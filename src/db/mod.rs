use super::config::Config;
use super::types;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::StreamExt;
use log::error;
use mongodb::bson::{self, doc, Bson};
use mongodb::{options::ClientOptions, Client};
use std::fmt;
use std::{collections::HashMap, error::Error};
pub(crate) mod params;
mod nominator;
mod validator;

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

        // Get a handle to the deployment.
        self.client = Some(Client::with_options(client_options)?);
        Ok(())
    }

    pub async fn get_chain_info(&self) -> Result<types::ChainInfo, DatabaseError> {
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                match db.collection("chainInfo").find_one(None, None).await {
                    Ok(cursor) => Ok(bson::from_bson(Bson::Document(cursor.unwrap())).unwrap()),
                    Err(_) => Err(DatabaseError {
                        message: "Get data from DB failed".to_string(),
                    }),
                }
            }
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }

    fn get_price_from_cache(&self, timestamp: i64) -> Result<f64, DatabaseError> {
        if self.price_cache.contains_key(&timestamp) {
            return Ok(self.price_cache[&timestamp]);
        }
        Err(DatabaseError {
            message: "Cache missed".to_string(),
        })
    }

    async fn get_price_of_day(&self, timestamp: i64) -> Result<types::CoinPrice, DatabaseError> {
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection("price")
                    .find(doc! {"timestamp": timestamp}, None)
                    .await
                    .unwrap();
                if let Some(coin_price) = cursor.next().await {
                    let doc = coin_price.unwrap();
                    let price = doc.get("price").unwrap().as_f64().unwrap_or(0.0);
                    let timestamp = doc.get("timestamp").unwrap().as_i32().unwrap_or(0);
                    if timestamp == 0 {
                        let timestamp = doc.get("timestamp").unwrap().as_i64().unwrap_or(0);
                        let price = types::CoinPrice {
                            timestamp: timestamp as i64,
                            price,
                        };
                        return Ok(price);
                    } else {
                        let price = types::CoinPrice {
                            timestamp: timestamp as i64,
                            price,
                        };
                        return Ok(price);
                    }
                }
                Err(DatabaseError {
                    message: "Cannot get price".to_string(),
                })
            }
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn get_stash_reward(
        &mut self,
        stash: &str,
    ) -> Result<types::StashRewards, DatabaseError> {
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection("stashInfo")
                    .find(doc! {"stash": stash}, None)
                    .await
                    .unwrap();
                let mut total_in_fiat = 0.0;
                let mut era_rewards: Vec<types::StashEraReward> = vec![];
                while let Some(stash_reward) = cursor.next().await {
                    let doc = stash_reward.unwrap();
                    let era;
                    match doc.get("era").unwrap().as_i32() {
                        Some(_era) => era = _era,
                        None => continue,
                    }
                    let amount = doc.get("amount").unwrap().as_f64().unwrap_or(0.0);
                    let _timestamp = doc.get("timestamp").unwrap().as_i64();
                    let timestamp: i64;
                    let naive: NaiveDateTime;
                    match _timestamp {
                        Some(__timestamp) => {
                            naive =
                            NaiveDateTime::from_timestamp((__timestamp / 1000) as i64, 0);
                            timestamp = __timestamp;
                        },
                        None => {
                            let  timestamp_f64 = doc.get("timestamp").unwrap().as_f64().unwrap();
                            naive =
                            NaiveDateTime::from_timestamp((timestamp_f64.round() / 1000.0) as i64, 0);
                            timestamp = timestamp_f64.round() as i64;
                        },
                    }
                    // Create a normal DateTime from the NaiveDateTime
                    let datetime: DateTime<Utc> = DateTime::from_utc(naive, Utc);
                    let t = datetime.date().and_hms(0, 0, 0).timestamp();
                    let result = self.get_price_from_cache(t);
                    let mut price = 0.0;
                    match result {
                        Ok(_price) => {
                            price = _price;
                        }
                        Err(_) => {
                            let _price = self.get_price_of_day(t).await;
                            if let Ok(_price) = _price {
                                price = _price.price;
                                self.price_cache.insert(t, price);
                            }
                        }
                    }
                    total_in_fiat += price * amount;
                    era_rewards.push(types::StashEraReward {
                        era,
                        amount,
                        timestamp: (timestamp) as i64,
                        price,
                        total: price * amount,
                    })
                }
                Ok(types::StashRewards {
                    stash: stash.to_string(),
                    era_rewards,
                    total_in_fiat
                })
            }
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }
}
