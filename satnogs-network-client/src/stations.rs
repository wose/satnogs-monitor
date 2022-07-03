use chrono::{DateTime, Utc};
use restson::{Error, RestPath};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum StationList {
    Array(Vec<StationInfo>),
}

impl RestPath<()> for StationList {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("/api/stations/"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Antenna {
    /// minimum frequency
    frequency: u64,
    /// maximum frequency
    frequency_max: u64,
    /// frequency band
    band: String,
    /// antenna type
    antenna_type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StationStatus {
    Online,
    Offline,
    Testing,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StationInfo {
    /// station id
    pub id: u64,
    /// station name
    pub name: String,
    /// station position altitude
    pub altitude: f64,
    /// minimum horizon
    pub min_horizon: f64,
    /// station position latitude
    pub lat: f64,
    /// station position longitude
    pub lng: f64,
    /// QTH locator
    pub qthlocator: String,
    /// antennas
    pub antenna: Vec<Antenna>,
    /// date and time the station was created
    pub created: DateTime<Utc>,
    /// date and time the station was last seen by the network
    pub last_seen: Option<DateTime<Utc>>,
    /// current station status ["Online", "Offline", "Testing"]
    pub status: StationStatus,
    /// number of observations
    pub observations: u64,
    /// station description provided by the operator
    pub description: String,
}

impl RestPath<u64> for StationInfo {
    fn get_path(id: u64) -> Result<String, Error> {
        Ok(format!("/api/stations/{}/", id))
    }
}
