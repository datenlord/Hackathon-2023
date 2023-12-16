use crate::{
    config::{self, YamlUser},
    result::WSResult,
    sys::{LogicalModule, LogicalModuleNewArgs, SimUserView},
    util::JoinHandleWrapper,
};
use async_trait::async_trait;
use std::sync::atomic::AtomicU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use ws_derive::LogicalModule;
enum AcessMode {
    Loop,
    Random,
    LoopParallel,
}

#[derive(LogicalModule)]
pub struct SimUser {
    focus_target: Vec<String>,
    acess_mode: AcessMode,
    view: SimUserView,
}

#[async_trait]
impl LogicalModule for SimUser {
    fn inner_new(args: LogicalModuleNewArgs) -> Self
    where
        Self: Sized,
    {
        // read from user.yaml
        let users = config::read_yaml_users(args.nodes_config.file_dir);
        let YamlUser { targets, access } = users.get(&args.nodes_config.this.0).unwrap().clone();
        Self {
            focus_target: targets,
            acess_mode: match &*access {
                "loop" => AcessMode::Loop,
                "random" => AcessMode::Random,
                "loop_parallel" => AcessMode::LoopParallel,
                _ => panic!("invalid access mode"),
            },
            view: SimUserView::new(args.logical_modules_ref.clone()),
        }
    }

    async fn start(&self) -> WSResult<Vec<JoinHandleWrapper>> {
        let view = self.view.clone();
        Ok(vec![tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            loop {
                match view.sim_user().acess_mode {
                    AcessMode::LoopParallel => {
                        for target in view.sim_user().focus_target.iter() {
                            let cnt = view.fs_node().file_block_cnt(target);
                            let block_cnter = Arc::new(AtomicU32::new(0));
                            let mut tasks = vec![];
                            for _ in 0..10 {
                                let block_cnter = block_cnter.clone();
                                let target = target.clone();
                                let view = view.clone();
                                tasks.push(tokio::spawn(async move {
                                    loop {
                                        let next = block_cnter.fetch_add(1, Ordering::Relaxed);
                                        if next < cnt {
                                            let res = view
                                                .fs_node()
                                                .read_file_block(false, &target, next)
                                                .await;
                                            if let Err(res) = res {
                                                tracing::error!(
                                                    "user {} read file block: {},{} err:{:?}",
                                                    view.p2p().nodes_config.this.0,
                                                    target,
                                                    next,
                                                    res
                                                );
                                            } else {
                                            }
                                        } else {
                                            break;
                                        }
                                    }
                                }));
                            }
                            for t in tasks {
                                let _ = t.await;
                            }
                            // for i in 0..cnt {
                            //     let view = view.clone();
                            //     let target=target.clone();
                            //     tasks.push(tokio::spawn(async move{
                            //         view.fs_node().read_file_block(false, &target, i).await
                            //     }));
                            // }
                            // for t in tasks {
                            //     let res = t.await.unwrap();
                            //     if let Err(res) = res {
                            //         tracing::error!(
                            //             "user {} read file block err:{:?}",
                            //             view.p2p().nodes_config.this.0,
                            //             // target,
                            //             // i,
                            //             res
                            //         );

                            //     } else {
                            //         // tracing::info!(
                            //         //     "user {} read file block: {},{} succ:{}",
                            //         //     view.p2p().nodes_config.this.0,
                            //         //     target,
                            //         //     i,
                            //         //     res.is_ok()
                            //         // );
                            //     }
                            // }
                        }
                    }
                    AcessMode::Loop => {
                        for target in view.sim_user().focus_target.iter() {
                            let cnt = view.fs_node().file_block_cnt(target);

                            for i in 0..cnt {
                                let res = view.fs_node().read_file_block(false, target, i).await;
                                if let Err(res) = res {
                                    tracing::error!(
                                        "user {} read file block: {},{} err:{:?}",
                                        view.p2p().nodes_config.this.0,
                                        target,
                                        i,
                                        res
                                    );
                                } else {
                                    // tracing::info!(
                                    //     "user {} read file block: {},{} succ:{}",
                                    //     view.p2p().nodes_config.this.0,
                                    //     target,
                                    //     i,
                                    //     res.is_ok()
                                    // );
                                }
                            }
                        }
                    }
                    AcessMode::Random => {
                        for target in view.sim_user().focus_target.iter() {
                            let cnt = view.fs_node().file_block_cnt(target);
                            for _ in 0..cnt {
                                let i = rand::random::<u32>() % cnt;
                                let res = view.fs_node().read_file_block(false, target, i).await;

                                if let Err(res) = res {
                                    tracing::error!(
                                        "user {} read file block: {},{} err:{:?}",
                                        view.p2p().nodes_config.this.0,
                                        target,
                                        i,
                                        res
                                    );
                                } else {
                                    // tracing::info!(
                                    //     "user {} read file block: {},{} succ:{}",
                                    //     view.p2p().nodes_config.this.0,
                                    //     target,
                                    //     i,
                                    //     res.is_ok()
                                    // );
                                }
                            }
                        }
                    }
                }
            }
        })
        .into()])
    }
}
