pub mod error;
pub mod logic;
pub mod node;
pub mod store;
pub mod tracker;
pub mod utils;

use std::{
    path::{Path, PathBuf},
    process::exit,
};

use anyhow::anyhow;
use axum::{
    extract::Host,
    handler::HandlerWithoutStateExt,
    http::{HeaderValue, StatusCode, Uri},
    response::Redirect,
    routing::get,
    Extension, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use hyper::header::SERVER;
use rand::Rng;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    BoxError,
};
use tracing::{debug, error, info};

use crate::constant::*;
use crate::server::store::NODE_ID;
use crate::{
    config::Config,
    server::{node::task::check::HeartBeatTask, tracker::task::check::CheckTask},
};

pub async fn start_server(
    config: &mut Config,
    mode: String,
    filename: String,
) -> anyhow::Result<()> {
    let mut port = config.http.tracker_port;
    let mut app = Router::new();

    // Get router and port
    match mode.as_str() {
        SERVER_MODE_TRACKER => {
            app = create_tracker_router(config.to_owned()).await;
            CheckTask::new(
                &NODE_ID,
                config.p2p.session_check_time,
                config.p2p.session_expire_time,
            )
            .start();
        },
        SERVER_MODE_NODE => {
            app = create_node_router(config.to_owned()).await;
            // Get random port
            port = rand::thread_rng().gen_range(
                config.http.range_port_start.clone()..config.http.range_port_end.clone(),
            );
            HeartBeatTask::new(
                config.to_owned(),
                &NODE_ID,
                format!("{}:{}", config.http.host, port).as_str(),
                config.p2p.session_check_time,
            )
            .start();
        },
        _ => {
            error!("Invalid server mode");
            exit(1);
        },
    }

    let host = format!("{}:{}", config.http.host, port);

    info!(
        "🚀🚀🚀 Server [{}:{}] has launched on http://{}",
        mode,
        NODE_ID.as_str(),
        host
    );

    // change the type from Vec<String> to Vec<HeaderValue> so that the http server can correctly detect CORS hosts
    let origins = config
        .http
        .cors
        .iter()
        .map(|e| e.parse().expect("Failed to parse CORS hosts"))
        .collect::<Vec<HeaderValue>>();
    app = app.layer(CorsLayer::new().allow_origin(AllowOrigin::list(origins)));

    // start http server
    axum::Server::bind(&host.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Err(anyhow!("Server unexpected stopped!"))
}

pub async fn create_tracker_router(config: Config) -> Router<()> {
    let app = Router::new()
        .nest("/api/v1", tracker::api::app())
        .route("/", get(tracker::api::health))
        // .layer(CorsLayer::new().allow_origin(AllowOrigin::list(origins)))
        .layer(Extension(config.clone()));

    app
}

pub async fn create_node_router(config: Config) -> Router<()> {
    let app = Router::new()
    .nest("/api/v1", node::api::app())
    .route("/", get(node::api::health))
    // .layer(CorsLayer::new().allow_origin(AllowOrigin::list(origins)))
    .layer(Extension(config.clone()));

    app
}
