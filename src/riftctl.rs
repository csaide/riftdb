// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use crate::grpc::kv;
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

    let mut client = match kv::KvClient::connect(cfg.remote_addr.clone()).await {
        Ok(client) => client,
        Err(err) => {
            crit!(root_logger, "Failed to connect to remote endpoint."; "error" => err.to_string(), "addr" => &cfg.remote_addr);
            return exitcode::IOERR;
        }
    };

    let key = Vec::from(b"i_am_a_key".as_ref());

    let request = tonic::Request::new(kv::KeyValue {
        ttl: 1000000000,
        key: key.clone(),
        value: Vec::from(b"i_am_a_value".as_ref()),
    });

    let response = match client.set(request).await {
        Ok(resp) => resp,
        Err(err) => {
            crit!(root_logger, "Failed to execute say hello on remote server."; "error" => err.to_string());
            return exitcode::IOERR;
        }
    };

    info!(root_logger, "RESPONSE={:?}", response);

    let request = tonic::Request::new(kv::Key { key: key.clone() });

    let response = match client.get(request).await {
        Ok(resp) => resp,
        Err(err) => {
            crit!(root_logger, "Failed to retrieve set value."; "error" => err.to_string());
            return exitcode::IOERR;
        }
    };

    info!(root_logger, "RESPONSE={:?}", response);

    exitcode::OK
}
