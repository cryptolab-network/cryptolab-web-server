use super::config::Config;
use mongodb::options::{Tls, TlsOptions, TlsOptionsBuilder};
use mongodb::{options::ClientOptions, Client};
use std::fmt;
use std::{collections::HashMap, error::Error};
pub(crate) mod params;
mod nominator;
mod validator;
mod chain_info;
mod staking_rewards;

// Define our error types. These may be customized for our error handling cases.
// Now we will be able to write our own errors, defer to an underlying error
// implementation, or do something in between.
#[derive(Debug, Clone)]
pub struct DatabaseError {
    message: String,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Database error: {}", self.message)
    }
}

#[derive(Debug, Clone)]
pub struct Database {
    ip: String,
    port: u16,
    db_name: String,
    client: Option<Client>,
    price_cache: HashMap<i64, f64>,
}

impl Database {
    pub fn new(ip: String, port: u16, db_name: &str) -> Self {
        Database {
            ip,
            port,
            db_name: db_name.to_string(),
            client: None,
            price_cache: HashMap::new(),
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let need_credential = Config::current().db_has_credential;
        let mut url = "mongodb://".to_string();
        if need_credential {
            if let Some(username) = Config::current().db_username.to_owned() {
                if let Some(password) = Config::current().db_password.to_owned() {
                    url += format!("{}:{}@", username, password).as_str();
                }
            }
        }
        url += format!("{}:{}/{}", self.ip, self.port, self.db_name).as_str();
        let mut client_options = ClientOptions::parse(url.as_str()).await?;
        // Manually set an option.
        client_options.app_name = Some("cryptolab".to_string());
        if Config::current().db_has_tls {
            let tls_options = TlsOptions::builder()
                .ca_file_path(Config::current().db_ca_file.clone())
                .cert_key_file_path(Config::current().db_cert_key_file.clone())
                .allow_invalid_certificates(true)
                .build();
            client_options.tls = Some(Tls::Enabled(tls_options));
        }
        // Get a handle to the deployment.
        self.client = Some(Client::with_options(client_options)?);
        Ok(())
    }
}
