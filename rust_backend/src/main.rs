#[macro_use]
extern crate log;
mod data;
mod docker;
mod files;
mod id_generator;
mod server;
mod task_queue;
mod tasks;
use axum::{
    routing::{get, post},
    Router,
};
use env_logger::{Builder, Env, Logger};
use log::LevelFilter;
use std::net::SocketAddr;
use tokio::task;

async fn start_queue_thread() {
    data::queue_thread().await;
}

#[tokio::main]
async fn main() {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info);
    builder.init();

    info!("Initializing");

    #[cfg(not(unix))]
    {
        warn!("Warning! Running on Windows. Docker will be unavailable!");
    }

    info!("Starting task queue thread");
    tokio::spawn(async {
        start_queue_thread().await;
    });
    // tracing_subscriber::fmt::init();

    info!("Starting axum router");
    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(server::root))
        .route("/upload", post(server::upload));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
