// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use crate::grpc::example;
use crate::log;

use exitcode::ExitCode;
use structopt::clap::{self, crate_version, ErrorKind};
use structopt::StructOpt;

const RIFTCTL: &str = "riftctl";

/// Overall riftd binary configuration.
#[derive(Debug, Clone, StructOpt)]
#[structopt(
    global_settings = &[clap::AppSettings::DeriveDisplayOrder],
    author = "Christian Saide <me@csaide.dev>",
    about = "Manage a riftd instance or cluster."
)]
struct RiftctlConfig {
    #[structopt(flatten)]
    log_config: log::Config,
    #[structopt(
        long = "remote-addr",
        short = "r",
        env = "RIFT_REMOTE_ADDR",
        help = "The remote address to send gRPC requests to.",
        default_value = "http://127.0.0.1:8081",
        takes_value = true
    )]
    remote_addr: String,
}

/// Execute riftctl.
pub async fn run() -> ExitCode {
    let setup_logger = log::default(RIFTCTL, crate_version!());
    let cfg = match RiftctlConfig::from_args_safe() {
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

    let root_logger = log::new(&cfg.log_config, RIFTCTL, crate_version!());
    info!(root_logger, "Hello world!");

    let mut client = match example::GreeterClient::connect(cfg.remote_addr.clone()).await {
        Ok(client) => client,
        Err(err) => {
            crit!(root_logger, "Failed to connect to remote endpoint."; "error" => err.to_string(), "addr" => &cfg.remote_addr);
            return exitcode::IOERR;
        }
    };

    let request = tonic::Request::new(example::HelloRequest {
        name: "Tonic".into(),
    });

    let response = match client.say_hello(request).await {
        Ok(resp) => resp,
        Err(err) => {
            crit!(root_logger, "Failed to execute say hello on remote server."; "error" => err.to_string());
            return exitcode::IOERR;
        }
    };

    info!(root_logger, "RESPONSE={:?}", response);
    exitcode::OK
}
