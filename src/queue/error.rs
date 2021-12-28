// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

use std::result;

use thiserror::Error;

/// Custom Result wrapper to simplify usage.
pub type Result<T> = result::Result<T, Error>;

/// Represents queue and slot related errors.
#[derive(Error, Debug)]
pub enum Error {
    /// An error which occurs when an operation like ack/nack are made against
    /// non-locked slots.
    #[error("the slot must be locked for this operation to complete")]
    MustBeLocked,
    /// An error which occurs when an operation like lock is made against
    /// non-filled slots.
    #[error("the slot must be filled for this operation to complete")]
    MustBeFilled,
    /// An error which occurs when an operation like fill is made against
    /// non-empty slots.
    #[error("the slot must be empty for this operation to complete")]
    MustBeEmpty,
    /// An error which occurs when attempting to ack/nack a lease with an invalid
    /// lease id.
    #[error("the specified lease is either invalid, missing, or expired")]
    InvalidOrExpiredLease,
    /// An error which occurs when there are no available empty slots.
    #[error("the queue is full and unable to accept new messages")]
    QueueFull,
}
