use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::{BTreeSet, HashMap};
use ws_derive::LogicalModule;

use crate::{
    network::proto::{self, cache::CacheTrans},
    result::WSResult,
    sys::{CacheRouterView, LogicalModule, LogicalModuleNewArgs, NodeID},
    util::JoinHandleWrapper,
};

#[derive(LogicalModule)]
pub struct CacheRouter {
    view: CacheRouterView,
    each_file_cache: RwLock<HashMap<(String, u32), RwLock<(usize, BTreeSet<NodeID>)>>>,
}

#[async_trait]
impl LogicalModule for CacheRouter {
    fn inner_new(args: LogicalModuleNewArgs) -> Self
    where
        Self: Sized,
    {
        Self {
            view: CacheRouterView::new(args.logical_modules_ref.clone()),
            each_file_cache: RwLock::new(HashMap::new()),
        }
    }

    async fn start(&self) -> WSResult<Vec<JoinHandleWrapper>> {
        let view = self.view.clone();
        self.view
            .p2p()
            .regist_rpc_with_receiver::<proto::cache::GetCacheViewRequest, _>(
                false,
                move |node_id, _p2p, task_id, req| {
                    // tracing::debug!("recv GetCacheViewRequest from node {}", node_id);
                    let view = view.clone();
                    let (offset, node_ids) = view
                        .cache_router()
                        .get_cache_view(&req.filename, req.block_id);
                    let _ = tokio::spawn(async move {
                        let _ = view
                            .p2p()
                            .send_resp(
                                node_id,
                                task_id,
                                proto::cache::GetCacheViewResponse { offset, node_ids },
                            )
                            .await;
                    });
                    Ok(())
                },
            );
        let view = self.view.clone();
        self.view.p2p().regist_dispatch(
            proto::cache::CacheTrans::default(),
            move |nid, _p2p, _taskid, ct| {
                view.cache_router().apply_cache_trans(nid, ct);
                Ok(())
            },
        );
        Ok(vec![])
    }
}

impl CacheRouter {
    pub fn get_cache_view(&self, filename: &str, block_id: u32) -> (u32, Vec<NodeID>) {
        let map = self.each_file_cache.read();
        let entry = map.get(&(filename.to_owned(), block_id));
        if let Some(entry) = entry {
            let mut entry = entry.write();
            entry.0 += 1;
            if entry.0 >= entry.1.len() + 1 {
                entry.0 = 0;
            }
            (entry.0 as u32, {
                let mut nodes = entry.1.iter().map(|v| *v).collect::<Vec<_>>();
                nodes.push(self.view.p2p().nodes_config.s3_node);
                nodes
            })
        } else {
            (0, vec![self.view.p2p().nodes_config.s3_node])
        }
    }
    fn apply_cache_trans(&self, from_node: NodeID, ct: CacheTrans) {
        let mut map = self.each_file_cache.write();
        let entry = map
            .entry((ct.filename, ct.block_id))
            .or_insert_with(|| RwLock::new((0, BTreeSet::new())));
        if ct.evict {
            let _ = entry.write().1.remove(&from_node);
        } else if ct.store {
            let _ = entry.write().1.insert(from_node);
        } else {
            panic!("invalid CacheTrans from {}", from_node);
        }
    }
}
