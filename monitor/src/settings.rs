use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StationConfig {
    pub local: bool,
    pub satnogs_id: u64,
    pub rt_ip: Option<String>,
    pub rt_port: Option<u32>,
}

impl StationConfig {
    pub fn new(id: u64) -> Self {
        StationConfig {
            local: false,
            satnogs_id: id,
            rt_ip: None,
            rt_port: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UiConfig {
    pub ground_track_num: u8,
}

impl UiConfig {
    pub fn new() -> Self {
        UiConfig {
            ground_track_num: 3,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub log_level: Option<u64>,
    pub ui: UiConfig,
    pub stations: Vec<StationConfig>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();
        settings.set_default("log_level", 0)?;
        settings.set_default("ui.ground_track_num", 3)?;
        settings.set_default("stations", Vec::<config::Value>::new())?;

        if let Some(project_dirs) = ProjectDirs::from("org", "SatNOGS", "satnogs-monitor") {
            let file = File::with_name(
                project_dirs
                    .config_dir()
                    .join("config.toml")
                    .to_str()
                    .ok_or(ConfigError::Message("Invalid project dir".to_string()))?,
            );
            settings.merge(file.required(false))?;
        }

        settings.try_into()
    }

    pub fn from_file(file: &str) -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name(file))?;
        settings.try_into()
    }
}
