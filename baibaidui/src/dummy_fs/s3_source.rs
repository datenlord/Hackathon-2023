use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
};

use async_trait::async_trait;
use ws_derive::LogicalModule;

use super::{file_block::FileBlock, meta_map::MetaMap};
use crate::{
    config,
    network::proto,
    result::WSResult,
    sys::{LogicalModule, LogicalModuleNewArgs, S3SourceView},
    util::JoinHandleWrapper,
};

#[derive(LogicalModule)]
pub struct S3Source {
    meta_map: MetaMap,
    view: S3SourceView,
}

// RPC recv
// - GetCache

#[async_trait]
impl LogicalModule for S3Source {
    fn inner_new(args: LogicalModuleNewArgs) -> Self
    where
        Self: Sized,
    {
        Self {
            view: S3SourceView::new(args.logical_modules_ref.clone()),
            meta_map: config::read_yaml_file_map(args.nodes_config.file_dir),
        }
    }

    async fn start(&self) -> WSResult<Vec<JoinHandleWrapper>> {
        // register rpc
        let view = self.view.clone();
        self.view
            .p2p()
            .regist_rpc_with_receiver::<proto::cache::GetCacheRequest, _>(
                false,
                move |nid, _p2p, taskid, req| {
                    // tracing::info!("recv rpc");
                    let view = view.clone();
                    let _ = tokio::spawn(async move {
                        let res = view
                            .s3_source()
                            .read_file_block(&req.filename, req.block_id)
                            .await;
                        let _ = view
                            .p2p()
                            .send_resp(
                                nid,
                                taskid,
                                proto::cache::GetCacheResponse {
                                    is_hit: true,
                                    data: res.data,
                                },
                            )
                            .await;
                    });
                    Ok(())
                },
            );

        Ok(vec![])
    }
}

impl S3Source {
    async fn read_file_block(&self, filename: &str, block_id: u32) -> FileBlock {
        let view = self.view.clone();
        let filename = filename.to_owned();
        tokio::task::spawn_blocking(move || {
            // 打开文件只读
            let mut file = File::open(
                view.p2p()
                    .nodes_config
                    .file_dir
                    .join("datas")
                    .join(filename),
            )
            .unwrap();
            let blocksize = view.p2p().nodes_config.block_size * 1024;
            // 定位到文件的第10个字节处

            let _ = file
                .seek(SeekFrom::Start((block_id * blocksize) as u64))
                .unwrap();
            let mut buf = vec![];
            let _ = file.take(blocksize as u64).read_to_end(&mut buf);
            FileBlock { data: buf }
        })
        .await
        .unwrap()
    }
}
