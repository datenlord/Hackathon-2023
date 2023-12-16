use std::time::{Duration, SystemTime};

use super::{
    file_block::FileBlock,
    local_cache::{Cache, FsCache},
    meta_map::MetaMap,
};
use crate::{
    config,
    metric::publisher::HitPosition,
    network::{
        p2p::P2PModule,
        proto::{self, cache::GetCacheResponse},
    },
    result::WSResult,
    sys::{FsNodeView, LogicalModule, LogicalModuleNewArgs},
    util::JoinHandleWrapper,
};
use async_trait::async_trait;
use ws_derive::LogicalModule;

#[derive(LogicalModule)]
pub struct FsNode {
    meta_map: MetaMap,
    view: FsNodeView,
    local_cache: FsCache,
}

#[async_trait]
impl LogicalModule for FsNode {
    fn inner_new(args: LogicalModuleNewArgs) -> Self
    where
        Self: Sized,
    {
        Self {
            view: FsNodeView::new(args.logical_modules_ref.clone()),
            meta_map: config::read_yaml_file_map(args.nodes_config.file_dir),
            local_cache: FsCache::new(FsNodeView::new(args.logical_modules_ref.clone())),
        }
    }

    async fn start(&self) -> WSResult<Vec<JoinHandleWrapper>> {
        // Communicate with other FsNode or S3Source
        let view = self.view.clone();
        self.view
            .p2p()
            .regist_rpc_with_receiver::<proto::cache::GetCacheRequest, _>(
                true,
                move |nid, _p2p, taskid, req| {
                    tracing::info!("recv GetCacheRequest from {}", nid);
                    let view = view.clone();
                    let _ = tokio::spawn(async move {
                        if let Ok(Some(block)) = view
                            .fs_node()
                            .read_file_block(true, &req.filename, req.block_id)
                            .await
                        {
                            let _ = view
                                .p2p()
                                .send_resp(
                                    nid,
                                    taskid,
                                    proto::cache::GetCacheResponse {
                                        is_hit: true,
                                        data: block.data,
                                    },
                                )
                                .await;
                        } else {
                            let _ = view
                                .p2p()
                                .send_resp(
                                    nid,
                                    taskid,
                                    proto::cache::GetCacheResponse {
                                        is_hit: false,
                                        data: vec![],
                                    },
                                )
                                .await;
                        }
                    });
                    Ok(())
                },
            );

        // Communicate with CacheRouter
        self.view
            .p2p()
            .regist_rpc_without_receiver::<proto::cache::GetCacheViewRequest>();

        Ok(vec![])
    }
}

impl FsNode {
    pub async fn read_file_block(
        &self,
        only_local: bool,
        filename: &str,
        block_id: u32,
    ) -> WSResult<Option<FileBlock>> {
        let begin = SystemTime::now();
        let res = self
            .read_file_block_inner3(only_local, filename, block_id)
            .await?;

        if let Some((fb, hit_pos, hit_query_time)) = res {
            if !only_local {
                let end = SystemTime::now();
                let duration = end
                    .duration_since(begin)
                    .unwrap_or_else(|_err| Duration::new(0, 0));
                self.view.metric_publisher().record_read_one_block(
                    hit_pos,
                    filename,
                    block_id,
                    duration.as_millis() as usize,
                    hit_query_time,
                );
            }
            Ok(Some(fb))
        } else {
            assert!(
                only_local,
                "A block will must return if only_local is false"
            );
            Ok(None)
        }
    }

