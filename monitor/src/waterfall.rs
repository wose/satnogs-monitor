use crate::event::Event;
use crate::Result;

use anyhow::bail;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use chrono::{DateTime, FixedOffset};
use crossbeam_channel::{unbounded, Receiver};
use itertools_num::linspace;
use lazy_static::lazy_static;
use notify::{immediate_watcher, Event as RawEvent, RecursiveMode, Watcher};
use regex::Regex;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::sync::mpsc::SyncSender;
use std::thread;

const HEADER_SIZE: u64 = 32 + 20;

lazy_static! {
    static ref RE: Regex = Regex::new(r".*/.*receiving_waterfall_(\d+)_.*\.dat.*").unwrap();
}

#[allow(unused)]
struct WaterfallHeader {
    /// Center frequency, the frequency your SDR is tuned to
    center_freq: f32,
    // This should probably be an enum, not sure how to parse it yet
    endianess: u32,
    /// FFT size
    fft_size: u32,
    // not sure what this does
    nfft_per_row: u32,
    /// Sample rate
    sample_rate: u32,
    /// Start of the observation
    timestamp: DateTime<FixedOffset>,
}

impl WaterfallHeader {
    pub fn from_reader<T>(reader: &mut T) -> Result<Self>
    where
        T: Read,
    {
        let mut buf = [0; 32];
        reader.read_exact(&mut buf)?;
        let timestamp = parse_timestamp(&buf)?;
        let fft_size = reader.read_u32::<BigEndian>()?;
        let sample_rate = reader.read_u32::<BigEndian>()?;
        let nfft_per_row = reader.read_u32::<BigEndian>()?;
        let center_freq = reader.read_f32::<BigEndian>()?;
        let endianess = reader.read_u32::<BigEndian>()?;

        Ok(WaterfallHeader {
            center_freq,
            endianess,
            fft_size,
            nfft_per_row,
            sample_rate,
            timestamp,
        })
    }
}

fn parse_timestamp(buf: &[u8]) -> Result<DateTime<FixedOffset>> {
    let end = buf.iter().position(|&c| c == b'\0').unwrap_or(buf.len());

    let timestamp = std::str::from_utf8(&buf[0..end])?;
    let datetime = DateTime::parse_from_rfc3339(timestamp)?;

    Ok(datetime)
}

struct WaterfallFile {
    fft_size: u64,
    _observation: u64,
    reader: BufReader<File>,
}

pub struct WaterfallWatcher {
    event_tx: SyncSender<Event>,
    file: Option<WaterfallFile>,
    watcher_rx: Receiver<std::result::Result<RawEvent, notify::Error>>,
    _watcher: notify::RecommendedWatcher,
}

