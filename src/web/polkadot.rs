use std::{collections::HashMap, convert::Infallible};
use warp::Filter;
use warp::http::{StatusCode};
use serde::Deserialize;
use super::super::cache;
use super::super::db::Database;

#[derive(Deserialize)]
struct ValidDetailOptions {
    option: String
}

fn get_validators() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    let path = warp::path("validators").and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_validators()));
    path
}

fn get_validator_trend(db: Database) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    warp::path("validator").and(with_db(db))
    .and(warp::path::param()).and(warp::path("trend")).and(warp::path::end())
    .and_then(|db: Database, stash: String| async move {
        let validator = db.get_validator(stash).await;
        match validator {
            Ok(v) => Ok(warp::reply::json(&[v])),
            Err(e) => Err(warp::reject::not_found())
        }
    })
}

fn get_1kv_validators() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    let path = warp::path("valid").and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_1kv_info_detail()));
    path
}

fn with_db(db: Database) -> impl Filter<Extract = (Database,), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn get_data_from_db(db: Database, era: u32) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    let result = db.get_all_validator_info_of_era(era, 0, 2000).await;
    Ok(warp::reply::with_status(warp::reply::json(&result.unwrap()), StatusCode::OK))
}

fn get_validator_detail() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    let path = warp::path("validDetail")
        .and(warp::path::end())
        .and(warp::query().map(|opt: ValidDetailOptions| 
            if opt.option == "1kv" {
                warp::reply::json(&cache::get_1kv_info_simple())
            }
            else if opt.option == "all" {
                warp::reply::json(&cache::get_validators())
            }
            else { 
                warp::reply::json(&cache::get_validators())
            }
        )
    );
    path
}

async fn handle_query_parameter_err() -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    Ok(warp::reply::with_status(warp::reply::json(&""), StatusCode::BAD_REQUEST))
}

pub fn routes(db: Database) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    let routes = warp::get().and(warp::path("api")).and(warp::path("dot"))
    .and(get_validators()
    .or(get_validator_detail())
    .or(get_validator_trend(db.clone()))
    .or(get_1kv_validators())
    .or(
        warp::path("allValidators").and(warp::path::end())
        .and(with_db(db.clone()))
        .and(warp::query::<HashMap<String, String>>())
        .and_then(|db: Database,p: HashMap<String, String>| async move { match p.get("size") {
            Some(size) => {
                let chain_info = db.get_chain_info().await.unwrap();
                get_data_from_db(db, chain_info.active_era).await
            },
            None => {
                handle_query_parameter_err().await
            }
        }})
    ));
    routes
}
