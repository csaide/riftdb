// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::{
    ops::Add,
    time::{Duration, Instant, SystemTime},
};

/// A lease tag is used to capture the various pieces of metadata to expose to the caller for this lease.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct LeaseTag {
    /// The identifier of the [Lease] instance that this tag represents.
    pub id: u64,
    /// The ttl of the [Lease].
    pub ttl: Duration,
    /// The system time of when this lease was originally created.
    pub leased_at: SystemTime,
    /// The estimated system time when this lease will expire.
    pub deadline: SystemTime,
}

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
    pub fn new(ttl: Duration, inner: T) -> (LeaseTag, Self) {
        let now = SystemTime::now();
        let leased_at_instant = Instant::now();
        let id = rand::random();
        (
            LeaseTag {
                id,
                ttl,
                leased_at: now,
                deadline: now.add(ttl),
            },
            Self {
                ttl,
                leased_at: leased_at_instant,
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
