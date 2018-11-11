use chrono::{DateTime, Utc};
use gpredict::{Location, Sat};
use satnogs_network_client as snc;
use crate::vessel::Vessel;

pub struct Job {
    job: snc::Job,
    pub observation: snc::Observation,
    pub vessel: Vessel,
}

impl Job {
    pub fn new(job: (snc::Job, snc::Observation), qth: Location) -> Self {
        let (job, observation) = job;
        Job {
            vessel: Vessel::new(
                observation.norad_cat_id,
                &job.tle0,
                &job.tle1,
                &job.tle2,
                qth,
            ),
            job,
            observation,
        }
    }

    pub fn id(&self) -> u64 {
        self.job.id
    }

    pub fn frequency_mhz(&self) -> f64 {
        self.job.frequency as f64 /  1_000_000.0
    }

    pub fn vessel_name(&self) -> &str {
        &self.vessel.tle.name
    }

    pub fn start(&self) -> DateTime<Utc> {
        self.job.start
    }

    pub fn end(&self) -> DateTime<Utc> {
        self.job.end
    }

    pub fn mode(&self) -> &str {
        &self.job.mode
    }

    pub fn update_position(&mut self) {
        self.vessel.update_position();
    }

    pub fn update_ground_track(&mut self) {
        self.vessel.update_ground_track();
    }

    pub fn sat(&self) -> &Sat {
        self.vessel.sat()
    }
}
