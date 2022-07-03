use restson::{blocking, Error, Response, RestClient};

use crate::{Job, JobList, Observation, ObservationFilter, ObservationList};
use crate::{StationInfo, StationList};

pub struct Client {
    client: blocking::RestClient,
}

impl Client {
    pub fn new(url: &str) -> Result<Self, Error> {
        let client = RestClient::new_blocking(url)?;
        Ok(Client { client })
    }

    pub fn with_api_key(url: &str, api_key: &str) -> Result<Self, Error> {
        let mut client = RestClient::new_blocking(url)?;
        client.set_header("Authorization", &format!("Token {}", api_key))?;
        Ok(Client { client })
    }

    pub fn jobs(&mut self, id: u64) -> Result<Vec<Job>, Error> {
        self.client
            .get_with((), &[("ground_station", &format!("{}", id))])
            .and_then(|resp: Response<JobList>| {
                let JobList(jobs) = resp.into_inner();
                Ok(jobs)
            })
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
                Ok(resp) => {
                    let ObservationList::Array(ref obs) = *resp;
                    observations.extend_from_slice(obs);
                    // check if we are surely on the last page
                    if obs.len() < 25 {
                        break;
                    }
                }
                Err(Error::HttpError(404, _)) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(observations)
    }

    pub fn observation(&mut self, id: u64) -> Result<Observation, Error> {
        self.client
            .get(id)
            .and_then(|resp: Response<Observation>| Ok(resp.into_inner()))
    }

    pub fn stations(&mut self) -> Result<StationList, Error> {
        self.client
            .get(())
            .and_then(|resp: Response<StationList>| Ok(resp.into_inner()))
    }

    pub fn station_info(&mut self, id: u64) -> Result<StationInfo, Error> {
        self.client
            .get(id)
            .and_then(|resp: Response<StationInfo>| Ok(resp.into_inner()))
    }
}
