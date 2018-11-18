use chrono::{Duration, Utc};
use satnogs_network_client as snc;
use std::fmt;

use crate::job::Job;

pub struct Station {
    pub info: snc::StationInfo,
    pub jobs: Vec<Job>,
}

impl Station {
    pub fn new(info: snc::StationInfo) -> Self {
        Station {
            info: info,
            jobs: vec![],
        }
    }

    pub fn id(&self) -> u64 {
        self.info.id
    }

    pub fn name(&self) -> &str {
        &self.info.name
    }

    pub fn remove_finished_jobs(&mut self) {
        self.jobs.retain(|job|
                         job.end() - Utc::now() > Duration::zero()
        );
    }

    pub fn update_jobs(&mut self, jobs: Vec<(snc::Job, snc::Observation)>) {
        for job in jobs {
            if self.jobs.iter().find(|j| j.id() == job.0.id).is_none() {
                self.jobs.push(Job::new(job, self.location()));
            }
        }
        self.jobs.sort_unstable_by_key(|job| job.start());
    }

    pub fn location(&self) -> gpredict::Location {
        gpredict::Location {
            lat_deg: self.info.lat,
            lon_deg: self.info.lng,
            alt_m: self.info.altitude,
        }
    }
}

impl fmt::Display for Station {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.id(), self.name())
    }
}
