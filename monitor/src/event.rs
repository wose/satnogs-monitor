use crate::satnogs::Data;
use crate::sysinfo::SysInfo;
use log::Level;

pub enum Event {
    Input(termion::event::Event),
    Log((Level, String)),
    CommandResponse(Data),
    Resize,
    RotatorPosition(f64, f64),
    SystemInfo(Vec<u64>, SysInfo),
    Tick,
    WaterfallCreated(u64, Vec<f32>),
    WaterfallData(i64, Vec<f32>),
    WaterfallClosed(u64),
}
