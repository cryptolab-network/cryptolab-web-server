use crate::cache_redis::Cache;
use serde::Deserialize;
use crate::staking_rewards_collector::StakingRewardsCollector;
use crate::staking_rewards_collector::{StakingRewardsAddress, StakingRewardsReport};
use crate::types::ValidatorNominationInfo;
use crate::web::Invalid;

// use super::super::cache;
use super::super::db::Database;
use super::params::ErrorCode;
use super::params::{AllValidatorOptions, InvalidParam};
use std::{convert::Infallible};
use log::{debug, error};
use warp::http::StatusCode;
use warp::{Filter, Rejection};

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
        let chain_info = db.get_chain_info().await.unwrap();
        let validators = get_validator_data_from_db(db, cache, chain.to_string(), chain_info.active_era, p).await;
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
    let nominator = db.get_nominator_info(id).await;
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
          let src = StakingRewardsCollector::new(p.start.unwrap_or(start), p.end.unwrap_or(end),
          p.currency.unwrap_or(currency), p.price_data.unwrap_or(true),
          vec![StakingRewardsAddress::new("".to_string(), stash.clone(), p.start_balance.unwrap_or(0.0))]);
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
              let chain_info = db.get_chain_info().await;
              match chain_info {
                  Ok(chain_info) => {
                      let result = db
                      .get_validator_info(&nominator.targets, &chain_info.active_era)
                      .await;
                      match result {
                          Ok(validators) => Ok(warp::reply::json(&validators)),
                          Err(_) => Err(warp::reject::not_found()),
                      }
                  },
                  Err(_) => {
                      Err(warp::reject::not_found())
                  },
              }
          }
          Err(_) => {
              error!("{}", "failed to get nominated list from the cache");
              Err(warp::reject::not_found())
          }
      }
  })
}

pub fn routes(
    chain: &'static str,
    db: Database,
    cache: Cache,
    src_path: String
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_all_validators(chain, db.clone(), cache.clone())
    .or(get_nominator_info(chain, db.clone()))
    .or(get_all_nominators(chain, cache.clone()))
    .or(get_nominated_validators(chain, db.clone(), cache.clone()))
    .or(get_validator_history(chain, db.clone()))
    .or(get_1kv_validators(chain, cache.clone()))
    .or(get_1kv_nominators(chain, cache))
    .or(get_stash_rewards_collector(src_path.clone()))
    .or(get_stash_rewards_collector_csv(src_path.clone()))
    .or(get_stash_rewards_collector_json(src_path))
    .or(get_validator_unclaimed_eras(chain, db))
}
