use std::{collections::HashMap, collections::HashSet, sync::Arc};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

pub type NodeId = String;
pub type PieceId = String;
pub type PieceToNode = HashMap<PieceId, HashSet<NodeId>>;
pub type NodeToPiece = HashMap<NodeId, HashSet<PieceId>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct PieceNodeInfo {
    piece_to_node: PieceToNode,
    node_to_piece: NodeToPiece,
}

pub struct PieceNodeInfoCache {
    inner: Arc<Mutex<PieceNodeInfo>>,
}

impl PieceNodeInfoCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(PieceNodeInfo {
                piece_to_node: HashMap::new(),
                node_to_piece: HashMap::new(),
            })),
        }
    }

    pub fn get(&self, piece_id: &PieceId) -> Option<HashSet<NodeId>> {
        let inner = self.inner.lock();
        inner.piece_to_node.get(piece_id).cloned()
    }

    pub fn set(&self, piece_id: PieceId, node_id: NodeId) {
        let mut inner = self.inner.lock();
        let node_set = inner
            .piece_to_node
            .entry(piece_id.clone())
            .or_insert(HashSet::new());
        node_set.insert(node_id.clone());

        let piece_set = inner.node_to_piece.entry(node_id).or_insert(HashSet::new());
        piece_set.insert(piece_id.clone());
    }

    pub fn remove(&self, piece_id: &PieceId, node_id: &NodeId) {
        let mut inner = self.inner.lock();
        if let Some(node_set) = inner.piece_to_node.get_mut(piece_id) {
            node_set.remove(node_id);
        }

        if let Some(piece_set) = inner.node_to_piece.get_mut(node_id) {
            piece_set.remove(piece_id);
        }
    }

    pub fn get_all(&self) -> Vec<(PieceId, HashSet<NodeId>)> {
        let inner = self.inner.lock();
        inner
            .piece_to_node
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub fn get_all_piece_id(&self) -> Vec<PieceId> {
        let inner = self.inner.lock();
        inner.piece_to_node.keys().cloned().collect()
    }

    pub fn get_all_node_id(&self) -> Vec<NodeId> {
        let inner = self.inner.lock();
        inner.node_to_piece.keys().cloned().collect()
    }

    pub fn get_all_map(&self) -> PieceToNode {
        let inner = self.inner.lock();
        inner.piece_to_node.clone()
    }

    pub fn get_piece_info(&self) -> PieceNodeInfo {
        let inner = self.inner.lock();
        PieceNodeInfo {
            piece_to_node: inner.piece_to_node.clone(),
            node_to_piece: inner.node_to_piece.clone(),
        }
    }

    pub fn get_node_list(&self, piece_id: &PieceId) -> Option<Vec<NodeId>> {
        let inner = self.inner.lock();
        inner
            .piece_to_node
            .get(piece_id)
            .map(|e| e.iter().cloned().collect())
    }

    pub fn get_piece_list(&self, node_id: &NodeId) -> Option<Vec<PieceId>> {
        let inner = self.inner.lock();
        inner
            .node_to_piece
            .get(node_id)
            .map(|e| e.iter().cloned().collect())
    }

    pub fn purge_node(&self, node_id: &NodeId) {
        let mut inner = self.inner.lock();
        if let Some(piece_set) = inner.node_to_piece.get_mut(node_id) {
            // TODO: Update borrow checker
            let mut inner_mut = self.inner.lock();
            for piece_id in piece_set.iter() {
                if let Some(node_set) = inner_mut.piece_to_node.get_mut(piece_id) {
                    node_set.remove(node_id);
                }
            }
        }
        inner.node_to_piece.remove(node_id);
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct PieceInfo {
    filename: String,
    index: u32,
    size: u32,
    checksum: String, // Same as PieceId
}

impl PieceInfo {
    pub fn new(filename: String, index: u32, size: u32, checksum: String) -> Self {
        Self {
            filename,
            index,
            size,
            checksum,
        }
    }

    pub fn get_filename(&self) -> String {
        self.filename.clone()
    }

    pub fn get_index(&self) -> u32 {
        self.index
    }

    pub fn get_size(&self) -> u32 {
        self.size
    }

    pub fn get_checksum(&self) -> String {
        self.checksum.clone()
    }
}

pub struct PieceInfoCache {
    inner: Arc<Mutex<HashMap<PieceId, PieceInfo>>>,
}

impl PieceInfoCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, piece_id: &PieceId) -> Option<PieceInfo> {
        let inner = self.inner.lock();
        inner.get(piece_id).cloned()
    }

    pub fn set(&self, piece_id: PieceId, piece_info: PieceInfo) {
        let mut inner = self.inner.lock();
        inner.insert(piece_id, piece_info);
    }

    pub fn contains(&self, piece_id: &PieceId) -> bool {
        let inner = self.inner.lock();
        inner.contains_key(piece_id)
    }

    pub fn remove(&self, piece_id: &PieceId) {
        let mut inner = self.inner.lock();
        inner.remove(piece_id);
    }

    pub fn get_all(&self) -> Vec<(PieceId, PieceInfo)> {
        let inner = self.inner.lock();
        inner.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn get_all_piece_id(&self) -> Vec<PieceId> {
        let inner = self.inner.lock();
        inner.keys().cloned().collect()
    }

    pub fn get_all_map(&self) -> HashMap<PieceId, PieceInfo> {
        let inner = self.inner.lock();
        inner.clone()
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct SeedInfo {
    filename: String,
    content_length: u32,        // file content size
    piece_list: Vec<PieceInfo>, // piece list(order by index)
}

impl SeedInfo {
    pub fn new(filename: String, content_length: u32, piece_list: Vec<PieceInfo>) -> Self {
        Self {
            filename,
            content_length,
            piece_list,
        }
    }

    pub fn get_filename(&self) -> String {
        self.filename.clone()
    }

    pub fn get_content_length(&self) -> u32 {
        self.content_length
    }

    pub fn get_piece_list(&self) -> Vec<PieceInfo> {
        self.piece_list.clone()
    }

    pub fn add_piece(&mut self, piece_info: PieceInfo) {
        self.piece_list.push(piece_info);
    }
}

type Filename = String;

pub struct SeedInfoCache {
    inner: Arc<Mutex<HashMap<Filename, SeedInfo>>>,
}

impl SeedInfoCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, filename: &Filename) -> Option<SeedInfo> {
        let inner = self.inner.lock();
        inner.get(filename).cloned()
    }

    pub fn set(&self, filename: Filename, seed_info: SeedInfo) {
        let mut inner = self.inner.lock();
        inner.insert(filename, seed_info);
    }

    pub fn remove(&self, filename: &Filename) {
        let mut inner = self.inner.lock();
        inner.remove(filename);
    }

    pub fn get_all(&self) -> Vec<(Filename, SeedInfo)> {
        let inner = self.inner.lock();
        inner.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn get_all_filename(&self) -> Vec<Filename> {
        let inner = self.inner.lock();
        inner.keys().cloned().collect()
    }

    pub fn get_all_map(&self) -> HashMap<Filename, SeedInfo> {
        let inner = self.inner.lock();
        inner.clone()
    }
}

pub struct PieceCache {
    inner: Arc<Mutex<HashMap<PieceId, Vec<u8>>>>,
}

impl PieceCache {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get(&self, piece_id: &PieceId) -> Option<Vec<u8>> {
        let inner = self.inner.lock();
        inner.get(piece_id).cloned()
    }

    pub fn set(&self, piece_id: PieceId, piece: Vec<u8>) {
        let mut inner = self.inner.lock();
        inner.insert(piece_id, piece);
    }

    pub fn remove(&self, piece_id: &PieceId) {
        let mut inner = self.inner.lock();
        inner.remove(piece_id);
    }

    pub fn get_all(&self) -> Vec<(PieceId, Vec<u8>)> {
        let inner = self.inner.lock();
        inner.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn get_all_piece_id(&self) -> Vec<PieceId> {
        let inner = self.inner.lock();
        inner.keys().cloned().collect()
    }

    pub fn get_all_map(&self) -> HashMap<PieceId, Vec<u8>> {
        let inner = self.inner.lock();
        inner.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piece_node_info_cache() {
        let piece_info_cache = PieceNodeInfoCache::new();
        let piece_id = "piece_id".to_string();
        let node_id = "node_id".to_string();
        let piece_id1 = "piece_id1".to_string();
        let node_id1 = "node_id1".to_string();
        let piece_id2 = "piece_id1".to_string();
        let node_id2 = "node_id".to_string();

        piece_info_cache.set(piece_id.clone(), node_id.clone());
        piece_info_cache.set(piece_id1.clone(), node_id1.clone());
        piece_info_cache.set(piece_id2.clone(), node_id2.clone());
        assert_eq!(
            piece_info_cache.get(&piece_id),
            Some(vec![node_id.clone()].into_iter().collect())
        );

        // PieceNodeInfo { piece_to_node: {"piece_id": {"node_id"}, "piece_id1": {"node_id1", "node_id"}}, node_to_piece: {"node_id": {"piece_id1", "piece_id"}, "node_id1": {"piece_id1"}} }
        println!("{:?}", piece_info_cache.get_piece_info());

        piece_info_cache.remove(&piece_id, &node_id);
        assert_eq!(
            piece_info_cache.get(&piece_id),
            Some(vec![].into_iter().collect())
        );

        piece_info_cache.set(piece_id.clone(), node_id.clone());
    }

    #[test]
    fn test_piece_info_cache() {
        let piece_info_cache = PieceInfoCache::new();
        let piece_id = "piece_id".to_string();
        let piece_info = PieceInfo {
            filename: "filename".to_string(),
            index: 0,
            size: 0,
            checksum: "checksum".to_string(),
        };

        piece_info_cache.set(piece_id.clone(), piece_info.clone());
        assert_eq!(piece_info_cache.get(&piece_id).unwrap(), piece_info.clone());

        // [("piece_id", PieceInfo { filename: "filename", index: 0, size: 0 })]
        println!("{:?}", piece_info_cache.get_all());

        piece_info_cache.remove(&piece_id);
        assert_eq!(piece_info_cache.get(&piece_id), None);

        piece_info_cache.set(piece_id.clone(), piece_info.clone());
        assert_eq!(
            piece_info_cache.get_all(),
            vec![(piece_id.clone(), piece_info.clone())]
        );
    }

    #[test]
    fn test_seed_info_cache() {
        let seed_info_cache = SeedInfoCache::new();
        let filename = "filename".to_string();
        let seed_info = SeedInfo {
            filename: "filename".to_string(),
            content_length: 0,
            piece_list: vec![],
        };

        seed_info_cache.set(filename.clone(), seed_info.clone());
        assert_eq!(seed_info_cache.get(&filename).unwrap(), seed_info.clone());

        // [("filename", SeedInfo { filename: "filename", content_length: 0, piece_list: [] })]
        println!("{:?}", seed_info_cache.get_all());

        seed_info_cache.remove(&filename);
        assert_eq!(seed_info_cache.get(&filename), None);

        seed_info_cache.set(filename.clone(), seed_info.clone());
        assert_eq!(
            seed_info_cache.get_all(),
            vec![(filename.clone(), seed_info.clone())]
        );
    }

    #[test]
    fn test_piece_cache() {
        let piece_cache = PieceCache::new();
        let piece_id = "piece_id".to_string();
        let piece = vec![0, 1, 2, 3];

        piece_cache.set(piece_id.clone(), piece.clone());
        assert_eq!(piece_cache.get(&piece_id).unwrap(), piece.clone());

        // [("piece_id", [0, 1, 2, 3])]
        println!("{:?}", piece_cache.get_all());

        piece_cache.remove(&piece_id);
        assert_eq!(piece_cache.get(&piece_id), None);

        piece_cache.set(piece_id.clone(), piece.clone());
        assert_eq!(
            piece_cache.get_all(),
            vec![(piece_id.clone(), piece.clone())]
        );
    }
}
