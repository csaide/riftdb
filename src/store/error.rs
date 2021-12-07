// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

// stdlib usings
use std::result;

// extern usings
use thiserror::Error;

/// Custom Result wrapper to simplify usage.
pub type Result<T> = result::Result<T, Error>;

/// Represents logging errors based on user configuration or OS
/// errors while attempting to configure log handlers.
#[derive(Error, Debug)]
pub enum Error {}