    async fn read_file_block_inner4(
        &self,
        only_local: bool,
        filename: &str,
        block_id: u32,
    ) -> WSResult<Option<(FileBlock, HitPosition, u64)>> {
        let mut query_time = 0;

        //1. local cache
        if let Some(block) = self.local_cache.get(filename, block_id) {
            return Ok(Some((block, HitPosition::Local, query_time)));
        }

        if only_local {
            return Ok(None);
        }

        for nid in 0..(self.view.p2p().nodes_config.peers.len() + 1) as u32 {
            let nid = nid + 1;

            if nid == self.view.p2p().nodes_config.router_node
                || nid == self.view.p2p().nodes_config.s3_node
                || nid == self.view.p2p().nodes_config.this.0
            {
                continue;
            }

            query_time += 1;
            if let Ok(res) = self
                .view
                .p2p()
                .call_rpc(
                    nid,
                    &proto::cache::GetCacheRequest {
                        filename: filename.to_owned(),
                        block_id,
                    },
                )
                .await
            {
                if res.is_hit {
                    self.local_cache.put(
                        filename,
                        block_id,
                        FileBlock {
                            data: res.data.clone(),
                        },
                    );
                    return Ok(Some((
                        FileBlock { data: res.data },
                        HitPosition::Remote,
                        query_time,
                    )));
                }
            }
        }
        query_time += 1;
        let res = self
            .view
            .p2p()
            .call_rpc(
                self.view.p2p().nodes_config.s3_node,
                &proto::cache::GetCacheRequest {
                    filename: filename.to_owned(),
                    block_id: block_id,
                },
            )
            .await?;
        self.local_cache.put(
            filename,
            block_id,
            FileBlock {
                data: res.data.clone(),
            },
        );
        Ok(Some((
            FileBlock { data: res.data },
            HitPosition::S3,
            query_time,
        )))
    }

    // Always from s3
    async fn read_file_block_inner3(
        &self,
        only_local: bool,
        filename: &str,
        block_id: u32,
    ) -> WSResult<Option<(FileBlock, HitPosition, u64)>> {
        let mut query_time = 0;
        //1. local cache
        if let Some(block) = self.local_cache.get(filename, block_id) {
            return Ok(Some((block, HitPosition::Local, query_time)));
        }

        if only_local {
            return Ok(None);
        }
        query_time += 1;
        let res = self
            .view
            .p2p()
            .call_rpc(
                self.view.p2p().nodes_config.s3_node,
                &proto::cache::GetCacheRequest {
                    filename: filename.to_owned(),
                    block_id: block_id,
                },
            )
            .await?;
        self.local_cache.put(
            filename,
            block_id,
            FileBlock {
                data: res.data.clone(),
            },
        );
        Ok(Some((
            FileBlock { data: res.data },
            HitPosition::S3,
            query_time,
        )))
    }

    // From next
    async fn read_file_block_inner2(
        &self,
        only_local: bool,
        filename: &str,
        block_id: u32,
    ) -> WSResult<Option<(FileBlock, HitPosition, u64)>> {
        let mut query_time = 0;
        //1. local cache
        if let Some(block) = self.local_cache.get(filename, block_id) {
            return Ok(Some((block, HitPosition::Local, query_time)));
        }

        if only_local {
            return Ok(None);
        }

        for node_offset in 0..self.view.p2p().nodes_config.peers.len() as u32 {
            let mut nid = self.view.p2p().nodes_config.this.0 + node_offset + 1;
            nid = (nid - 1) % (self.view.p2p().nodes_config.peers.len() as u32 + 1) + 1;

            if nid == self.view.p2p().nodes_config.router_node
                || nid == self.view.p2p().nodes_config.s3_node
            {
                continue;
            }

            query_time += 1;
            if let Ok(res) = self
                .view
                .p2p()
                .call_rpc(
                    nid,
                    &proto::cache::GetCacheRequest {
                        filename: filename.to_owned(),
                        block_id,
                    },
                )
                .await
            {
                if res.is_hit {
                    self.local_cache.put(
                        filename,
                        block_id,
                        FileBlock {
                            data: res.data.clone(),
                        },
                    );
                    return Ok(Some((
                        FileBlock { data: res.data },
                        HitPosition::Remote,
                        query_time,
                    )));
                }
            }
        }
        query_time += 1;
        let res = self
            .view
            .p2p()
            .call_rpc(
                self.view.p2p().nodes_config.s3_node,
                &proto::cache::GetCacheRequest {
                    filename: filename.to_owned(),
                    block_id: block_id,
                },
            )
            .await?;
        self.local_cache.put(
            filename,
            block_id,
            FileBlock {
                data: res.data.clone(),
            },
        );
        Ok(Some((
            FileBlock { data: res.data },
            HitPosition::S3,
            query_time,
        )))
    }

