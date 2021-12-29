// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::time::{Duration, Instant};

/// A slot lease handles tying a slot index to an opaque identifier, ttl,
/// and lease start time. This is used to monitor the life cycle of leased slots
/// awaiting ack/nack operations.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Lease<T> {
    ttl: Duration,
    leased_at: Instant,
    id: u64,
    inner: T,
}

impl<T> Lease<T> {
    /// Create a new lease with the supplied ttl.
    pub fn new(ttl: Duration, inner: T) -> (u64, Self) {
        let leased_at = Instant::now();
        let id = rand::random();
        (
            id,
            Self {
                ttl,
                leased_at,
                id,
                inner,
            },
        )
    }

    /// Return the identifier for this lease.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Unwrap this lease into its inner type.
    pub fn into_inner(self) -> T {
        self.inner
    }

    /// Check to see if this lease is expired.
    pub fn expired(&self) -> bool {
        self.leased_at.elapsed().ge(&self.ttl)
    }

    /// Checks wether or not the supplied id matches this leases'.
    pub fn valid(&self, o: u64) -> bool {
        self.id == o
    }
}
