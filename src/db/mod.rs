
use std::fmt;
use std::net::Ipv4Addr;
use std::error::Error;
use futures::StreamExt;
use mongodb::{Client, options::ClientOptions};
use mongodb::bson::{self, Bson, doc, bson};
use types::{ValidatorNominationInfo};
use super::types;
use super::config::Config;

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
    client: Option<Client>
}

impl Database {
    pub fn new(ip: Ipv4Addr, port: u16, db_name: &str) -> Self {
        Database {
            ip: ip,
            port: port,
            db_name: db_name.to_string(),
            client: None
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

    pub async fn get_validator(&self, stash: String) -> Result<types::ValidatorNominationTrend, DatabaseError> {
        let cloned_stash = stash.clone();
        let match_command = doc! {
            "$match":{
                "id": stash
            },
        };
        let lookup_command = doc! {
            "$lookup": {
                "from": "nomination",
                "localField": "id",
                "foreignField": "validator",
                "as": "info"
            },
        };
        // let mut array = Vec::new();
        match self.client.as_ref().ok_or(DatabaseError {message: "Mongodb client is not working as expected.".to_string()}) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db.collection("validator")
                    .aggregate(vec![match_command, lookup_command], None).await.unwrap();
                while let Some(result) = cursor.next().await {
                    let unwrapped = result.unwrap();
                    let info: types::ValidatorNominationTrend = bson::from_bson(Bson::Document(unwrapped)).unwrap();
                    return Ok(info)
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

    pub async fn get_all_validator_info_of_era(&self, era: u32, page: u32, size: u32) -> Result<Vec<types::ValidatorNominationInfo>, DatabaseError> {
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
        let skip_command = doc! {
            "$skip": page * size,
        };
        let limit_command = doc! {
            "$limit": size,
        };
        match self.client.as_ref().ok_or(DatabaseError {message: "Mongodb client is not working as expected.".to_string()}) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db.collection("nomination")
                    .aggregate(vec![match_command, lookup_command, skip_command, limit_command], None).await.unwrap();
                while let Some(result) = cursor.next().await {
                    let doc = result.unwrap();
                    // println!("{}", doc);
                    let _data = &doc.get_array("data").unwrap()[0];
                    
                    let id  = _data.as_document().unwrap().get("id");
                    let status_change =  _data.as_document().unwrap().get("statusChange");
                    let identity = _data.as_document().unwrap().get("identity");
                    let default_identity = bson!({
                        "display": ""
                    });
                    let output = doc! {
                        "id": id.unwrap(),
                        "statusChange": status_change.unwrap(),
                        "identity": identity.unwrap_or_else(|| &default_identity),
                        "info": {
                            "nominators": doc.get_array("nominators").unwrap(),
                            "era": doc.get("era").unwrap(),
                            "commission": doc.get("commission").unwrap(),
                            "apy": doc.get("apy").unwrap(),
                            "exposure": doc.get("exposure").unwrap(),
                        }
                    };
                    // info: {
                    //     nominators: nomination.nominators,
                    //     era: nomination.era,
                    //     exposure: nomination.exposure,
                    //     commission: nomination.commission,
                    //     apy: nomination.apy
                    //   }
                    let info: ValidatorNominationInfo = bson::from_bson(Bson::Document(output)).unwrap();
                    array.push(info);
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
        match self.client.as_ref().ok_or(DatabaseError {message: "Mongodb client is not working as expected.".to_string()}) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let cursor = db.collection("chainInfo")
                    .find_one(None, None).await.unwrap();
                let data = bson::from_bson(Bson::Document(cursor.unwrap())).unwrap();
                Ok(data)
            }
            Err(e) => {
                println!("{}", e);
                Err(e)
            }
        }
    }
}