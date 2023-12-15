use std::fs;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub http: ConfigHttp,
    pub p2p: ConfigP2p,
    pub s3: ConfigS3,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigHttp {
    pub host: String,
    pub tracker_port: u16,
    pub range_port_start: u16,
    pub range_port_end: u16,
    pub cors: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigP2p {
    pub piece_size: u64,
    pub session_expire_time: u64,
    pub session_check_time: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigS3 {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
}

impl Config {
    /// Parse configuration file.
    pub fn parse(path: &str) -> anyhow::Result<Self> {
        let config_str = fs::read_to_string(path)?;

        let config = toml::from_str(&config_str)?;

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test default configuration file.
    #[test]
    fn test_config() {
        Config::parse("./config.toml").expect("Failed to parse configuration file");
    }
}
