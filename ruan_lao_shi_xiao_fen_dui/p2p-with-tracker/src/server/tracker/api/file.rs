use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, log::LevelFilter};

use crate::config::Config;
use crate::server::logic::file::progress_report;
use crate::server::store::piece::SeedInfo;
use crate::server::{error::*, logic};

pub async fn report(
    Extension(config): Extension<Config>,
    Json(request): Json<ReportRequest>,
) -> Result<Json<ReportResponse>> {
    debug!(
        "Receive piece report from node: {piece_id}, pieceId: {node_id} progress: {progress} filename: {filename}",
        piece_id = request.piece_id,
        node_id = request.node_id,
        progress = request.progress,
        filename = request.filename,
    );

    if request.node_id == "" || request.piece_id == "" {
        return Err(Error::ParamMissing);
    }

    // Update to progress info
    progress_report(
        request.node_id.as_str(),
        request.piece_id.as_str(),
        request.progress,
        request.filename.as_str(),
    )
    .await?;

    let response = ReportResponse {
        code: StatusCode::OK.to_string(),
        msg: "ok".to_string(),
    };

    Ok(Json(response))
}

pub async fn fetch_seed(
    Extension(config): Extension<Config>,
    Json(request): Json<FetchSeedRequest>,
) -> Result<Json<FetchSeedResponse>> {
    debug!(
        "Receive fetch seed filename: {filename}",
        filename = request.filename,
    );

    if request.filename == "" {
        return Err(Error::ParamMissing);
    }

    let response = match logic::file::fetch_seed(config, request.filename.as_str()).await {
        Ok(seed_info) => FetchSeedResponse {
            code: StatusCode::OK.to_string(),
            msg: "ok".to_string(),
            seed_info: seed_info,
        },
        Err(e) => FetchSeedResponse {
            code: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
            msg: e.to_string(),
            seed_info: SeedInfo::new("".to_string(), 0, vec![]),
        },
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct ReportRequest {
    pub node_id: String,
    pub piece_id: String,
    pub progress: f64,
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReportResponse {
    pub code: String,
    pub msg: String,
}

#[derive(Debug, Deserialize)]
pub struct FetchSeedRequest {
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct FetchSeedResponse {
    pub code: String,
    pub msg: String,
    pub seed_info: SeedInfo,
}
