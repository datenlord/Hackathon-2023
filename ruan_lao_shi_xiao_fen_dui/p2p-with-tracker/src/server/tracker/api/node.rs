use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, log::LevelFilter};

use crate::config::Config;
use crate::server::error::*;
use crate::server::logic::node::get_piece_node_list;

/// Get online node list by piece Id
pub async fn node_list(
    Extension(config): Extension<Config>,
    Json(request): Json<NodeListRequest>,
) -> Result<Json<NodeListResponse>> {
    debug!(
        "Receive query pieceId with pieceId: {piece_id}",
        piece_id = request.piece_id,
    );

    // Change to addr list
    let node_list = get_piece_node_list(request.piece_id.as_str()).await?;

    let response = NodeListResponse {
        code: StatusCode::OK.to_string(),
        msg: "ok".to_string(),
        node_list: node_list,
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct NodeListRequest {
    pub piece_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct NodeListResponse {
    pub code: String,
    pub msg: String,
    pub node_list: Vec<String>,
}
