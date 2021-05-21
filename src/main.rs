mod cache;
mod config;
mod db;
mod types;
mod cache_redis;
mod config;
mod polkadot_cache;
mod web;

use config::Config;
use db::Database;
use env_logger;
use web::{WebServer, WebServerOptions};

#[tokio::main]
async fn main() {
    // env::set_var("RUST_LOG", "warp");
    env_logger::init();
    Config::init();
    let mut kusama_db = Database::new(
        Config::current().db_address.parse().unwrap(),
        Config::current().db_port,
        Config::current().kusama_db_name.as_str(),
    );
    let result = kusama_db.connect().await;
    match result {
        Ok(_) => {
            let mut polkadot_db = Database::new(
                Config::current().db_address.parse().unwrap(),
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
