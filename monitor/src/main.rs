use anyhow::{bail, Result};
use satnogs_network_client::Client;
use std::thread;
use systemstat::{Platform, System};

mod event;
mod job;
mod logger;
mod rotctld_client;
mod satnogs;
mod settings;
mod state;
mod station;
mod sysinfo;
mod ui;
mod vessel;
mod waterfall;
mod widgets;

use self::event::Event;
use self::rotctld_client::RotCtldClient;
use self::settings::{Settings, StationConfig};
use self::station::Station;
use self::sysinfo::SysInfo;
use self::waterfall::WaterfallWatcher;

use clap::{ArgGroup, Parser};

/// Monitors the current and future jobs of SatNOGS ground stations.
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = None,
    max_term_width = 100,
    group(
        ArgGroup::new("stations")
            .args(["local_station", "station"])
            .required(true)
            .multiple(true)
    )
)]
struct Cli {
    /// Sets the SatNOGS network api endpoint url
    #[arg(short, long = "api", value_name = "URL")]
    api_url: Option<String>,

    /// Adds a station running on the same machine as this monitor
    /// with this SatNOGS network id to to the list of monitored stations
    #[arg(short, long = "local", value_name = "ID", num_args(1..))]
    local_station: Vec<u64>,

    /// Adds a station with this SatNOGS network id to the
    /// list of monitored stations
    #[arg(short, long = "station", value_name = "ID", num_args(1..))]
    station: Vec<u64>,

    /// Sets custom config file
    #[arg(short, long = "config", value_name = "FILE")]
    config: Option<String>,

    /// Sets the number of orbits plotted on the map
    #[arg(short, long = "orbits", value_name = "NUM", default_value_t = 3, value_parser = clap::value_parser!(u8).range(1..))]
    orbits: u8,

    /// Sets the level of log verbosity
    #[arg(short = 'v', action = clap::ArgAction::Count)]
    verbosity: u8,

    /// Enables the spectrum and waterfall plot if set to the
    /// SatNOGS client data path
    #[arg(long = "data-path", value_name = "PATH")]
    data_path: Option<String>,

    /// Enables rotator monitoring if set to a rotctld address
    #[arg(long = "rotctld-address", value_name = "IP:PORT")]
    rotctld_address: Option<String>,

    /// Polls the rotator position every INTERVAL seconds
    #[arg(long = "rotctld-interval", value_name = "INTERVAL")]
    rotctld_interval: Option<u64>,

    /// Sets the lower dB bound of the spectrum and waterfall plot
    #[arg(long="db-min", value_name="DB",
    // value_parser = clap::value_parser!(f32).range(-200.0..0.0)
    )]
    db_min: Option<f32>,

    /// Sets the upper dB bound of the spectrum and waterfall plot
    #[arg(long = "db-max", value_name = "DB",
    // value_parser = clap::value_parser!(f32).range(-200.0..0.0)
    )]
    db_max: Option<f32>,

    /// Enables the spectrum plot
    #[arg(long = "spectrum")]
    spectrum: bool,

    /// Enables the waterfall plot
    #[arg(long = "waterfall")]
    waterfall: bool,

    /// Zooms the spectrum and waterfall plot (1.0 - 10.0)
    #[arg(long = "waterfall-zoom", value_name = "FACTOR")]
    waterfall_zoom: Option<f32>,

    /// Polls the network for new jobs every SECONDS
    #[arg(long = "job-update-interval", value_name = "SECONDS")]
    job_update_interval: Option<u64>,
}

fn main() -> Result<()> {
    run()
}

