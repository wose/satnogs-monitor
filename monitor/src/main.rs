use log;

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

fn main() {
    let settings = Settings::new().expect("Config error");
    let mut tui = ui::Ui::new(&settings);

    log::set_boxed_logger(Box::new(logger::Logger::new(tui.sender())))
        .expect("Unable to create global logger");
    log::set_max_level(log::LevelFilter::Info);

    tui.update_ground_tracks();
    tui.run();
}
