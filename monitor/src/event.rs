use crate::satnogs::Data;
use crate::sysinfo::SysInfo;
use log::Level;

pub enum Event {
    Input(termion::event::Event),
    Log((Level, String)),
    CommandResponse(Data),
    Resize,
    SystemInfo(Vec<u64>, SysInfo),
    Tick,
}
