use core::fmt;

use chrono::format;
use rand::seq::SliceRandom;
use serde::{Serialize, Deserialize};
use tracing::{debug, info};

use crate::server::error::*;
use crate::server::node::client::file::{FileClient, self};
use crate::server::node::client::node::NodeClient;
use crate::server::node::client::piece::PieceClient;
use crate::server::store::piece::PieceInfo;
use crate::server::store::{
    FINISHED_NODE_CACHE, GLOBAL_TIMESTAMP_CACHE, NODE_INFO_CACHE, PIECE_INFO_CACHE,
    PIECE_NODE_INFO_CACHE, SEED_INFO_CACHE, NODE_ID, PIECE_CACHE,
};
use crate::server::utils::s3_helper::S3Client;
use crate::server::utils::time_helper::get_current_timestamp;
use crate::{config::Config, server::store::piece::SeedInfo};

pub async fn fetch_seed(config: Config, filename: &str) -> Result<SeedInfo> {
    // Get seed from cache
    match SEED_INFO_CACHE.get(&filename.to_string()) {
        Some(seed_info) => {
            debug!("Fetch seed info from cache: {:?}", seed_info);
            return Ok(seed_info);
        },
        None => {
            debug!("Fetch seed info from s3");
        },
    }

    let endpoint = config.s3.endpoint;
    let bucket = config.s3.bucket;
    let region = config.s3.region;
    let access_key = config.s3.access_key;
    let secret_key = config.s3.secret_key;

    let client = S3Client::new(endpoint, bucket, region, access_key, secret_key);
    let metadata = match client.get_object_metadata(filename.to_string()).await {
        Ok(metadata) => metadata,
        Err(e) => {
            return Err(Error::FetchMetadataFailed);
        },
    };
    let mut pieces: Vec<PieceInfo> = vec![];
    let mut piece_size = config.p2p.piece_size;

    let file_size = metadata.content_length.unwrap() as u64;
    let mut start = 0;
    let mut end = piece_size;
    while start < file_size {
        if end > file_size {
            end = file_size;
            piece_size = end - start;
        }
        let checksum = uuid::Uuid::new_v4();
        let piece: PieceInfo = PieceInfo::new(
            filename.to_string(),
            start as u32,
            piece_size as u32,
            checksum.to_string(),
        );
        pieces.push(piece.clone());

        // Update to piece info cache
        PIECE_INFO_CACHE.set(checksum.to_string(), piece.clone());

        // Iterate to next piece
        start += piece_size;
        end += piece_size;
    }

    let seed_info: SeedInfo = SeedInfo::new(
        filename.to_string(),
        metadata.content_length.unwrap_or_default() as u32,
        pieces,
    );
    SEED_INFO_CACHE.set(filename.to_string(), seed_info.clone());

    Ok(seed_info)
}

pub async fn progress_report(
    node_id: &str,
    piece_id: &str,
    progress: f64,
    filename: &str,
) -> Result<()> {
    PIECE_NODE_INFO_CACHE.set(piece_id.to_string(), node_id.to_string());

    // Calc finish time
    #[cfg(debug_assertions)]
    {
        calc_download_time(node_id, progress).await;
    }

    Ok(())
}

pub async fn calc_download_time(node_id: &str, progress: f64) {
    if progress >= 1.0 {
        FINISHED_NODE_CACHE.insert(node_id.to_string());
    }

    // check if all online nodes finished
    let nodes = NODE_INFO_CACHE.get_all_node_id();
    for node in nodes {
        if !FINISHED_NODE_CACHE.contains(&node) {
            return;
        }
    }

    match GLOBAL_TIMESTAMP_CACHE.get_last_timestamp() {
        Some(timestamp) => {
            let current_timestamp = get_current_timestamp();
            let download_time = current_timestamp - timestamp;
            debug!("All nodes finished at timestamp: {} ms", download_time)
        },
        None => {
            debug!("No timestamp found");
        },
    }

    // Update global timestamp
    let timestamp = get_current_timestamp();
    // Update current finished time
    GLOBAL_TIMESTAMP_CACHE.insert(timestamp);
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadResult {
    pub data: Vec<u8>,
    pub success_count: f64,
    pub failed_count: f64,
    pub total_count: f64,
}

impl DownloadResult {
    pub fn new(data: Vec<u8>, success_count: f64, failed_count:f64, total_count: f64) -> Self {
        DownloadResult {
            data: data,
            success_count: success_count,
            failed_count: failed_count,
            total_count: total_count,
        }
    }

    pub fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }

    pub fn get_success_count(&self) -> f64 {
        self.success_count
    }

    pub fn get_failed_count(&self) -> f64 {
        self.failed_count
    }

    pub fn get_total_count(&self) -> f64 {
        self.total_count
    }

    pub fn get_progress(&self) -> f64 {
        self.success_count / self.total_count
    }

    pub fn is_finished(&self) -> bool {
        self.get_progress() >= 1.0
    }

    pub fn set_data(&mut self, data: Vec<u8>) {
        self.data = data;
    }

    pub fn set_success_count(&mut self, success_count: f64) {
        self.success_count = success_count;
    }

    pub fn set_failed_count(&mut self, failed_count: f64) {
        self.failed_count = failed_count;
    }

    pub fn set_total_count(&mut self, total_count: f64) {
        self.total_count = total_count;
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.success_count = progress * self.total_count;
    }
}

