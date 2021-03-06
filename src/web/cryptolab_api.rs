use crate::cache_redis::Cache;
use crate::config::Config;
use crate::referer;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::json;
use validator::Validate;
use crate::staking_rewards_collector::StakingRewardsCollector;
use crate::staking_rewards_collector::{StakingRewardsAddress, StakingRewardsReport};
use crate::types::{NewsletterSubscriberOptions, NominationOptions, NominationResultOptions, NominationResultParams, OverSubscribeEventOutput, RefKeyOptions, StakingEvents, UserEventMappingOptions, ValidatorNominationInfo};
use crate::web::Invalid;

// use super::super::cache;
use super::super::db::Database;
use super::params::{ErrorCode, EventFilterOptions, OperationFailed};
use super::params::{AllValidatorOptions, InvalidParam};
use std::path::Path;
use std::process::Command;
use std::{convert::Infallible};
use log::{debug, error, info};
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

#[derive(Deserialize)]
struct StakingRewardsOptions {
  pub start: Option<String>,
  pub end: Option<String>,
  pub currency: Option<String>,
  pub price_data: Option<bool>,
  pub start_balance: Option<f64>
}

fn validate_get_all_validators() -> impl Filter<Extract = (AllValidatorOptions,), Error = Rejection> + Copy {
  warp::filters::query::query().and_then(|params: AllValidatorOptions| async move {
    if !(0.0..=1.0).contains(&params.apy_max()) {
      return Err(warp::reject::custom(InvalidParam::new("apy_max must be between 0 ~ 1.", 
      ErrorCode::InvalidApy)));
    }
    if !(0.0..=1.0).contains(&params.apy_min()) {
      return Err(warp::reject::custom(InvalidParam::new("apy_min must be between 0 ~ 1.",
      ErrorCode::InvalidApy)));
    }
    if !(0.0..=1.0).contains(&params.commission_min()) {
      return Err(warp::reject::custom(InvalidParam::new("commission_min must be between 0 ~ 1.",
      ErrorCode::InvalidCommission)));
    }
    if !(0.0..=1.0).contains(&params.commission_max()) {
      return Err(warp::reject::custom(InvalidParam::new("commission_max must be between 0 ~ 1.",
      ErrorCode::InvalidCommission)));
    }
    Ok(params)
  })
}

fn validate_event_filters() -> impl Filter<Extract = (EventFilterOptions,), Error = Rejection> + Copy {
  warp::filters::query::query().and_then(|params: EventFilterOptions| async move {
    if params.from_era() > params.to_era() {
      return Err(warp::reject::custom(InvalidParam::new("from_era cannot be greater than to_era", 
      ErrorCode::InvalidApy)));
    }
    Ok(params)
  })
}

fn validate_ref_key_options() -> impl Filter<Extract = (RefKeyOptions,), Error = Rejection> + Copy {
  warp::filters::body::json().and_then(|params: RefKeyOptions| async move {
    if params.ref_key.is_empty() {
      return Err(warp::reject::custom(InvalidParam::new("ref_key cannot be empty", 
      ErrorCode::EmptyRefKey)));
    }
    Ok(params)
  })
}


fn validate_newsletter_subscription() -> impl Filter<Extract = (NewsletterSubscriberOptions,), Error = Rejection> + Copy {
  warp::filters::body::json().and_then(|params: NewsletterSubscriberOptions| async move {
    match params.validate() {
      Ok(_) => {
        Ok(params)
      },
      Err(e) => {
        println!("{:?}", e);
        Err(warp::reject::custom(InvalidParam::new("Must be a valid email address",
      ErrorCode::InvalidEmailAddress)))
      }
    }
  })
}

