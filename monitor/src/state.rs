use crate::station::Station;
use crate::vessel::Vessel;
use std::collections::{BTreeMap, HashMap};

pub struct State {
    pub stations: BTreeMap<u64, Station>,
    pub vessels: HashMap<u64, Vessel>,
}

impl State {
    pub fn new() -> Self {
        State {
            stations: BTreeMap::new(),
            vessels: HashMap::new(),
        }
    }

    pub fn add_station(&mut self, station: Station) {
        self.stations.insert(station.id(), station);
    }

    pub fn add_vessel(&mut self, vessel: Vessel) {
        self.vessels.insert(vessel.id, vessel);
    }
}
