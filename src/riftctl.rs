// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

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
    exitcode::OK
}
