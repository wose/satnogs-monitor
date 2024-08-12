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

        let mut pages_remaining = true;
        let mut filter_cursor = "".to_owned();
        while pages_remaining {
            pages_remaining = false;
            let mut filter = filter.clone();
            filter.push(("cursor", &filter_cursor));
            match self.client.get_with((), &filter) {
                Ok(resp) => {
                    let resp_data: Response<ObservationList> = resp; 
                    let resp_headers = resp_data.headers();
                    if resp_headers.contains_key("link") {
                        let link_header = &resp_headers["Link"];
                        let res = parse_link_header::parse(link_header.to_str().unwrap());
                        assert!(res.is_ok());

                        let val = res.unwrap();
                        let next_link = val.get(&Some("next".to_string()));
                        if next_link.is_some() {
                            let next_url = &next_link.unwrap();
                            filter_cursor = next_url.queries["cursor"].to_string();
                            pages_remaining = true;
                        }
                    }

                    let ObservationList::Array(obs) = resp_data.into_inner();
                    observations.extend_from_slice(&obs);
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
