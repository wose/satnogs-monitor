use clap::{crate_authors, crate_version, value_t, values_t, App, Arg};
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

fn settings() -> Result<Settings> {
    let author = env!("CARGO_PKG_AUTHORS").replace(":", "\n");
    let version = env!("CARGO_PKG_VERSION");
    let app = App::new("satnogs-monitor")
        .version(version)
        .author(&*author)
        .about("Monitors the current and future jobs of SatNOGS ground stations.")
        .max_term_width(100)
        .arg(
            Arg::with_name("api_url")
                .short("a")
                .long("api")
                .help("Sets the SatNOGS network api endpoint url")
                .value_name("URL")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .help("Sets custom config file")
                .value_name("FILE")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("local_station")
                .short("l")
                .long("local")
                .help(
                    "Adds a station running on the same machine as this monitor \
                     with this SatNOGS network id to to the list of monitored stations",
                )
                .value_name("ID")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("orbits")
                .short("o")
                .long("orbits")
                .help("Sets the number of orbits plotted on the map")
                .value_name("NUM")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("station")
                .short("s")
                .long("station")
                .help(
                    "Adds a station with this SatNOGS network id to the list of \
                     monitored stations",
                )
                .value_name("ID")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the level of log verbosity"),
        )
        .arg(
            Arg::with_name("data_path")
                .long("data-path")
                .value_name("PATH")
                .takes_value(true)
                .help(
                    "Enables the spectrum and waterfall plot if set to the SatNOGS \
                     client data path (/tmp/.satnogs/data/)",
                ),
        )
        .arg(
            Arg::with_name("rotctld_address")
                .long("rotctld-address")
                .value_name("IP:PORT")
                .takes_value(true)
                .help("Enables rotator monitoring if set to a rotctld address"),
        )
        .arg(
            Arg::with_name("rotctld_interval")
                .long("rotctld-interval")
                .value_name("INTERVAL")
                .takes_value(true)
                .help("Polls the rotator position every INTERVAL seconds (5)"),
        )
        .arg(
            Arg::with_name("db_min")
                .long("db-min")
                .value_name("DB")
                .takes_value(true)
                .help("Sets the lower dB bound of the spectrum and waterfall plot (-100)"),
        )
        .arg(
            Arg::with_name("db_max")
                .long("db-max")
                .value_name("DB")
                .takes_value(true)
                .help("Sets the upper dB bound of the spectrum and waterfall plot (0)"),
        )
        .arg(
            Arg::with_name("spectrum")
                .long("spectrum")
                .help("Enables the spectrum plot"),
        )
        .arg(
            Arg::with_name("waterfall")
                .long("waterfall")
                .help("Enables the waterfall plot"),
        )
        .arg(
            Arg::with_name("waterfall_zoom")
                .long("waterfall-zoom")
                .value_name("FACTOR")
                .takes_value(true)
                .help("Zooms the spectrum and waterfall plot (1.0 - 10.0)"),
        )
        .arg(
            Arg::with_name("job_update_interval")
                .long("job-update-interval")
                .value_name("SECONDS")
                .takes_value(true)
                .help("Polls the network for new jobs every SECONDS (600)"),
        );

    let matches = app.get_matches();

    let mut settings = matches
        .value_of("config")
        .map_or(Settings::new(), |config| Settings::from_file(config))?;

    let log_level = std::cmp::max(
        matches.occurrences_of("verbosity"),
        settings.log_level.unwrap_or(0),
    );
    let log_filter = match log_level {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _3_or_more => log::LevelFilter::Trace,
    };

    log::set_max_level(log_filter);

    if let Ok(api_endpoint) = value_t!(matches.value_of("api_url"), String) {
        settings.api_endpoint = api_endpoint;
    }

    if let Ok(ids) = values_t!(matches.values_of("local_station"), u64) {
        for id in ids {
            // if the station was already configured in the config file we just overwrite the local flag
            if let Some(sc) = settings.stations.iter_mut().find(|sc| sc.satnogs_id == id) {
                (*sc).local = true;
            } else {
                let mut sc = StationConfig::new(id);
                sc.local = true;
                settings.stations.push(sc);
            }
        }
    }

    if let Ok(ids) = values_t!(matches.values_of("station"), u64) {
        for id in ids {
            if settings
                .stations
                .iter()
                .find(|&sc| sc.satnogs_id == id)
                .is_none()
            {
                settings.stations.push(StationConfig::new(id));
            }
        }
    }

    if settings.stations.is_empty() {
        bail!("no station provided");
    }

    // only one entry per station
    settings.stations.sort_unstable_by_key(|sc| sc.satnogs_id);
    settings.stations.dedup_by_key(|sc| sc.satnogs_id);

    if let Ok(orbits) = value_t!(matches.value_of("orbits"), u8) {
        settings.ui.ground_track_num = std::cmp::max(1, orbits);
    }

    if let Ok(rotctld_address) = value_t!(matches.value_of("rotctld_address"), String) {
        settings.rotctld_address = Some(rotctld_address);
    }

    if let Ok(rotctld_interval) = value_t!(matches.value_of("rotctld_interval"), u64) {
        settings.rotctld_interval = rotctld_interval;
    }

    if let Ok(data_path) = value_t!(matches.value_of("data_path"), String) {
        settings.data_path = Some(data_path);
    }

    if let Ok(db_min) = value_t!(matches.value_of("db_min"), f32) {
        settings.ui.db_min = db_min;
    }

    if let Ok(db_max) = value_t!(matches.value_of("db_max"), f32) {
        settings.ui.db_max = db_max;
    }

    if settings.ui.db_min >= settings.ui.db_max {
        bail!("invalid dB range: {} >= {}", settings.ui.db_min, settings.ui.db_max);
    }

    settings.ui.spectrum_plot |= matches.is_present("spectrum");
    settings.ui.waterfall |= matches.is_present("waterfall");

    if let Ok(mut waterfall_zoom) = value_t!(matches.value_of("waterfall_zoom"), f32) {
        if waterfall_zoom < 1.0 {
            waterfall_zoom = 1.0;
        }
        if waterfall_zoom > 10.0 {
            waterfall_zoom = 10.0;
        }

        settings.waterfall_zoom = waterfall_zoom;
    }

    if let Ok(job_update_interval) = value_t!(matches.value_of("job_update_interval"), u64) {
        settings.job_update_interval = job_update_interval;
    }

    Ok(settings)
}
