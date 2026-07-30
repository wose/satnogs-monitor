use chrono::{DateTime, Utc};
use restson::{Error, RestPath};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Satellite {
    pub sat_id: String,
    pub norad_cat_id: u64,
    pub norad_follow_id: Option<u64>,
    pub name: String,
    pub names: String,
    pub image: String,
    pub status: String,
    pub decayed: Option<DateTime<Utc>>,
    pub launched: DateTime<Utc>,
    pub deployed: Option<DateTime<Utc>>,
    pub website: String,
    pub operator: String,
    pub countries: String,
    pub updated: DateTime<Utc>,
    pub citation: String,
    pub is_frequency_violator: bool,
    pub associated_satellites: Vec<String>,
}

impl RestPath<String> for Satellite {
    fn get_path(id: String) -> Result<String, Error> {
        Ok(format!("/api/satellites/{}/", id))
    }
}
