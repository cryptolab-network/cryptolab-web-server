use warp::Filter;
mod kusama;

pub struct WebServer {
    port: u16,
}

impl WebServer {
    pub fn new(port: u16) -> Self {
        WebServer {
            port: port
        }
    }

    fn initialize_routes(&self) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let hello_world = warp::path::end().map(|| "Hello, World at root!");
        let routes = warp::get().and(
            hello_world
        );
        let routes = routes.or(kusama::routes());
        routes
    }

    pub async fn start(&self) {
        let routes = self.initialize_routes();
        warp::serve(routes)
        .run(([127, 0, 0, 1], self.port))
        .await;
    }
}