fn get_all_validators(chain: &'static str, db: Database, cache: Cache) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
    .and(warp::path("v1"))
    .and(warp::path("validators"))
    .and(warp::path(chain))
    .and(warp::path::end())
    .and(with_db(db))
    .and(with_cache(cache))
    .and(validate_get_all_validators())
    .and_then(move |db: Database, cache: Cache, p: AllValidatorOptions| async move {
        let mut era = cache.get_current_era(chain);
        if era == 0 {
          era = db.get_chain_info().await.unwrap().active_era;
        }
        let validators = get_validator_data_from_db(db, cache, chain.to_string(), era, p).await;
        validators
    })
}

fn get_nominator_info(chain: &'static str, db: Database) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("nominator"))
  .and(warp::path("id"))
  .and(warp::path::param())
  .and(warp::path(chain))
  .and(warp::path::end())
  .and(with_db(db))
  .and_then(|id: String, mut db: Database| async move {
    let nominator = db.get_nominator_info(&id).await;
    if let Ok(nominator) = nominator {
      Ok(warp::reply::with_status(
        warp::reply::json(&nominator),
        StatusCode::OK,
      ))
    } else {
      Err(warp::reject::not_found())
    }
  })
}

fn get_validator_history(
  chain: &'static str,
  db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("validator"))
  .and(with_db(db))
  .and(warp::path::param())
  .and(warp::path(chain))
  .and(warp::path::end())
  .and_then(|db: Database, stash: String| async move {
      let validator = db.get_validator(stash).await;
      match validator {
          Ok(v) => Ok(warp::reply::json(&[v])),
          Err(_) => Err(warp::reject::not_found()),
      }
  })
}

fn get_all_nominators(
  chain: &'static str,
  cache: Cache
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("nominators"))
  .and(warp::path(chain))
  .and(warp::path::end())
  .map(move || warp::reply::json(&cache.get_nominators(chain)))
}

fn get_1kv_validators(
  chain: &'static str,
  cache: Cache
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path("api")
    .and(warp::path("v1"))
    .and(warp::path("1kv"))
    .and(warp::path("validators"))
    .and(warp::path(chain))
    .and(warp::path::end())
    .map(move || warp::reply::json(&cache.get_1kv_info_detail(chain)))
}

fn get_1kv_nominators(
  chain: &'static str,
  cache: Cache
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("1kv"))
  .and(warp::path("nominators"))
  .and(warp::path(chain))
  .and(warp::path::end())
  .map(move || warp::reply::json(&cache.get_1kv_nominators(chain)))
}

fn get_stash_rewards_collector(src_path: String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("stash"))
      .and(warp::path::param())
      .and(with_string(src_path))
      .and(warp::path("rewards"))
      .and(warp::path("collector"))
      .and(warp::path::end())
      .and(warp::query::<StakingRewardsOptions>())
      .and_then(|stash: String, src_path: String, p: StakingRewardsOptions| async move {
          let start = "2020-01-01".to_string();
          let end = chrono::Utc::now().format("%Y-%m-%d").to_string();
          let currency = "USD".to_string();
          let mut network = "Kusama";
          if stash.starts_with("1") {
            network = "Polkadot";
          }
          let src = StakingRewardsCollector::new(p.start.unwrap_or(start), p.end.unwrap_or(end),
          p.currency.unwrap_or(currency), p.price_data.unwrap_or(true),
          vec![StakingRewardsAddress::new("".to_string(), stash.clone(), p.start_balance.unwrap_or(0.0), network.to_string())]);
          match src {
              Ok(src) => {
                  let result = src.call_exe(src_path.to_string());
                  match result {
                      Ok(v) => {
                          Ok(warp::reply::json(&v))
                      },
                      Err(e) => {
                          error!("{}", e);
                          if e.err_code == -2 {
                              Err(warp::reject::not_found())
                          } else {
                              Err(warp::reject::custom(e))
                          }
                      },
                  }
              },
              Err(e) => {
                  Err(warp::reject::custom(e))
              },
          }
      })
}