fn run() -> Result<()> {
    let settings = settings()?;
    // get the station info from the network
    let mut client = Client::new(&settings.api_endpoint)?;

    let mut state = state::State::new();

    for station in &settings.stations {
        state.add_station(client.station_info(station.satnogs_id).map(Station::new)?);

        if state.active_station == 0 {
            state.active_station = station.satnogs_id;
        }
    }

    state.update_ground_tracks(settings.ui.ground_track_num);

    let data_path = settings.data_path.clone();
    let rotctld_address = settings.rotctld_address.clone();
    let rotctld_interval = settings.rotctld_interval.clone();

    let local_stations: Vec<_> = settings
        .stations
        .iter()
        .filter(|sc| sc.local)
        .map(|sc| sc.satnogs_id)
        .collect();
    let tui = ui::Ui::new(settings, client, state)?;
    log::set_boxed_logger(Box::new(logger::Logger::new(tui.sender())))?;

    if !local_stations.is_empty() {
        let tx = tui.sender();
        thread::spawn(move || {
            while let Ok(sys_info) = get_sysinfo() {
                match tx.send(Event::SystemInfo(local_stations.clone(), sys_info)) {
                    Ok(_) => thread::sleep(std::time::Duration::new(4, 0)),
                    Err(e) => {
                        log::error!("Failed to send system info: {}", e);
                        break;
                    }
                }
            }
        });
    }

    // watch for waterfall if enabled
    if let Some(data_path) = data_path {
        log::info!("Starting waterfall watcher for {}", data_path);

        let tx = tui.sender();
        let mut waterfall_watcher = WaterfallWatcher::new(&data_path, tx)?;

        thread::spawn(move || {
            if let Err(err) = waterfall_watcher.run() {
                log::error!("Waterfall watcher stopped with error: {}", err);
            }
        });
    };

    if let Some(rotctld_address) = rotctld_address {
        log::info!("Connecting to rotctld at {}", rotctld_address);

        let tx = tui.sender();
        let mut client = RotCtldClient::new(&rotctld_address)?;

        log::info!(
            "Connected to rotctld at {} polling every {} seconds",
            rotctld_address,
            rotctld_interval
        );

        thread::spawn(move || {
            while let Ok(pos) = client.position() {
                log::trace!("RotCtl: {} / {}", pos.0, pos.1);
                match tx.send(Event::RotatorPosition(pos.0, pos.1)) {
                    Ok(_) => thread::sleep(std::time::Duration::new(rotctld_interval, 0)),
                    Err(e) => {
                        log::error!("Failed to send rotator position: {}", e);
                        break;
                    }
                }
            }

            log::error!("Lost connection to rotctld");
        });
    }

    tui.run()
}

fn get_sysinfo() -> Result<SysInfo> {
    let sys = System::new();
    let cpu_load = sys.cpu_load();
    thread::sleep(std::time::Duration::new(1, 0));

    Ok(SysInfo {
        cpu_load: cpu_load.and_then(|load| load.done()).ok(),
        cpu_temp: sys.cpu_temp().ok(),
        mem: sys.memory().ok(),
        uptime: sys.uptime().ok(),
    })
}

/// Generates the internal settings representation for the app. CLI options will
/// override the options loaded from config files.
fn settings() -> Result<Settings> {
    let cli = Cli::parse();

    let mut settings = match cli.config {
        Some(path) => Settings::from_file(&path)?,
        None => Settings::new()?,
    };

    let log_level = std::cmp::max(cli.verbosity as u64, settings.log_level.unwrap_or(0));

    let log_filter = match log_level {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };

    log::set_max_level(log_filter);

    if let Some(api_endpoint) = cli.api_url {
        settings.api_endpoint = api_endpoint;
    }

    for id in &cli.local_station {
        // if the station was already configured in the config file we just overwrite the local flag
        if let Some(sc) = settings.stations.iter_mut().find(|sc| sc.satnogs_id == *id) {
            sc.local = true;
        } else {
            let mut sc = StationConfig::new(*id);
            sc.local = true;
            settings.stations.push(sc);
        }
    }

    for &id in &cli.station {
        if settings
            .stations
            .iter()
            .find(|sc| sc.satnogs_id == id)
            .is_none()
        {
            settings.stations.push(StationConfig::new(id));
        }
    }

    if settings.stations.is_empty() {
        bail!("no station provided");
    }

    // only one entry per station
    settings.stations.sort_unstable_by_key(|sc| sc.satnogs_id);
    settings.stations.dedup_by_key(|sc| sc.satnogs_id);

    settings.ui.ground_track_num = cli.orbits;

    if let Some(addr) = cli.rotctld_address {
        settings.rotctld_address = Some(addr);
    }

    if let Some(i) = cli.rotctld_interval {
        settings.rotctld_interval = i;
    }

    if let Some(path) = cli.data_path {
        settings.data_path = Some(path);
    }

    if let Some(v) = cli.db_min {
        settings.ui.db_min = v;
    }

    if let Some(v) = cli.db_max {
        settings.ui.db_max = v;
    }

    if settings.ui.db_min >= settings.ui.db_max {
        bail!(
            "invalid dB range: {} >= {}",
            settings.ui.db_min,
            settings.ui.db_max
        );
    }

    settings.ui.spectrum_plot |= cli.spectrum;
    settings.ui.waterfall |= cli.waterfall;

    if let Some(mut waterfall_zoom) = cli.waterfall_zoom {
        if waterfall_zoom < 1.0 {
            waterfall_zoom = 1.0;
        }
        if waterfall_zoom > 10.0 {
            waterfall_zoom = 10.0;
        }

        settings.waterfall_zoom = waterfall_zoom;
    }

    if let Some(i) = cli.job_update_interval {
        settings.job_update_interval = i;
    }

    Ok(settings)
}
