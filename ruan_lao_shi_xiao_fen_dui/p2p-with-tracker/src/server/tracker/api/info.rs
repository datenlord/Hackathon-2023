use std::collections::HashSet;

use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, log::LevelFilter};

use crate::config::Config;
use crate::server::error::*;
use crate::server::store::piece::PieceInfo;
use crate::server::store::{
    NODE_INFO_CACHE, PIECE_INFO_CACHE, PIECE_NODE_INFO_CACHE, SESSION_INFO_CACHE,
};

pub async fn info(Extension(config): Extension<Config>) -> Result<Json<InfoResponse>> {
    debug!("Get tracker info");

    let session_info: Vec<(String, u128)> = SESSION_INFO_CACHE.get_all();
    let node_info: Vec<(String, String)> = NODE_INFO_CACHE.get_all();
    let piece_node_info: Vec<(String, HashSet<String>)> = PIECE_NODE_INFO_CACHE.get_all();
    let piece_info: Vec<(String, PieceInfo)> = PIECE_INFO_CACHE.get_all();

    let response = InfoResponse {
        code: StatusCode::OK.to_string(),
        msg: "ok".to_string(),
        session_info,
        node_info,
        piece_node_info,
        piece_info,
    };

    Ok(Json(response))
}

#[derive(Serialize, Deserialize)]
pub struct InfoResponse {
    pub code: String,
    pub msg: String,
    pub session_info: Vec<(String, u128)>,
    pub node_info: Vec<(String, String)>,
    pub piece_node_info: Vec<(String, HashSet<String>)>,
    pub piece_info: Vec<(String, PieceInfo)>,
}
