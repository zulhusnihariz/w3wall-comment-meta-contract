use serde::{Deserialize, Serialize};

#[derive(Deserialize, Clone)]
pub struct DataStructFork {
    pub owner: String,
    pub data_key: String,
    pub cid: String,
}

#[derive(Serialize, Deserialize)]
pub struct OpenSeaAttributes {
  pub display_type: String,
  pub trait_type: String,
  pub value: i32,
}