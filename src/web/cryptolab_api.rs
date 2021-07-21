use crate::cache_redis::Cache;

// use super::super::cache;
use super::super::db::Database;
use super::params::ErrorCode;
use super::params::{AllValidatorOptions, InvalidParam};
use std::{convert::Infallible};
use warp::http::StatusCode;
use warp::{Filter, Rejection};

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

fn get_all_validators(chain: &'static str, db: Database) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
    .and(warp::path("v1"))
    .and(warp::path("validators"))
    .and(warp::path(chain))
    .and(warp::path::end())
    .and(with_db(db))
    .and(validate_get_all_validators())
    .and_then(|db: Database, p: AllValidatorOptions| async move {
        let chain_info = db.get_chain_info().await.unwrap();
        get_validator_data_from_db(db, chain_info.active_era, p).await
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

fn with_db(
    db: Database,
) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn get_validator_data_from_db(
    db: Database,
    era: u32,
    options: AllValidatorOptions,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    let result = db.get_all_validator_info_of_era(era, options.to_db_all_validator_options()).await;
    Ok(warp::reply::with_status(
        warp::reply::json(&result.unwrap()),
        StatusCode::OK,
    ))
}

pub fn routes(
    chain: &'static str,
    db: Database,
    cache: Cache
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_all_validators(chain, db.clone())
    .or(get_nominator_info(chain, db.clone()))
    .or(get_all_nominators(chain, cache.clone()))
    .or(get_validator_history(chain, db))
    .or(get_1kv_validators(chain, cache.clone()))
    .or(get_1kv_nominators(chain, cache))
}
