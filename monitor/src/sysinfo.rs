use systemstat::data::{CPULoad, Duration, Memory};

#[derive(Default)]
pub struct SysInfo {
    pub cpu_temp: Option<f32>,
    pub cpu_load: Option<Vec<CPULoad>>,
    pub mem: Option<Memory>,
    pub uptime: Option<Duration>,
}
