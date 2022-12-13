extern crate core;

use crate::config::WasmConfig;
use crate::storage::StorageLibrary;
use clap::Parser;
use many_server::transport::http::HttpServer;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use tracing::info;
use tracing::level_filters::LevelFilter;

mod abi;
mod config;
mod executor;
mod storage;
mod wasm_engine;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the configuration file.
    config: PathBuf,

    /// Whether to create databases.
    #[clap(long, default_value = "false")]
    init: bool,

    /// Path to the PEM file for the identity.
    #[clap(long)]
    pem: PathBuf,

    /// Binding socket.
    #[clap(long)]
    bind: SocketAddr,

    /// Increase output logging verbosity to DEBUG level.
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Suppress all output logging. Can be used multiple times to suppress more.
    #[clap(short, long, action = clap::ArgAction::Count)]
    quiet: u8,
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let opts: Opts = Opts::parse();
    let verbose_level = 3 + opts.verbose - opts.quiet;
    let log_level = match verbose_level {
        x if x > 4 => LevelFilter::TRACE,
        4 => LevelFilter::DEBUG,
        3 => LevelFilter::INFO,
        2 => LevelFilter::WARN,
        1 => LevelFilter::ERROR,
        x if x <= 0 => LevelFilter::OFF,
        _ => unreachable!(),
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();
    info!("opts = {:?}", opts);

    let config: WasmConfig = {
        let content = std::fs::read_to_string::<&Path>(opts.config.as_ref())
            .expect("Could not read config file.");
        json5::from_str(&content).expect("Could not parse config.")
    };

    let config_dir = opts.config.parent().unwrap_or_else(|| Path::new(""));
    let config_dir = std::env::current_dir().unwrap().join(config_dir);
    let storage = StorageLibrary::create(config.storages, &config_dir, opts.init)
        .expect("Could not create storage.");

    let key = many_identity_dsa::CoseKeyIdentity::from_pem(
        std::fs::read_to_string(opts.pem).expect("Could not read PEM file."),
    )
    .expect("Could not parse PEM file.");

    let engine = wasm_engine::WasmEngine::new(config.modules, &config_dir, storage, opts.init)
        .expect("Could not load WASM.");
    let executor = executor::WasmExecutor::new(engine, key);
    let server = HttpServer::new(executor);

    server.bind(opts.bind).await.unwrap();
}
