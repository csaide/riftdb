// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::time;

use bytes::Bytes;

mod error;
mod hash;

pub use error::{Error, Result};
pub use hash::HashStore;

/// The Store trait represents a backing store for the KV service. The trait encompasses
/// the various methods every Store requires to be leveraged by rift.
#[tonic::async_trait]
pub trait Store {
    /// Retrieve the specified value if it exists. Returning the old valu if it exists,
    /// and/or any errors encountered.
    async fn get(&self, key: &Bytes) -> Result<Option<Bytes>>;
    /// Set the specified value at the specified key with the specified ttl. Returning
    /// any errors encountered.
    async fn set(&self, key: Bytes, value: Bytes, ttl: time::Duration) -> Result<Option<Bytes>>;
    /// Selete the specified value if it exists. Returing the old value if it exists,
    /// and/or any errors encountered.
    async fn delete(&self, key: &Bytes) -> Result<Option<Bytes>>;
}
