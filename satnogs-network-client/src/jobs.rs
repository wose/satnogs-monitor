use chrono::{DateTime, Utc};
use restson::{Error, RestPath};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Deserialize, Debug)]
pub struct JobList ( pub Vec<Job> );

impl RestPath<()> for JobList {
    fn get_path(_: ()) -> Result<String, Error> {
        Ok(String::from("/api/jobs/"))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Job {
    pub id: u64,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub ground_station: u64,
    pub tle0: String,
    pub tle1: String,
    pub tle2: String,
    pub frequency: u64,
    #[serde(deserialize_with = "deserialize_null_default")]
    pub mode: String,
    pub transmitter: String,
    pub baud: Option<f64>,
}

impl RestPath<u64> for Job {
    fn get_path(id: u64) -> Result<String, Error> {
        Ok(format!("/api/jobs/{}/", id))
    }
}

fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de>,
    D: Deserializer<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}
