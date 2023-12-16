use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, log::LevelFilter};

use crate::config::Config;
use crate::server::error::*;
use crate::server::store::PIECE_CACHE;

pub async fn fetch_piece(
    Extension(config): Extension<Config>,
    Json(request): Json<FetchPieceRequest>,
) -> Result<Json<FetchPieceResponse>> {
    debug!(
        "Receive fetch piece pieceId: {piece_id}",
        piece_id = request.piece_id,
    );

    match PIECE_CACHE.get(&request.piece_id) {
        Some(piece) => {
            let response = FetchPieceResponse {
                code: StatusCode::OK.to_string(),
                msg: "ok".to_string(),
                data: piece,
            };
            return Ok(Json(response))
        }
        None => {
            let response = FetchPieceResponse {
                code: StatusCode::OK.to_string(),
                msg: "Not find".to_string(),
                data: vec![],
            };
            return Ok(Json(response))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchPieceRequest {
    pub piece_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchPieceResponse {
    pub code: String,
    pub msg: String,
    pub data: Vec<u8>,
}