use serde::Serialize;
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum Error {
    // `/tracker` error
    #[error("Parameter is missing.")]
    ParamMissing,
    #[error("Fetch metadata failed.")]
    FetchMetadataFailed,

    // `/node` error
    #[error("Fetch seed info failed.")]
    FetchSeedInfoFailed,
    #[error("Download file failed.")]
    DownloadFailed,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "kebab-case")]
enum ResponseError {
    Error(String),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;

        #[cfg(debug_assertions)]
        error!("Error: {:?}", self);

        let status = match self {
            _ => StatusCode::BAD_REQUEST,
        };

        let mut response = axum::Json(ResponseError::Error(self.to_string())).into_response();
        *response.status_mut() = status;

        response
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
