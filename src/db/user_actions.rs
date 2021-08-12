use log::error;
use mongodb::bson::{doc};

use crate::types::{NewsletterSubscriberOptions, NominationOptions};

use super::{Database, DatabaseError};

impl Database {
  pub async fn insert_nomination_action(&self, options: NominationOptions) -> Result<(), DatabaseError> {
    match self.client.as_ref().ok_or(DatabaseError {
        message: "Mongodb client is not working as expected.".to_string(),
    }) {
        Ok(client) => {
            let db = client.database(&self.db_name);
            match db.collection("nominationRecords").insert_one(doc! {
              "stash": options.stash,
              "validators": options.validators,
              "amount": options.amount.to_string(),
              "strategy": options.strategy as u32,
            }, None).await {
                Ok(_) => Ok(()),
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
                      Err(_) => {
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