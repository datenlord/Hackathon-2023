use std::time::{SystemTime, UNIX_EPOCH};

use crate::server::error::*;
use crate::server::store::{
    NODE_INFO_CACHE, PIECE_INFO_CACHE, PIECE_NODE_INFO_CACHE, SESSION_INFO_CACHE,
};

pub async fn get_tracker_info() -> Result<()> {
    let session_info = SESSION_INFO_CACHE.get_all();
    let node_info = NODE_INFO_CACHE.get_all();
    let piece_node_info = PIECE_NODE_INFO_CACHE.get_piece_info();
    let piece_info = PIECE_INFO_CACHE.get_all();

    Ok(())
}
