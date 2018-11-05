use config::{ConfigError, Config, File};
use serde;
use serde_derive::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct HeaderStyle {
    background: String,
    foreground: String,
}

#[derive(Debug, Deserialize)]
pub struct Theme {
    header: HeaderStyle,

}

impl Theme {
    pub fn from_file(_file: &Path) -> Result<Self, ConfigError> {
        Ok(Theme{
            header: HeaderStyle {
                background: "".into(),
                foreground: "".into(),
            }
        })
    }
}
