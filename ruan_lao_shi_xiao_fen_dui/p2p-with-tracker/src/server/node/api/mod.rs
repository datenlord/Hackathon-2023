pub mod info;
pub mod file;
pub mod node;

use axum::routing::*;

pub async fn health() -> &'static str {
    "Node is running!"
}

pub fn app() -> Router {
    Router::new()
        .route("/start_download", post(node::start_download))
        .route("/fetch_piece", post(file::fetch_piece))
        .route("/info", get(info::info))
        .route("/health", get(health))
}
