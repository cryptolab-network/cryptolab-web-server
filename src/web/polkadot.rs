use log::error;
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible};
use super::params::AllValidatorOptions;
use super::params::{ValidDetailOptions};
use warp::http::StatusCode;
use warp::Filter;
use crate::cache_redis::Cache;
use crate::config::Config;
use crate::staking_rewards_collector::StakingRewardsReport;
use crate::web::Invalid;

use super::super::staking_rewards_collector::{StakingRewardsCollector, StakingRewardsAddress};

use super::super::db::Database;

#[derive(Deserialize, Debug)]
struct StakingRewardsOptions {
    pub start: Option<String>,
    pub end: Option<String>,
    pub currency: Option<String>,
    pub price_data: Option<bool>,
    pub start_balance: Option<f64>
}

fn get_validators(cache: Cache) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("validators")
    .and(warp::path::end())
    .map(move || warp::reply::json(&cache.get_validators("DOT")))
}

fn get_1kv_validators(cache: Cache) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path("valid")
    .and(warp::path::end())
    .map(move || warp::reply::json(&cache.get_1kv_info_detail("DOT")))
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

fn get_validator_detail(cache: Cache) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path("validDetail")
    .and(warp::path::end())
    .and(warp::query().map(move |opt: ValidDetailOptions| {
        if opt.option == "1kv" {
            warp::reply::json(&cache.get_1kv_info_simple("DOT"))
        } else {
            warp::reply::json(&cache.get_validators("DOT"))
        }
    }))
}

fn get_nominators(cache: Cache) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("nominators")
    .and(warp::path::end())
    .map(move || warp::reply::json(&cache.get_nominators("DOT")))
}

fn get_1kv_nominators(cache: Cache) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::path("1kv")
    .and(warp::path("nominators"))
    .and(warp::path::end())
    .map(move || warp::reply::json(&cache.get_1kv_nominators("DOT")))
}

fn get_nominated_validators(
    cache: Cache,
    db: Database,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("nominated")
    .and(with_db(db))
    .and(with_cache(cache))
    .and(warp::path("stash"))
    .and(warp::path::param())
    .and(warp::path::end())
    .and_then(|db: Database, cache: Cache, stash: String| async move {
        let result = cache.get_nominator("DOT", stash);
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

fn get_stash_rewards_collector(src_path: String) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("stash")
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
        vec![StakingRewardsAddress::new("".to_string(), stash.clone(), p.start_balance.unwrap_or(0.0), "Polkadot".to_string())]);
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
    warp::path("stash")
        .and(warp::path::param())
        .and(with_string(src_path))
        .and(warp::path("rewards"))
        .and(warp::path("collector"))
        .and(warp::path("csv"))
        .and(warp::path::end())
        .and_then(|stash: String, src_path: String| async move{
            // validate stash
            if !stash.chars().all(char::is_alphanumeric) {
                error!("{}", stash);
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
    warp::path("stash")
        .and(warp::path::param())
        .and(with_string(src_path))
        .and(warp::path("rewards"))
        .and(warp::path("collector"))
        .and(warp::path("json"))
        .and(warp::path::end())
        .and_then(|stash: String, src_path: String| async move{
            // validate stash
            if !stash.chars().all(char::is_alphanumeric) {
                error!("{}", stash);
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
    cache: Cache,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    // let routes_v2 = get_all_validators_formal(db.clone());
    warp::get()
    .and(warp::path("api"))
    .and(warp::path("dot"))
    .and(
        get_validators(cache.clone())
            .or(get_validator_detail(cache.clone()))
            .or(get_validator_trend(db.clone()))
            .or(get_nominators(cache.clone()))
            .or(get_nominated_validators(cache.clone(), db.clone()))
            .or(get_validator_unclaimed_eras(db.clone()))
            .or(get_stash_rewards(db.clone()))
            .or(get_stash_rewards_collector(Config::current().staking_rewards_collector_dir.to_string()))
            .or(get_stash_rewards_collector_csv(Config::current().staking_rewards_collector_dir.to_string()))
            .or(get_stash_rewards_collector_json(Config::current().staking_rewards_collector_dir.to_string()))
            .or(get_1kv_validators(cache.clone()))
            .or(get_1kv_nominators(cache))
            .or(warp::path("allValidators")
                .and(warp::path::end())
                .and(with_db(db))
                .and(warp::query::<HashMap<String, String>>())
                .and_then(|db: Database, p: HashMap<String, String>| async move {
                    match p.get("size") {
                        Some(_) => {
                            let chain_info = db.get_chain_info().await.unwrap();
                            get_data_from_db(db, chain_info.active_era, AllValidatorOptions::new()).await
                        }
                        None => handle_query_parameter_err().await,
                    }
                })),
    )
}
