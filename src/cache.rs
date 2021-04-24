use std::{fmt, fs, path::{Path}};
use super::types;
use super::config::Config;

#[derive(Debug, Clone)]
pub struct CacheError {
    message: String,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cache error: {}", self.message)
    }
}

pub fn get_validators() -> Vec<types::ValidatorInfo> {
    Config::init();
    let config = Config::current();
    let folder = config.new_cache_folder.as_str();
    let path = Path::new(folder).join("validDetailAll.json");
    let data = fs::read_to_string(path).expect("Unable to read the cache file");
    let json: Option<types::ValidatorDetailAll> =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");

    json.unwrap().valid
}

pub fn get_1kv_info_simple() -> types::ValidatorDetail1kv {
    Config::init();
    let config = Config::current();
    let folder = config.new_cache_folder.as_str();
    let path = Path::new(folder).join("onekv.json");
    let data = fs::read_to_string(path).expect("Unable to read the cache file");
    let json: Option<types::ValidatorDetail1kv> =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.unwrap()
}

pub fn get_1kv_info_detail() -> types::ValidatorDetail1kv {
    Config::init();
    let config = Config::current();
    let folder = config.new_cache_folder.as_str();
    let path = Path::new(folder).join("onekv.json");
    let data = fs::read_to_string(path).expect("Unable to read the cache file");
    let json: Option<types::ValidatorDetail1kv> =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    // form StakingInfo
    // element.commission = detail.stakingInfo.validatorPrefs.commission;
    //           element.stakeSize = detail.stakingInfo.stakingLedger.total;
    json.unwrap()
}

pub fn get_nominators() -> Vec<types::NominatorNomination> {
    Config::init();
    let config = Config::current();
    let folder = config.new_cache_folder.as_str();
    let path = Path::new(folder).join("nominators.json");
    let data = fs::read_to_string(path).expect("Unable to read the cache file");
    let json: Option<Vec<types::NominatorNomination>> =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.unwrap()
}

pub fn get_nominator(stash: String) -> Result<types::NominatorNomination, CacheError> {
    let nominators = get_nominators();
    for nominator in nominators {
        if nominator.account_id == stash {
            return Ok(nominator)
        }
    }
    Err(CacheError {
        message: "Cannot find stash in nominator cache".to_string(),
    })
}

pub fn get_1kv_nominators() -> types::OneKvNominators {
    Config::init();
    let config = Config::current();
    let folder = config.new_cache_folder.as_str();
    let path = Path::new(folder).join("onekvNominators.json");
    let data = fs::read_to_string(path).expect("Unable to read the cache file");
    let json: Option<types::OneKvNominators> =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.unwrap()
}