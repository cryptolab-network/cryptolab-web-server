use std::fs;
use super::types;
use super::config::Config;

pub fn get_validators() -> Vec<types::ValidatorInfo> {
    Config::init();
    let data = fs::read_to_string(Config::current().cache_file_path.as_str()).expect("Unable to read the cache file");
    let json: types::PolkadotApiValidators =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");

    json.valid_detail_all.unwrap().valid
}

pub fn get_1kv_info_simple() -> types::ValidatorDetail1kv {
    Config::init();
    let data = fs::read_to_string(Config::current().cache_file_path.as_str()).expect("Unable to read the cache file");
    let json: types::PolkadotApiValidators =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.valid_detail_1kv.unwrap()
}

pub fn get_1kv_info_detail() -> types::Validator1kvSimple {
    Config::init();
    let data = fs::read_to_string(Config::current().cache_file_path.as_str()).expect("Unable to read the cache file");
    let json: types::PolkadotApiValidators =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.valid.unwrap()
}

pub fn get_nominators() -> Vec<types::NominatorNomination> {
    Config::init();
    let data = fs::read_to_string(Config::current().cache_file_path.as_str()).expect("Unable to read the cache file");
    let json: types::PolkadotApiValidators =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.nominators.unwrap()
}

pub fn get_1kv_nominators() -> types::OneKvNominators {
    Config::init();
    let data = fs::read_to_string(Config::current().cache_file_path.as_str()).expect("Unable to read the cache file");
    let json: types::PolkadotApiValidators =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.one_kv_nominators.unwrap()
}