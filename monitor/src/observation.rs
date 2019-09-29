use crate::vessel::Vessel;
use time;

pub struct Observation {
    pub norad_id: u64,
    pub start: time::TM,
    pub end: time::TM,
    pub rise: f64,
    pub max: f64,
    pub set: f64,
    pub polar_plot: Vec<(f64, f64)>,
}

impl Observation {
    pub fn new(start: time::TM, end: time::TM, vessel: &mut Vessel) -> Self {
        vessel.predict.update(start);
        let rise = vessel.predict.sat.az_deg;
        let set = vessel.predict.sat.az_deg;
        let mut observation = Observation {
            norad_id: vessel.id,
            start: start,
            end: end,
            rise: rise,
            max: 0.0,
            set: set,
            polar_plot: vec![],
        };
        observation.update_polar_plot(&mut vessel);

        observation
    }

    fn update_polar_plot(&mut self, vessel: &mut Vessel) {}
}
