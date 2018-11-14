use crate::event::Event;
use log::{Log, Metadata, Record};
use std::sync::mpsc::SyncSender;

pub struct Logger {
    sender: SyncSender<Event>,
}

impl Logger {
    pub fn new(sender: SyncSender<Event>) -> Self {
        Logger {
            sender,
        }
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target().starts_with("satnogs")
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let messgae = format!("{}", record.args());
            let _ = self.sender.send(Event::Log((record.level(), messgae)));
        }
    }

    fn flush(&self) {
    }
}
