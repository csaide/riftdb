// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

#![warn(missing_docs)]

//! Librift encapsulates all logic for riftd and riftctl.

// macro usings
#[macro_use]
extern crate slog;
#[macro_use]
extern crate prometheus;

/// The main gRPC server/client implementations.
pub mod grpc;
/// Debugging/Control Plane HTTP handling.
pub mod http;
/// General log related functionality, based ontop of the [slog] ecosystem.
pub mod log;
/// Prometheus metrics logic and handling.
pub mod metric;
/// Pubsub implementation.
pub mod pubsub;
/// Entrypoint logic for riftctl.
pub mod riftctl;
/// Entrypoint logic for riftd.
pub mod riftd;
