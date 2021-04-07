use warp::Filter;
use super::db::Database;
mod kusama;

pub struct WebServer {
    port: u16,
    db: Database,
}

impl WebServer {
    pub fn new(port: u16, db: Database) -> Self {
        WebServer {
            port: port,
            db: db
        }
    }

    fn initialize_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let routes = kusama::routes(self.db.clone());
        routes
    }

    pub async fn start(&self) {
        let cors = warp::cors()
        .allow_origin("http://127.0.0.1:8080")
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers", "Content-Type"])
        .allow_methods(&[warp::http::Method::GET, warp::http::Method::OPTIONS]);

        let routes = self.initialize_routes().with(cors).with(warp::compression::gzip()).with(warp::log("warp_request"));
        warp::serve(routes)
        .run(([127, 0, 0, 1], self.port))
        .await;
    }
}