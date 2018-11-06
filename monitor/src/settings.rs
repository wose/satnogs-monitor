use config::{Config, ConfigError, File};
use directories::ProjectDirs;
use serde;
use serde_derive::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct StationConfig {
    pub satnogs_id: u32,
    pub name: String,
    pub rt_ip: Option<String>,
    pub rt_port: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub stations: Vec<StationConfig>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut settings = Config::new();
        if let Some(project_dirs) = ProjectDirs::from("org", "SatNOGS", "satnogs-monitor") {
            let file = File::with_name(
                project_dirs
                    .config_dir()
                    .join("config.toml")
                    .to_str()
                    .ok_or(ConfigError::Message("Invalid project dir".to_string()))?
            );
            settings.merge(file)?;
        }

        settings.try_into()
    }

    pub fn from_file(file: &str) -> Result<Self, ConfigError> {
        let mut settings = Config::new();

        settings.merge(File::with_name(file))?;
        settings.try_into()
    }
}