fn get_stash_rewards_collector_csv(src_path: String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
      .and(warp::path("v1"))
      .and(warp::path("stash"))
      .and(warp::path::param())
      .and(with_string(src_path))
      .and(warp::path("rewards"))
      .and(warp::path("collector"))
      .and(warp::path("csv"))
      .and(warp::path::end())
      .and_then(|stash: String, src_path: String| async move{
          // validate stash
          if !stash.chars().all(char::is_alphanumeric) {
              debug!("{}", stash);
              Err(warp::reject::custom(Invalid))
          } else {
              // get file from src path
              let srr = StakingRewardsReport::new(src_path, stash, "csv".to_string());
              let file = srr.get_report();
              match file {
                  Ok(data) => {
                      Ok(data)
                  },
                  Err(err) => {
                      Err(warp::reject::custom(err))
                  },
              }
          }
          
      })
}

fn get_stash_rewards_collector_json(src_path: String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("stash"))
      .and(warp::path::param())
      .and(with_string(src_path))
      .and(warp::path("rewards"))
      .and(warp::path("collector"))
      .and(warp::path("json"))
      .and(warp::path::end())
      .and_then(|stash: String, src_path: String| async move{
          // validate stash
          if !stash.chars().all(char::is_alphanumeric) {
              debug!("{}", stash);
              Err(warp::reject::custom(Invalid))
          } else {
              // get file from src path
              let srr = StakingRewardsReport::new(src_path, stash, "json".to_string());
              let file = srr.get_report();
              match file {
                  Ok(data) => {
                      Ok(data)
                  },
                  Err(err) => {
                      Err(warp::reject::custom(err))
                  },
              }
          }
          
      })
}

fn get_validator_unclaimed_eras(
  chain: &'static str,
  db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("validator"))
  .and(with_db(db))
  .and(warp::path::param())
  .and(warp::path("unclaimedEras"))
  .and(warp::path(chain))
  .and(warp::path::end())
  .and_then(|db: Database, stash: String| async move {
      let validator = db.get_validator_unclaimed_eras(stash).await;
      match validator {
          Ok(v) => Ok(warp::reply::json(&v)),
          Err(_) => Err(warp::reject::not_found()),
      }
  })
}

fn get_validator_slashes(
  chain: &'static str,
  db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("validator"))
  .and(with_db(db))
  .and(warp::path::param())
  .and(warp::path("slashes"))
  .and(warp::path(chain))
  .and(warp::path::end())
  .and_then(|db: Database, stash: String| async move {
      let validator = db.get_validator_slashes(stash).await;
      match validator {
          Ok(v) => Ok(warp::reply::json(&v)),
          Err(_) => Err(warp::reject::not_found()),
      }
  })
}

fn with_db(
    db: Database,
) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_cache(
  cache: Cache,
) -> impl Filter<Extract = (Cache,), Error = std::convert::Infallible> + Clone {
  warp::any().map(move || cache.clone())
}

fn with_string(
  s: String
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
  warp::any().map(move || s.clone())
}

async fn gen_ref_key(
  db: Database,
  stash: &str
) -> Result<warp::reply::Json, Infallible> {
  match db.get_validator_ref_key(&stash).await {
    Ok(ref_key) => {
      Ok(warp::reply::json(&json!({
        "refKey": ref_key
      })))
    },
    Err(_) => {
      let ref_key = referer::gen_ref_key(&stash);
      Ok(warp::reply::json(&json!({
        "refKey": ref_key
      })))
    },
  }
}

