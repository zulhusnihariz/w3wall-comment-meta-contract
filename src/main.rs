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
use serde_json::to_value;
use types::Block;
use types::MetaContract;
use types::Metadata;
use types::Transaction;
use types::{SerdeMetadata, FinalMetadata, MetaContractResult, FinalComment};
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
    let final_comment: FinalComment;
    let mut cid: String = "".to_string();
    let mut parent_cid: String = "".to_string();
    let mut content: Vec<serde_json::Value> = vec![];
    
    let serde_metadata: Result<SerdeMetadata, serde_json::Error> = serde_json::from_str(&transaction.data.clone());

    match serde_metadata {
      Ok(tx_data) => {

        if tx_data.cid.is_empty() { 
          return MetaContractResult {
            result: false,
            metadatas: Vec::new(),
            error_string: "Cid cannot be empty.".to_string(),
         };
        }

        if tx_data.content.text.is_empty() { 
          return MetaContractResult {
            result: false,
            metadatas: Vec::new(),
            error_string: "Text cannot be empty.".to_string(),
         };
        }

        if is_profane(&tx_data.content.text) {
          return MetaContractResult {
              result: false,
              metadatas: Vec::new(),
              error_string: "Profanity found in the text.".to_string(),
          };
        }

        if tx_data.is_invalid_media_link() {
          return MetaContractResult {
              result: false,
              metadatas: Vec::new(),
              error_string: "Invalid media link format.".to_string(),
          };
        }

        parent_cid = tx_data.cid.clone();
        final_comment= FinalComment::new(transaction.public_key.clone(), tx_data.content.text);

        for metadata in metadatas.clone(){
          if metadata.cid == tx_data.cid && metadata.alias =="comments" {
            cid = metadata.cid;
          }
        }
      }
      Err(_) => {
        return MetaContractResult {
          result: false,
          metadatas: Vec::new(),
          error_string: "Data does not follow the required JSON schema".to_string(),
        }
      }
    }

    if !cid.is_empty() {
      let ipfs_get_result = get(cid, "".to_string(), 0);
      let block: Block = serde_json::from_str(&ipfs_get_result).unwrap();
      content = block.content
    }

    let final_comment_as_value = to_value(&final_comment).unwrap();
    content.push(final_comment_as_value);

    let serialized_content= serde_json::to_string(&content);

    match serialized_content{
      Ok(content) => {

        finals.push(FinalMetadata {
            public_key: transaction.meta_contract_id,
            alias: "comments".to_string(),
            content,
            version: parent_cid,
            loose: 0,
        });

        MetaContractResult {
            result: true,
            metadatas: finals,
            error_string: "".to_string(),
        }
      }
      Err(_) => {
        return MetaContractResult {
          result: false,
          metadatas: Vec::new(),
          error_string: "Unable to serialize content".to_string(),
        }
      }
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
                                  version: data.version,
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

/**
 * For now leaving it empty. Freedom of speech
 */
pub fn is_profane(text: &str) -> bool {
  let profane_words = vec!["", ""];
  profane_words.iter().any(|&word| {
    if word != "" {
      return text.contains(word)
    }
    false
  })
}

pub fn is_nft_storage_link(link: &str) -> bool {
  link == "" || link.starts_with("https://nftstorage.link/ipfs/")
}