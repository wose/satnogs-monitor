//use termion::event::Event;
use log::Level;
use crate::satnogs::Data;
use crate::sysinfo::SysInfo;

pub enum Event {
    Input(termion::event::Event),
    Log((Level, String)),
    CommandResponse(Data),
    NoSatnogsNetworkConnection,
    Resize,
    Shutdown,
    SystemInfo(u64, SysInfo),
    Tick,
}
