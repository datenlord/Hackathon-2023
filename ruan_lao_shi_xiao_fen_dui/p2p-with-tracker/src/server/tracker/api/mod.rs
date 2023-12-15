mod cluster;
mod file;
mod info;
mod node;

use axum::routing::*;

pub async fn health() -> &'static str {
    "Tracker is running!"
}

pub fn app() -> Router {
    Router::new()
        .route("/heart_beat", post(cluster::heart_beat))
        .route("/node_list", post(node::node_list))
        .route("/fetch_seed", post(file::fetch_seed))
        .route("/report", post(file::report))
        .route("/info", get(info::info))
        .route("/health", get(health))
}
