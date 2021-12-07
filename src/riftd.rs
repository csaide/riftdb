// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::net::SocketAddr;

use crate::grpc::kv;
use crate::http;
use crate::log;
use crate::metric;
use crate::store;

use exitcode::ExitCode;
use structopt::clap::{self, crate_version, ErrorKind};
use structopt::StructOpt;
use tonic::transport::Server;

const RIFTD: &str = "riftd";

/// Overall riftd binary configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    global_settings = &[clap::AppSettings::DeriveDisplayOrder],
    author = "Christian Saide <me@csaide.dev>",
    about = "Run an instance of riftd.",
    version = crate_version!()
)]
struct RiftdConfig {
    #[structopt(flatten)]
    log_config: log::Config,
    #[structopt(
        long = "grpc-addr",
        short = "g",
        env = "RIFT_GRPC_ADDR",
        help = "The address to listen on for incoming gRPC requests.",
        long_help = "This sets the listen address for all incoming gRPC requests.",
        default_value = "[::]:8081",
        takes_value = true
    )]
    grpc_addr: SocketAddr,
    #[structopt(
        long = "http-addr",
        short = "a",
        env = "RIFT_HTTP_ADDR",
        help = "The address to listen on for incoming HTTP requests.",
        long_help = "This sets the listen address for all incoming HTTP control plane requests.",
        default_value = "[::]:8080",
        takes_value = true
    )]
    http_addr: SocketAddr,
}

/// Execute riftd.
pub async fn run() -> ExitCode {
    let setup_logger = log::default(RIFTD, crate_version!());
    let cfg = match RiftdConfig::from_args_safe() {
        Ok(cfg) => cfg,
        Err(err)
            if err.kind == ErrorKind::HelpDisplayed || err.kind == ErrorKind::VersionDisplayed =>
        {
            println!("{}", err.message);
            return exitcode::USAGE;
        }
        Err(err) => {
            crit!(setup_logger, "Failed to parse provided configuration."; "error" => err.to_string());
            return exitcode::CONFIG;
        }
    };

    let root_logger = log::new(&cfg.log_config, RIFTD, crate_version!());
    info!(root_logger, "Hello world!");

    let mm = metric::Manager::new(
        "rift".to_string(),
        "grpc".to_string(),
        "riftd".to_string(),
        crate_version!().to_string(),
    );

    let memory = store::HashStore::new();
    let kv_impl = kv::KVImpl::new(memory);

    let grpc_logger = root_logger.new(o!("mod" => "grpc"));
    let grpc_handle = async move {
        if let Err(err) = Server::builder()
            .add_service(kv::KvServer::with_interceptor(
                kv_impl,
                crate::grpc::interceptor::RiftInterceptor::new(&grpc_logger, mm),
            ))
            .serve(cfg.grpc_addr)
            .await
        {
            crit!(&grpc_logger, "Failed to listen and serve gRPC."; "error" => err.to_string());
        }
    };

    let http_logger = root_logger.new(o!("mod" => "http"));
    let http_handle = async move {
        if let Err(err) = http::listen(&cfg.http_addr).await {
            crit!(&http_logger, "Failed to listen and serve HTTP."; "error" => err.to_string());
        }
    };

    tokio::select! {
        _ = grpc_handle => {},
        _ = http_handle => {},
    };

    exitcode::IOERR
}
