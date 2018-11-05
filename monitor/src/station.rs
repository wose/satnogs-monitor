use log::info;
use satnogs_network_client::{Observation, ObservationList};
use std::fmt;

#[derive(Copy, Clone)]
pub enum StationStatus {
    Idle,
    Observing,
    Offline,
}

pub struct Station {
    pub active: bool,
    pub id: u32,
    pub name: String,
    pub status: StationStatus,
    pub observations: Vec<Observation>,
}

impl Station {
    pub fn new(id: u32, name: &str) -> Self {
        Station {
            active: false,
            id: id,
            name: name.into(),
            status: StationStatus::Offline,
            observations: vec![],
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn status(&self) -> StationStatus {
        self.status
    }

    pub fn set_status(&mut self, status: StationStatus) {
        self.status = status;
    }

    pub fn update_observations(&mut self, observations: &[Observation]) {
        for obs in observations {
            match self
                .observations
                .iter_mut()
                .find(|ref observation| { observation.id == obs.id })
            {
                Some(mut observation) => {
                    info!("Update Observation {} on station {} ({})", obs.id, self.name, self.id);
                    observation = &mut obs.clone()
                },
                None => {
                    info!("New observation {} on station {} ({})", obs.id, self.name, self.id);
                    self.observations.push(obs.clone())
                },
            }
        }
    }
}

impl fmt::Display for Station {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} - {}", self.id, self.name)
    }
}
