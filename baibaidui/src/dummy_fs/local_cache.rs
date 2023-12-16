use crate::{network::proto, sys::FsNodeView};

use super::file_block::FileBlock;
use lru::LruCache;
use parking_lot::Mutex;

pub type FsCache = LRULocalCache;

// pub struct CacheHolder<C> {
//     cache: C,
//     view: Option<FsNodeView>,
// }
// impl<C> CacheHolder<C> {
//     pub fn new(view: FsNodeView, cache: C) -> Self {
//         Self {
//             cache,
//             view: Some(view),
//         }
//     }
// }
// impl<C> Deref for CacheHolder<C> {
//     type Target = C;
//     fn deref(&self) -> &Self::Target {
//         &self.cache
//     }
// }
// impl<C> Drop for CacheHolder<FileBlock> {
//     fn drop(&mut self) {
//         let view = self.view.take().unwrap();
//         let filename=self.
//         tokio::spawn(async move {
//             let router = view.p2p().nodes_config.router_node;
//             view.p2p().send_resp(
//                 router,
//                 0, /* dummy id */
//                 proto::cache::CacheTrans {
//                     filename: todo!(),
//                     block_id: todo!(),
//                     store: todo!(),
//                     evict: todo!(),
//                 },
//             )
//         });
//     }
// }

pub struct LRULocalCache {
    lru_cache: Mutex<LruCache<(String, u32), FileBlock>>,
    view: FsNodeView,
}

impl LRULocalCache {
    pub fn new(view: FsNodeView) -> Self {
        Self {
            lru_cache: Mutex::new(LruCache::new(201)),
            view,
        }
    }
}

pub trait Cache {
    fn put(&self, file_name: &str, index: u32, fb: FileBlock);
    fn get(&self, file_name: &str, index: u32) -> Option<FileBlock>;
}

impl Cache for LRULocalCache {
    fn put(&self, file_name: &str, index: u32, fb: FileBlock) {
        let mut lru_cache = self.lru_cache.lock();
        if lru_cache.put((file_name.to_owned(), index), fb).is_none() {
            let view = self.view.clone();
            let filename = file_name.to_owned();
            let _ = tokio::spawn(async move {
                let _ = view
                    .p2p()
                    .send_resp(
                        view.p2p().nodes_config.router_node,
                        0, /* dummy id */
                        proto::cache::CacheTrans {
                            filename,
                            block_id: index,
                            store: true,
                            evict: false,
                        },
                    )
                    .await;
            });
        }
        if lru_cache.len() == lru_cache.cap() - 1 {
            let ((fname, blockid), _) = lru_cache.pop_lru().unwrap();
            let view = self.view.clone();
            let _ = tokio::spawn(async move {
                let _ = view
                    .p2p()
                    .send_resp(
                        view.p2p().nodes_config.router_node,
                        0, /* dummy id */
                        proto::cache::CacheTrans {
                            filename: fname,
                            block_id: blockid,
                            store: false,
                            evict: true,
                        },
                    )
                    .await;
            });
        }
    }
    fn get(&self, file_name: &str, index: u32) -> Option<FileBlock> {
        let mut lru_cache = self.lru_cache.lock();
        lru_cache
            .get(&(file_name.to_owned(), index))
            .map(|v| (*v).clone())
    }
}
