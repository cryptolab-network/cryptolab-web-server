// use super::super::cache;
use super::super::db::Database;
use super::params::{AllValidatorOptions, InvalidParam};
use std::{convert::Infallible};
use warp::http::StatusCode;
use warp::{Filter, Rejection};

fn validate_get_all_validators() -> impl Filter<Extract = (AllValidatorOptions,), Error = Rejection> + Copy {
  warp::filters::query::query().and_then(|params: AllValidatorOptions| async move {
    if !(0.0..=1.0).contains(&params.apy_max()) {
      return Err(warp::reject::custom(InvalidParam));
    }
    if !(0.0..=1.0).contains(&params.apy_min()) {
      return Err(warp::reject::custom(InvalidParam));
    }
    if !(0.0..=1.0).contains(&params.commission_min()) {
      return Err(warp::reject::custom(InvalidParam));
    }
    if !(0.0..=1.0).contains(&params.commission_max()) {
      return Err(warp::reject::custom(InvalidParam));
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
        get_data_from_db(db, chain_info.active_era, p).await
    })
}

fn with_db(
    db: Database,
) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn get_data_from_db(
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
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get_all_validators(chain, db)
}
