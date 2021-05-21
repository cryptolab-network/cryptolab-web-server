// use super::super::cache;
use super::super::db::Database;
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible};
use warp::http::StatusCode;
use warp::Filter;
use super::super::cache_redis as cache;

#[derive(Deserialize)]
struct ValidDetailOptions {
    option: String,
}

fn get_validators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let path = warp::path("api")
        .and(warp::path("validators"))
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_validators()));
    path
}

fn get_validator_trend(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("validator"))
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

fn get_1kv_validators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let path = warp::path("api")
        .and(warp::path("valid"))
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_1kv_info_detail()));
    path
}

fn with_db(
    db: Database,
) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn get_validator_unclaimed_eras(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("validator"))
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

async fn get_data_from_db(
    db: Database,
    era: u32,
) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    let result = db.get_all_validator_info_of_era(era, 0, 2000).await;
    Ok(warp::reply::with_status(
        warp::reply::json(&result.unwrap()),
        StatusCode::OK,
    ))
}

fn get_validator_detail() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let path = warp::path("api")
        .and(warp::path("validDetail"))
        .and(warp::path::end())
        .and(warp::query().map(|opt: ValidDetailOptions| {
            if opt.option == "1kv" {
                warp::reply::json(&cache::get_1kv_info_simple())
            } else if opt.option == "all" {
                warp::reply::json(&cache::get_validators())
            } else {
                warp::reply::json(&cache::get_validators())
            }
        }));
    path
}

fn get_nominators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let path = warp::path("api")
        .and(warp::path("nominators"))
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_nominators()));
    path
}

fn get_nominated_validators(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("nominated"))
        .and(with_db(db))
        .and(warp::path("stash"))
        .and(warp::path::param())
        .and(warp::path::end())
        .and_then(|db: Database, stash: String| async move {
            let result = cache::get_nominator(stash);
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
                Err(_) => {
                    println!("{}", "failed to get nominated list from the cache");
                    Err(warp::reject::not_found())
                }
            }
        })
}

fn get_1kv_nominators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    let path = warp::path("api")
        .and(warp::path("1kv"))
        .and(warp::path("nominators"))
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_1kv_nominators()));
    path
}

fn get_stash_rewards(
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
        .and(warp::path("stash"))
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
    let hello_world = warp::path::end().map(|| "Hello, World at root Kusama!");
    let routes = warp::get().and(hello_world);
    let routes = routes
        .or(get_validators())
        .or(get_validator_detail())
        .or(get_validator_trend(db.clone()))
        .or(get_1kv_validators())
        .or(get_nominators())
        .or(get_nominated_validators(db.clone()))
        .or(get_1kv_nominators())
        .or(get_validator_unclaimed_eras(db.clone()))
        .or(get_stash_rewards(db.clone()))
        .or(warp::path("api")
            .and(warp::path("allValidators"))
            .and(warp::path::end())
            .and(with_db(db.clone()))
            .and(warp::query::<HashMap<String, String>>())
            .and_then(|db: Database, p: HashMap<String, String>| async move {
                match p.get("size") {
                    Some(_) => {
                        let chain_info = db.get_chain_info().await.unwrap();
                        get_data_from_db(db, chain_info.active_era).await
                    }
                    None => handle_query_parameter_err().await,
                }
            }));
    routes
}
