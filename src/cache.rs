use std::{fs, path::{Path}};
use super::types;
use super::config::Config;

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