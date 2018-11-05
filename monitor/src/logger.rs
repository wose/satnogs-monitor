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
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let _message = format!(
            "{}, {}, {}",
            record.args(),
            record.file().unwrap_or("?"),
            record.line().unwrap_or(0)
        );

        let messgae = format!("{}", record.args());
        let _ = self.sender.send(Event::Log((record.level(), messgae)));
    }

    fn flush(&self) {
    }
}
