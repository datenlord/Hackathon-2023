use reqwest::Client;
use serde::{Deserialize, Serialize};

use reqwest::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionClient {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartBeatRequest {
    pub node_id: String,
    pub addr: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeartBeatResponse {
    pub code: String,
    pub msg: String,
}

impl SessionClient {
    pub fn new(host: &str, port: u16) -> Self {
        SessionClient {
            host: host.to_string(),
            port: port,
        }
    }

    pub async fn heart_beat(&self, addr: &str, node_id: &str) -> Result<HeartBeatResponse, Error> {
        let url = format!("http://{}:{}/api/v1/heart_beat", self.host, self.port);
        let client = Client::new();
        let payload = HeartBeatRequest {
            addr: addr.to_string(),
            node_id: node_id.to_string(),
        };

        let response = client
            .post(url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        let response_body: HeartBeatResponse = response.json().await?;

        Ok(response_body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_heart_beat() {
        let client = SessionClient::new("127.0.0.1", 38080);

        let addr = "192.168.1.100:3933";
        let node_id = "D526D2F5-920F-42A7-B5B9-BC643AC6B71B";

        let response = client.heart_beat(addr, node_id).await.unwrap();
        println!("{:?}", response);
    }
}
