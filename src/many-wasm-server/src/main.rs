extern crate core;

use clap::Parser;
use many_server::transport::http::HttpServer;
use std::net::SocketAddr;
use std::path::PathBuf;
use tracing::info;
use tracing::level_filters::LevelFilter;

mod abi;
mod executor;
mod wasm_engine;

#[derive(Debug, Parser)]
struct Opts {
    /// Path to the WASM to host.
    wasm: PathBuf,

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
    let verbose_level = 2 + opts.verbose - opts.quiet;
    let log_level = match verbose_level {
        x if x > 3 => LevelFilter::TRACE,
        3 => LevelFilter::DEBUG,
        2 => LevelFilter::INFO,
        1 => LevelFilter::WARN,
        0 => LevelFilter::ERROR,
        x if x <= 0 => LevelFilter::OFF,
        _ => unreachable!(),
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();
    info!("opts = {:?}", opts);

    let key = many_identity_dsa::CoseKeyIdentity::from_pem(
        std::fs::read_to_string(opts.pem).expect("Could not read PEM file."),
    )
    .expect("Could not parse PEM file.");

    let engine = wasm_engine::WasmEngine::new(opts.wasm).expect("Could not load WASM.");
    let executor = executor::WasmExecutor::new(engine, key);
    let server = HttpServer::new(executor);

    server.bind(opts.bind).await.unwrap();
}
