use chrono::{DateTime, Utc};
use restson::{Error, RestPath};
use serde_derive::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum JobList {
    Array(Vec<Job>),
}

impl RestPath<()> for JobList {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("/api/jobs/"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub id: i64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub ground_station: i64,
    pub tle0: String,
    pub tle1: String,
    pub tle2: String,
    pub frequency: u64,
    pub mode: String,
    pub transmitter: String,
    pub baud: Option<f64>,
}

impl RestPath<i64> for Job {
    fn get_path(id: i64) -> Result<String, Error> {
        Ok(format!("/api/jobs/{}/", id))
    }
}
