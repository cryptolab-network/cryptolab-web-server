use crate::types::NominatorNomination;

use self::params::AllValidatorOptions;
use super::config::Config;
use super::types;
use chrono::{DateTime, NaiveDateTime, Utc};
use futures::StreamExt;
use log::error;
use mongodb::bson::{self, bson, doc, Bson, Document};
use mongodb::{options::ClientOptions, Client};
use std::fmt;
use std::{collections::HashMap, error::Error};
use types::{ValidatorNominationInfo};
pub(crate) mod params;

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

    pub async fn get_validator(
        &self,
        stash: String,
    ) -> Result<types::ValidatorNominationTrend, DatabaseError> {
        let match_command = doc! {
            "$match":{
                "id": &stash
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
                "averageApy": 1,
                "stakerPoints": 1,
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
                "averageApy": { "$first" : "$averageApy" },
                "stakerPoints": { "$first" : "$stakerPoints" },
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
                    .aggregate(
                        vec![
                            match_command,
                            lookup_command,
                            unwind_command,
                            project_command,
                            group_command,
                        ],
                        None,
                    )
                    .await
                    .unwrap();
                if let Some(result) = cursor.next().await {
                    let unwrapped = result.unwrap();
                    let mut info: types::ValidatorNominationTrend =
                        bson::from_bson(Bson::Document(unwrapped)).unwrap();
                    let mut cursor2 = db
                        .collection("nomination")
                        .aggregate(
                            vec![
                                doc! {
                                    "$match":{
                                        "validator": &stash
                                    },
                                },
                                doc! {
                                    "$sort": {"era": -1}
                                },
                                doc! {
                                    "$limit": 1
                                },
                                doc! {
                                    "$lookup": {
                                        "from": "nominator",
                                        "localField": "nominators",
                                        "foreignField": "address",
                                        "as": "nominators",
                                    }
                                },
                                doc! {
                                    "$project": {
                                        "era": 1,
                                        "exposure": 1,
                                        "commission": 1,
                                        "apy": 1,
                                        "validator": 1,
                                        "nominatorCount": {
                                            "$size": "$nominators"
                                        },
                                        "nominators": 1,
                                    }
                                },
                            ],
                            None,
                        )
                        .await
                        .unwrap();
                    if let Some(result2) = cursor2.next().await {
                        let info2: types::NominationInfo =
                            bson::from_bson(Bson::Document(result2.unwrap())).unwrap();
                        let mut index: i32 = -1;
                        for (i, era_info) in info.info.iter().enumerate() {
                            if era_info.era == info2.era {
                                index = i as i32;
                            }
                        }
                        if index >= 0 {
                            info.info[index as usize]
                                .set_nominators(info2.nominators.unwrap_or_else(std::vec::Vec::new));
                        }
                    }
                    return Ok(info);
                }
                Err(DatabaseError {
                    message: format!("Failed to find validator with stash {}", &stash),
                })
            }
            Err(e) => {
                error!("{}", e);
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
                error!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn get_all_validator_info_of_era(
        &self,
        era: u32,
        options: AllValidatorOptions,
    ) -> Result<Vec<types::ValidatorNominationInfo>, DatabaseError> {
        let array = Vec::new();
        let match_command = doc! {
            "$match":{
                "era": era,
                "apy": {"$lte": options.apy_max, "$gte": options.apy_min},
                "commission": {"$lte": options.commission_max * 100.0, "$gte": options.commission_min * 100.0},
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
        let match_command2 = doc! {
            "$match": {
                "data.identity.isVerified": true
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
        let lookup_command3 = doc! {
            "$lookup": {
                "from": "validatorSlash",
                "localField": "validator",
                "foreignField": "address",
                "as": "slashes"
            },
        };
        let skip_command = doc! {
            "$skip": options.page * options.size,
        };
        let limit_command = doc! {
            "$limit": options.size,
        };
        if options.has_verified_identity {
            self.do_get_validator_info(
                array,
                vec![
                    match_command,
                    lookup_command,
                    match_command2,
                    lookup_command2,
                    lookup_command3,
                    skip_command,
                    limit_command,
                ],
            )
            .await
        } else {
            self.do_get_validator_info(
                array,
                vec![
                    match_command,
                    lookup_command,
                    lookup_command2,
                    lookup_command3,
                    skip_command,
                    limit_command,
                ],
            )
            .await
        }
    }

    pub async fn get_validator_info(
        &self,
        stashes: &[String],
        era: &u32,
    ) -> Result<Vec<types::ValidatorNominationInfo>, DatabaseError> {
        let array = Vec::new();
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
        let unwind_command = doc! {
            "$unwind": {
                "path": "$nominators",
                "includeArrayIndex": "nominatorIndex",
                "preserveNullAndEmptyArrays": false
            }
        };
        let lookup_command3 = doc! {
            "$lookup": {
                "from": "nominator",
                "localField": "nominators",
                "foreignField": "address",
                "as": "nominators"
            },
        };
        let unwind_command2 = doc! {
            "$unwind": {
                "path": "$nominators",
                "includeArrayIndex": "nominatorIndex2",
                "preserveNullAndEmptyArrays": false
            }
        };
        // let match_command2 = doc! {
        //     "$match": {
        //         "$expr": {
        //             "$eq": ["$nominators.era", "$era"]
        //           }
        //     }
        // };
        let group_command = doc! {
            "$group": {
                "_id": "$_id",
                "era": { "$first" : "$era" },
                "exposure": { "$first" : "$exposure" },
                "commission": { "$first" : "$commission" },
                "apy": { "$first" : "$apy" },
                "validator": {"$first": "$validator"},
                "nominators": {"$push": "$nominators"},
                "data": {"$first": "$data"},
            }
        };
        self.do_get_validator_info(
            array,
            vec![
                match_command,
                lookup_command,
                lookup_command2,
                unwind_command,
                lookup_command3,
                unwind_command2,
                // match_command2,
                group_command,
            ],
        )
        .await
    }

    async fn do_get_validator_info(
        &self,
        mut array: Vec<ValidatorNominationInfo>,
        pipeline: Vec<Document>,
    ) -> Result<Vec<ValidatorNominationInfo>, DatabaseError> {
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection("nomination")
                    .aggregate(pipeline, None)
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
                    let staker_points = _data.as_document().unwrap().get("stakerPoints");
                    let average_apy = _data.as_document().unwrap().get("averageApy");
                    let slashes = doc.get("slashes");
                    let default_identity = bson!({
                        "display": "",
                        "parent": "",
                        "sub": "",
                        "is_verified": false,
                    });
                    let nominators = doc.get_array("nominators").unwrap();
                    let mut _nominators = vec![];

                    for n in nominators {
                        if let Some(n) = n.as_str() {
                            _nominators.push(bson!({
                                "address": n.to_string(),
                            }));
                        } else {
                            _nominators.push(n.clone());
                        }
                    }

                    let mut output = doc! {
                        "id": id.unwrap(),
                        "statusChange": status_change.unwrap(),
                        "identity": identity.unwrap_or(&default_identity),
                        "info": {
                            "nominators": &_nominators,
                            "nominatorCount": doc.get_array("nominators").unwrap().len() as u32,
                            "era": doc.get("era").unwrap(),
                            "commission": doc.get("commission").unwrap(),
                            "apy": doc.get("apy").unwrap(),
                            "exposure": doc.get("exposure").unwrap(),
                            "unclaimedEras": unclaimed_era_infos[0].as_document().unwrap().get_array("eras").unwrap(),
                            "total": doc.get("total").unwrap_or(&Bson::String("0x00".to_string())),
                        },
                        "stakerPoints": staker_points.unwrap(),
                        "averageApy": average_apy.unwrap_or(&Bson::Int32(0)),
                    };
                    let info = output.get_document_mut("info").unwrap();
                    if unclaimed_era_infos.is_empty() {
                        info.insert("unclaimedEras", bson! ([]));
                    } else {
                        info.insert("unclaimedEras", unclaimed_era_infos[0].as_document().unwrap().get_array("eras").unwrap());
                    }
                    match slashes {
                        Some(slashes) => {

                            output.insert("slashes", slashes);
                        },
                        None => {
                            output.insert("slashes", bson! (null));
                        }
                    }
                    let info: ValidatorNominationInfo =
                        bson::from_bson(Bson::Document(output)).unwrap();
                    array.push(info);
                }
                Ok(array)
            }
            Err(e) => {
                error!("{}", e);
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

    pub async fn get_nominator_info(
    &mut self,
    stash: String,
    ) -> Result<NominatorNomination, DatabaseError> {
        match self.get_stash_reward(&stash).await {
            Ok(rewards) => {
                match self.do_get_nominator_info(&stash).await {
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

        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection("nominator")
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
                    Err(DatabaseError {
                        message: format!("Cannot find stash {}.", &stash),
                    })
                }
            }
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }
}
