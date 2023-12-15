use std::thread;
use std::time::Duration;

use tracing::{debug, info};

use crate::server::store::{SESSION_INFO_CACHE, NODE_INFO_CACHE};
use crate::server::utils::time_helper::get_current_timestamp;
pub struct CheckTask {
    tracker_uuid: String,
    session_check_time: u64,
    session_expire_time: u64,
}

impl CheckTask {
    pub fn new(uuid: &str, session_check_time: u64, session_expire_time: u64) -> Self {
        Self {
            tracker_uuid: uuid.to_string(),
            session_check_time,
            session_expire_time,
        }
    }

    pub fn start(&self) {
        let session_check_time = self.session_check_time;
        let session_expire_time = self.session_expire_time;
        let tracker_uuid = self.tracker_uuid.clone();

        thread::spawn(move || loop {
            let interval = Duration::from_secs(session_check_time);
            let expire_time = Duration::from_secs(session_expire_time);
            let timestamp = get_current_timestamp();
            let expired_node_list =
                SESSION_INFO_CACHE.find_expired(timestamp, expire_time.as_millis());
            if expired_node_list.is_empty() {
                debug!("[CheckTask-Tracker:{tracker_uuid}] No expired node");
            } else {
                for node_id in expired_node_list.iter() {
                    SESSION_INFO_CACHE.remove(node_id);
                    NODE_INFO_CACHE.remove(node_id);
                }
                info!(
                    "[CheckTask-Tracker:{tracker_uuid}] Remove expired node list: {:?}",
                    expired_node_list
                );
            }

            thread::sleep(interval);
        });
    }
}
