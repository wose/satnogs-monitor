use gpredict::{Location, Predict, Sat, Tle};
use time;

pub struct Vessel {
    pub ground_track: Vec<(f64, f64)>,
    pub id: u64,
    pub qth: Location,
    sat: Sat,
    pub tle: Tle,
}

impl Vessel {
    pub fn new(id: u64, name: &str, tle1: &str, tle2: &str, qth: Location) -> Self {
        let tle = Tle {
            name: name.to_string(),
            line1: tle1.to_string(),
            line2: tle2.to_string(),
        };

        let mut predict = Predict::new(&tle, &qth);
        predict.update(None);

        Vessel {
            ground_track: vec![],
            id,
            sat: predict.sat,
            tle: tle,
            qth: qth,
        }
    }

    pub fn name(&self) -> &str {
        &self.tle.name
    }

    pub fn sat(&self) -> &Sat {
        &self.sat
    }

    pub fn update_position(&mut self, orbits: u8) {
        let mut predict = Predict::new(&self.tle, &self.qth);
        predict.update(None);
        let update_ground_track = self.sat.orbit_nr != predict.sat.orbit_nr || self.ground_track.is_empty();
        self.sat = predict.sat;
        if update_ground_track {
            self.update_ground_track(orbits);
        }
    }

    pub fn update_ground_track(&mut self, orbits: u8) {
        // get the current orbit number and go back in time to the start of the orbit
        // then go forward in time until the next orbit (or orgbit numbers)
        let mut predict = Predict::new(&self.tle, &self.qth);
        predict.update(None);

        let mut current_orbit = self.sat.orbit_nr;
        let this_orbit = current_orbit;
        let mut time = time::now_utc();

        while current_orbit == this_orbit {
            time = time - time::Duration::seconds(15);
            predict.update(Some(time));
            current_orbit = predict.sat.orbit_nr;
        }

        current_orbit = this_orbit;
        self.ground_track.clear();
        while current_orbit < this_orbit + orbits as u64 {
            time = time + time::Duration::seconds(15);
            predict.update(Some(time));
            self.ground_track
                .push((predict.sat.lon_deg, predict.sat.lat_deg));
            current_orbit = predict.sat.orbit_nr;
        }
    }
}
