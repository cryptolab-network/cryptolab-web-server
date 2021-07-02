use super::config::Config;
use super::types;
use std::{fmt, fs, path::Path, time::UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct CacheError {
    message: String,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cache error: {}", self.message)
    }
}

// fn get_folder(chain: &str) -> String {
//     Config::init();
//     let config = Config::current();
//     if chain == "KSM" {
//         config.new_cache_folder.clone()
//     } else {
//         config.new_cache_folder_polkadot.clone()
//     }
// }

// pub fn get_validators(chain: &str) -> Vec<types::ValidatorInfo> {
//     let path = Path::new(get_folder(chain).as_str()).join("validDetailAll.json");
//     let data = fs::read_to_string(path).expect("Unable to read the cache file");
//     let json: Option<types::ValidatorDetailAll> =
//         serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");

//     json.unwrap().valid
// }

// pub fn get_1kv_info_simple(chain: &str) -> types::ValidatorDetail1kv {
//     let path = Path::new(get_folder(chain).as_str()).join("onekv.json");
//     let data = fs::read_to_string(path).expect("Unable to read the cache file");
//     let json: Option<types::ValidatorDetail1kv> =
//         serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");

//     json.unwrap()
// }

// pub fn get_1kv_info_detail(chain: &str) -> types::ValidatorDetail1kv {
//     let path = Path::new(get_folder(chain).as_str()).join("onekv.json");
//     let data = fs::read_to_string(path.clone()).expect("Unable to read the cache file");
//     let metadata = fs::metadata(path.clone());
//     let json_option: Option<types::ValidatorDetail1kv> =
//         serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
//     let mut json = json_option.unwrap();
//     if let Ok(metadata) = metadata {
//         if let Ok(modified_time) = metadata.modified() {
//             let timestamp = modified_time
//             .duration_since(UNIX_EPOCH)
//             .expect("Time went backwards").as_secs();
//             json.modified_time = Some(timestamp);
//         }
//     }
//     json
// }

// pub fn get_nominators(chain: &str) -> Vec<types::NominatorNomination> {
//     let path = Path::new(get_folder(chain).as_str()).join("nominators.json");
//     let data = fs::read_to_string(path).expect("Unable to read the cache file");
//     let json: Option<Vec<types::NominatorNomination>> =
//         serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
//         json.unwrap()
// }

// pub fn get_nominator(chain: &str, stash: String) -> Result<types::NominatorNomination, CacheError> {
//     let nominators = get_nominators(chain);
//     for nominator in nominators {
//         if nominator.account_id == stash {
//             return Ok(nominator);
//         }
//     }
//     Err(CacheError {
//         message: "Cannot find stash in nominator cache".to_string(),
//     })
// }

// pub fn get_1kv_nominators(chain: &str) -> types::OneKvNominators {
//     let path = Path::new(get_folder(chain).as_str()).join("onekvNominators.json");
//     let data = fs::read_to_string(path).expect("Unable to read the cache file");
//     let json: Option<types::OneKvNominators> =
//         serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");

//     json.unwrap()
// }
