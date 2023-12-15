use reqwest::Client;
use serde::{Deserialize, Serialize};

use reqwest::Error;

use crate::server::store::piece::SeedInfo;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileClient {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportRequest {
    pub node_id: String,
    pub piece_id: String,
    pub progress: f64,
    pub filename: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportResponse {
    pub code: String,
    pub msg: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FetchSeedRequest {
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub struct FetchSeedResponse {
    pub code: String,
    pub msg: String,
    pub seed_info: SeedInfo,
}

impl FileClient {
    pub fn new(host: &str, port: u16) -> Self {
        FileClient {
            host: host.to_string(),
            port: port,
        }
    }

    pub async fn report(
        &self,
        node_id: &str,
        piece_id: &str,
        progress: f64,
        filename: &str,
    ) -> Result<ReportResponse, Error> {
        let url = format!("http://{}:{}/api/v1/report", self.host, self.port);
        let client = Client::new();
        let payload = ReportRequest {
            node_id: node_id.to_string(),
            piece_id: piece_id.to_string(),
            progress: progress,
            filename: filename.to_string(),
        };

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let response_body: ReportResponse = response.json().await?;

        Ok(response_body)
    }

    pub async fn fetch_seed(&self, filename: &str) -> Result<FetchSeedResponse, Error> {
        let url = format!("http://{}:{}/api/v1/fetch_seed", self.host, self.port);
        let client = Client::new();
        let payload = FetchSeedRequest {
            filename: filename.to_string(),
        };

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let response_body: FetchSeedResponse = response.json().await?;

        Ok(response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_report() {
        let client = FileClient::new("127.0.0.1", 38080);

        let node_id = "D526D2F5-920F-42A7-B5B9-BC643AC6B71B";
        let piece_id = "D526D2F5-920F-42A7-B5B9-BC643AC6B71B";
        let progress = 0.5;
        let filename = "test.txt";

        let response = client
            .report(node_id, piece_id, progress, filename)
            .await
            .unwrap();
        println!("{:?}", response);
    }

    #[tokio::test]
    async fn test_fetch_seed() {
        let client = FileClient::new("127.0.0.1", 38080);

        let filename = "output_10MB.txt";

        let response = client.fetch_seed(filename).await.unwrap();
        println!("{:#?}", response.seed_info);
    }
}
