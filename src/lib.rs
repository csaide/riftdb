// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

#![warn(missing_docs)]

//! Librift encapsulates all logic for riftd and riftctl.

// macro usings
#[macro_use]
extern crate slog;

/// The main gRPC server/client implementations.
pub mod grpc;
/// General log related functionality, based ontop of the [slog] ecosystem.
pub mod log;
/// Entrypoint logic for riftctl.
pub mod riftctl;
/// Entrypoint logic for riftd.
pub mod riftd;
