#![allow(improper_ctypes)]

mod types;

use std::collections::HashMap;
use marine_rs_sdk::marine;
use marine_rs_sdk::module_manifest;
use marine_rs_sdk::MountedBinaryResult;
use marine_rs_sdk::WasmLoggerBuilder;
use types::MetaContract;
use types::Metadata;
use types::SerdeMetadata;
use types::Transaction;
use types::{FinalMetadata, MetaContractResult};
use ethabi::{decode, ParamType};

module_manifest!();

pub fn main() {
    WasmLoggerBuilder::new()
        .with_log_level(log::LevelFilter::Info)
        .build()
        .unwrap();
}

#[marine]
pub fn on_execute(
    contract: MetaContract,
    metadatas: Vec<Metadata>,
    transaction: Transaction,
) -> MetaContractResult {
    let mut finals: Vec<FinalMetadata> = vec![];
    
    let serde_metadata: Result<SerdeMetadata, serde_json::Error> = serde_json::from_str(&transaction.mcdata.clone());
    let loose;

    match serde_metadata {
      Ok(sm) => loose = sm.loose,
      _ => loose = 1,
    }
    finals.push(FinalMetadata {
        public_key: transaction.public_key,
        alias: transaction.alias,
        content: transaction.data,
        loose,
    });

    MetaContractResult {
        result: true,
        metadatas: finals,
        error_string: "".to_string(),
    }
}

#[marine]
pub fn on_clone() -> bool {
    return true;
}

#[marine]
pub fn on_mint(contract: MetaContract, data_key: String, token_id: String, data: String) -> MetaContractResult {

    MetaContractResult {
        result: true,
        metadatas: Vec::new(),
        error_string: "".to_string(),
    }
}