async fn get_validator_data_from_db(
    db: Database,
    cache: Cache,
    chain: String,
    era: u32,
    options: AllValidatorOptions,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    let mut result = db.get_all_validator_info_of_era(era, options.to_db_all_validator_options()).await.unwrap();
    if result.is_empty() {
      result = db.get_all_validator_info_of_era(era - 1, options.to_db_all_validator_options()).await.unwrap();
    }
    if options.has_joined_1kv() {
      if chain == "WND" {
        Ok(warp::reply::with_status(
          warp::reply::json(&json!([])),
          StatusCode::OK,
        ))
      } else {
        let one_kv = cache.get_1kv_info_detail(&chain);
        let mut one_kv_nodes: Vec<ValidatorNominationInfo> = [].to_vec();
        for v in result {
          let mut is1kv = false;
          for o in &one_kv.valid {
              if o.stash == v.id {
                is1kv = true;
                break;
              }
          }
          if is1kv {
            one_kv_nodes.push(v);
          }
        }
        Ok(warp::reply::with_status(
          warp::reply::json(&one_kv_nodes),
          StatusCode::OK,
        ))
      }
    } else {
      Ok(warp::reply::with_status(
          warp::reply::json(&result),
          StatusCode::OK,
      ))
    }
}

fn get_nominated_validators(
  chain: &'static str,
  db: Database,
  cache: Cache,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("nominated"))
  .and(with_db(db))
  .and(with_cache(cache))
  .and(warp::path("stash"))
  .and(warp::path::param())
  .and(warp::path(chain))
  .and(warp::path::end())
  .and_then(move |db: Database, cache: Cache, stash: String| async move {
      let result = &cache.get_nominator(&chain, stash);
      match result {
          Ok(nominator) => {
            let mut era = cache.get_current_era(chain);
            if era == 0 {
              era = db.get_chain_info().await.unwrap().active_era;
            }
            let result = db
              .get_validator_info(&nominator.targets, &era)
              .await;
              match result {
                  Ok(validators) => Ok(warp::reply::json(&validators)),
                  Err(_) => Err(warp::reject::not_found()),
              }
          }
          Err(_) => {
              error!("{}", "failed to get nominated list from the cache");
              Err(warp::reject::not_found())
          }
      }
  })
}

fn get_events(
  chain: &'static str,
  db: Database,
  user_db: Database,
  cache: Cache,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("events"))
  .and(with_db(db))
  .and(with_db(user_db))
  .and(with_cache(cache))
  .and(warp::path("stash"))
  .and(warp::path::param())
  .and(warp::path(chain))
  .and(warp::path::end())
  .and(validate_event_filters())
  .and_then(move |mut db: Database, mut user_db: Database, cache: Cache, stash: String, filters: EventFilterOptions| async move {
      let result = db.get_nominator_info(&stash).await;
      match result {
          Ok(nominator) => {
              let mut era = cache.get_current_era(chain);
              if era == 0 {
                era = db.get_chain_info().await.unwrap().active_era;
              }
              let mut to_era = era;
              if filters.to_era() > 0 {
                to_era = filters.to_era();
              }
              let mut from_era = era - 84;
              if filters.from_era() > 0 {
                from_era = filters.from_era();
              }
              match user_db.get_nomination_records(&stash).await {
                  Ok(c) => {
                    let options = UserEventMappingOptions {
                      stash,
                      from_era,
                      to_era,
                      event_types: vec![0, 1, 2, 3, 4, 5, 6],
                    };
                    let events = db.get_user_events_by_mapping(options).await;
                    match events {
                        Ok(events) => {
                          Ok(warp::reply::json(&events))
                        },
                        Err(_) => {
                          Err(warp::reject::not_found())
                        },
                    }
                  },
                  Err(_) => {
                    let commission = db
                    .get_is_commission_changed(&nominator.targets, from_era, to_era)
                    .await;
                    let slash = db
                    .get_multiple_validators_slashes(&nominator.targets, from_era, to_era)
                    .await;
                    let inactive = db.get_all_validators_inactive(&stash, from_era, to_era).await;
                    let stale_payouts =
                      db.get_nominated_validators_stale_payout_events(&nominator.targets, from_era, to_era).await;
                    let payouts = db.get_nominated_validators_payout_events(nominator.account_id, from_era, to_era).await;
                    let kicks = db.get_kick_events(&stash, &from_era, &to_era).await;
                    let chills = db.get_chill_events(&nominator.targets, &from_era, &to_era).await;
                    let oversubscribes = db.get_oversubscribe_events(&stash, &from_era, &to_era).await.unwrap_or_default();
                    let mut o = Vec::<OverSubscribeEventOutput>::new();
                    for ele in oversubscribes {
                        o.push(OverSubscribeEventOutput {
                          era: ele.era,
                          address: ele.address,
                          nominator: stash.clone(),
                          amount: ele.nominators.iter().find(|&x| x.who == stash.clone()).unwrap().value.clone()
                        });
                    }
                    let events = StakingEvents {
                      commissions: commission.unwrap_or_default(),
                      slashes: slash.unwrap_or_default(),
                      inactive: inactive.unwrap_or_default(),
                      stale_payouts: stale_payouts.unwrap_or_default(),
                      payouts: payouts.unwrap_or_default(),
                      kicks: kicks.unwrap_or_default(),
                      chills: chills.unwrap_or_default(),
                      over_subscribes: o,
                    };
                    Ok(warp::reply::json(&events))
                  }
              }
          }
          Err(_) => {
              error!("{}", "failed to get nominated list from the cache");
              Err(warp::reject::not_found())
          }
      }
  })
}

