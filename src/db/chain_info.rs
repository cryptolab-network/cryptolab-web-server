
use mongodb::bson::{self, Bson};

use crate::types;

use super::{Database, DatabaseError};

impl Database {
  pub async fn get_chain_info(&self) -> Result<types::ChainInfo, DatabaseError> {
    let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
    if let Ok(client) = client {
        let db = client.database(&self.db_name);
        match db.collection("chainInfo").find_one(None, None).await {
            Ok(cursor) => Ok(bson::from_bson(Bson::Document(cursor.unwrap())).unwrap()),
            Err(e) => {
                println!("{:?}", e);
                Err(DatabaseError::GetFailed)
            },
        }
    } else {
        Err(DatabaseError::Disconnected)
    }
  }
}