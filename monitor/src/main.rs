use clap::{crate_authors, crate_version, value_t, values_t, App, Arg};
use failure::Fail;
use log::{error, info};
use satnogs_network_client::Client;
use std::process;
use std::thread;
use systemstat::{Platform, System};

mod event;
mod job;
mod logger;
mod satnogs;
mod settings;
mod state;
mod station;
mod sysinfo;
mod theme;
mod ui;
mod vessel;
//mod waterfall;

use self::event::Event;
use self::settings::{Settings, StationConfig};
use self::station::Station;
use self::sysinfo::SysInfo;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
#[fail(display = "No station provided")]
struct NoStationError;


fn main() {
    if let Err(err) = run() {
        eprintln!("{}", format_error(&err));
        let backtrace = err.backtrace().to_string();
        if !backtrace.trim().is_empty() {
            eprintln!("{}", backtrace);
        }
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let settings = settings()?;
    // get the station info from the network
    let mut client = Client::new("https://network.satnogs.org/api/")?;

    let mut state = state::State::new();

    for station in &settings.stations {
        state.add_station(
            client
                .station_info(station.satnogs_id)
                .map(|si| Station::new(si))?,
        );
    }

    let local_station = settings.local_station;
    let mut tui = ui::Ui::new(settings, client, state)?;

    if local_station {
        let tx = tui.sender();
        thread::spawn(move || {
            while let Ok(sys_info) = get_sysinfo() {
                match tx.send(Event::SystemInfo(175, sys_info)) {
                    Ok(_) => thread::sleep(std::time::Duration::new(4, 0)),
                    Err(e) => {
                        error!("Failed to send system info: {}", e);
                        break;
                    }
                }
            }
        });
    }

    log::set_boxed_logger(Box::new(logger::Logger::new(tui.sender())))?;

    tui.update_ground_tracks();
    tui.run()
}

fn get_sysinfo() -> Result<SysInfo> {
    let sys = System::new();
    let cpu_load = sys.cpu_load();
    thread::sleep(std::time::Duration::new(1, 0));

    info!("{:?}", sys.cpu_temp());

    Ok(
        SysInfo {
            cpu_load: cpu_load.and_then(|load| load.done()).ok(),
            cpu_temp: sys.cpu_temp().ok(),
            mem: sys.memory().ok(),
            uptime: sys.uptime().ok(),
        }
    )
}

fn format_error(err: &failure::Error) -> String {
    let mut out = "Error occurred: ".to_string();
    out.push_str(&err.to_string());
    let mut prev = err.as_fail();
    while let Some(next) = prev.cause() {
        out.push_str("\n -> ");
        out.push_str(&next.to_string());
        prev = next;
    }
    out
}

fn settings() -> Result<Settings> {
    let app = App::new("satnogs-monitor")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("Monitors the current and future jobs of SatNOGS ground stations.")
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
                .help("Set if a sation runs on the same machine as the monitor")
        )
        .arg(
            Arg::with_name("orbits")
                .short("o")
                .long("orbits")
                .help("Sets the number of orbits plotted on the map")
                .value_name("NUM")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("station")
                .short("s")
                .long("station")
                .help("Adds a station with this SatNOGS network id for this session")
                .value_name("ID")
                .takes_value(true)
                .multiple(true),
        )
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
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

    if let Ok(ids) = values_t!(matches.values_of("station"), u64) {
        for id in ids {
            settings.stations.push(StationConfig::new(id));
        }
    }

    if settings.stations.is_empty() {
        return Err(NoStationError.into());
    }

    // only one entry per station
    settings.stations.sort_unstable_by_key(|sc| sc.satnogs_id);
    settings.stations.dedup_by_key(|sc| sc.satnogs_id);

    if let Ok(orbits) = value_t!(matches.value_of("orbits"), u8) {
        settings.ui.ground_track_num = std::cmp::max(1, orbits);
    }

    if matches.is_present("local_station") {
        settings.local_station = true;
    }

    Ok(settings)
}
