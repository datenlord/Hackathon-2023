//! Datenlord p2p server
//!

use std::{fs::File, io::Write, path::Path};

use clap::Parser;

use tracing::{info, warn};

use p2p_with_tracker::config::Config;
use p2p_with_tracker::constant::*;
use p2p_with_tracker::logger;
use p2p_with_tracker::server;

#[derive(Debug, Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    version = env!("CARGO_PKG_VERSION"),
)]
struct Cli {
    #[clap(short = 'c', long = "config", help = "Configuration file path", default_value = DEFAULT_CONFIG_FILE, display_order = 1)]
    config: String,
    #[clap(short = 'm', long = "mode", help = "Server running mode", default_value = DEFAULT_SERVER_MODE, display_order = 2)]
    mode: String,
    #[clap(short = 'f', long = "filename", help = "Download s3 filename", default_value = DEFAULT_FILENAME, display_order = 3)]
    filename: String,
}

#[tokio::main]
async fn main() {
    logger::init();

    let args = Cli::parse();

    let config_path = args.config;

    // if configuration file doesn't exist, create it
    if !Path::new(&config_path).exists() {
        warn!("Configuration file doesn't exists.");

        let mut file =
            File::create(config_path).expect("notrace - Failed to create configuration file");

        file.write_all(DEFAULT_CONFIG_CONTENT)
            .expect("notrace - Failed to write default configuration to config file");

        info!("Created configuration file. Exiting...");

        std::process::exit(0);
    }

    let mut config =
        Config::parse(&config_path).expect("notrace - Failed to parse configuration file");

    // Print configuration
    info!("Configuration: {:?}", config);

    server::start_server(&mut config, args.mode, args.filename)
        .await
        .expect("notrace - HTTP Server error");
}
