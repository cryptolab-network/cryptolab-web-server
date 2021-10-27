use futures::StreamExt;
use log::info;
use serde::{Deserialize};
use mongodb::bson::{self, Bson, Document, doc};
use rand::{Rng, thread_rng};

use crate::{referer, types::{CBStashEraReward,ValidatorStalePayoutEvent, ChillEvent, KickEvent, NewsletterSubscriberOptions, NominationOptions, NominationResultOptions, OverSubscribeEventOutput, StakingEvents, UserEventMapping, UserEventMappingOptions, ValidatorCommission, ValidatorSlash}};

use super::{Database, DatabaseError, params::DbRefKeyOptions};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct NominationRecords {
    pub stash: String,
    pub validators: Vec<String>,
    pub amount: String,
    pub strategy: u32,
    pub tag: String,
    pub extrinsic_hash: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RefKeyRecords {
    pub stash: String,
    pub ref_key: String,
    pub timestamp: u32,
}

impl Database {
  pub async fn get_user_events_by_mapping(&self, options: UserEventMappingOptions) -> Result<StakingEvents, DatabaseError> {
    let mut array0: Vec<UserEventMapping> = Vec::new();
    let mut payouts: Vec<CBStashEraReward> = Vec::new();
    let mut inactive: Vec<u32> = Vec::new();
    let mut stale_payouts: Vec<ValidatorStalePayoutEvent> = Vec::new();
    let mut kicks: Vec<KickEvent> = Vec::new();
    let mut chills: Vec<ChillEvent> = Vec::new();
    let mut over_subscribes: Vec<OverSubscribeEventOutput> = Vec::new();
    let mut commissions: Vec<ValidatorCommission> = Vec::new();
    let mut slashes: Vec<ValidatorSlash> = Vec::new();
    let match_command = doc! {
      "$match":{
        "$and": [
          {
            "address": options.stash
          }, {
            "era": {
              "$gte": options.from_era,
              "$lte": options.to_era
            }
          }, {
            "type": {
              "$in": &options.event_types
            }
          }
        ]
      }
    };
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
      let db = client.database(&self.db_name);
      let mut cursor = db
          .collection::<Document>("userEventMapping")
          .aggregate(
              vec![
                  match_command,
              ],
              None,
          )
          .await
          .unwrap();
      while let Some(result) = cursor.next().await {
          let doc = result.unwrap();
          let em: UserEventMapping = bson::from_bson(Bson::Document(doc)).unwrap();
          if em.event_type == 0 {
            array0.push(em);
          }
      }
      let mut array_payouts = Vec::new();
      for ele in array0 {
        array_payouts.push(ele.mapping);
      }
      let mut cursor = db
        .collection::<Document>("stashInfo")
        .find(doc! {"_id": {"$in": &array_payouts}}, None)
        .await
        .unwrap();
        while let Some(result) = cursor.next().await {
          let doc = result.unwrap();
          let payout = bson::from_bson(Bson::Document(doc)).unwrap();
          payouts.push(payout);
        }
      Ok(StakingEvents {
        commissions,
        slashes,
        inactive,
        stale_payouts,
        payouts,
        kicks,
        chills,
        over_subscribes,
      })
    } else {
        Err(DatabaseError::Disconnected)
    }
  }

  pub async fn insert_nomination_action(&self, chain: String, options: NominationOptions) -> Result<String, DatabaseError> {
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
      let rand_string: String = thread_rng()
      .sample_iter(&rand::distributions::Alphanumeric)
      .take(16)
      .map(char::from)
      .collect();
        let db = client.database(&self.db_name);
        match db.collection("nominationRecords").insert_one(doc! {
          "stash": options.stash,
          "validators": options.validators,
          "amount": options.amount.to_string(),
          "strategy": options.strategy as u32,
          "tag": &rand_string,
          "chain": chain,
        }, None).await {
            Ok(_) => Ok(rand_string),
            Err(e) => {
                println!("{:?}", e);
                Err(DatabaseError::WriteFailed)
            },
        }
    } else {
        Err(DatabaseError::Disconnected)
    }
  }

  pub async fn insert_nomination_result(&self, options: NominationResultOptions) -> Result<(), DatabaseError> {
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
      let db = client.database(&self.db_name);
      match db.collection::<NominationRecords>("nominationRecords").find_one_and_update(doc! {
        "tag": &options.tag,
      }, doc! {
        "$set": {
          "extrinsicHash": options.extrinsic_hash,
        }
      }, None).await {
          Ok(m) => {
            if m.is_none() {
              return Err(DatabaseError::GetFailed);
            }
            Ok(())
          },
          Err(e) => {
              println!("{:?}", e);
              Err(DatabaseError::WriteFailed)
          },
      }
    } else {
        Err(DatabaseError::Disconnected)
    }
  }

  pub async fn get_nomination_records(&self, stash: &str) -> Result<NominationRecords, DatabaseError> {
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
      let db = client.database(&self.db_name);
      match db.collection::<NominationRecords>("nominationRecords").find_one(doc! {"stash": stash}, None).await {
        Ok(c) => {
          match c {
            Some(c) => {
              Ok(c)
            },
            None => {
              Err(DatabaseError::GetFailed)
            }
          }
        },
        Err(_) => {
          Err(DatabaseError::GetFailed)
        }
      }
    } else {
        Err(DatabaseError::Disconnected)
    }
  }

  pub async fn get_validator_ref_key(&self, stash: &str) -> Result<String, DatabaseError> {
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
      let db = client.database(&self.db_name);
      match db.collection::<RefKeyRecords>("refKeyRecords").find_one(doc! {"stash": stash}, None).await {
        Ok(c) => {
          info!("{:?}", c);
          match c {
            Some(c) => {
              Ok(c.ref_key)
            },
            None => {
              Err(DatabaseError::GetFailed)
            }
          }
        },
        Err(_) => {
          Err(DatabaseError::GetFailed)
        }
      }
    } else {
        Err(DatabaseError::Disconnected)
    }
  }

  pub async fn decode_validator_ref_key(&self, ref_key: &str) -> Result<DbRefKeyOptions, DatabaseError> {
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
      let db = client.database(&self.db_name);
      match db.collection::<RefKeyRecords>("refKeyRecords").find_one(doc! {"refKey": ref_key}, None).await {
        Ok(c) => {
          info!("{:?}", c);
          match c {
            Some(c) => {
              let decoded = referer::decrypt_ref_key(&c.ref_key).unwrap();
              Ok(decoded)
            },
            None => {
              Err(DatabaseError::GetFailed)
            }
          }
        },
        Err(_) => {
          Err(DatabaseError::GetFailed)
        }
      }
    } else {
        Err(DatabaseError::Disconnected)
    }
  }

  pub async fn insert_validator_ref_key(&self, options: DbRefKeyOptions) -> Result<(), DatabaseError> {
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
      let db = client.database(&self.db_name);
      match db.collection::<RefKeyRecords>("refKeyRecords").find_one(doc! {"stash": &options.stash}, None).await {
        Ok(c) => {
          match c {
            Some(_) => {
              let result = db.collection::<RefKeyRecords>("refKeyRecords").update_one(doc! {
                "stash": options.stash,
              }, doc! {
                "$set": {
                  "refKey": options.ref_key,
                  "timestamp": options.timestamp,
                }
              }, None).await;
              match result {
                  Ok(_) => {Ok(())},
                  Err(_) => {
                    Err(DatabaseError::WriteFailed)
                  },
              }
            },
            None => {
              match db.collection("refKeyRecords").insert_one(doc! {
                "stash": options.stash,
                "timestamp": options.timestamp,
                "refKey": options.ref_key,
              }, None).await {
                  Ok(_) => Ok(()),
                  Err(e) => {
                    println!("{:?}", e);
                    Err(DatabaseError::WriteFailed)
                  },
              }
            }
          }
          
        },
        Err(_) => {
          Err(DatabaseError::WriteFailed)
        }
      }
    } else {
        Err(DatabaseError::Disconnected)
    }
  }

  pub async fn insert_newsletter_subsriber(&self, options: NewsletterSubscriberOptions) -> Result<(), DatabaseError> {
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
      let db = client.database(&self.db_name);
      match db.collection::<NewsletterSubscriberOptions>("newsletter").find_one(doc! {"email": &options.email}, None).await {
        Ok(c) => {
          match c {
            Some(_) => {
              Err(DatabaseError::Duplicated)
            },
            None => {
              match db.collection("newsletter").insert_one(doc! {
                "email": options.email,
                "timestamp": chrono::Utc::now().naive_utc().timestamp(),
              }, None).await {
                  Ok(_) => Ok(()),
                  Err(e) => {
                    println!("{:?}", e);
                    Err(DatabaseError::WriteFailed)
                  },
              }
            }
          }
          
        },
        Err(_) => {
          match db.collection("newsletter").insert_one(doc! {
            "email": options.email,
            "timestamp": chrono::Utc::now().naive_utc().timestamp(),
          }, None).await {
              Ok(_) => Ok(()),
              Err(e) => {
                  println!("{:?}", e);
                  Err(DatabaseError::WriteFailed)
              },
          }
        }
      }
    } else {
        Err(DatabaseError::Disconnected)
    }
  }
}