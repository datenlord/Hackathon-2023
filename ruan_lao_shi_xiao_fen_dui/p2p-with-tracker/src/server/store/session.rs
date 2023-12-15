use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

pub type NodeId = String;
pub type Timestamp = u128;
// pub type NodeInfoCache = Arc<Mutex<HashMap<NodeId, Addr>>>;

pub struct SessionInfoCache {
    inner: Arc<Mutex<HashMap<NodeId, Timestamp>>>,
}

impl SessionInfoCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, node_id: &NodeId) -> Option<Timestamp> {
        let inner = self.inner.lock();
        inner.get(node_id).cloned()
    }

    pub fn set(&self, node_id: NodeId, timestamp: Timestamp) {
        let mut inner = self.inner.lock();
        inner.insert(node_id, timestamp);
    }

    pub fn remove(&self, node_id: &NodeId) {
        let mut inner = self.inner.lock();
        inner.remove(node_id);
    }

    pub fn get_all(&self) -> Vec<(NodeId, Timestamp)> {
        let inner = self.inner.lock();
        inner.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn get_all_node_id(&self) -> Vec<NodeId> {
        let inner = self.inner.lock();
        inner.keys().cloned().collect()
    }

    pub fn get_all_timestamp(&self) -> Vec<Timestamp> {
        let inner = self.inner.lock();
        inner.values().cloned().collect()
    }

    pub fn get_all_map(&self) -> HashMap<NodeId, Timestamp> {
        let inner = self.inner.lock();
        inner.clone()
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock();
        inner.clear();
    }

    pub fn find_expired(&self, current_timestamp: Timestamp, timeout: Timestamp) -> Vec<NodeId> {
        let inner = self.inner.lock();
        inner
            .iter()
            .filter(|(_, timestamp)| current_timestamp - **timestamp > timeout)
            .map(|(node_id, _)| node_id.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_session_info_cache() {
        let cache = SessionInfoCache::new();
        let node_id = "node_id".to_string();

        let current_time = SystemTime::now();

        let duration = current_time
            .duration_since(UNIX_EPOCH)
            .expect("Failed to get current time");
        let milliseconds = duration.as_millis();
        let timestamp = milliseconds;

        cache.set(node_id.clone(), timestamp);

        println!("{:?}", cache.get_all());

        assert_eq!(cache.get(&node_id), Some(timestamp));
        assert_eq!(cache.get_all_node_id(), vec![node_id.clone()]);
        assert_eq!(cache.get_all_timestamp(), vec![timestamp]);
        assert_eq!(
            cache.get_all_map(),
            vec![(node_id.clone(), timestamp)].into_iter().collect()
        );
        cache.remove(&node_id);
    }
}
