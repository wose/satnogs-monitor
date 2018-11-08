use restson::{RestClient, Error};

//use demoddata::DemodData;
//use jobs::*;
//use observations::*;
//use stations::*;

use crate::{JobList, Job, ObservationList, ObservationFilter};
use crate::{StationList, StationInfo};

pub struct Client {
    api_key: Option<String>,
    client: RestClient,
}

impl Client {
    pub fn new(url: &str) -> Result<Self, Error> {
        let client = RestClient::new(url)?;
        Ok(
            Client {
                api_key: None,
                client: client,
            }
        )
    }

    pub fn with_api_key(url: &str, api_key: &str) -> Result<Self, Error> {
        let mut client = RestClient::new(url)?;
        client.set_header("Authorization", &format!("Token {}", api_key))?;
        Ok(
            Client {
                api_key: Some(api_key.into()),
                client: client,
            })
    }

    pub fn jobs(&mut self, ground_station: i64) -> Result<JobList, Error> {
        self.client.get_with((), &[("ground_station", &format!("{}", ground_station))])
    }

    pub fn observations(&mut self) -> Result<ObservationList, Error> {
        // we have to specify that we want the result in json format
        self.client.get_with((), &[("format", "json")])
    }

    pub fn observations_with_filter(&mut self, filter: &ObservationFilter) -> Result<ObservationList, Error> {
        let filter: Vec<_> = (filter).into();
        self.client.get_with((), &filter)
    }

    pub fn stations(&mut self) -> Result<StationList, Error> {
        self.client.get(())
    }

    pub fn station_info(&mut self, station_id: u32) -> Result<StationInfo, Error> {
        self.client.get(station_id)
    }
}
