use restson::{Error, RestClient};

use crate::{JobList, Observation, ObservationFilter, ObservationList};
use crate::{StationInfo, StationList};

pub struct Client {
    client: RestClient,
}

impl Client {
    pub fn new(url: &str) -> Result<Self, Error> {
        let client = RestClient::new(url)?;
        Ok(Client { client: client })
    }

    pub fn with_api_key(url: &str, api_key: &str) -> Result<Self, Error> {
        let mut client = RestClient::new(url)?;
        client.set_header("Authorization", &format!("Token {}", api_key))?;
        Ok(Client { client: client })
    }

    pub fn jobs(&mut self, id: u64) -> Result<JobList, Error> {
        self.client
            .get_with((), &[("ground_station", &format!("{}", id))])
    }

    pub fn observations(&mut self, filter: &ObservationFilter) -> Result<Vec<Observation>, Error> {
        let filter: Vec<_> = filter.into();
        let mut observations = vec![];

        // We cannot use the response headers to know if there is next page with more
        // results. So we iterate over every page and stop if we receive an HttpError 404.
        for page in 1.. {
            let mut filter = filter.clone();
            let page = format!("{}", page);
            filter.push(("page", &page));
            match self.client.get_with((), &filter) {
                Ok(ObservationList::Array(ref obs)) => {
                    observations.extend_from_slice(obs);
                    // check if we are surely on the last page
                    if obs.len() < 25 {
                        break
                    }

                },
                Err(Error::HttpError(404, _)) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(observations)
    }

    pub fn observation(&mut self, id: u64) -> Result<Observation, Error> {
        self.client.get(id)
    }

    pub fn stations(&mut self) -> Result<StationList, Error> {
        self.client.get(())
    }

    pub fn station_info(&mut self, id: u64) -> Result<StationInfo, Error> {
        self.client.get(id)
    }
}
