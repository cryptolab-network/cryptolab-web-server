use std::{collections::HashMap, convert::Infallible};
use warp::Filter;
use warp::http::{StatusCode};
use super::super::cache;
use super::super::db::Database;

const API_PREFIX: &str =  "/api";
const ONEKV_PREFIX: &str = "/1kv";

// const API = {
//   ValidCandidates: API_PREFIX + '/valid',
//   OnekvNominators: API_PREFIX + ONEKV_PREFIX + '/nominators',
//   Nominators: API_PREFIX + '/nominators',
//   Nominator: API_PREFIX + 'nominator/:stash',
//   Statistic: API_PREFIX + '/statistic/:stash',
//   FalseNominations: API_PREFIX + '/falseNominations',
//   Validators: API_PREFIX + '/validators',
//   onekvlist: API_PREFIX + '/onekvlist',
//   ValidDetail: API_PREFIX + '/validDetail',
//   test: API_PREFIX + '/test',
//   polkadot: API_PREFIX + '/polkadot/:stash',
//   kusama: API_PREFIX + '/kusama/:stash',
//   validatorTrend: API_PREFIX + '/validator/:stash/trend',
//   validatorDetail: API_PREFIX + '/validator/:stash',
//   AllValidators: API_PREFIX + '/allValidators',
// }

fn get_validators() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    let path = warp::path("api").and(warp::path("validators")).and(warp::path::end())
        .map(|| warp::reply::json(&cache::get_validators()));
    path
}

fn with_db(db: Database) -> impl Filter<Extract = (Database,), Error=std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

async fn get_data_from_db(db: Database, era: u32) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    let result = db.get_all_validator_info_of_era(era, 0, 2000).await;
    Ok(warp::reply::with_status(warp::reply::json(&result.unwrap()), StatusCode::OK))
}

async fn handle_query_parameter_err() -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    Ok(warp::reply::with_status(warp::reply::json(&""), StatusCode::BAD_REQUEST))
}

pub fn routes(db: Database) -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    let hello_world = warp::path::end().map(|| "Hello, World at root Kusama!");
    let routes = warp::get().and(
        hello_world
    );
    let routes = routes.or(get_validators()).or(
        warp::path("api").and(warp::path("allValidators")).and(warp::path::end())
        .and(with_db(db))
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
    );
    routes
}
