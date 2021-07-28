use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, RwLock};
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub db_address: String,
    pub db_port: u16,
    pub kusama_db_name: String,
    pub polkadot_db_name: String,
    pub port: u16,
    pub cors_url: Vec<String>,
    pub db_has_credential: bool,
    pub db_username: Option<String>,
    pub db_password: Option<String>,
    pub db_has_tls: bool,
    pub db_ca_file: Option<String>,
    pub db_cert_key_file: Option<String>,

    pub new_cache_folder: String,
    pub new_cache_folder_polkadot: String,

    pub redis: String,
    pub redis_port: u16,

    pub staking_rewards_collector_dir: String,
    pub serve_www: Option<bool>,
}

impl Config {
    pub fn init() {
        let config = read_config("./config/config.json".to_string()).unwrap();
        config.make_current();
    }
    pub fn current() -> Arc<Config> {
        CURRENT_CONFIG.with(|c| c.read().unwrap().clone())
    }
    pub fn make_current(self) {
        CURRENT_CONFIG.with(|c| *c.write().unwrap() = Arc::new(self))
    }
}

thread_local! {
    static CURRENT_CONFIG: RwLock<Arc<Config>> = RwLock::new(Default::default());
}

fn read_config(path: String) -> Result<Config, Box<dyn Error>> {
    let file = File::open(path);
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let config: Config = serde_json::from_reader(reader)?;
            let config = read_env(config);
            println!("{:?}", config);
            Ok(config)
        }
        Err(e) => Err(Box::new(e)),
    }
}

fn read_env(mut config: Config) -> Config {
    config.kusama_db_name = env::var("KUSAMA_DB_NAME").unwrap_or(config.kusama_db_name);
    config.polkadot_db_name = env::var("POLKADOT_DB_NAME").unwrap_or(config.polkadot_db_name);
    config.port =
        str::parse::<u16>(&env::var("PORT").unwrap_or_else(|_| config.port.to_string())).unwrap();
    config.redis = env::var("REDIS").unwrap_or(config.redis);
    let redis_port = str::parse::<u16>(
        &env::var("REDIS_PORT").unwrap_or_else(|_| config.redis_port.to_string()),
    );
    config.redis_port = redis_port.unwrap();
    let serve_www = str::parse::<bool>(
        &env::var("SERVE_WWW").unwrap_or_else(|_| config.serve_www.unwrap_or(false).to_string()),
    );
    config.serve_www = Some(serve_www.unwrap());
    config.staking_rewards_collector_dir =
        env::var("STAKING_REWARDS_COLLECTOR_DIR").unwrap_or(config.staking_rewards_collector_dir);
    let cors_url = serde_json::from_str(
        &env::var("CORS_URL").unwrap_or_else(|_| serde_json::to_string(&config.cors_url).unwrap()),
    );
    config.cors_url = cors_url.unwrap();
    config.db_address = env::var("DB_ADDRESS").unwrap_or(config.db_address);
    let db_has_credential = str::parse::<bool>(
        &env::var("DB_HAS_CREDENTIAL").unwrap_or_else(|_| config.db_has_credential.to_string()),
    );
    config.db_has_credential = db_has_credential.unwrap();
    config.db_password =
        Some(env::var("DB_PASSWORD").unwrap_or_else(|_| config.db_password.clone().unwrap()));
    config.db_port =
        str::parse::<u16>(&env::var("DB_PORT").unwrap_or_else(|_| config.db_port.to_string()))
            .unwrap();
    config.db_username =
        Some(env::var("DB_USERNAME").unwrap_or_else(|_| config.db_username.clone().unwrap()));
    let db_has_tls = str::parse::<bool>(
        &env::var("DB_HAS_TLS").unwrap_or_else(|_| config.db_has_tls.to_string()),
    );
    config.db_has_tls = db_has_tls.unwrap();
    let db_cert_key_file = Some(
        env::var("DB_CERT_KEY_FILE").unwrap_or_else(|_| config.db_cert_key_file.clone().unwrap()),
    );
    if db_cert_key_file.is_some() && db_cert_key_file.unwrap().is_empty() {
        config.db_cert_key_file = None;
    } else {
        config.db_cert_key_file = Some(
            env::var("DB_CERT_KEY_FILE").unwrap_or_else(|_| config.db_cert_key_file.clone().unwrap()),
        );
    }
    config.db_ca_file =
        Some(env::var("DB_CA_FILE").unwrap_or_else(|_| config.db_ca_file.clone().unwrap()));
    config
}