fn get_ref_key(
  chain: &'static str,
  db: Database,
  user_db: Database
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("refKey"))
  .and(with_db(db))
  .and(with_db(user_db))
  .and(warp::path("stash"))
  .and(warp::path::param())
  .and(warp::path(chain))
  .and(warp::path::end())
  .and_then(move |db: Database, user_db: Database, stash: String| async move {
    gen_ref_key(user_db, &stash).await
  })
}

fn json_body<T: DeserializeOwned + Send>() -> impl Filter<Extract = (T,), Error = warp::Rejection> + Clone {
  // When accepting a body, we want a JSON body
  // (and to reject huge payloads)...
  warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

fn post_nominated_records(
  chain: &'static str,
  db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("nominate"))
  .and(with_db(db))
  .and(warp::path(chain))
  .and(warp::path::end())
  .and(json_body::<NominationOptions>())
  .and(warp::post())
  .and_then(move |db: Database, options: NominationOptions| async move {
    let result = db.insert_nomination_action(chain.to_string(), options).await;
    if result.is_ok() {
      let tag = result.unwrap();
      Ok(warp::reply::with_status(
        tag,
        StatusCode::OK,
      ))
    } else {
      let err = result.err().unwrap();
      Err(warp::reject::custom(
        OperationFailed::new(&err.to_string(), ErrorCode::OperationFailed)
      ))
    }
  })
}

fn post_nominated_result(
  chain: &'static str,
  db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("nominated"))
  .and(with_db(db))
  .and(warp::path(chain))
  .and(warp::path::end())
  .and(json_body::<NominationResultOptions>())
  .and(warp::query::<NominationResultParams>())
  .and(warp::post())
  .and_then(move |db: Database, mut options: NominationResultOptions, params: NominationResultParams| async move { 
    options.ref_key = params.ref_key;
    let result = db.insert_nomination_result(options).await;
    if result.is_ok() {
      Ok(warp::reply::with_status(
        "",
        StatusCode::OK,
      ))
    } else {
      let err = result.err().unwrap();
      Err(warp::reject::custom(
        OperationFailed::new(&err.to_string(), ErrorCode::OperationFailed)
      ))
    }
  })
}

fn post_subscribe_newsletter(
  db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("newsletter"))
  .and(with_db(db))
  .and(warp::path::end())
  .and(warp::post())
  .and(validate_newsletter_subscription())
  .and_then(move |db: Database, options: NewsletterSubscriberOptions| async move { 
    let result = db.insert_newsletter_subsriber(options).await;
    if result.is_ok() {
      Ok(warp::reply::with_status(
        "",
        StatusCode::OK,
      ))
    } else {
      let err = result.err().unwrap();
      error!("{}", err);
      Err(warp::reject::custom(
        OperationFailed::new(&err.to_string(), ErrorCode::OperationFailed)
      ))
    }
  })
}

