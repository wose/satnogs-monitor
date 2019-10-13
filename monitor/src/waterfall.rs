use crate::event::Event;
use crate::Result;

use byteorder::{LittleEndian, ReadBytesExt};
use crossbeam_channel::{unbounded, Receiver};
use lazy_static::lazy_static;
use notify::{immediate_watcher, Op, RawEvent, RecursiveMode, Watcher};
use regex::Regex;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, Seek, SeekFrom};
use std::sync::mpsc::SyncSender;
use std::thread;

lazy_static! {
    static ref RE: Regex = Regex::new(r".*/.*receiving_waterfall_(\d+)_.*\.dat.*").unwrap();
}

struct WaterfallFile {
    fft_size: u64,
    observation: u64,
    reader: BufReader<File>,
}

pub struct WaterfallWatcher {
    event_tx: SyncSender<Event>,
    file: Option<WaterfallFile>,
    watcher_rx: Receiver<RawEvent>,
    watcher: notify::RecommendedWatcher,
}

impl WaterfallWatcher {
    pub fn new(path: &str, event_tx: SyncSender<Event>) -> Result<Self> {
        let (watcher_tx, watcher_rx) = unbounded();

        let mut watcher = immediate_watcher(watcher_tx)?;
        watcher.watch(path, RecursiveMode::NonRecursive)?;

        Ok(WaterfallWatcher {
            event_tx,
            file: None,
            watcher_rx,
            watcher,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        'event_loop: loop {
            match self.watcher_rx.recv() {
                Ok(event) => {
                    if let (Ok(op), Some(path)) = (event.op, &event.path) {
                        // we are only interested in waterfall files
                        if let Some(obs_id) = RE.captures(path.to_str().unwrap_or("")) {
                            let observation: u64 = obs_id[1].parse().unwrap();

                            // a new waterfall was created
                            // wait until the header has been written and read it
                            if op.contains(Op::CREATE) {
                                match OpenOptions::new().read(true).open(path) {
                                    Ok(file) => {
                                        // wait until the fft size is written
                                        // if it takes longer than 5 seconds we stop and handle the next
                                        // event
                                        let now = std::time::SystemTime::now();
                                        while file.metadata().unwrap().len() < 4 {
                                            match now.elapsed() {
                                                Ok(elapsed) if elapsed.as_secs() <= 5 => {
                                                    thread::sleep(std::time::Duration::from_millis(
                                                        10,
                                                    ))
                                                }
                                                _ => continue 'event_loop,
                                            };
                                        }

                                        let mut reader = BufReader::new(file);
                                        let fft_size =
                                            reader.read_f32::<LittleEndian>().unwrap() as u64;

                                        // wait until the frequencies are written
                                        let now = std::time::SystemTime::now();
                                        while reader.get_ref().metadata().unwrap().len()
                                            < 4 + 4 * fft_size
                                        {
                                            match now.elapsed() {
                                                Ok(elapsed) if elapsed.as_secs() <= 5 => {
                                                    thread::sleep(std::time::Duration::from_millis(
                                                        10,
                                                    ))
                                                }
                                                _ => continue 'event_loop,
                                            };
                                        }

                                        let mut frequencies = vec![];
                                        frequencies.reserve(fft_size as usize);
                                        for _ in 0..fft_size {
                                            frequencies
                                                .push(reader.read_f32::<LittleEndian>().unwrap());
                                        }

                                        if let Err(err) = self
                                            .event_tx
                                            .send(Event::WaterfallCreated(observation, frequencies))
                                        {
                                            log::error!(
                                                "Failed to send waterfall creation event: {}",
                                                err
                                            );
                                        }

                                        self.file = Some(WaterfallFile {
                                            fft_size,
                                            observation,
                                            reader,
                                        });
                                    }
                                    Err(err) => {
                                        log::error!(
                                            "Failed to open waterfall file {}: {}",
                                            path.to_str().unwrap_or(""),
                                            err
                                        );
                                        continue;
                                    }
                                }
                            }

                            // some data has been written, check if it's at least one complete
                            // spectrum line and send it to the ui
                            if op.contains(Op::WRITE) {
                                while self.is_data_available() {
                                    if let Some(file) = self.file.as_mut() {
                                        let seconds =
                                            file.reader.read_f32::<LittleEndian>().unwrap();
                                        let mut power = vec![];
                                        power.reserve(file.fft_size as usize);

                                        for _ in 0..file.fft_size {
                                            power.push(
                                                file.reader.read_f32::<LittleEndian>().unwrap(),
                                            );
                                        }

                                        if let Err(err) =
                                            self.event_tx.send(Event::WaterfallData(seconds, power))
                                        {
                                            log::error!(
                                                "Failed to send waterfall data event: {}",
                                                err
                                            );
                                        }
                                    }
                                }
                            }

                            // the waterfall is closed by the satnogs client
                            // all data has been written and was read here so we can discard the
                            // file notify the ui
                            if op.contains(Op::CLOSE_WRITE) {
                                log::info!("Closed waterfall file for observation {}", observation);
                                self.file = None;
                                if let Err(err) =
                                    self.event_tx.send(Event::WaterfallClosed(observation))
                                {
                                    log::error!("Failed to send waterfall closing event: {}", err);
                                }
                            }
                        }
                    };
                }
                Err(err) => {
                    log::error!(
                        "Failed to receive waterfall watcher event: {}. Stopping watcher.",
                        err
                    );
                    break;
                }
            }
        }

        Ok(())
    }

    fn is_data_available(&mut self) -> bool {
        if let Some(file) = self.file.as_mut() {
            let size = file.reader.get_ref().metadata().unwrap().len();
            let position = file.reader.seek(SeekFrom::Current(0)).unwrap();

            (size - position >= file.fft_size * 4 + 4)
        } else {
            false
        }
    }
}
