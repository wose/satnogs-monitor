use chrono::{DateTime, Utc};
use restson::{Error, RestPath};
use serde_derive::{Serialize, Deserialize};
use std::convert::From;

use crate::demoddata::DemodData;

#[derive(Default)]
pub struct ObservationFilter {
    ground_station: String,
    start: String,
    end: String,
    norad_cat_id: String,
}

impl<'a> ObservationFilter {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn ground_station(mut self, id: u64) -> Self {
        self.ground_station = format!("{}", id);
        self
    }

    pub fn start(mut self, dt: DateTime<Utc>) -> Self {
        self.start = dt.to_rfc3339();
        self
    }

    pub fn end(mut self, dt: DateTime<Utc>) -> Self {
        self.end = dt.to_rfc3339();
        self
    }

    pub fn norad_cat_id(mut self, id: u64) -> Self {
        self.norad_cat_id = format!("{}", id);
        self
    }
}

impl<'a> From<&'a ObservationFilter> for Vec<(&'a str, &'a str)> {
    fn from(filter: &'a ObservationFilter) -> Vec<(&'a str, &'a str)> {
        let mut params = vec![];
        if !filter.ground_station.is_empty() {
            params.push(("ground_station", filter.ground_station.as_str()));
        }

        if !filter.start.is_empty() {
            params.push(("start", filter.start.as_str()));
        }

        if !filter.end.is_empty() {
            params.push(("end", filter.end.as_str()));
        }

        if !filter.norad_cat_id.is_empty() {
            params.push(("satellite__norad_cat_id", filter.norad_cat_id.as_str()));
        }
        params
    }
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum ObservationList {
    Array(Vec<Observation>),
}

impl RestPath<()> for ObservationList {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("/api/observations/"))
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Observation {
    pub id: u64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub ground_station: u64,
    pub transmitter: String,
    pub norad_cat_id: u64,
    pub payload: Option<String>,
    pub waterfall: Option<String>,
    pub demoddata: Vec<DemodData>,
    pub station_name: String,
    pub station_lat: f64,
    pub station_lng: f64,
    pub station_alt: f64,
    pub vetted_status: String,
    pub rise_azimuth: f64,
    pub set_azimuth: f64,
    pub max_altitude: f64,
    pub archived: bool,
    pub archive_url: Option<String>,
    pub client_version: String,
    pub client_metadata: String,
}

impl RestPath<u64> for Observation {
    fn get_path(id: u64) -> Result<String, Error> {
        Ok(format!("/api/observations/{}/", id))
    }
}
