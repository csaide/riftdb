// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;
use std::time;

use bytes::Bytes;
use tokio::sync::RwLock;

use super::{Result, Store};

struct Value {
    ttl: time::Duration,
    created: time::Instant,
    payload: Bytes,
}

/// A [HashStore] instance represents an in-memory [HashMap] based backing store.
pub struct HashStore {
    data: RwLock<HashMap<Bytes, Value>>,
}

impl HashStore {
    /// Create a new [HashStore] with a default capacity of `1024`.
    pub fn new() -> HashStore {
        let data = HashMap::with_capacity(1024);
        let data = RwLock::new(data);
        HashStore { data }
    }

    async fn insert(
        &self,
        key: Bytes,
        payload: Bytes,
        ttl: time::Duration,
    ) -> Result<Option<Bytes>> {
        let mut guard = self.data.write().await;
        let old = guard
            .insert(
                key,
                Value {
                    ttl,
                    created: time::Instant::now(),
                    payload,
                },
            )
            .map(|val| val.payload);
        Ok(old)
    }

    async fn retrieve(&self, key: &Bytes) -> Result<Option<Bytes>> {
        let guard = self.data.read().await;
        let value = match guard.get(key) {
            None => None,
            Some(value) if value.ttl <= value.created.elapsed() => {
                return self.remove(key).await;
            }
            Some(value) => Some(value.payload.clone()),
        };
        Ok(value)
    }

    async fn remove(&self, key: &Bytes) -> Result<Option<Bytes>> {
        let mut guard = self.data.write().await;
        let value = guard.remove(key).map(|val| val.payload);
        Ok(value)
    }
}

impl Default for HashStore {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl Store for HashStore {
    /// Retrieve the specified value if it exists. Returning the old valu if it exists,
    /// and/or any errors encountered.
    #[inline]
    async fn get(&self, key: &Bytes) -> Result<Option<Bytes>> {
        self.retrieve(key).await
    }

    /// Set the specified value at the specified key with the specified ttl. Returning
    /// any errors encountered.
    #[inline]
    async fn set(&self, key: Bytes, value: Bytes, ttl: time::Duration) -> Result<Option<Bytes>> {
        self.insert(key, value, ttl).await
    }

    /// Selete the specified value if it exists. Returing the old value if it exists,
    /// and/or any errors encountered.
    #[inline]
    async fn delete(&self, key: &Bytes) -> Result<Option<Bytes>> {
        self.remove(key).await
    }
}
