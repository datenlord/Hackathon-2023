use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, log::LevelFilter};

use crate::config::Config;
use crate::server::logic::node::{add_node, update_session};
use crate::server::{error::*, logic};

pub async fn heart_beat(
    Extension(config): Extension<Config>,
    Json(request): Json<HeartBeatRequest>,
) -> Result<Json<HeartBeatResponse>> {
    info!(
        "Receive heart beat from node: {node_id}, addr: {addr}",
        node_id = request.node_id,
        addr = request.addr
    );

    // Update session
    update_session(request.node_id.as_str()).await?;

    // Store node info
    add_node(request.node_id.as_str(), &request.addr.as_str()).await?;

    let response = HeartBeatResponse {
        code: StatusCode::OK.to_string(),
        msg: "ok".to_string(),
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct HeartBeatRequest {
    pub node_id: String,
    pub addr: String,
}

#[derive(Serialize, Deserialize)]
pub struct HeartBeatResponse {
    pub code: String,
    pub msg: String,
}
