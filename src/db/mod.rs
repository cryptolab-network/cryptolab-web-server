
use std::net::Ipv4Addr;
use std::error::Error;
use mongodb::{Client, options::ClientOptions};
use super::types;

pub struct Database {
    ip: Ipv4Addr,
    port: u16,
    db_name: String
}

impl Database {
    pub fn new(ip: Ipv4Addr, port: u16, db_name: &str) -> Self {
        Database {
            ip: ip,
            port: port,
            db_name: db_name.to_string()
        }
    }

    pub async fn connect(&self) -> Result<(), Box<dyn Error>> {
        // Parse a connection string into an options struct.
        let url = format!("mongodb://{}:{}/{}", self.ip, self.port, self.db_name);
        let mut client_options = ClientOptions::parse(url.as_str()).await?;
        // Manually set an option.
        client_options.app_name = Some("cryptolab".to_string());

        // Get a handle to the deployment.
        let client = Client::with_options(client_options)?;
        Ok(())
    }

    // pub fn async get_all_validator_info_of_era(&self, era: u32) -> types::ValidatorInfo {

    // }
}