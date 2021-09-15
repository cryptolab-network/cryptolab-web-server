use futures::StreamExt;
use mongodb::bson::{self, Bson, Document, doc, bson};

use crate::types::{self, ValidatorCommission, ValidatorNominationInfo, ValidatorSlash, ValidatorStalePayoutEvent};
use log::error;
use super::{Database, DatabaseError, params::AllValidatorOptions};

impl Database {
  
    pub async fn get_multiple_validators_slashes(
        &self,
        validators: &[String],
    ) -> Result<Vec<ValidatorSlash>, DatabaseError> {
        let mut array = Vec::new();
        let match_command = doc! {
            "$match": {
                "address": {
                    "$in": validators
                }
            }
        };
    
        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection::<Document>("validatorSlash")
                    .aggregate(vec![match_command], None)
                    .await
                    .unwrap();
                while let Some(result) = cursor.next().await {
                    let doc = result.unwrap();
                    let slash: ValidatorSlash = bson::from_bson(Bson::Document(doc)).unwrap();
                    array.push(slash);
                }
                Ok(array)
            }
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }

  pub async fn get_is_commission_changed(
      &self,
      validators: &Vec<String>,
      from: u32,
      to: u32,
  ) -> Result<Vec<types::ValidatorCommission>, DatabaseError> {
    let mut array: Vec<types::ValidatorCommission> = Vec::new();
    let match_command = doc! {
        "$match":{
            "$and": [
                {"address": {
                    "$in": validators
                }},  {
                    "era": {
                        "$gte": from,
                        "$lte": to
                    }
                }
            ]
        },
    };
    match self.client.as_ref().ok_or(DatabaseError {
        message: "Mongodb client is not working as expected.".to_string(),
    }) {
        Ok(client) => {
            let db = client.database(&self.db_name);
            let mut cursor = db
              .collection::<Document>("commission")
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
                let vc: ValidatorCommission =
                      bson::from_bson(Bson::Document(doc)).unwrap();
                      array.push(vc);
            }
            Ok(array)
        }
        Err(e) => {
            error!("{}", e);
            Err(e)
        }
    }
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
                },
                "total": 1,
                "selfStake": 1,
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
                .collection::<Document>("validator")
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
                    .collection::<Document>("nomination")
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
                                    "total": 1,
                                    "selfStake": 1,
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
                  .collection::<Document>("unclaimedEraInfo")
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

    pub async fn get_validator_slashes(
        &self,
        stash: String,
    ) -> Result<Vec<ValidatorSlash>, DatabaseError> {
        let mut array = Vec::new();
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
                    .collection::<Document>("validatorSlash")
                    .aggregate(vec![match_command], None)
                    .await
                    .unwrap();
                while let Some(result) = cursor.next().await {
                    let doc = result.unwrap();
                    let slash: ValidatorSlash = bson::from_bson(Bson::Document(doc)).unwrap();
                    array.push(slash);
                }
                Ok(array)
            }
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }

    pub async fn get_nominated_validators_stale_payout_events(
        &self,
        validators: &[String],
        from: u32,
        to: u32,
    ) -> Result<Vec<ValidatorStalePayoutEvent>, DatabaseError> {
        let mut array = Vec::new();
        let match_command = doc! {
            "$match":{
                "$and": [
                    {"address": {
                        "$in": validators
                    }},  {
                        "era": {
                            "$gte": from,
                            "$lte": to
                        }
                    }
                ]
            },
        };

        match self.client.as_ref().ok_or(DatabaseError {
            message: "Mongodb client is not working as expected.".to_string(),
        }) {
            Ok(client) => {
                let db = client.database(&self.db_name);
                let mut cursor = db
                    .collection::<Document>("stalePayouts")
                    .aggregate(vec![match_command], None)
                    .await
                    .unwrap();
                while let Some(result) = cursor.next().await {
                    let doc = result.unwrap();
                    let events: ValidatorStalePayoutEvent = bson::from_bson(Bson::Document(doc)).unwrap();
                    array.push(events);
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
                  .collection::<Document>("nomination")
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
                  let blocked = _data.as_document().unwrap().get("blocked");
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
                  let mut output: Document;
                  if !unclaimed_era_infos.is_empty() {
                    output = doc! {
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
                            "selfStake": doc.get("selfStake").unwrap_or(&Bson::String("0x00".to_string())),
                        },
                        "stakerPoints": staker_points.unwrap(),
                        "averageApy": average_apy.unwrap_or(&Bson::Int32(0)),
                        "blocked": blocked.unwrap_or(&Bson::Boolean(false)),
                    };
                  } else {
                    output = doc! {
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
                            "unclaimedEras": [],
                            "total": doc.get("total").unwrap_or(&Bson::String("0x00".to_string())),
                            "selfStake": doc.get("selfStake").unwrap_or(&Bson::String("0x00".to_string())),
                        },
                        "stakerPoints": staker_points.unwrap(),
                        "averageApy": average_apy.unwrap_or(&Bson::Int32(0)),
                        "blocked": blocked.unwrap_or(&Bson::Boolean(false)),
                    };
                  }
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
                          output.insert("slashes", bson! ([]));
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

}