pub async fn download(config: Config, filename: &str) -> Result<DownloadResult, Error> {
    // Fetch seed info
    let file_client = FileClient::new(
        &config.http.host,
        config.http.tracker_port,
    );
    let node_client = NodeClient::new(
        &config.http.host,
        config.http.tracker_port,
    );
    let s3_client = S3Client::new(
        config.s3.endpoint.to_string(),
        config.s3.bucket.to_string(),
        config.s3.region.to_string(),
        config.s3.access_key.to_string(),
        config.s3.secret_key.to_string(),
    );
    match file_client.fetch_seed(filename).await {
        Ok(seed_info) => {
            let seed_info = seed_info.seed_info;
            // Save to local memory
            SEED_INFO_CACHE.set(filename.to_string(), seed_info.to_owned());

            let mut pieces = seed_info.get_piece_list();
            let mut shuffled_pieces =  seed_info.get_piece_list();
            // shuffle piece list
            shuffled_pieces.shuffle(&mut rand::thread_rng());
            debug!("Fetch seed info: {:?}", seed_info);

            let mut success_count: f64 = 0.0;
            let mut failed_count: f64 = 0.0;
            let total_count: f64 = pieces.len() as f64;
        
            // Download pieces
            for piece in shuffled_pieces {
                let piece_id = piece.get_checksum().clone();
                let mut piece_node_list: Vec<String> = Vec::new();
                // Fetch piece node info cache
                match node_client.node_list(piece_id.as_str()).await {
                    Ok(node_list) => {
                        debug!("Fetch node list: {:?}", node_list);
                        piece_node_list = node_list.node_list;
                    },
                    Err(e) => {
                        debug!("Fetch node list failed: {:?}", e);
                        continue;
                    },
                }

                // 1. Check if the piece is downloaded or not
                if PIECE_INFO_CACHE.contains(&piece_id) {
                    debug!("Piece: {:?} is already downloaded", piece);
                    success_count+=1.0;
                    continue;
                }
                
                // Fetch piece from S3 or node
                if piece_node_list.len() == 0 {
                    debug!("No node found for piece: {:?} from tracker", piece_id);

                    // Download from s3
                    let start = piece.get_index() as u64;
                    let end = start + piece.get_size() as u64;
                    match s3_client.get_object_range_data(filename.to_string(), start, end).await {
                        Ok(data) => {
                            debug!("Download piece: {:?} from s3 success", piece);
                            success_count+=1.0;

                            // Update piece info cache
                            PIECE_INFO_CACHE.set(piece_id.clone(), piece.clone());
                            PIECE_CACHE.set(piece_id.clone(), data.clone());
                            let _ = file_client.report(&NODE_ID, piece_id.as_str(), success_count/total_count, filename).await;
                        },
                        Err(e) => {
                            debug!("Download piece: {:?} from s3 failed", piece);
                            continue;
                        },
                    }
                } else {
                    // Download from node
                    // Iterate node list to fetch data
                    for node_addr in piece_node_list {
                        // TODO: Skip current node
                        // if node_addr.eq(format!("{}:{}", config.http.host, config.http.port).as_str()) {
                        //     continue;
                        // }

                        // Fetch piece data from node
                        let piece_client = PieceClient::new(node_addr.as_str());

                        match piece_client.fetch_piece(&piece_id).await {
                            Ok(response) => {
                                if response.data.is_empty() {
                                    continue;
                                }

                                debug!("Download piece: {:?} from node addr: {:?} success", piece, node_addr);
                                success_count+=1.0;

                                // Update piece info cache
                                PIECE_INFO_CACHE.set(piece_id.clone(), piece.clone());
                                PIECE_CACHE.set(piece_id.clone(), response.data.clone());
                                let _ = file_client.report(&NODE_ID, piece_id.as_str(), success_count/total_count, filename).await;
                                break;
                            },
                            Err(e) => {
                                debug!("Download piece: {:?} from node addr: {:?} failed", piece, node_addr);
                                failed_count+=1.0;
                                continue;
                            },
                        }
                    }
                }
            }
        
            // Assemble pieces
            let mut file = vec![];
            for piece in pieces {
                let piece_id: String = piece.get_checksum().clone();
                match PIECE_CACHE.get(&piece_id.clone()) {
                    Some(piece) => {
                        file.extend(piece);
                    },
                    None => {
                        debug!("Piece: {:?} not found", piece);
                        continue;
                    },
                }
            }

            // return download result
            let download_result = DownloadResult::new(file, success_count, failed_count, total_count);
            return Ok(download_result);
        },
        Err(e) => {
            return Err(Error::FetchSeedInfoFailed);
        },
    }
}