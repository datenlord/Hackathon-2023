pub mod helper;
pub mod node;
pub mod piece;
pub mod session;

use lazy_static::lazy_static;
use tracing::{debug, error, info};

lazy_static! {
    // Tracker cache
    pub static ref NODE_INFO_CACHE: node::NodeInfoCache = node::NodeInfoCache::new();
    pub static ref PIECE_NODE_INFO_CACHE: piece::PieceNodeInfoCache = piece::PieceNodeInfoCache::new();
    pub static ref SESSION_INFO_CACHE: session::SessionInfoCache = session::SessionInfoCache::new();

    // Node cache
    pub static ref PIECE_INFO_CACHE: piece::PieceInfoCache = piece::PieceInfoCache::new();
    pub static ref PIECE_CACHE: piece::PieceCache = piece::PieceCache::new();

    // Seed cache
    pub static ref SEED_INFO_CACHE: piece::SeedInfoCache = piece::SeedInfoCache::new();

    // NodeId
    pub static ref NODE_ID: node::NodeId = uuid::Uuid::new_v4().to_string();

    /// For Test: Store finished node
    pub static ref FINISHED_NODE_CACHE: helper::FinishedNodeCache = helper::FinishedNodeCache::new();
    pub static ref GLOBAL_TIMESTAMP_CACHE: helper::GlobalTimestampCache = helper::GlobalTimestampCache::new();
}