    async fn read_file_block_inner(
        &self,
        only_local: bool,
        filename: &str,
        block_id: u32,
    ) -> WSResult<Option<(FileBlock, HitPosition, u64)>> {
        let mut query_time = 0;
        //1. local cache
        if let Some(block) = self.local_cache.get(filename, block_id) {
            return Ok(Some((block, HitPosition::Local, query_time)));
        }

        if only_local {
            return Ok(None);
        }

        query_time += 1;
        //2. get CacheView from cache_route
        let (offset, node_ids) =
            if self.view.p2p().nodes_config.router_node == self.view.p2p().nodes_config.this.0 {
                self.view.cache_router().get_cache_view(filename, block_id)
            } else {
                let res = self
                    .view
                    .p2p()
                    .call_rpc(
                        self.view.p2p().nodes_config.router_node,
                        &proto::cache::GetCacheViewRequest {
                            filename: filename.to_string(),
                            block_id,
                        },
                    )
                    .await;
                let res = match res {
                    Ok(res) => res,
                    Err(e) => {
                        tracing::error!(
                            "get cache view from cache router failed: {}, node {}",
                            e,
                            self.view.p2p().nodes_config.router_node
                        );
                        return Err(e);
                    }
                };

                (res.offset, res.node_ids)
            };
        //3. according to cache router's guide,
        //   check cache parallelly or one by one,
        //   if not, check s3

        // 基数个 n1 n2 n3 n4 n5 n6 n7 offset=2
        //  遍历顺序 n3 n2 n4 n1 n5 n7 n6
        // 偶数个 n1 n2 n3 n4 n5 n6 offset=2
        //  遍历顺序 n3 n2 n4 n1 n5 n6

        // offset, offset - 1, offset + 1, offset -2, offset + 2, offset - 3, offset + 3 ....
        async fn try_read_cache(
            filename: &str,
            block_id: u32,
            p2p: &P2PModule,
            mut offset: i32,
            nodeids: &Vec<u32>,
        ) -> WSResult<GetCacheResponse> {
            if offset < 0 {
                offset += nodeids.len() as i32;
            } else if offset >= nodeids.len() as i32 {
                offset -= nodeids.len() as i32;
            }

            let nodeid = nodeids[offset as usize];
            p2p.call_rpc(
                nodeid,
                &proto::cache::GetCacheRequest {
                    filename: filename.to_owned(),
                    block_id,
                },
            )
            .await
        }
        let loop_time = (node_ids.len() / 2) as i32;
        for i in 0..loop_time {
            let offset1 = offset as i32 + i;
            query_time += 1;
            if let Ok(res) =
                try_read_cache(filename, block_id, self.view.p2p(), offset1, &node_ids).await
            {
                if res.is_hit {
                    self.local_cache.put(
                        filename,
                        block_id,
                        FileBlock {
                            data: res.data.clone(),
                        },
                    );
                    return Ok(Some((
                        FileBlock { data: res.data },
                        HitPosition::Remote,
                        query_time,
                    )));
                }
            }
            let offset2 = offset as i32 - i - 1;
            query_time += 1;
            if let Ok(res) =
                try_read_cache(filename, block_id, self.view.p2p(), offset2, &node_ids).await
            {
                if res.is_hit {
                    self.local_cache.put(
                        filename,
                        block_id,
                        FileBlock {
                            data: res.data.clone(),
                        },
                    );
                    return Ok(Some((
                        FileBlock { data: res.data },
                        HitPosition::Remote,
                        query_time,
                    )));
                }
            }
        }
        query_time += 1;
        if node_ids.len() % 2 == 1 {
            let offset = offset as i32 + loop_time;
            if let Ok(res) =
                try_read_cache(filename, block_id, self.view.p2p(), offset, &node_ids).await
            {
                if res.is_hit {
                    self.local_cache.put(
                        filename,
                        block_id,
                        FileBlock {
                            data: res.data.clone(),
                        },
                    );
                    return Ok(Some((
                        FileBlock { data: res.data },
                        HitPosition::Remote,
                        query_time,
                    )));
                }
            }
        }
        query_time += 1;
        let res = self
            .view
            .p2p()
            .call_rpc(
                self.view.p2p().nodes_config.s3_node,
                &proto::cache::GetCacheRequest {
                    filename: filename.to_owned(),
                    block_id: block_id,
                },
            )
            .await?;
        self.local_cache.put(
            filename,
            block_id,
            FileBlock {
                data: res.data.clone(),
            },
        );
        Ok(Some((
            FileBlock { data: res.data },
            HitPosition::S3,
            query_time,
        )))
    }
    pub fn file_block_cnt(&self, filename: &str) -> u32 {
        self.meta_map.file_block_cnt(filename)
    }
}
