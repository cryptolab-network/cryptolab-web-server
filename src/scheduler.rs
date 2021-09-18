use std::{env, time::Duration};

use crate::{cache_redis::Cache, config::Config, db::Database};


pub fn cache_era_info(chain: &'static str) {
  tokio::spawn(async move {
    let mongo_ip = env::var("MONGO_IP_ADDR");
    let mongo_ip = mongo_ip.unwrap_or_else(|_| Config::current().db_address.parse().unwrap());
    let mut db: Database;
    if chain == "KSM" {
      db = Database::new(
        mongo_ip.clone(),
        Config::current().db_port,
        Config::current().kusama_db_name.as_str(),
      );
    } else if chain == "DOT" {
      db = Database::new(
        mongo_ip.clone(),
        Config::current().db_port,
        Config::current().polkadot_db_name.as_str(),
      );
    } else {
      db = Database::new(
        mongo_ip.clone(),
        Config::current().db_port,
        Config::current().westend_db_name.as_str(),
      );
    }
    let result = db.connect().await;
    if let Ok(()) = result {
      // create a cache client
      loop {
        // put data from db to cache
        let chain_info = db.get_chain_info().await;
        if let Ok(chain_info) = chain_info {
          let era = chain_info.active_era;
          let cache = Cache {};
          cache.cache_current_era(chain, era);
        }
        // sleep for 10 minutes
        tokio::time::sleep(Duration::from_secs(600)).await;
      }   
    }
  });
}