use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Config {
    pub db_address: String,
    pub db_port: u16,
    pub kusama_db_name: String,
    pub polkadot_db_name: String,
    pub port: u16,
    pub cache_file_path: String,
    pub cors_url: String,
}

impl Config {
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