impl WaterfallWatcher {
    pub fn new(path: &str, event_tx: SyncSender<Event>) -> Result<Self> {
        let (watcher_tx, watcher_rx) = unbounded();

        let mut watcher = immediate_watcher(move |evt| watcher_tx.send(evt).unwrap())?;
        watcher.watch(path, RecursiveMode::NonRecursive)?;

        Ok(WaterfallWatcher {
            event_tx,
            file: None,
            watcher_rx,
            _watcher: watcher,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        loop {
            match self.watcher_rx.recv() {
                Ok(Ok(event)) => {
                    let (kind, paths) = (event.kind, &event.paths);
                    log::trace!("EventKind: {:?} Paths: {:?}", kind, paths);
                    if paths.is_empty() {
                        continue;
                    }
                    let path = &paths[0];
                    // we are only interested in waterfall files
                    if let Some(obs_id) = RE.captures(path.to_str().unwrap_or("")) {
                        let observation: u64 = obs_id[1].parse().unwrap();

                        match kind {
                            // a new waterfall was created
                            // wait until the header has been written and read it
                            notify::EventKind::Create(notify::event::CreateKind::File) => {
                                if let Err(err) = self.on_waterfall_file_created(observation, &path)
                                {
                                    log::error!(
                                        "Failed to open waterfall file {}: {}",
                                        path.to_str().unwrap_or(""),
                                        err
                                    );
                                    continue;
                                }
                            }

                            // some data has been written, check if it's at least one complete
                            // spectrum line and send it to the ui
                            notify::EventKind::Modify(notify::event::ModifyKind::Data(_)) => {
                                if let Err(err) = self.on_waterfall_data_changed() {
                                    log::error!("Failed to send waterfall data event: {}", err);
                                }
                            }

                            // the waterfall is closed by the satnogs client
                            // all data has been written and was read here so we can discard the
                            // file notify the ui
                            notify::EventKind::Access(notify::event::AccessKind::Close(_)) => {
                                if let Err(err) = self.on_waterfall_closed(observation) {
                                    log::error!("Failed to send waterfall closing event: {}", err);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Err(err)) => {
                    log::error!("Notify error: {}. Stopping watcher.", err);
                    break;
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

    fn on_waterfall_file_created(
        &mut self,
        observation: u64,
        path: &std::path::PathBuf,
    ) -> Result<()> {
        match OpenOptions::new().read(true).open(&path) {
            Ok(file) => {
                // wait until the header is written
                // if it takes longer than 5 seconds we stop and handle the next
                // event, this also means we're skipping this waterfall
                let now = std::time::SystemTime::now();
                while file.metadata().unwrap().len() < HEADER_SIZE {
                    match now.elapsed() {
                        Ok(elapsed) if elapsed.as_secs() <= 5 => {
                            thread::sleep(std::time::Duration::from_millis(10))
                        }
                        _ => bail!("Missing Waterfall Header"),
                    };
                }

                let mut reader = BufReader::new(file);
                let header = WaterfallHeader::from_reader(&mut reader).unwrap();

                let frequencies: Vec<_> = linspace::<f32>(
                    -0.5 * header.sample_rate as f32,
                    0.5 * header.sample_rate as f32,
                    header.fft_size as usize,
                )
                .collect();

                if let Err(err) = self
                    .event_tx
                    .send(Event::WaterfallCreated(observation, frequencies))
                {
                    log::error!("Failed to send waterfall creation event: {}", err);
                }

                self.file = Some(WaterfallFile {
                    fft_size: header.fft_size as u64,
                    _observation: observation,
                    reader,
                });

                Ok(())
            }
            Err(err) => {
                log::error!(
                    "Failed to open waterfall file {}: {}",
                    path.to_str().unwrap_or(""),
                    err
                );
                Err(err.into())
            }
        }
    }

    fn on_waterfall_data_changed(&mut self) -> Result<()> {
        while self.is_data_available() {
            if let Some(file) = self.file.as_mut() {
                let seconds = file
                    .reader
                    .read_i64::<LittleEndian>()?;
                let mut power = vec![];
                power.reserve(file.fft_size as usize);

                for _ in 0..file.fft_size {
                    power.push(
                        file.reader
                            .read_f32::<LittleEndian>()?,
                    );
                }

                self.event_tx
                    .send(Event::WaterfallData(seconds, power))?;
            }
        }

        Ok(())
    }

    fn on_waterfall_closed(&mut self, observation: u64) -> Result<()> {
        log::info!("Closed waterfall file for observation {}", observation);
        self.file = None;
        self.event_tx
            .send(Event::WaterfallClosed(observation))
            .map_err(|e| e.into())
    }

    fn is_data_available(&mut self) -> bool {
        if let Some(file) = self.file.as_mut() {
            let size = file.reader.get_ref().metadata().unwrap().len();
            let position = file.reader.seek(SeekFrom::Current(0)).unwrap();

            size - position >= file.fft_size * 4 + 8
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_datetime_string() {
        let buf: &[u8] = b"2020-03-23T09:34:47.193416Z\x00\xde\xad\xc0\xde";
        let _datetime = parse_timestamp(buf).unwrap();
    }

    #[test]
    #[should_panic]
    fn err_on_invalid_waterfall_timestamp() {
        let buf: &[u8] = b"2020-03!23T09:34:47.193416Z\x00\xde\xad\xc0\xde";
        let _datetime = parse_timestamp(buf).unwrap();
    }
}
