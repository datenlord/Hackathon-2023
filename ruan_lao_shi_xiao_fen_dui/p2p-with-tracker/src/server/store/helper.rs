use std::{collections::HashSet, sync::Arc};

use parking_lot::Mutex;

pub struct FinishedNodeCache {
    inner: Arc<Mutex<HashSet<String>>>,
}

impl FinishedNodeCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn insert(&self, node_id: String) {
        let mut inner = self.inner.lock();
        inner.insert(node_id);
    }

    pub fn remove(&self, node_id: &str) {
        let mut inner = self.inner.lock();
        inner.remove(node_id);
    }

    pub fn contains(&self, node_id: &str) -> bool {
        let inner = self.inner.lock();
        inner.contains(node_id)
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock();
        inner.clear();
    }
}

pub struct GlobalTimestampCache {
    inner: Arc<Mutex<Vec<u128>>>,
}

impl GlobalTimestampCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn insert(&self, timestamp: u128) {
        let mut inner = self.inner.lock();
        inner.push(timestamp);
    }

    pub fn get(&self) -> Vec<u128> {
        let inner = self.inner.lock();
        inner.clone()
    }

    pub fn clear(&self) {
        let mut inner = self.inner.lock();
        inner.clear();
    }

    pub fn get_last_timestamp(&self) -> Option<u128> {
        let inner = self.inner.lock();
        inner.last().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finished_node_cache() {
        let cache = FinishedNodeCache::new();
        cache.insert("1".to_string());
        cache.insert("2".to_string());
        cache.insert("3".to_string());
        assert!(cache.contains("1"));
        assert!(cache.contains("2"));
        assert!(cache.contains("3"));
        cache.remove("1");
        assert!(!cache.contains("1"));
        cache.clear();
        assert!(!cache.contains("2"));
        assert!(!cache.contains("3"));
    }

    #[test]
    fn test_global_timestamp_cache() {
        let cache = GlobalTimestampCache::new();
        cache.insert(1);
        cache.insert(2);
        cache.insert(3);
        assert_eq!(cache.get(), vec![1, 2, 3]);
        cache.clear();
        assert_eq!(cache.get(), vec![]);
    }
}
