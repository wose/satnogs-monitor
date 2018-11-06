use crate::event::Event;
use log::{error, info, trace, warn};
use std::sync::mpsc::{sync_channel, SendError, SyncSender};
use std::thread;

pub enum Data {
    Jobs(u32, Vec<satnogs_network_client::Job>),
    Observations(satnogs_network_client::ObservationList),
    StationInfo(u32, satnogs_network_client::Station),
}

pub enum Command {
    GetJobs(Option<i64>),
    GetObservation(Option<u32>),
    GetStationInfo(u32),
}

pub struct Connection {
//    client: satnogs_network_client::Client,
    command_tx: SyncSender<Command>,
//    data: SyncSender<Data>,
}

impl Connection {
    pub fn new(data_tx: SyncSender<Event>) -> Self {
        let (command_tx, command_rx) = sync_channel(100);
        thread::spawn(move || {
            let mut client = satnogs_network_client::Client::new("https://network.satnogs.org/api/").unwrap();

            while let Ok(command) = command_rx.recv() {
                match command {
                    Command::GetJobs(Some(ground_station)) => {
                        trace!("GetJobs({})", ground_station);
                        if let Ok(satnogs_network_client::JobList::Array(jobs)) = client.jobs(ground_station) {
                            data_tx.send(Event::CommandResponse(Data::Jobs(ground_station as u32, jobs))).unwrap();
                        } else {
                            data_tx.send(Event::NoSatnogsNetworkConnection).unwrap();
                        }
                    },
                    Command::GetStationInfo(ground_station) => {
                        trace!("GetStationInfo({})", ground_station);
                        if let Ok(station_info) = client.station_info(ground_station) {
                            data_tx.send(Event::CommandResponse(Data::StationInfo(ground_station, station_info))).unwrap();
                        } else {
                            data_tx.send(Event::NoSatnogsNetworkConnection).unwrap();
                        }
                    }
                    Command::GetJobs(None) => {
                        info!("GetJobs(None)");
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
        //    client: satnogs_network_client::Client::new("https://network.satnogs.org/api/").unwrap(),
            command_tx: command_tx,
        //    data: data_tx,
        }
    }

    pub fn send(&mut self, command: Command) -> Result<(), SendError<Command>> {
        self.command_tx.send(command)
    }
}
