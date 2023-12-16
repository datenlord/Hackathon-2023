use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

pub type NodeId = String;
pub type Addr = String;
// pub type NodeInfoCache = Arc<Mutex<HashMap<NodeId, Addr>>>;

pub struct NodeInfoCache {
    inner: Arc<Mutex<HashMap<NodeId, Addr>>>,
}

impl NodeInfoCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, node_id: &NodeId) -> Option<Addr> {
        let inner = self.inner.lock();
        inner.get(node_id).cloned()
    }

    pub fn get_addr(&self, node_id: &NodeId) -> Option<Addr> {
        let inner = self.inner.lock();
        inner.get(node_id).cloned()
    }

    pub fn set(&self, node_id: NodeId, addr: Addr) {
        let mut inner = self.inner.lock();
        inner.insert(node_id, addr);
    }

    pub fn remove(&self, node_id: &NodeId) {
        let mut inner = self.inner.lock();
        inner.remove(node_id);
    }

    pub fn get_all(&self) -> Vec<(NodeId, Addr)> {
        let inner = self.inner.lock();
        inner.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn get_all_node_id(&self) -> Vec<NodeId> {
        let inner = self.inner.lock();
        inner.keys().cloned().collect()
    }

    pub fn get_all_addr(&self) -> Vec<Addr> {
        let inner = self.inner.lock();
        inner.values().cloned().collect()
    }

    pub fn get_all_map(&self) -> HashMap<NodeId, Addr> {
        let inner = self.inner.lock();
        inner.clone()
    }

    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock();
        inner.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_info_cache() {
        let node_info_cache = NodeInfoCache::new();
        let node_id = "node_id".to_string();
        let addr = "addr".to_string();

        node_info_cache.set(node_id.clone(), addr.clone());
        assert_eq!(node_info_cache.get(&node_id), Some(addr.clone()));

        println!("{:?}", node_info_cache.get_all());

        node_info_cache.remove(&node_id);
        assert_eq!(node_info_cache.get(&node_id), None);

        node_info_cache.set(node_id.clone(), addr.clone());
        assert_eq!(
            node_info_cache.get_all(),
            vec![(node_id.clone(), addr.clone())]
        );

        assert_eq!(node_info_cache.get_all_node_id(), vec![node_id.clone()]);
        assert_eq!(node_info_cache.get_all_addr(), vec![addr.clone()]);

        let mut map = HashMap::new();
        map.insert(node_id.clone(), addr.clone());
        assert_eq!(node_info_cache.get_all_map(), map);
    }
}
