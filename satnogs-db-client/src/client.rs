use restson::{blocking, Error, Response, RestClient};

use crate::Satellite;

pub struct Client {
    client: blocking::RestClient,
}

impl Client {
    pub fn new(url: &str) -> Result<Self, Error> {
        let client = RestClient::new_blocking(url)?;
        Ok(Client { client })
    }

    pub fn satellite(&mut self, id: String) -> Result<Satellite, Error> {
        self.client
            .get(id)
            .and_then(|resp: Response<Satellite>| Ok(resp.into_inner()))
    }
}
