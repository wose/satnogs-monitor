use chrono::{DateTime, Utc};
use restson::{Error, RestPath};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum StationList {
    Array(Vec<Station>),
}

impl RestPath<()> for StationList {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("/api/stations/"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Station {
    pub id: i64,
    pub name: String,
    pub altitude: f64,
    pub min_horizon: f64,
    pub lat: f64,
    pub lng: f64,
    pub qthlocator: String,
    pub location: String,
    pub antenna: Vec<String>,
    pub created: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub status: String,
    pub observations: u64,
    pub description: String,
}

impl RestPath<i64> for Station {
    fn get_path(id: i64) -> Result<String, Error> {
        Ok(format!("/api/stations/{}/", id))
    }
}
