use serde::{Deserialize};
use mongodb::bson::{doc};
use rand::{Rng, thread_rng};

use crate::types::{NewsletterSubscriberOptions, NominationOptions, NominationResultOptions};

use super::{Database, DatabaseError};

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

impl Database {
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