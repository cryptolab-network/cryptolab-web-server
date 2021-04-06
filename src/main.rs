mod web;
mod db;
mod types;
mod cache;

use std::net::Ipv4Addr;
use web::WebServer;
use db::Database;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut db = Database::new(Ipv4Addr::new(127, 0, 0, 1), 27017, "kusama");
    db.connect().await;

    let server = WebServer::new(3030, db);
    server.start().await;
}
