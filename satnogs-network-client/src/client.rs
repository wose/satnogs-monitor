use restson::{RestClient, Error};

use crate::{JobList, Observation, ObservationList, ObservationFilter};
use crate::{StationList, StationInfo};

pub struct Client {
    client: RestClient,
}

impl Client {
    pub fn new(url: &str) -> Result<Self, Error> {
        let client = RestClient::new(url)?;
        Ok(
            Client {
                client: client,
            }
        )
    }

    pub fn with_api_key(url: &str, api_key: &str) -> Result<Self, Error> {
        let mut client = RestClient::new(url)?;
        client.set_header("Authorization", &format!("Token {}", api_key))?;
        Ok(
            Client {
                client: client,
            })
    }

    pub fn jobs(&mut self, ground_station: u64) -> Result<JobList, Error> {
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

    pub fn observation(&mut self, id: u64) -> Result<Observation, Error> {
        self.client.get(id)
    }

    pub fn stations(&mut self) -> Result<StationList, Error> {
        self.client.get(())
    }

    pub fn station_info(&mut self, station_id: u64) -> Result<StationInfo, Error> {
        self.client.get(station_id)
    }
}
