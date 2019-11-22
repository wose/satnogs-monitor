use crate::event::Event;
use chrono::Utc;
use log::{debug, error, trace, warn};
use satnogs_network_client::{Job, Observation, ObservationFilter};
use std::sync::mpsc::{sync_channel, SendError, SyncSender};
use std::thread;

pub enum Data {
    Jobs(u64, Vec<(Job, Observation)>),
}

pub enum Command {
    GetJobs(u64),
}

pub struct Connection {
    command_tx: SyncSender<Command>,
}

impl Connection {
    pub fn new(data_tx: SyncSender<Event>, api_endpoint: String) -> Self {
        let (command_tx, command_rx) = sync_channel(100);
        thread::spawn(move || {
            let mut client = satnogs_network_client::Client::new(&api_endpoint).unwrap();

            while let Ok(command) = command_rx.recv() {
                match command {
                    Command::GetJobs(id) => {
                        if let Ok(observations) = client.observations(
                            &ObservationFilter::new()
                                .start(Utc::now())
                                .ground_station(id),
                        ) {
                            client
                                .jobs(id)
                                .and_then(|jobs| {
                                    let jobs = jobs
                                        .into_iter()
                                        .filter_map(|job| {
                                            if let Some(obs) = observations
                                                .iter()
                                                .find(|observation| observation.id == job.id)
                                            {
                                                trace!("Got all infos for job {}", job.id);
                                                Some((job, obs.clone()))
                                            } else {
                                                debug!("No observation for job {} found", job.id);
                                                None
                                            }
                                        })
                                        .collect::<Vec<_>>();
                                    Ok(jobs)
                                })
                                .and_then(|jobs| {
                                    data_tx
                                        .send(Event::CommandResponse(Data::Jobs(id, jobs)))
                                        .unwrap_or_else(|e| {
                                            error!("Failed to send Data::Job response: {}", e)
                                        });
                                    Ok(())
                                })
                                .unwrap_or_else(|e| {
                                    error!("Failed to map jobs to observations: {}", e)
                                });
                        } else {
                            error!("Failed to get observations for station {}", id);
                        }
                    }
                }
            }

            warn!("Command channel closed");
        });

        Self {
            command_tx,
        }
    }

    pub fn send(&mut self, command: Command) -> Result<(), SendError<Command>> {
        self.command_tx.send(command)
    }
}
