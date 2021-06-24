use crate::config::Config;
use crate::staking_rewards_collector::{StakingRewardsAddress, StakingRewardsReport};
use crate::staking_rewards_collector::StakingRewardsCollector;
use crate::web::Invalid;

use super::super::cache;
use super::super::db::Database;
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible};
use warp::http::StatusCode;
use warp::Filter;

#[derive(Deserialize)]
struct ValidDetailOptions {
    option: String,
}

#[derive(Deserialize)]
struct StakingRewardsOptions {
    pub start: Option<String>,
    pub end: Option<String>,
    pub currency: Option<String>,
    pub price_data: Option<bool>,
    pub start_balance: Option<f64>
}

fn get_validators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let path = warp::path("api")
        .and(warp::path("validators"))
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_validators("KSM")));
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
        .map(|| warp::reply::json(&cache::get_1kv_info_detail("KSM")));
    path
}

fn with_db(
    db: Database,
) -> impl Filter<Extract = (Database,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_string(
    s: String
) -> impl Filter<Extract = (String,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || s.clone())
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
                warp::reply::json(&cache::get_1kv_info_simple("KSM"))
            } else if opt.option == "all" {
                warp::reply::json(&cache::get_validators("KSM"))
            } else {
                warp::reply::json(&cache::get_validators("KSM"))
            }
        }));
    path
}

fn get_nominators() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let path = warp::path("api")
        .and(warp::path("nominators"))
        .and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_nominators("KSM")));
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
            let result = cache::get_nominator("KSM", stash);
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
        .map(|| warp::reply::json(&cache::get_1kv_nominators("KSM")));
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

fn get_stash_rewards_collector(src_path: String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
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
            let result = src.call_exe(src_path.to_string());
            match result {
                Ok(v) => {
                    Ok(warp::reply::json(&v))
                },
                Err(err) => {
                    println!("{}", err);
                    if err.err_code == -2 {
                        Err(warp::reject::not_found())
                    } else {
                        Err(warp::reject::custom(err))
                    }
                },
            }
        })
}

fn get_stash_rewards_collector_csv(src_path: String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("api")
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
                println!("{}", stash);
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
                println!("{}", stash);
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
        .or(get_stash_rewards_collector(Config::current().staking_rewards_collector_dir.to_string()))
        .or(get_stash_rewards_collector_csv(Config::current().staking_rewards_collector_dir.to_string()))
        .or(get_stash_rewards_collector_json(Config::current().staking_rewards_collector_dir.to_string()))
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
