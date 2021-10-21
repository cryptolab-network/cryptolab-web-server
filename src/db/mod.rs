use super::config::Config;
use mongodb::options::{Tls, TlsOptions};
use mongodb::{options::ClientOptions, Client};
use std::fmt;
use std::path::PathBuf;
use std::{collections::HashMap};
pub(crate) mod params;
mod nominator;
mod validator;
mod chain_info;
mod staking_rewards;
mod user_actions;

#[derive(Debug)]
pub enum DatabaseError {
    Mongo(mongodb::error::Error),
    GetFailed,
    Disconnected,
    CacheMissed,
    WriteFailed,
    Duplicated
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Database error: {:?}", self)
    }
}

impl std::error::Error for DatabaseError {
    
}

impl From<mongodb::error::Error> for DatabaseError {
    fn from(err: mongodb::error::Error) -> DatabaseError {
        DatabaseError::Mongo(err)
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

    pub async fn connect(&mut self) -> Result<(), DatabaseError> {
        let need_credential = Config::current().db_has_credential;
        let mut url = "mongodb://".to_string();
        if need_credential {
            if let Some(username) = Config::current().db_username.to_owned() {
                if let Some(password) = Config::current().db_password.to_owned() {
                    url += format!("{}:{}@", username, password).as_str();
                }
            }
            url += format!("{}:{}/{}?authSource=admin", self.ip, self.port, self.db_name).as_str();
        } else {
            url += format!("{}:{}/{}", self.ip, self.port, self.db_name).as_str();
        }
        let client_options = ClientOptions::parse(url.as_str()).await.map_err(DatabaseError::Mongo);
        match client_options {
            Ok(mut client_options) => {
                // Manually set an option.
                client_options.app_name = Some("cryptolab".to_string());
                client_options.retry_writes = Some(false);
                let mut ca_file_path: Option<PathBuf> = None;
                if Config::current().db_ca_file.clone().is_some() {
                    ca_file_path = Some(PathBuf::from(Config::current().db_ca_file.clone().unwrap()));
                }
                let mut db_cert_file_path: Option<PathBuf> = None;
                if Config::current().db_cert_key_file.clone().is_some() {
                    db_cert_file_path = Some(PathBuf::from(Config::current().db_cert_key_file.clone().unwrap()));
                }
                if Config::current().db_has_tls {
                    let tls_options = TlsOptions::builder()
                        .ca_file_path(ca_file_path)
                        .cert_key_file_path(db_cert_file_path)
                        .allow_invalid_certificates(true)
                        .build();
                    client_options.tls = Some(Tls::Enabled(tls_options));
                }
                // Get a handle to the deployment.
                let o = Client::with_options(client_options).map_err(|e| e.to_string());
                self.client = Some(o.unwrap());
                Ok(())
            },
            Err(e) => {
                Err(e)
            },
        }
    }
}
