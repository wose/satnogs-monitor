use chrono::{DateTime, Utc};
use satnogs_network_client::{Client, Observation, ObservationFilter, ObservationList};

fn main() {
    let mut client = Client::new("https://network.satnogs.org/api/").unwrap();

    let jobs = client.jobs(175).unwrap();
    println!("{:?}", jobs);


    let filter = ObservationFilter::new()
        .ground_station(175)
        .start(Utc::now())
        .norad_cat_id(40907);
    let ObservationList::Array(obs) = client.observations_with_filter(&filter).unwrap();
    println!("Count: {}", obs.len());
}
