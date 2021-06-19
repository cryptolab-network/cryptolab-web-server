use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible};
use super::params::{AllValidatorOptions, ValidDetailOptions};
use warp::http::StatusCode;
use warp::Filter;

use super::super::db::Database;
use super::super::cache;

fn get_validators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let path = warp::path("validators")
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_validators("DOT")));
    path
}

fn get_1kv_validators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let path = warp::path("valid")
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_1kv_info_detail("DOT")));
    path
}

fn get_validator_trend(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("validator")
        .and(with_db(db))
        .and(warp::path::param())
        .and(warp::path("trend"))
        .and(warp::path::end())
        .and_then(|db: Database, stash: String| async move {
            let validator = db.get_validator(stash).await;
            match validator {
                Ok(v) => Ok(warp::reply::json(&[v])),
                Err(_) => Err(warp::reject::not_found()),
            }
        })
}

fn get_validator_unclaimed_eras(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("validator")
        .and(with_db(db))
        .and(warp::path::param())
        .and(warp::path("unclaimedEras"))
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

async fn get_data_from_db(
    db: Database,
    era: u32,
    size: Option<u32>,
    page: Option<u32>,
    apy_min: Option<f32>,
    apy_max: Option<f32>,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    let result = db.get_all_validator_info_of_era(era, page.unwrap_or(0),
     size.unwrap_or(4000), apy_min.unwrap_or(0.0), apy_max.unwrap_or(100.0)).await;
    Ok(warp::reply::with_status(
        warp::reply::json(&result.unwrap()),
        StatusCode::OK,
    ))
}

fn get_validator_detail() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let path = warp::path("validDetail")
        .and(warp::path::end())
        .and(warp::query().map(|opt: ValidDetailOptions| {
            if opt.option == "1kv" {
                warp::reply::json(&cache::get_1kv_info_simple("DOT"))
            } else if opt.option == "all" {
                warp::reply::json(&cache::get_validators("DOT"))
            } else {
                warp::reply::json(&cache::get_validators("DOT"))
            }
        }));
    path
}

fn get_nominators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let path = warp::path("nominators")
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_nominators("DOT")));
    path
}

fn get_1kv_nominators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let path = warp::path("1kv")
        .and(warp::path("nominators"))
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_1kv_nominators("DOT")));
    path
}

fn get_nominated_validators(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("nominated")
        .and(with_db(db))
        .and(warp::path("stash"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(|db: Database, stash: String| async move {
            let result = cache::get_nominator("DOT", stash);
            match result {
                Ok(nominator) => {
                    let chain_info = db.get_chain_info().await.unwrap();
                    let result = db
                        .get_validator_info(&nominator.targets, &chain_info.active_era)
                        .await;
                    match result {
                        Ok(validators) => Ok(warp::reply::json(&validators)),
                        Err(_) => Err(warp::reject::not_found()),
                    }
                }
                Err(_) => Err(warp::reject::not_found()),
            }
        })
}

fn get_stash_rewards(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("stash")
        .and(with_db(db))
        .and(warp::path::param())
        .and(warp::path("rewards"))
        .and(warp::path::end())
        .and_then(|mut db: Database, stash: String| async move {
            let validator = db.get_stash_reward(&stash).await;
            match validator {
                Ok(v) => Ok(warp::reply::json(&v)),
                Err(_) => Err(warp::reject::not_found()),
            }
        })
}

async fn handle_query_parameter_err(
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    Ok(warp::reply::with_status(
        warp::reply::json(&""),
        StatusCode::BAD_REQUEST,
    ))
}

pub fn routes(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // let routes_v2 = get_all_validators_formal(db.clone());
    let routes = warp::get()
        .and(warp::path("api"))
        .and(warp::path("dot"))
        .and(
            get_validators()
                .or(get_validator_detail())
                .or(get_validator_trend(db.clone()))
                .or(get_nominators())
                .or(get_nominated_validators(db.clone()))
                .or(get_validator_unclaimed_eras(db.clone()))
                .or(get_stash_rewards(db.clone()))
                .or(get_1kv_validators())
                .or(get_1kv_nominators())
                .or(warp::path("allValidators")
                    .and(warp::path::end())
                    .and(with_db(db.clone()))
                    .and(warp::query::<HashMap<String, String>>())
                    .and_then(|db: Database, p: HashMap<String, String>| async move {
                        match p.get("size") {
                            Some(_) => {
                                let chain_info = db.get_chain_info().await.unwrap();
                                get_data_from_db(db, chain_info.active_era, None, None, None, None).await
                            }
                            None => handle_query_parameter_err().await,
                        }
                    })),
        );
    routes
}
