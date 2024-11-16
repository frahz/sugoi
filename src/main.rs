mod api;
mod status;

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use api::get_api_routes;
use axum::{routing::get, Router};
use status::Status;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing::info;

pub struct AppState {
    statuses: Arc<Mutex<Vec<Status>>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting sugoi client");

    let shared_state = Arc::new(AppState {
        statuses: Arc::new(Mutex::new(Vec::new())),
    });

    let compression_layer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    let app = Router::new()
        .route("/status", get(status::status_root))
        .route("/status/refresh", get(status::status_refresh))
        .nest("/api", get_api_routes())
        .with_state(shared_state)
        .nest_service("/assets", ServeDir::new(get_assets_dir()))
        .layer(compression_layer);

    let port = get_port();
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await.unwrap();
    info!("Listening on 0.0.0.0:{}", port);

    axum::serve(listener, app).await.unwrap();
}

fn get_assets_dir() -> PathBuf {
    env::var("ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets"))
}

fn get_port() -> u16 {
    env::var("PORT")
        .ok()
        .and_then(|port| port.parse().ok())
        .unwrap_or_else(|| 8080)
}
