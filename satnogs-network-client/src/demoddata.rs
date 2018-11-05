use serde_derive::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DemodData {
    pub payload_demod: String,
}
