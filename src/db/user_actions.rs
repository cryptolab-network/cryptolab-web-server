use log::error;
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
    pub extrinsic_hash: String,
}

impl Database {
  pub async fn insert_nomination_action(&self, options: NominationOptions) -> Result<String, DatabaseError> {
    match self.client.as_ref().ok_or(DatabaseError {
        message: "Mongodb client is not working as expected.".to_string(),
    }) {
        Ok(client) => {
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
            }, None).await {
                Ok(_) => Ok(rand_string),
                Err(e) => {
                    println!("{:?}", e);
                    Err(DatabaseError { message: "Failed to write nomination record to db".to_string()}
                )},
            }
        }
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
  }

  pub async fn insert_nomination_result(&self, options: NominationResultOptions) -> Result<(), DatabaseError> {
    match self.client.as_ref().ok_or(DatabaseError {
        message: "Mongodb client is not working as expected.".to_string(),
    }) {
        Ok(client) => {
            let db = client.database(&self.db_name);
            match db.collection::<NominationRecords>("nominationRecords").find_one_and_update(doc! {
              "tag": &options.tag,
            }, doc! {
              "$set": {
                "extrinsicHash": options.extrinsic_hash,
              }
            }, None).await {
                Ok(m) => {
                  println!("{:?}", m);
                  Ok(())
                },
                Err(e) => {
                    println!("{:?}", e);
                    Err(DatabaseError { message: "Failed to write nomination record to db".to_string()}
                )},
            }
        }
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
  }

  pub async fn insert_newsletter_subsriber(&self, options: NewsletterSubscriberOptions) -> Result<(), DatabaseError> {
    match self.client.as_ref().ok_or(DatabaseError {
      message: "Mongodb client is not working as expected.".to_string(),
  }) {
      Ok(client) => {
          let db = client.database(&self.db_name);
          match db.collection::<NewsletterSubscriberOptions>("newsletter").find_one(doc! {"email": &options.email}, None).await {
            Ok(c) => {
              match c {
                Some(_) => {
                  Err(DatabaseError { message: "This email has already registered.".to_string()})
                },
                None => {
                  match db.collection("newsletter").insert_one(doc! {
                    "email": options.email,
                    "timestamp": chrono::Utc::now().naive_utc().timestamp(),
                  }, None).await {
                      Ok(_) => Ok(()),
                      Err(e) => {
                        println!("{:?}", e);
                        Err(DatabaseError { message: "Failed to write nomination record to db".to_string()}
                      )},
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
                      Err(DatabaseError { message: "Failed to write nomination record to db".to_string()}
                  )},
              }
            }
          }
      }
      Err(e) => {
          error!("{}", e);
          Err(e)
      }
    }
  }
}