use super::config::Config;
use super::types;
use futures::StreamExt;
use mongodb::bson::{self, Bson, Document, bson, doc};
use mongodb::{options::ClientOptions, Client};
use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;
use types::{StashRewards, ValidatorNominationInfo};

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
    ip: Ipv4Addr,
    port: u16,
    db_name: String,
    client: Option<Client>,
}

impl Database {
    pub fn new(ip: Ipv4Addr, port: u16, db_name: &str) -> Self {
        Database {
            ip: ip,
            port: port,
            db_name: db_name.to_string(),
            client: None,
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

    pub async fn get_validator(
        &self,
        stash: String,
    ) -> Result<types::ValidatorNominationTrend, DatabaseError> {
        let cloned_stash = stash.clone();
        let match_command = doc! {
            "$match":{
                "id": stash.clone()
            },
        };
        let lookup_command = doc! {
            "$lookup": {
                "from": "nomination",
                "localField": "id",
                "foreignField": "validator",
                "as": "info",
            },
        };
        let unwind_command = doc! {
            "$unwind": {
                "path": "$info",
                "includeArrayIndex": "infoIndex",
                "preserveNullAndEmptyArrays": false
            }
        };

        let project_command = doc! {
            "$project": {
                "id": 1,
                "identity": 1,
                "statusChange": 1,
                "rewards": 1,
                "info": {
                    "era": 1,
                    "exposure": 1,
                    "commission": 1,
                    "apy": 1,
                    "validator": 1,
                    "nominatorCount": {
                        "$size": "$info.nominators"
                    }
                },
            }
        };

        let group_command = doc! {
            "$group": {
                "_id": "$_id",
                "id": { "$first" : "$id" },
                "identity": { "$first" : "$identity" },
                "statusChange": { "$first" : "$statusChange" },
                "rewards": { "$first" : "$rewards" },
                "info": {"$push": "$info"}
            }
        };
        // let mut array = Vec::new();
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection("validator")
                    .aggregate(vec![match_command, lookup_command, unwind_command, project_command, group_command], None)
                    .await
                    .unwrap();
                while let Some(result) = cursor.next().await {
                    let unwrapped = result.unwrap();
                    // println!("{:?}", unwrapped);
                    let mut info: types::ValidatorNominationTrend =
                        bson::from_bson(Bson::Document(unwrapped)).unwrap();
                    let mut cursor2 = db
                    .collection("nomination")
                    .aggregate(vec![doc! {
                        "$match":{
                            "validator": stash.clone()
                        },
                    }, doc! {
                        "$sort": {"era": -1}
                    }, doc! {
                        "$limit": 1
                    }, doc! {
                        "$project": {
                            "era": 1,
                            "exposure": 1,
                            "apy": 1,
                            "commission": 1,
                            "validator": 1,
                            "nominators": 1,
                            "nominatorCount": {"$size": "$nominators"},
                        }
                    }], None)
                    .await
                    .unwrap();
                    while let Some(result2) = cursor2.next().await {
                        let info2: types::NominationInfo = bson::from_bson(Bson::Document(result2.unwrap())).unwrap();
                        let mut index: i32 = -1;
                        for (i, era_info) in info.info.iter().enumerate() {
                            if era_info.era == info2.era {
                                index = i as i32;
                            }
                        }
                        println!("{:?}", info);
                        if index >= 0 {
                            info.info[index as usize].set_nominators(info2.nominators.unwrap_or_else(||vec![]));
                        }
                        break;
                    }
                    return Ok(info);
                }
                Err(DatabaseError {
                    message: format!("Failed to find validator with stash {}", cloned_stash),
                })
            }
            Err(e) => {
                println!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn get_validator_unclaimed_eras(
        &self,
        stash: String,
    ) -> Result<Vec<i32>, DatabaseError> {
        let mut array = Vec::new();
        let match_command = doc! {
            "$match":{
                "validator": stash
            },
        };

        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection("unclaimedEraInfo")
                    .aggregate(vec![match_command], None)
                    .await
                    .unwrap();
                while let Some(result) = cursor.next().await {
                    let doc = result.unwrap();
                    let eras = doc.get_array("eras").unwrap();
                    for era in eras {
                        array.push(era.as_i32().unwrap());
                    }
                }
                Ok(array)
            }
            Err(e) => {
                println!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn get_all_validator_info_of_era(
        &self,
        era: u32,
        page: u32,
        size: u32,
    ) -> Result<Vec<types::ValidatorNominationInfo>, DatabaseError> {
        let mut array = Vec::new();
        let match_command = doc! {
            "$match":{
                "era": era
            },
        };
        let lookup_command = doc! {
            "$lookup": {
                "from": "validator",
                "localField": "validator",
                "foreignField": "id",
                "as": "data"
            },
        };
        let lookup_command2 = doc! {
            "$lookup": {
                "from": "unclaimedEraInfo",
                "localField": "validator",
                "foreignField": "validator",
                "as": "unclaimedEraInfo"
            },
        };
        let skip_command = doc! {
            "$skip": page * size,
        };
        let limit_command = doc! {
            "$limit": size,
        };
        self.do_get_validator_info(array,  vec![
                match_command,
                lookup_command,
                lookup_command2,
                skip_command,
                limit_command,
            ]).await
    }

    pub async fn get_validator_info(&self, stashes: Vec<String>, era: u32) -> Result<Vec<types::ValidatorNominationInfo>, DatabaseError> {
        println!("{:?}", stashes);
        let mut array = Vec::new();
        let match_command = doc! {
            "$match":{
                "$and": [
                    {"era": era},
                    {"validator": {
                        "$in": stashes
                    }}
                ]
            },
        };
        let lookup_command = doc! {
            "$lookup": {
                "from": "validator",
                "localField": "validator",
                "foreignField": "id",
                "as": "data"
            },
        };
        let lookup_command2 = doc! {
            "$lookup": {
                "from": "unclaimedEraInfo",
                "localField": "validator",
                "foreignField": "validator",
                "as": "unclaimedEraInfo"
            },
        };
        let skip_command = doc! {
        };
        let limit_command = doc! {
        };
        self.do_get_validator_info(array, vec! [
            match_command,
            lookup_command,
            lookup_command2,
        ]).await
    }

    async fn do_get_validator_info(&self, mut array: Vec<ValidatorNominationInfo>, pipeline: Vec<Document> ) 
        -> Result<Vec<ValidatorNominationInfo>, DatabaseError> {
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection("nomination")
                    .aggregate( pipeline,
                        None,
                    )
                    .await
                    .unwrap();
                while let Some(result) = cursor.next().await {
                    let doc = result.unwrap();
                    let _data = &doc.get_array("data").unwrap()[0];
                    let id = _data.as_document().unwrap().get("id");
                    let default_unclaimed_eras = vec![bson! ({
                        "eras": [],
                        "validator": id.unwrap(),
                    })];
                    let unclaimed_era_infos = doc
                        .get_array("unclaimedEraInfo")
                        .unwrap_or(&default_unclaimed_eras);
                    let status_change = _data.as_document().unwrap().get("statusChange");
                    let identity = _data.as_document().unwrap().get("identity");
                    let default_identity = bson!({
                        "display": ""
                    });
                    let mut output = doc! {};
                    if unclaimed_era_infos.len() == 0 {
                        output = doc! {
                            "id": id.unwrap(),
                            "statusChange": status_change.unwrap(),
                            "identity": identity.unwrap_or_else(|| &default_identity),
                            "info": {
                                "nominators": doc.get_array("nominators").unwrap(),
                                "nominatorCount": doc.get_array("nominators").unwrap().len() as u32,
                                "era": doc.get("era").unwrap(),
                                "commission": doc.get("commission").unwrap(),
                                "apy": doc.get("apy").unwrap(),
                                "exposure": doc.get("exposure").unwrap(),
                                "unclaimedEras": bson! ([]),
                            }
                        };
                    } else {
                        output = doc! {
                            "id": id.unwrap(),
                            "statusChange": status_change.unwrap(),
                            "identity": identity.unwrap_or_else(|| &default_identity),
                            "info": {
                                "nominators": doc.get_array("nominators").unwrap(),
                                "nominatorCount": doc.get_array("nominators").unwrap().len() as u32,
                                "era": doc.get("era").unwrap(),
                                "commission": doc.get("commission").unwrap(),
                                "apy": doc.get("apy").unwrap(),
                                "exposure": doc.get("exposure").unwrap(),
                                "unclaimedEras": unclaimed_era_infos[0].as_document().unwrap().get_array("eras").unwrap(),
                            }
                        };
                    }
                    // println!("{:?}", output);
                    let info: ValidatorNominationInfo =
                        bson::from_bson(Bson::Document(output)).unwrap();
                    array.push(info.clone());
                    // println!("{:?}", unclaimed_era_info.as_document().unwrap().get_array("eras").unwrap());
                    // println!("{:?}", info);
                }
                Ok(array)
            }
            Err(e) => {
                println!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn get_chain_info(&self) -> Result<types::ChainInfo, DatabaseError> {
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let cursor = db
                    .collection("chainInfo")
                    .find_one(None, None)
                    .await
                    .unwrap();
                let data = bson::from_bson(Bson::Document(cursor.unwrap())).unwrap();
                Ok(data)
            }
            Err(e) => {
                println!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn get_stash_reward(
        &self,
        stash: String,
    ) -> Result<types::StashRewards, DatabaseError> {
        let _stash = stash.clone();
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection("stashInfo")
                    .find(doc! {"stash": _stash}, None)
                    .await
                    .unwrap();
                let mut era_rewards: Vec<types::StashEraReward> = vec![];
                while let Some(stash_reward) = cursor.next().await {
                    let doc = stash_reward.unwrap();
                    let mut era = 0;
                    match doc.get("era").unwrap().as_i32() {
                        Some(_era) => era = _era,
                        None => continue,
                    }
                    let amount = doc.get("amount").unwrap().as_f64().unwrap_or_else(|| 0.0);
                    let timestamp = doc.get("timestamp").unwrap().as_f64().unwrap();
                    era_rewards.push(types::StashEraReward {
                        era: era,
                        amount: amount,
                        timestamp: (timestamp).round() as i64,
                    })
                }
                Ok(types::StashRewards {
                    stash: stash,
                    era_rewards: era_rewards,
                })
            }
            Err(e) => {
                println!("{}", e);
                Err(e)
            }
        }
    }
}
