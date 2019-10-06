use chrono::{DateTime, Utc};
use gpredict::{Location, Predict, Sat, Tle};
use time;

pub struct Vessel {
    pub footprint: Vec<(f64, f64)>,
    pub ground_track: Vec<(f64, f64)>,
    pub polar_track: Vec<(f64, f64)>,
    pub id: u64,
    pub qth: Location,
    sat: Sat,
    pub tle: Tle,
}

impl Vessel {
    pub fn new(
        id: u64,
        name: &str,
        tle1: &str,
        tle2: &str,
        qth: Location,
        aos: DateTime<Utc>,
        los: DateTime<Utc>,
    ) -> Self {
        let tle = Tle {
            name: name.to_string(),
            line1: tle1.to_string(),
            line2: tle2.to_string(),
        };

        let mut predict = Predict::new(&tle, &qth);
        let polar_track = calc_polar_track(&mut predict, aos, los);

        predict.update(None);

        Vessel {
            footprint: vec![],
            ground_track: vec![],
            polar_track,
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
        let update_ground_track =
            self.sat.orbit_nr != predict.sat.orbit_nr || self.ground_track.is_empty();
        self.sat = predict.sat;
        if update_ground_track {
            self.update_ground_track(orbits);
        }
        self.update_footprint();
    }

    pub fn update_footprint(&mut self) {
        use std::f64::consts::PI;
        let xkmper = 6.378135E3;
        let footprint = 12756.33 * (xkmper / (xkmper + self.sat.alt_km)).acos();
        let beta = (0.5 * footprint) / xkmper;

        self.footprint.clear();

        for azi in 0..180 {
            let azimuth = (azi as f64).to_radians();
            let sat_lat = self.sat.lat_deg.to_radians();
            let sat_lon = self.sat.lon_deg.to_radians();

            let range_lat =
                (sat_lat.sin() * beta.cos() + azimuth.cos() * beta.sin() * sat_lat.cos()).asin();

            let num = beta.cos() - sat_lat.sin() * range_lat.sin();
            let dem = sat_lat.cos() * range_lat.cos();

            let mut range_lon = match (num, dem) {
                (x, y) if (x / y).abs() > 1.0 => sat_lon,
                (x, y) if y > 0.0 => sat_lon - (x / y).acos(),
                (x, y) if y < 0.0 => sat_lon + (x / y).acos() + PI,
                _ => 0.0,
            };

            while range_lon < -PI {
                range_lon += 2.0 * PI;
            }

            while range_lon > PI {
                range_lon -= 2.0 * PI;
            }

            let range_lon_deg = range_lon.to_degrees();

            let mut diff = self.sat.lon_deg - range_lon_deg;
            while diff < 0.0 {
                diff += 360.0;
            }
            while diff > 360.0 {
                diff -= 360.0;
            }

            let mut mirror_lon_deg = self.sat.lon_deg + diff.abs();
            while mirror_lon_deg > 180.0 {
                mirror_lon_deg -= 360.0;
            }
            while mirror_lon_deg < -180.0 {
                mirror_lon_deg += 360.0;
            }

            self.footprint.push((range_lon_deg, range_lat.to_degrees()));
            self.footprint
                .push((mirror_lon_deg, range_lat.to_degrees()));
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
            time = time - time::Duration::seconds(10);
            predict.update(Some(time));
            current_orbit = predict.sat.orbit_nr;
        }

        current_orbit = this_orbit;
        self.ground_track.clear();
        while current_orbit < this_orbit + orbits as u64 {
            time = time + time::Duration::seconds(10);
            predict.update(Some(time));
            self.ground_track
                .push((predict.sat.lon_deg, predict.sat.lat_deg));
            current_orbit = predict.sat.orbit_nr;
        }
    }
}

pub fn calc_polar_track(
    predict: &mut Predict,
    aos: DateTime<Utc>,
    los: DateTime<Utc>,
) -> Vec<(f64, f64)> {
    let mut polar_track = vec![];
    let time_aos = time::at_utc(time::Timespec::new(aos.timestamp(), 0));
    let time_los = time::at_utc(time::Timespec::new(los.timestamp(), 0));

    let mut time = time_aos;
    while time <= time_los {
        predict.update(Some(time));
        polar_track.push((predict.sat.az_deg, predict.sat.el_deg));
        time = time + time::Duration::seconds(2);
    }

    polar_track
}
