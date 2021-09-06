use std::fmt;
extern crate redis;
use redis::{Commands, RedisError};

use crate::{config::Config, types};

#[derive(Debug, Clone)]
pub struct CacheError {
    message: String,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cache error: {}", self.message)
    }
}

#[derive(Debug, Clone)]
pub struct Cache {
}

impl Cache {
  pub fn connect(&self) -> Result<redis::Connection, RedisError> {
    Config::init();
    let config = Config::current();
    let client = redis::Client::open(format!("redis://{}:{}/", config.redis, config.redis_port));
    match client {
        Ok(client) => {
          let con = client.get_connection()?;
          Ok(con)
        },
        Err(e) => {Err(e)},
    }
  }
  
  pub fn get_validators(&self, chain: &str) -> Vec<types::ValidatorInfo> {
    let result: Result<String, RedisError> = self.connect().unwrap().get(format!("{}validDetailAll", chain));
    match result {
        Ok(data) => {
          let json: Option<types::ValidatorDetailAll> =
            serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
          json.unwrap().valid
        },
        Err(_) => {vec![]},
    }
  }
  
  pub fn get_1kv_info_simple(&self, chain: &str) -> types::ValidatorDetail1kv {
    let result: Result<String, RedisError> = self.connect().unwrap().get(format!("{}onekv", chain));
    let json: Option<types::ValidatorDetail1kv> =
    serde_json::from_str(result.unwrap().as_str()).expect("JSON was not well-formatted");
    json.unwrap()
  }
  
  pub fn get_1kv_info_detail(&self, chain: &str) -> types::ValidatorDetail1kv {
    let result: Result<String, RedisError> = self.connect().unwrap().get(format!("{}onekv", chain));
    let timestamp_result: Result<String, RedisError> = self.connect().unwrap().get(format!("{}onekv_timestamp", chain));
    let json: Option<types::ValidatorDetail1kv> =
    serde_json::from_str(result.unwrap().as_str()).expect("JSON was not well-formatted");
    let mut data = json.unwrap();
    for mut v in data.valid.iter_mut() {
      let mut valid = true;
      for validity in &v.validity {
          if !validity.valid {
            valid = false;
          }
      }
      if valid {
        v.valid = Some(true);
      }
    }
    let modified_time = timestamp_result.unwrap_or_else(|_| "0".to_string()).parse::<u64>().ok();
    data.modified_time = modified_time;
    data
  }
  
  pub fn get_nominators(&self, chain: &str) -> Vec<types::NominatorNomination> {
    let result: Result<String, RedisError> = self.connect().unwrap().get(format!("{}nominators", chain));
    match result {
        Ok(data) => {
          let json: Option<Vec<types::NominatorNomination>> =
            serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
          json.unwrap()
        },
        Err(_) => {vec![]},
    }
  }
  
  pub fn get_nominator(&self, chain: &str, stash: String) -> Result<types::NominatorNomination, CacheError> {
      let nominators = self.get_nominators(chain);
      for nominator in nominators {
          if nominator.account_id == stash {
              return Ok(nominator)
          }
      }
      Err(CacheError {
          message: "Cannot find stash in nominator cache".to_string(),
      })
  }
  
  pub fn get_1kv_nominators(&self, chain: &str) -> types::OneKvNominators {
    let result: Result<String, RedisError> = self.connect().unwrap().get(format!("{}onekvNominators", chain));
    let json: Option<types::OneKvNominators> =
    serde_json::from_str(result.unwrap().as_str()).expect("JSON was not well-formatted");
    json.unwrap()
  
  }    
}
