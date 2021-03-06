#![recursion_limit="256"]
#[macro_use]
extern crate lazy_static;
mod cache;
mod config;
mod db;
mod types;
mod cache_redis;
// mod polkadot_cache;
mod web;
mod staking_rewards_collector;
mod scheduler;
mod referer;

use config::Config;
use db::Database;
use log::debug;
use web::{WebServer, WebServerOptions};
use std::{env};

use crate::{cache_redis::Cache, scheduler::cache_era_info};

#[tokio::main]
async fn main() {
    // env::set_var("RUST_LOG", "warp");
    env_logger::builder().filter_module("hyper", log::LevelFilter::Info).filter_level(log::LevelFilter::Debug).init();
    Config::init();
    let mongo_ip = env::var("MONGO_IP_ADDR");
    debug!("{:?}", mongo_ip);
    let mongo_ip = mongo_ip.unwrap_or_else(|_| Config::current().db_address.parse().unwrap());
    debug!("{}", mongo_ip);
    let mut kusama_db = Database::new(
        mongo_ip.clone(),
        Config::current().db_port,
        Config::current().kusama_db_name.as_str(),
    );
    let result = kusama_db.connect().await;
    match result {
        Ok(_) => {
            let mut polkadot_db = Database::new(
                mongo_ip.clone(),
                Config::current().db_port,
                Config::current().polkadot_db_name.as_str(),
            );
            let _ = polkadot_db.connect().await;
            let mut users_db = Database::new(
                mongo_ip.clone(),
                Config::current().db_port,
                Config::current().users_db_name.as_str(),
            );
            let _ = users_db.connect().await;
            if Config::current().support_westend {
                let mut westend_db = Database::new(
                    mongo_ip.clone(),
                    Config::current().db_port,
                    Config::current().westend_db_name.as_str(),
                );
                let _ = westend_db.connect().await;
                let options = WebServerOptions {
                    kusama_db,
                    polkadot_db,
                    users_db,
                    westend_db: Some(westend_db),
                    cache: Cache{},
                };
                cache_era_info("KSM");
                cache_era_info("DOT");
                cache_era_info("WND");
                let server = WebServer::new(Config::current().port, options);
                server.start().await;
            } else {
                let options = WebServerOptions {
                    kusama_db,
                    polkadot_db,
                    users_db,
                    westend_db: None,
                    cache: Cache{},
                };
                cache_era_info("KSM");
                cache_era_info("DOT");
                let server = WebServer::new(Config::current().port, options);
                server.start().await;
            }
        }
        Err(_) => panic!("Failed to connect to the Kusama Database"),
    }
}
