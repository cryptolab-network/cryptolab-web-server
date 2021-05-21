use super::db::Database;
use warp::Filter;
mod kusama;
mod polkadot;
use super::config::Config;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::Rejection;

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
    ) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
        let routes = kusama::routes(self.kusama_db.clone())
            .or(polkadot::routes(self.polkadot_db.clone()))
            .recover(|error: Rejection| async move {
                // Do prettier error reporting for the default error here.
                if error.is_not_found() {
                    Ok(warp::reply::with_status(
                        String::from("Data not found"),
                        StatusCode::NOT_FOUND,
                    ))
                } else {
                    Ok(warp::reply::with_status(
                        String::from("Internal error"),
                        StatusCode::INTERNAL_SERVER_ERROR,
                    ))
                }
            });
        routes
    }

    pub async fn start(&self) {
        let cors = warp::cors()
            .allow_origin(Config::current().cors_url.as_str())
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

        let routes = self
            .initialize_routes()
            .with(cors)
            .with(warp::compression::gzip())
            .with(warp::log("warp_request"));
        warp::serve(routes).run(([127, 0, 0, 1], self.port)).await;
    }
}
