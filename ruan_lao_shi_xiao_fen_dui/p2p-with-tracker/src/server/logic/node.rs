use tracing::debug;

use crate::server::error::*;
use crate::server::store::{
    piece, GLOBAL_TIMESTAMP_CACHE, NODE_INFO_CACHE, PIECE_NODE_INFO_CACHE, SESSION_INFO_CACHE,
};
use crate::server::utils::time_helper::get_current_timestamp;

pub async fn update_session(node_id: &str) -> Result<()> {
    let timestamp = get_current_timestamp();

    // Check if node is the first to connect
    #[cfg(debug_assertions)]
    {
        let is_empty = NODE_INFO_CACHE.is_empty();
        if is_empty {
            debug!(
                "Node {} is first time to connect - timestamp:{}",
                node_id, timestamp
            );

            // Update global time
            GLOBAL_TIMESTAMP_CACHE.insert(timestamp);
        }
    }

    SESSION_INFO_CACHE.set(node_id.to_string(), timestamp);

    Ok(())
}

pub async fn add_node(node_id: &str, addr: &str) -> Result<()> {
    NODE_INFO_CACHE.set(node_id.to_string(), addr.to_string());

    Ok(())
}

pub async fn get_piece_node_list(piece_id: &str) -> Result<Vec<String>> {
    let node_list = match PIECE_NODE_INFO_CACHE.get_node_list(&piece_id.to_string()) {
        Some(node_list) => {
            // Convert to addr list
            let mut addr_list = vec![];
            for node_id in node_list.iter() {
                match NODE_INFO_CACHE.get_addr(node_id) {
                    Some(addr) => {
                        addr_list.push(addr);
                    }
                    None => {
                        debug!("Node {} is not online", node_id);
                    }
                }
            }
            return Ok(addr_list);
        },
        None => {
            vec![]
        },
    };

    Ok(node_list)
}
