use std::fs;

use axum::{Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, log::LevelFilter};

use crate::config::Config;
use crate::server::error::*;
use crate::server::logic::file;

// Start download
pub async fn start_download(
    Extension(config): Extension<Config>,
    Json(request): Json<StartDownloadRequest>,
) -> Result<Json<StartDownloadResponse>> {
    info!(
        "Receive start download request with filename: {filename}",
        filename = request.filename,
    );

    #[cfg(debug_assertions)]
    {
        // Sync in debug mode
        match file::download(config, &request.filename.as_str()).await {
            Ok(file) => {
                info!("Download success!");

                let response = StartDownloadResponse {
                    code: StatusCode::OK.to_string(),
                    msg: "ok".to_string(),
                    success_count: file.success_count as f64,
                    failed_count: file.failed_count as f64,
                    total_count: file.total_count as f64,
                };
            
                fs::write(request.filename.as_str(), file.data).unwrap();

                return Ok(Json(response))
            },
            Err(e) => {
                info!("Download failed!");
            },
        }

        // Check if the file is downloaded or not
    }

    #[cfg(not(debug_assertions))]
    {
        // Async in release mode
        tokio::spawn(async move {
            file::download(&request.filename.as_str()).await.unwrap();
        });
    }

    let response = StartDownloadResponse {
        code: StatusCode::OK.to_string(),
        msg: "ok".to_string(),
        success_count: 0.0,
        failed_count: 0.0,
        total_count: 0.0,
    };

    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct StartDownloadRequest {
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct StartDownloadResponse {
    pub code: String,
    pub msg: String,
    pub success_count: f64,
    pub failed_count: f64,
    pub total_count: f64,
}
