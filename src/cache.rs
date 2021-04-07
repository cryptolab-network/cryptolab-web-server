use std::fs;
use super::types;

pub fn get_validators() -> Vec<types::ValidatorInfo> {
    let file_path =  "E:/git/validator/src/data/data.json";
    let data = fs::read_to_string(file_path).expect("Unable to read the cache file");
    let json: types::PolkadotApiValidators =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");

    json.valid_detail_all.unwrap().valid
}

pub fn get_1kv_info_simple() -> types::ValidatorDetail1kv {
    let file_path =  "E:/git/validator/src/data/data.json";
    let data = fs::read_to_string(file_path).expect("Unable to read the cache file");
    let json: types::PolkadotApiValidators =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.valid_detail_1kv.unwrap()
}

pub fn get_1kv_info_detail() -> types::Validator1kvSimple {
    let file_path =  "E:/git/validator/src/data/data.json";
    let data = fs::read_to_string(file_path).expect("Unable to read the cache file");
    let json: types::PolkadotApiValidators =
    serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
    
    json.valid.unwrap()
}