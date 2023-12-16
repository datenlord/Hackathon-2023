use std::thread;
use std::time::Duration;

use tracing::{debug, info};

use crate::config::Config;
use crate::server::node::client::session::SessionClient;
use std::sync::Arc;

pub struct HeartBeatTask {
    config: Config,
    node_uuid: String,
    node_addr: String,
    session_check_time: u64,
}

impl HeartBeatTask {
    pub fn new(config: Config, uuid: &str, addr: &str, session_check_time: u64) -> Self {
        Self {
            config,
            node_uuid: uuid.to_string(),
            node_addr: addr.to_string(),
            session_check_time,
        }
    }

    pub fn start(&self) {
        let session_check_time = self.session_check_time;
        let node_uuid = self.node_uuid.clone();
        let node_addr = self.node_addr.clone();
        let config = Arc::new(self.config.clone());

        tokio::spawn(async move {
            let interval = Duration::from_secs(session_check_time);

            let tracker_host = config.http.host.clone();
            let tracker_port = config.http.tracker_port;
            let session_client = SessionClient::new(tracker_host.as_str(), tracker_port);
            let session_client = Arc::new(session_client);

            loop {
                let node_addr = node_addr.clone();
                let node_uuid = node_uuid.clone();
                let session_client = Arc::clone(&session_client);

                let heartbeat_task = async move {
                    match session_client
                        .heart_beat(node_addr.as_str(), node_uuid.as_str())
                        .await
                    {
                        Ok(_) => {
                            debug!(
                                "[HeartBeatTask-Node:{node_uuid}] Send heart beat to tracker",
                                node_uuid = node_uuid
                            );
                        },
                        Err(e) => {
                            debug!("[HeartBeatTask-Node:{node_uuid}] Send heart beat to tracker failed, error: {error}", node_uuid = node_uuid, error = e);
                        },
                    }
                    // debug!("[HeartBeatTask-Node:{node_uuid}] Send heart beat to tracker");
                };

                tokio::pin!(heartbeat_task);

                heartbeat_task.await;

                tokio::time::sleep(interval).await;
            }
        });
    }
}
