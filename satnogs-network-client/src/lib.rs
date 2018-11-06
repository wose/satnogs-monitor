//extern crate chrono;
//extern crate restson;

//#[macro_use]
//extern crate serde_derive;
//extern crate serde;

#![feature(custom_attribute)]

mod demoddata;
mod observations;
mod client;
mod jobs;
mod stations;

pub use crate::client::Client;
pub use crate::jobs::{Job, JobList};
pub use crate::observations::{Observation, ObservationList, ObservationFilter};
pub use crate::stations::Station;
