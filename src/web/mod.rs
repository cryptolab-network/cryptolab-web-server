use std::convert::Infallible;

use crate::cache_redis::Cache;
use crate::staking_rewards_collector::SRCError;
use self::params::{InvalidParam, OperationFailed};

use super::db::Database;

use serde_json::json;
use warp::hyper::StatusCode;
use warp::{Filter, Rejection};
use warp::reject::Reject;
mod kusama;
mod polkadot;
mod cryptolab_api;
mod params;
use super::config::Config;

impl Reject for SRCError {}
#[derive(Debug)]
struct Invalid;
impl Reject for Invalid {}

pub struct WebServerOptions {
    pub kusama_db: Database,
    pub polkadot_db: Database,
    pub westend_db: Option<Database>,
    pub users_db: Database,
    pub cache: Cache,
}

pub struct WebServer {
    port: u16,
    kusama_db: Database,
    polkadot_db: Database,
    westend_db: Option<Database>,
    pub users_db: Database,
    cache: Cache,
}

impl WebServer {
    pub fn new(port: u16, options: WebServerOptions) -> Self {
        WebServer {
            port,
            kusama_db: options.kusama_db,
            polkadot_db: options.polkadot_db,
            westend_db: options.westend_db,
            users_db: options.users_db,
            cache: options.cache,
        }
    }

    fn initialize_legacy_routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        kusama::routes(self.kusama_db.clone(), self.cache.clone())
        .or(polkadot::routes(self.polkadot_db.clone(), self.cache.clone()))
    }

    fn initialize_routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        cryptolab_api::get_routes("KSM", self.kusama_db.clone(), self.cache.clone(),
        Config::current().staking_rewards_collector_dir.to_string())
        .or(cryptolab_api::get_routes("DOT", self.polkadot_db.clone(), self.cache.clone(),
        Config::current().staking_rewards_collector_dir.to_string()))
        .or(cryptolab_api::get_routes("WND", self.westend_db.clone().unwrap(), self.cache.clone(),
        Config::current().staking_rewards_collector_dir.to_string()))
        .or(cryptolab_api::post_routes("KSM", self.users_db.clone()))
        .or(cryptolab_api::post_routes("DOT", self.users_db.clone()))
    }

    pub async fn start(&self) {
        let config = Config::current();
        let origins: Vec<&str> = config.cors_url.iter().map(|s| &**s).collect();
        let cors = warp::cors()
            .allow_origins(origins)
            .allow_headers(vec![
                "User-Agent",
                "Sec-Fetch-Mode",
                "Referer",
                "Origin",
                "Access-Control-Request-Method",
                "Access-Control-Request-Headers",
                "Content-Type",
            ])
            .allow_methods(&[warp::http::Method::GET, warp::http::Method::POST, warp::http::Method::OPTIONS]);
        let routes = warp::fs::dir("./www/static");
        let tool_routes = warp::path("tools").and(warp::fs::dir("./www/static"));
        let validator_status_routes = warp::path("tools").and(warp::path("validatorStatus")).and(warp::fs::dir("./www/static"));
        let ksmvn_routes = warp::path("tools").and(warp::path("ksmVN")).and(warp::fs::dir("./www/static"));
        let dotvn_routes = warp::path("tools").and(warp::path("dotVN")).and(warp::fs::dir("./www/static"));
        let dotsr_routes = warp::path("tools").and(warp::path("dotSR")).and(warp::fs::dir("./www/static"));
        let onekv_routes = warp::path("tools").and(warp::path("oneKValidators")).and(warp::fs::dir("./www/static"));
        let onekv_dot_routes = warp::path("tools").and(warp::path("oneKValidatorsDot")).and(warp::fs::dir("./www/static"));
        let contact_routes = warp::path("contact").and(warp::fs::dir("./www/static"));

        let api_routes = 
            self.initialize_routes()
            .or(self.initialize_legacy_routes())
            .recover(handle_rejection)
            .with(cors)
            .with(warp::compression::gzip())
            .with(warp::log("warp_request"));
        if Config::current().serve_www.unwrap_or_default() {
            warp::serve(api_routes.or(routes).or(tool_routes).or(validator_status_routes)
            .or(ksmvn_routes).or(dotvn_routes).or(dotsr_routes)
            .or(onekv_routes).or(onekv_dot_routes).or(contact_routes)
            ).run(([0, 0, 0, 0], self.port)).await;
        } else {
            warp::serve(api_routes).run(([0, 0, 0, 0], self.port)).await;
        }
    }
}

async fn handle_rejection(err: Rejection) -> Result<warp::reply::WithStatus<warp::reply::Json>, Infallible> {
    if err.is_not_found() {
        Ok(warp::reply::with_status(
        warp::reply::json(&""),
        StatusCode::NOT_FOUND,
        ))
    } else if let Some(e) = err.find::<InvalidParam>() {
        Ok(warp::reply::with_status(
        warp::reply::json(&json! ({
            "message": e.message,
            "code": e.err_code
        })),
        StatusCode::BAD_REQUEST,
        ))
    } else if let Some(e) = err.find::<OperationFailed>() {
        Ok(warp::reply::with_status(
        warp::reply::json(&json! ({
            "message": e.message,
            "code": e.err_code
        })),
        StatusCode::BAD_REQUEST,
        ))
    } else {
        Ok(warp::reply::with_status(
        warp::reply::json(&""),
        StatusCode::BAD_REQUEST,
        ))
    }
}
