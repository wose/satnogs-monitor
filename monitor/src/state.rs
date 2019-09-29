use crate::station::Station;
use crate::vessel::Vessel;

use satnogs_network_client as snc;

use std::collections::{BTreeMap, HashMap};

pub struct State {
    pub active_station: u64,
    pub stations: BTreeMap<u64, Station>,
    pub vessels: HashMap<u64, Vessel>,
}

impl State {
    pub fn new() -> Self {
        State {
            active_station: 0,
            stations: BTreeMap::new(),
            vessels: HashMap::new(),
        }
    }

    pub fn add_station(&mut self, station: Station) {
        self.stations.insert(station.id(), station);
    }

    pub fn get_active_station(&self) -> &Station {
        self.stations.get(&self.active_station).unwrap()
    }

    pub fn get_active_station_mut(&mut self) -> &mut Station {
        self.stations.get_mut(&self.active_station).unwrap()
    }

    pub fn update_jobs(&mut self, id: u64, jobs: Vec<(snc::Job, snc::Observation)>) {
        self.stations
            .entry(id)
            .and_modify(|station| station.update_jobs(jobs));
    }

    pub fn update_ground_tracks(&mut self, ground_tracks: u8) {
        let station = self.get_active_station_mut();
        if let Some(job) = station.jobs.iter_mut().next() {
            job.update_ground_track(ground_tracks);
        }
    }

    pub fn update_vessel_position(&mut self, ground_tracks: u8) {
        let station = self.get_active_station_mut();
        if let Some(job) = station.jobs.iter_mut().next() {
            job.update_position(ground_tracks);
        }
    }

    pub fn next_station(&mut self) {
        if self.stations.len() > 1 {
            self.active_station = *self
                .stations
                .keys()
                .skip_while(|id| **id != self.active_station)
                .skip(1)
                .next()
                .unwrap_or(self.stations.keys().next().unwrap());
        }
    }

    pub fn prev_station(&mut self) {
        if self.stations.len() > 1 {
            self.active_station = *self
                .stations
                .keys()
                .rev()
                .skip_while(|id| **id != self.active_station)
                .skip(1)
                .next()
                .unwrap_or(self.stations.keys().rev().next().unwrap());
        }
    }
}
