// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::time::{Duration, Instant};

use super::{Error, Result};

/// A slot lease handles tying a slot index to an opaque identifier, ttl,
/// and lease start time. This is used to monitor the life cycle of leased slots
/// awaiting ack/nack operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotLease {
    ttl: Duration,
    leased_at: Instant,
    id: u64,
    idx: usize,
}

impl SlotLease {
    /// Create a new lease with the supplied ttl.
    pub fn new(idx: usize, ttl: Duration) -> Self {
        let leased_at = Instant::now();
        let id = rand::random();
        Self {
            ttl,
            leased_at,
            id,
            idx,
        }
    }

    /// Check to see if this lease is expired.
    pub fn expired(&self) -> bool {
        self.leased_at.elapsed().ge(&self.ttl)
    }

    /// Retrieve the slot index pointed to by this lease.
    pub fn index(&self) -> usize {
        self.idx
    }

    /// Retrieve the lease ID for this lease.
    pub fn id(&self) -> u64 {
        self.id
    }
}

/// A queue slot implementation.
#[derive(Debug, Copy, Clone)]
pub enum Slot<T>
where
    T: Copy + Clone + Send + Sync,
{
    /// An empty slot is available for writing a message to.
    Empty,
    /// A filled slot represents a slot that has a pending message available to be read.
    Filled(T),
    /// A locked slot represents a slot that has a message that is awaiting an Ack or Nack.
    Locked(T),
}

impl<T> Default for Slot<T>
where
    T: Copy + Clone + Send + Sync,
{
    #[inline]
    fn default() -> Self {
        Self::Empty
    }
}

impl<T> Slot<T>
where
    T: Copy + Clone + Send + Sync,
{
    fn unwrap(self) -> T {
        match self {
            Self::Empty => panic!("called `Slot::unwrap()` on a `Empty` value"),
            Self::Filled(value) => value,
            Self::Locked(value) => value,
        }
    }

    /// Check to see if this slot is currently empty.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            _ => false,
        }
    }

    /// Check to see if this slot is currently filled and ready for reading.
    pub fn is_filled(&self) -> bool {
        match self {
            Self::Filled(..) => true,
            _ => false,
        }
    }

    /// Check to see if this slot is currently locked and waiting for an ack/nack/expiration.
    pub fn is_locked(&self) -> bool {
        match self {
            Self::Locked(..) => true,
            _ => false,
        }
    }

    /// Check to see if this slot is empty returning an error if not.
    pub fn check_empty(&self) -> Result<()> {
        if self.is_empty() {
            Ok(())
        } else {
            Err(Error::MustBeEmpty)
        }
    }

    /// Check to see if this slot is filled returning an error if not.
    pub fn check_filled(&self) -> Result<()> {
        if self.is_filled() {
            Ok(())
        } else {
            Err(Error::MustBeFilled)
        }
    }

    /// Check to see if this slot is locked returning an error if not.
    pub fn check_locked(&self) -> Result<()> {
        if self.is_locked() {
            Ok(())
        } else {
            Err(Error::MustBeLocked)
        }
    }

    /// Fill this slot with the supplied value, returning an error if the current slot
    /// is not a [Slot::Empty] variant.
    pub fn fill(&mut self, value: T) -> Result<()> {
        self.check_empty()?;

        *self = Self::Filled(value);
        Ok(())
    }

    /// Lock this slots internal value, while setting a sane TTL to wait for an ack/nack. Returns
    /// an error if the slot is not currently a [Slot::Filled] variant.
    pub fn lock(&mut self) -> Result<T> {
        self.check_filled()?;

        let value = self.unwrap();
        *self = Slot::Locked(value.clone());
        Ok(value)
    }

    /// Ack this slot which will forget the  previously stored value and set this slot to
    /// [Slot::Empty]. Returns an error if this slot is not currently a [Slot::Locked] variant.
    pub fn ack(&mut self) -> Result<()> {
        self.check_locked()?;

        *self = Slot::Empty;
        Ok(())
    }

    /// Nack this slot which will reset this slot back to [Slot::Filled] with the existing
    /// value. Returns an error if this slot is not currently a [Slot::Locked] variant.
    pub fn nack(&mut self) -> Result<()> {
        self.check_locked()?;

        *self = Slot::Filled(self.unwrap());
        Ok(())
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_empty() {
        let mut slot = Slot::<usize>::Empty;
        assert!(slot.is_empty());
        assert!(!slot.is_filled());
        assert!(!slot.is_locked());

        let res = slot.lock();
        assert!(res.is_err());

        let res = slot.ack();
        assert!(res.is_err());

        let res = slot.nack();
        assert!(res.is_err());

        // Ensure we panic on unwrap.
        slot.unwrap();
    }

    #[test]
    fn test_filled() {
        let mut slot = Slot::<usize>::Empty;

        let val = 0;
        let res = slot.fill(val);
        assert!(res.is_ok());

        // Check we have a filled slot now.
        assert!(!slot.is_empty());
        assert!(slot.is_filled());
        assert!(!slot.is_locked());

        // Lock the slot and then test the value is correct.
        let res = slot.lock();
        assert!(res.is_ok());

        let actual = res.unwrap();
        assert_eq!(val, actual);

        // Nack the slot which should mean we have a filled slot again.
        let res = slot.nack();
        assert!(res.is_ok());
        assert!(slot.is_filled());

        // Lock the slot again.
        let res = slot.lock();
        assert!(res.is_ok());

        // Now ack the slot which should mean we have a empty slot.
        let res = slot.ack();
        assert!(res.is_ok());
        assert!(slot.is_empty());
    }
}
