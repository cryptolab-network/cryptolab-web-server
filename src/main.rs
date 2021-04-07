mod web;
mod db;
mod types;
mod cache;

use std::net::Ipv4Addr;
use web::{WebServer, WebServerOptions};
use db::Database;
use env_logger;

#[tokio::main]
async fn main() {
    // env::set_var("RUST_LOG", "warp");
    env_logger::init();

    let mut kusama_db = Database::new(Ipv4Addr::new(127, 0, 0, 1), 27017, "kusama");
    let result = kusama_db.connect().await;
    match result {
        Ok(r) => {
            let mut polkadot_db = Database::new(Ipv4Addr::new(127, 0, 0, 1), 27017, "polkadot");
            polkadot_db.connect().await;
            let options = WebServerOptions {
                kusama_db: kusama_db,
                polkadot_db: polkadot_db,
            };
        
            let server = WebServer::new(3030, options);
            server.start().await;
        }
        Err(e) => panic!("Failed to connect to the Kusama Database")
    }
}
