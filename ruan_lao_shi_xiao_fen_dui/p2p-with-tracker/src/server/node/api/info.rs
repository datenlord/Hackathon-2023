use std::collections::HashSet;

use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, log::LevelFilter};

use crate::config::Config;
use crate::server::error::*;
use crate::server::store::piece::{PieceInfo, PieceId};
use crate::server::store::{
    NODE_INFO_CACHE, PIECE_INFO_CACHE, PIECE_NODE_INFO_CACHE, SESSION_INFO_CACHE, PIECE_CACHE, SEED_INFO_CACHE,
};

pub async fn info(Extension(config): Extension<Config>) -> Result<Json<InfoResponse>> {
    debug!("Get node info");

    let piece_info_cache: Vec<(PieceId, PieceInfo)> = PIECE_INFO_CACHE.get_all();
    let piece_cache: Vec<(String, Vec<u8>)> = PIECE_CACHE.get_all();

    let response = InfoResponse {
        code: StatusCode::OK.to_string(),
        msg: "ok".to_string(),
        piece_info_cache,
        piece_cache,
    };

    Ok(Json(response))
}

#[derive(Serialize, Deserialize)]
pub struct InfoResponse {
    pub code: String,
    pub msg: String,
    pub piece_info_cache: Vec<(PieceId, PieceInfo)>,
    pub piece_cache: Vec<(String, Vec<u8>)>,
}
