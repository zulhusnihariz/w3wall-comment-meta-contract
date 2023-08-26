#![allow(improper_ctypes)]

mod types;
mod data;
mod defaults;

use std::collections::HashMap;
use data::DataStructFork;
use defaults::DEFAULT_IPFS_MULTIADDR;
use defaults::DEFAULT_TIMEOUT_SEC;
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
    let mut error: Option<String> = None;
    let mut finals: Vec<FinalMetadata> = vec![];
    // extract out data
    if data.len() > 0 {

        let data_bytes = &hex::decode(&data);

        match data_bytes {
          Ok(decoded) => {
            let param_types = vec![
              ParamType::String,
              ParamType::String,
              ParamType::String,
            ];

            let results = decode(&param_types, decoded);

            match results {
              Ok(result) => {
                if result.len() == 3 {
                  
                  let ipfs_multiaddr = result[1].clone().to_string();
                  let cid = result[2].clone().to_string();
                  
                  let datasets = get(cid, ipfs_multiaddr, 0);
                  let result: Result<Vec<DataStructFork>, serde_json::Error> =
                      serde_json::from_str(&datasets);

                  match result {
                      Ok(datas) => {

                          for data in datas {

                              finals.push(FinalMetadata {
                                  public_key: data.owner,
                                  alias: "".to_string(),
                                  content: data.cid,
                                  loose: 0,
                              });

                          }
                      }
                      Err(e) => error = Some(format!("Invalid data structure: {}", e.to_string())),
                  }
                }
              },
              Err(e) => error = Some(format!("Invalid data structure: {}", e.to_string())),
            }
          },
          Err(e) => error = Some(format!("Invalid data structure: {}", e.to_string())),
        }
    }

    if !error.is_none() {
      return MetaContractResult {
        result: false,
        metadatas: Vec::new(),
        error_string: error.unwrap().to_string(),
      };
    }

    MetaContractResult {
        result: true,
        metadatas: finals,
        error_string: "".to_string(),
    }
}

/**
 * Get data from ipfs
 */
fn get(hash: String, api_multiaddr: String, timeout_sec: u64) -> String {
  let address: String;
  let t;

  if api_multiaddr.is_empty() {
      address = DEFAULT_IPFS_MULTIADDR.to_string();
  } else {
      address = api_multiaddr;
  }

  if timeout_sec == 0 {
      t = DEFAULT_TIMEOUT_SEC;
  } else {
      t = timeout_sec;
  }

  let args = vec![String::from("dag"), String::from("get"), hash];

  let cmd = make_cmd_args(args, address, t);

  let result = ipfs(cmd);

  String::from_utf8(result.stdout).unwrap()
}

pub fn make_cmd_args(args: Vec<String>, api_multiaddr: String, timeout_sec: u64) -> Vec<String> {
  args.into_iter()
      .chain(vec![
          String::from("--timeout"),
          get_timeout_string(timeout_sec),
          String::from("--api"),
          api_multiaddr,
      ])
      .collect()
}

#[inline]
pub fn get_timeout_string(timeout: u64) -> String {
  format!("{}s", timeout)
}

// Service
// - curl

#[marine]
#[link(wasm_import_module = "host")]
extern "C" {
  pub fn ipfs(cmd: Vec<String>) -> MountedBinaryResult;
}