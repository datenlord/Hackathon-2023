use reqwest::Client;
use serde::{Deserialize, Serialize};

use reqwest::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeClient {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeListRequest {
    pub piece_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeListResponse {
    pub code: String,
    pub msg: String,
    pub node_list: Vec<String>,
}

impl NodeClient {
    pub fn new(host: &str, port: u16) -> Self {
        NodeClient {
            host: host.to_string(),
            port: port,
        }
    }

    pub async fn node_list(&self, piece_id: &str) -> Result<NodeListResponse, Error> {
        let url = format!("http://{}:{}/api/v1/node_list", self.host, self.port);
        let client = Client::new();
        let payload = NodeListRequest {
            piece_id: piece_id.to_string(),
        };

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let response_body: NodeListResponse = response.json().await?;

        Ok(response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_list() {
        let client = NodeClient::new("127.0.0.1", 38080);

        let piece_id = "D526D2F5-920F-42A7-B5B9-BC643AC6B71B";

        let response = client.node_list(piece_id).await.unwrap();
        println!("{:?}", response);
    }
}
