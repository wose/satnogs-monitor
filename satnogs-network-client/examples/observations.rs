use satnogs_network_client::{Client, ObservationFilter};

fn main() {
    let mut client = Client::new("https://network.satnogs.org/api/").unwrap();

    let station = 175;
    let norad_cat_id = 40907;

    let filter = ObservationFilter::new()
        .ground_station(station)
        .norad_cat_id(norad_cat_id);
    let obs = client.observations(&filter).unwrap();

    println!(
        "Got {} observations for sat {} on station {}",
        obs.len(),
        norad_cat_id,
        station
    );
    //    println!("{:?}", obs);
}
