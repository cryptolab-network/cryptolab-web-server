use chrono::{DateTime, NaiveDateTime, Utc};
use futures::StreamExt;
use log::error;
use mongodb::bson::doc;
use crate::types;
use super::{Database, DatabaseError};

impl Database {
  
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
                let _timestamp = _timestamp.map_or_else(|| {
                    let timestamp_f64 = doc.get("timestamp").unwrap().as_f64().unwrap();
                    let naive =
                    NaiveDateTime::from_timestamp((timestamp_f64.round() / 1000.0) as i64, 0);
                    let timestamp = timestamp_f64.round() as i64;
                    (timestamp, naive)
                  },move |t| {
                    let naive =
                    NaiveDateTime::from_timestamp((t / 1000) as i64, 0);
                    let timestamp = t;
                    (timestamp, naive)
                  }
                );
                // Create a normal DateTime from the NaiveDateTime
                let datetime: DateTime<Utc> = DateTime::from_utc(_timestamp.1, Utc);
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
                    timestamp: (_timestamp.0) as i64,
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