#![allow(clippy::all)]
#![deny(
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::unnecessary_mut_passed,
    unused_results
)]

use std::env;

use clap::Parser;
use cmd_arg::CmdArgs;

use sys::Sys;
use tracing_subscriber::{
    prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

// #[macro_use]
// pub mod module_view;
pub mod cmd_arg;
pub mod config;
pub mod dummy_fs;
pub mod event;
pub mod metric;
pub mod network;
pub mod result;
mod sys;
pub mod util;

#[tokio::main]
async fn main() {
    start_tracing();

    // tracing_subscriber::fmt::init();
    tracing::info!("current dir:{:?}", env::current_dir());
    let args = CmdArgs::parse();
    // if args.deploy.is_some() {
    //     deploy::deploy(args).await;
    //     return;
    // }
    let config = config::read_config(args.this_id, args.files_dir);
    tracing::info!("config: {:?}", config);
    // dist_kv_raft::tikvraft_proxy::start();
    Sys::new(config).wait_for_end().await;
}

pub fn start_tracing() {
    let my_filter = tracing_subscriber::filter::filter_fn(|v| {
        // println!("{}", v.module_path().unwrap());
        // println!("{}", v.name());
        // if v.module_path().unwrap().contains("quinn_proto") {
        //     return false;
        // }

        // if v.module_path().unwrap().contains("qp2p::wire_msg") {
        //     return false;
        // }

        // println!("{}", v.target());
        if let Some(mp) = v.module_path() {
            if mp.contains("hyper") {
                return false;
            }
        }

        // if v.module_path().unwrap().contains("less::network::p2p") {
        //     return false;
        // }

        // v.level() == &tracing::Level::ERROR
        //     || v.level() == &tracing::Level::WARN
        //     || v.level() == &tracing::Level::INFO
        v.level() != &tracing::Level::TRACE
        // v.level() == &tracing::Level::INFO
        // true
    });
    let my_layer = tracing_subscriber::fmt::layer();
    tracing_subscriber::registry()
        .with(my_layer.with_filter(my_filter))
        .init();
}

pub fn new_test_systems() -> Vec<Sys> {
    let mut systems = vec![];
    for i in 1..4 {
        let config = config::read_config(i, "node_config.yaml");
        tracing::info!("config: {:?}", config);
        systems.push(Sys::new(config));
    }
    systems
}
