use std::fmt;

use redis::{Commands, RedisError};

use crate::{config::Config, types};

extern crate redis;

#[derive(Debug, Clone)]
pub struct CacheError {
    message: String,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Cache error: {}", self.message)
    }
}

fn connect() -> Result<redis::Connection, RedisError> {
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

pub fn get_validators() -> Vec<types::ValidatorInfo> {
  let result: Result<String, RedisError> = connect().unwrap().get("DOTvalidDetailAll");
  match result {
      Ok(data) => {
        let json: Option<types::ValidatorDetailAll> =
          serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
        json.unwrap().valid
      },
      Err(e) => {vec![]},
  }
}

pub fn get_1kv_info_simple() -> types::ValidatorDetail1kv {
  let result: Result<String, RedisError> = connect().unwrap().get("DOTonekv");
  let json: Option<types::ValidatorDetail1kv> =
  serde_json::from_str(result.unwrap().as_str()).expect("JSON was not well-formatted");
  json.unwrap()
}

pub fn get_1kv_info_detail() -> types::ValidatorDetail1kv {
  let result: Result<String, RedisError> = connect().unwrap().get("DOTonekv");
  let json: Option<types::ValidatorDetail1kv> =
  serde_json::from_str(result.unwrap().as_str()).expect("JSON was not well-formatted");
  json.unwrap()
}

pub fn get_nominators() -> Vec<types::NominatorNomination> {
  let result: Result<String, RedisError> = connect().unwrap().get("DOTnominators");
  match result {
      Ok(data) => {
        let json: Option<Vec<types::NominatorNomination>> =
          serde_json::from_str(data.as_str()).expect("JSON was not well-formatted");
        json.unwrap()
      },
      Err(e) => {vec![]},
  }
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
  let result: Result<String, RedisError> = connect().unwrap().get("DOTonekvNominators");
  let json: Option<types::OneKvNominators> =
  serde_json::from_str(result.unwrap().as_str()).expect("JSON was not well-formatted");
  json.unwrap()

}