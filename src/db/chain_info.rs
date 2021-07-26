use log::error;
use mongodb::bson::{self, Bson};

use crate::types;

use super::{Database, DatabaseError};

impl Database {
  pub async fn get_chain_info(&self) -> Result<types::ChainInfo, DatabaseError> {
    match self.client.as_ref().ok_or(DatabaseError {
        message: "Mongodb client is not working as expected.".to_string(),
    }) {
        Ok(client) => {
            let db = client.database(&self.db_name);
            match db.collection("chainInfo").find_one(None, None).await {
                Ok(cursor) => Ok(bson::from_bson(Bson::Document(cursor.unwrap())).unwrap()),
                Err(e) => {
                    println!("{:?}", e);
                    Err(DatabaseError { message: "Get data from DB failed".to_string()}
                )},
            }
        }
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
  }
}