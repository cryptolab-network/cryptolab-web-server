use crate::staking_rewards_collector::SRCError;

use super::db::Database;
use warp::Filter;
use warp::reject::Reject;
mod kusama;
mod polkadot;
use super::config::Config;

impl Reject for SRCError {}
#[derive(Debug)]
struct Invalid;
impl Reject for Invalid {}

pub struct WebServerOptions {
    pub kusama_db: Database,
    pub polkadot_db: Database,
}

pub struct WebServer {
    port: u16,
    kusama_db: Database,
    polkadot_db: Database,
}

impl WebServer {
    pub fn new(port: u16, options: WebServerOptions) -> Self {
        WebServer {
            port: port,
            kusama_db: options.kusama_db,
            polkadot_db: options.polkadot_db,
        }
    }

    fn initialize_routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let routes = kusama::routes(self.kusama_db.clone())
            .or(polkadot::routes(self.polkadot_db.clone()));
            // .recover(|error: Rejection| async move {
            //     // Do prettier error reporting for the default error here.
            //     if error.is_not_found() {
            //         Ok(warp::reply::with_status(
            //             String::from("Data not found"),
            //             StatusCode::NOT_FOUND,
            //         ))
            //     } else {
            //         Ok(warp::reply::with_status(
            //             String::from("Internal error"),
            //             StatusCode::INTERNAL_SERVER_ERROR,
            //         ))
            //     }
            // });
        routes
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
            .allow_methods(&[warp::http::Method::GET, warp::http::Method::OPTIONS]);
        let routes = warp::fs::dir("./www/static");
        let tool_routes = warp::path("tools").and(warp::fs::dir("./www/static"));
        let validator_status_routes = warp::path("tools").and(warp::path("validatorStatus")).and(warp::fs::dir("./www/static"));
        let ksmvn_routes = warp::path("tools").and(warp::path("ksmVN")).and(warp::fs::dir("./www/static"));
        let dotvn_routes = warp::path("tools").and(warp::path("dotVN")).and(warp::fs::dir("./www/static"));
        let dotsr_routes = warp::path("tools").and(warp::path("dotSR")).and(warp::fs::dir("./www/static"));
        let onekv_routes = warp::path("tools").and(warp::path("oneKValidators")).and(warp::fs::dir("./www/static"));
        let onekv_dot_routes = warp::path("tools").and(warp::path("oneKValidatorsDot")).and(warp::fs::dir("./www/static"));
        let contact_routes = warp::path("contact").and(warp::fs::dir("./www/static"));

        let api_routes = self
            .initialize_routes()
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
