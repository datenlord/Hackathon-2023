use reqwest::Client;
use serde::{Deserialize, Serialize};

use reqwest::Error;

use crate::server::store::piece::SeedInfo;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PieceClient {
    endpoint: String, // host:port
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

impl PieceClient {
    pub fn new(endpoint: &str) -> Self {
        PieceClient {
            endpoint: endpoint.to_string(),
        }
    }

    pub async fn fetch_piece(
        &self,
        piece_id: &str,
    ) -> Result<FetchPieceResponse, Error> {
        let url = format!("http://{}/api/v1/fetch_piece", self.endpoint);
        let client = Client::new();
        let payload = FetchPieceRequest {
            piece_id: piece_id.to_string(),
        };

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let response_body: FetchPieceResponse = response.json().await?;

        Ok(response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_report() {
        let client = PieceClient::new("127.0.0.1:48349");

        let piece_id = "88120779-00bc-4912-a2a5-c53f52297a03";

        let response = client
            .fetch_piece(piece_id)
            .await
            .unwrap();
        println!("{:?}", response);
    }
}
