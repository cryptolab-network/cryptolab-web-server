use futures::StreamExt;
use mongodb::bson::{self, Document, doc};

use crate::{db::params::Inactive, types::NominatorNomination};

use super::{Database, DatabaseError};

impl Database {
    pub async fn get_all_validators_inactive(
    &mut self,
    stash: &str,
    from: u32,
    to: u32,
    ) -> Result<Vec<u32>, DatabaseError> {
        let match_command = doc! {
            "$match":{
                "$and": [
                    {
                        "address": stash
                    }, {
                        "era": {
                            "$gte": from,
                            "$lte": to
                        }
                    }
                ]
            },
            
        };
        let mut eras = Vec::<u32>::new();
        let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
        if let Ok(client) = client {
            let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection::<Document>("inactiveEvents")
                    .aggregate(vec![match_command], None)
                    .await
                    .unwrap();
                while let Some(result) = cursor.next().await {
                    let doc = result.unwrap();
                    let n: Inactive = bson::from_document(doc).unwrap();
                    eras.push(n.era);
                }
                Ok(eras)
        } else {
            Err(DatabaseError::Disconnected)
        }
    }

  pub async fn get_nominator_info(
    &mut self,
    stash: &str,
    ) -> Result<NominatorNomination, DatabaseError> {
        match self.get_stash_reward(stash).await {
            Ok(rewards) => {
                match self.do_get_nominator_info(stash).await {
                    Ok(mut n) => {
                        n.rewards = Some(rewards);
                        Ok(n)
                    },
                    Err(e) => {Err(e)},
                }
            },
            Err(e) => {
                Err(e)
            }
        }
    }

    async fn do_get_nominator_info(&self, stash: &str)
        -> Result<NominatorNomination, DatabaseError> {
        let match_command = doc! {
            "$match":{
                "address": stash
            },
        };
        let client = self.client.as_ref().ok_or(DatabaseError::Mongo);
        if let Ok(client) = client {
            let db = client.database(&self.db_name);
            let mut cursor = db
                .collection::<Document>("nominator")
                .aggregate(vec![match_command], None)
                .await
                .unwrap();
            if let Some(result) = cursor.next().await {
                let doc = result.unwrap();
                let n = doc! {
                    "accountId": doc.get("address").unwrap().as_str().unwrap().to_string(),
                    "balance": doc.get("balance").unwrap(),
                    "targets": doc.get_array("targets").unwrap()
                };
                let n = bson::from_document(n).unwrap();
                Ok(n)
            } else {
                Err(DatabaseError::GetFailed)
            }
        } else {
            Err(DatabaseError::Disconnected)
        }
    }
}