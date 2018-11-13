use crate::event::Event;
use log::{error, info, trace, warn};
use satnogs_network_client::{Job, Observation, ObservationList, StationInfo};
use std::sync::mpsc::{sync_channel, SendError, SyncSender};
use std::thread;

pub enum Data {
    Jobs(u64, Vec<(Job, Observation)>),
    Observations(ObservationList),
    StationInfo(u64, StationInfo),
}

pub enum Command {
    GetJobs(u64),
    GetObservation(Option<u32>),
    GetStationInfo(u64),
}

pub struct Connection {
    command_tx: SyncSender<Command>,
}

impl Connection {
    pub fn new(data_tx: SyncSender<Event>) -> Self {
        let (command_tx, command_rx) = sync_channel(100);
        thread::spawn(move || {
            let mut client = satnogs_network_client::Client::new("https://network.satnogs.org/api/").unwrap();

            while let Ok(command) = command_rx.recv() {
                match command {
                    Command::GetStationInfo(ground_station) => {
                        trace!("GetStationInfo({})", ground_station);
                        if let Ok(station_info) = client.station_info(ground_station) {
                            data_tx.send(Event::CommandResponse(Data::StationInfo(ground_station, station_info))).unwrap();
                        } else {
                            data_tx.send(Event::NoSatnogsNetworkConnection).unwrap();
                        }
                    }
                    Command::GetJobs(id) => {
                        if let Ok(satnogs_network_client::JobList::Array(jobs)) = client.jobs(id) {
                            let mut jobs_obs = vec![];
                            for job in jobs {
                                if let Ok(observation) = client.observation(job.id) {
                                    jobs_obs.push((job, observation));
                                } else {
                                    warn!("No observation for job {} on station {}", job.id, id);
                                }
                            }

                            data_tx.send(Event::CommandResponse(Data::Jobs(id, jobs_obs))).unwrap();
                        } else {
                            error!("Couldn't get jobs for station {}", id);
                        }
                    }
                    Command::GetObservation(Some(id)) => {
                        info!("GetObservation({})", id);
                    },
                    Command::GetObservation(None) => {
                        info!("GetObservation(None)");
                    },
                }
            }

            warn!("Command channel closed");
        });



        Self {
            command_tx: command_tx,
        }
    }

    pub fn send(&mut self, command: Command) -> Result<(), SendError<Command>> {
        self.command_tx.send(command)
    }
}
