use serde_json;
use warp::Filter;
use super::super::cache;

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

pub fn routes() -> impl Filter<Extract=impl warp::Reply, Error=warp::Rejection> + Clone {
    let hello_world = warp::path::end().map(|| "Hello, World at root Kusama!");
    let routes = warp::get().and(
        hello_world
    );
    let routes = routes.or(get_validators());
    routes
}