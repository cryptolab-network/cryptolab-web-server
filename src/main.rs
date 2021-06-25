#[macro_use]
extern crate lazy_static;
mod cache;
mod config;
mod db;
mod types;
mod web;
mod staking_rewards_collector;

use config::Config;
use db::Database;
use env_logger;
use web::{WebServer, WebServerOptions};
use std::{env};

#[tokio::main]
async fn main() {
    // env::set_var("RUST_LOG", "warp");
    env_logger::init();
    Config::init();
    let mongo_ip = env::var("MONGO_IP_ADDR");
    println!("{:?}", mongo_ip);
    let mongo_ip = mongo_ip.unwrap_or(Config::current().db_address.parse().unwrap());
    println!("{}", mongo_ip);
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
            let options = WebServerOptions {
                kusama_db: kusama_db,
                polkadot_db: polkadot_db,
            };

            let server = WebServer::new(Config::current().port, options);
            server.start().await;
        }
        Err(_) => panic!("Failed to connect to the Kusama Database"),
    }
}
