// use super::super::cache;
use super::super::db::Database;
use super::params::{AllValidatorOptions, InvalidParam};
use std::{convert::Infallible};
use warp::http::StatusCode;
use warp::{Filter, Rejection};

fn validate_get_all_validators() -> impl Filter<Extract = (AllValidatorOptions,), Error = Rejection> + Copy {
  warp::filters::query::query().and_then(|params: AllValidatorOptions| async move {
    if let Some(apy_max) = params.apy_max {
      if apy_max < 0.0 || apy_max > 100.0 {
        return Err(warp::reject::custom(InvalidParam));
      }
    }
    if let Some(apy_min) = params.apy_min {
      if apy_min < 0.0 || apy_min > 100.0 {
        return Err(warp::reject::custom(InvalidParam));
      }
    }
    if let Some(commission_min) = params.commission_min {
      if commission_min < 0.0 || commission_min > 100.0 {
        return Err(warp::reject::custom(InvalidParam));
      }
    }
    if let Some(commission_max) = params.commission_max {
      if commission_max < 0.0 || commission_max > 100.0 {
        return Err(warp::reject::custom(InvalidParam));
      }
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
    .and(with_db(db.clone()))
    .and(validate_get_all_validators())
    .and_then(|db: Database, p: AllValidatorOptions| async move {
        let chain_info = db.get_chain_info().await.unwrap();
        get_data_from_db(db, chain_info.active_era, p.size, p.page, p.apy_min, p.apy_max, p.commission_min, p.commission_max).await
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
    size: Option<u32>,
    page: Option<u32>,
    apy_min: Option<f32>,
    apy_max: Option<f32>,
    commission_min: Option<f32>,
    commission_max: Option<f32>,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    let result = db.get_all_validator_info_of_era(era, page.unwrap_or(0),
     size.unwrap_or(4000), apy_min.unwrap_or(0.0), apy_max.unwrap_or(1.0),
          commission_min.unwrap_or(0.0), commission_max.unwrap_or(1.0)).await;
    Ok(warp::reply::with_status(
        warp::reply::json(&result.unwrap()),
        StatusCode::OK,
    ))
}

pub fn routes(
    chain: &'static str,
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let routes = get_all_validators(chain, db.clone());
    routes
}
