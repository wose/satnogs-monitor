//use termion::event::Event;
use log::Level;
use crate::satnogs::Data;

pub enum Event {
    Input(termion::event::Event),
    Log((Level, String)),
    CommandResponse(Data),
    NoSatnogsNetworkConnection,
    Resize,
    Shutdown,
    Tick,
}
