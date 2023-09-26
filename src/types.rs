use marine_rs_sdk::marine;
use serde::Deserialize; 
use serde::Serialize; 
use std::time::{ SystemTime, UNIX_EPOCH };

#[marine]
pub struct MetaContractResult {
    pub result: bool,
    pub metadatas: Vec<FinalMetadata>,
    pub error_string: String,
}

#[marine]
pub struct FinalMetadata {
    pub public_key: String,
    pub alias: String,
    pub content: String,
    pub loose: i64,
    pub version: String,
}

#[marine]
#[derive(Debug, Clone)]
pub struct Metadata {
    pub hash: String,
    pub token_key: String,
    pub data_key: String,
    pub meta_contract_id: String,
    pub token_id: String,
    pub alias: String,
    pub cid: String,
    pub public_key: String,
    pub version: String,
    pub loose: i64,
}

#[marine]
#[derive(Debug, Clone)]
pub struct Transaction {
    pub hash: String,
    pub method: String,
    pub meta_contract_id: String,
    pub data_key: String,
    pub token_key: String,
    pub data: String,
    pub public_key: String,
    pub alias: String,
    pub timestamp: u64,
    pub chain_id: String,
    pub token_address: String,
    pub token_id: String,
    pub version: String,
    pub status: i64,
    pub mcdata: String,
}

#[marine]
#[derive(Debug, Default, Clone)]
pub struct MetaContract {
    pub hash: String,
    pub token_key: String,
    pub meta_contract_id: String,
    pub public_key: String,
    pub cid: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct SerdeMetadata {
  pub loose: i64,
}

#[derive(Debug, Default, Deserialize)]
pub struct CommentMetadata {
    pub text: String,
    pub image: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FinalComment {
   pub from: String,
   pub message: String,
   pub timestamp: u64,
}

impl FinalComment {
   pub fn new(from: String, message: String) -> Self {
        let now = SystemTime::now();
        let timestamp= now.duration_since(UNIX_EPOCH).expect("Time went backwards").as_millis() as u64;
        FinalComment {
            from,
            message,
            timestamp
        }
    }
}
