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
        let hello_world = warp::path::end().map(|| "Hello, World at root!");
        let routes = warp::get().and(
            hello_world
        );
        let routes = routes.or(kusama::routes(self.db.clone())).with(warp::compression::gzip());
        routes
    }

    pub async fn start(&self) {
        let routes = self.initialize_routes();
        warp::serve(routes)
        .run(([127, 0, 0, 1], self.port))
        .await;
    }
}