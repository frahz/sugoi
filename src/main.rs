mod api;
mod db;
mod status;

use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use api::get_api_routes;
use axum::response::Redirect;
use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, services::ServeDir};
use tracing::info;

use crate::db::Database;

pub struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Starting sugoi client");

    let db_path = get_db_path();
    info!("Database path: {}", db_path.display());
    let db = Database::new(db_path)
        .await
        .expect("Couldn't open database");

    let shared_state = Arc::new(AppState { db });

    let compression_layer = CompressionLayer::new()
        .br(true)
        .deflate(true)
        .gzip(true)
        .zstd(true);

    let app = Router::new()
        .route("/", get(status::status))
        .route("/status", get(|| async { Redirect::permanent("/") }))
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
        .unwrap_or(8080)
}

fn get_db_path() -> PathBuf {
    env::var("SUGOI_DB_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("db/sugoi.db"))
}