fn verify_ref_key(
  chain: &'static str,
  db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("refKey"))
  .and(with_db(db))
  .and(warp::path("stash"))
  .and(warp::path::param())
  .and(warp::path(chain))
  .and(warp::path("verify"))
  .and(warp::path::end())
  .and(validate_ref_key_options())
  .and_then(move |db: Database, stash: String, options: RefKeyOptions| async move {
    let buf = Path::new(Config::current().signature_verifier.as_str()).join("src").join("index.js").to_path_buf();
    let path  = buf.to_str().unwrap();
    let cmd = Command::new("node")
    .args([path, "--msg", &options.ref_key,
      "--signature", &options.encoded.unwrap(), "--address", &stash])
    .output()
    .expect("failed to execute process");
    let output = String::from_utf8(cmd.stdout);
    info!("{:?}", output);
    match output {
        Ok(output) => {
          if output.contains("true") {
            let ref_key_options = referer::decrypt_ref_key(&options.ref_key);
            match ref_key_options {
                Ok(ref_key_options) => {
                  db.insert_validator_ref_key(ref_key_options).await;
                  Ok(warp::reply::with_status(
                    "true",
                    StatusCode::OK,
                  ))
                },
                Err(_) => {
                  Err(warp::reject::custom(
                    OperationFailed::new("", ErrorCode::OperationFailed)
                  ))
                },
            }
          } else {
            Ok(warp::reply::with_status(
              "false",
              StatusCode::OK,
            ))
          }
        },
        Err(err) => {
          error!("{}", err);
          Err(warp::reject::custom(
            OperationFailed::new(&err.to_string(), ErrorCode::OperationFailed)
          ))
        },
    }
    
  })
}

fn decode_ref_key(
  chain: &'static str,
  db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  warp::path("api")
  .and(warp::path("v1"))
  .and(warp::path("refKey"))
  .and(with_db(db))
  .and(warp::path("decode"))
  .and(warp::path::end())
  .and(validate_ref_key_options())
  .and_then(move |db: Database, options: RefKeyOptions| async move {
    match db.decode_validator_ref_key(&options.ref_key).await {
        Ok(c) => {
          Ok(warp::reply::with_status(
            c.stash,
            StatusCode::OK,
          ))
        },
        Err(err) => {
          Err(warp::reject::custom(
            OperationFailed::new(&err.to_string(), ErrorCode::OperationFailed)
          ))
        },
    }
  })
}

pub fn get_routes(
    chain: &'static str,
    db: Database,
    user_db: Database,
    cache: Cache,
    src_path: String
) -> impl Filter<Extract = (impl Reply,), Error = warp::Rejection> + Clone {
    warp::get().and(get_all_validators(chain, db.clone(), cache.clone())
    .or(get_nominator_info(chain, db.clone()))
    .or(get_all_nominators(chain, cache.clone()))
    .or(get_nominated_validators(chain, db.clone(), cache.clone()))
    .or(get_validator_history(chain, db.clone()))
    .or(get_1kv_validators(chain, cache.clone()))
    .or(get_1kv_nominators(chain, cache.clone()))
    .or(get_stash_rewards_collector(src_path.clone()))
    .or(get_stash_rewards_collector_csv(src_path.clone()))
    .or(get_stash_rewards_collector_json(src_path))
    .or(get_validator_unclaimed_eras(chain, db.clone()))
    .or(get_validator_slashes(chain, db.clone())))
    .or(get_events(chain, db, user_db, cache))
}

pub fn post_routes(
  chain: &'static str,
  db: Database,
  chain_db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
  post_nominated_records(chain, db.clone())
  .or(post_subscribe_newsletter(db.clone()))
  .or(post_nominated_result(chain, db.clone()))
  .or(verify_ref_key(chain, db.clone()))
  .or(get_ref_key(chain, chain_db, db.clone()))
  .or(decode_ref_key(chain, db))
}
