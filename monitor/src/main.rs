use clap::{App, Arg, crate_version, crate_authors};
use failure;
use log;
use std::process;

mod event;
mod logger;
mod station;
mod ui;
mod satnogs;
mod settings;
mod theme;
mod vessel;
//mod waterfall;

use self::settings::Settings;

type Result<T> = std::result::Result<T, failure::Error>;

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
    let mut tui = ui::Ui::new(&settings);

    log::set_boxed_logger(Box::new(logger::Logger::new(tui.sender())))?;

    tui.update_ground_tracks();
    tui.run();

    Ok(())
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
                .takes_value(true)
                .display_order(0),
        )
        .arg(Arg::with_name("verbosity")
             .short("v")
             .multiple(true)
             .help("Sets the level of verbosity")
        );

    let matches = app.get_matches();

    let settings = if matches.is_present("config") {
        let config_file = matches.value_of("config").unwrap();
        Settings::from_file(config_file)?
    } else {
        Settings::new()?
    };

    let log_level = std::cmp::max(matches.occurrences_of("verbosity"), settings.log_level.unwrap_or(0));
    let log_filter = match log_level {
        0 => log::LevelFilter::Warn,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _3_or_more => log::LevelFilter::Trace,
    };

    log::set_max_level(log_filter);
    Ok(settings)
}
