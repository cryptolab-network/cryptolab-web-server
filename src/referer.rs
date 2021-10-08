use chrono::Utc;
use log::{error, info};
use rand::{Rng, distributions::Alphanumeric, thread_rng};

use crate::{db::params::DbRefKeyOptions};
#[derive(Debug)]
pub struct RefKeyError {
  repr: ErrorRepr,
}

#[derive(Debug)]
enum ErrorRepr {
  IncorrectRefKey(String),
}

pub fn gen_ref_key(stash: &str) -> String {
  let timestamp = Utc::now().timestamp();
  let rand_string: String = thread_rng()
    .sample_iter(&Alphanumeric)
    .take(30)
    .map(char::from)
    .collect();
  let ref_key = format!("{}|{}|{}", stash, timestamp, rand_string);
  info!("{}", ref_key);
  let encoded = bs58::encode(ref_key).into_string();
  info!("{:?}", encoded);
  encoded
}

pub fn decrypt_ref_key(ref_key: &str) -> Result<DbRefKeyOptions, RefKeyError> {
  let vec = bs58::decode(ref_key).into_vec();
  match vec {
      Ok(vec) => {
        let raw = std::str::from_utf8(&vec).unwrap();
        let tokens: Vec<&str> = raw.split('|').collect();
        let timestamp = str::parse::<u32>(tokens[1]);
        Ok(DbRefKeyOptions {
          stash: tokens[0].to_string(),
          ref_key: ref_key.to_string(),
          timestamp: timestamp.unwrap(),
          rand: tokens[2].to_string(),
        })
      },
      Err(err) => {
        error!("{}", err);
        Err(RefKeyError {
          repr: ErrorRepr::IncorrectRefKey(err.to_string())
        })
      },
  }
}