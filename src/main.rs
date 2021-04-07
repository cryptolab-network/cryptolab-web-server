mod web;
mod db;
mod types;
mod cache;
mod config;

use std::error::Error;
use std::net::Ipv4Addr;
use std::fs::File;
use std::io::BufReader;
use config::Config;
use web::{WebServer, WebServerOptions};
use db::Database;
use env_logger;

#[tokio::main]
async fn main() {
    // env::set_var("RUST_LOG", "warp");
    env_logger::init();

    let config = read_config("./config/config.json".to_string()).unwrap();
    config.make_current();

    let mut kusama_db = Database::new(Config::current().db_address.parse().unwrap(),
        Config::current().db_port, Config::current().kusama_db_name.as_str());
    let result = kusama_db.connect().await;
    match result {
        Ok(r) => {
            let mut polkadot_db = Database::new(Config::current().db_address.parse().unwrap(),
                Config::current().db_port, Config::current().polkadot_db_name.as_str());
            polkadot_db.connect().await;
            let options = WebServerOptions {
                kusama_db: kusama_db,
                polkadot_db: polkadot_db,
            };
        
            let server = WebServer::new(Config::current().port, options);
            server.start().await;
        }
        Err(e) => panic!("Failed to connect to the Kusama Database")
    }
}

fn read_config(path: String) -> Result<Config, Box<Error>> {
    let file = File::open(path);
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let config: Config = serde_json::from_reader(reader)?;
            Ok(config)
        }
        Err(e) => Err(Box::new(e))
    }
}
