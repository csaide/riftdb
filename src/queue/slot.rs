// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::time::Duration;

use super::{Error, Lease, Result};

/// A queue slot implementation.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub enum Slot<T> {
    /// An empty slot is available for writing a message to.
    Empty,
    /// A filled slot represents a slot that has a pending message available to be read.
    Filled(T),
    /// A locked slot represents a slot that has a message that is awaiting an Ack or Nack.
    Locked(Lease<T>),
}

impl<T> Default for Slot<T> {
    #[inline]
    fn default() -> Self {
        Self::Empty
    }
}

impl<T> Slot<T>
where
    T: Clone,
{
    fn unwrap(self) -> T {
        match self {
            Self::Empty => panic!("called `Slot::unwrap()` on a `Empty` value"),
            Self::Filled(value) => value,
            Self::Locked(.., value) => value.into_inner(),
        }
    }

    /// Check to see if this slot is currently empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    /// Check to see if this slot is currently filled and ready for reading.
    #[inline]
    pub fn is_filled(&self) -> bool {
        matches!(self, Self::Filled(..))
    }

    /// Check to see if this slot is currently locked and waiting for an ack/nack/expiration.
    #[inline]
    pub fn is_locked(&self) -> bool {
        matches!(self, Self::Locked(..))
    }

    /// Check to see if this slot is currently locked and also has an expired lease.
    #[inline]
    pub fn is_expired(&self) -> bool {
        matches!(self, Self::Locked(lease,..) if lease.expired())
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

    /// Checks whether or not the given locked slot is actually expired, if it is
    /// expired this function will transmute self into a Filled slot ready for a
    /// subscription to read.
    pub fn expired(&mut self) -> Result<bool> {
        self.check_locked()?;

        let lease = match self {
            Slot::Locked(lease) => lease,
            _ => unreachable!(),
        };
        if lease.expired() {
            let id = lease.id();
            self.nack(id).map(|_| true)
        } else {
            Ok(false)
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
    pub fn lock(&mut self, ttl: Duration) -> Result<(u64, T)> {
        self.check_filled()?;

        let value = std::mem::take(self).unwrap();
        let (lease_id, lease) = Lease::new(ttl, value.clone());
        *self = Slot::Locked(lease);
        Ok((lease_id, value))
    }

    /// Ack this slot which will forget the  previously stored value and set this slot to
    /// [Slot::Empty]. Returns an error if this slot is not currently a [Slot::Locked] variant.
    pub fn ack(&mut self, id: u64) -> Result<()> {
        self.check_locked()?;

        let lease = match self {
            Slot::Locked(lease, ..) => lease,
            _ => unreachable!(),
        };

        if !lease.valid(id) {
            return Err(Error::InvalidOrExpiredLease);
        }

        *self = Slot::Empty;
        Ok(())
    }

    /// Nack this slot which will reset this slot back to [Slot::Filled] with the existing
    /// value. Returns an error if this slot is not currently a [Slot::Locked] variant.
    pub fn nack(&mut self, id: u64) -> Result<()> {
        self.check_locked()?;

        let lease = match self {
            Slot::Locked(lease, ..) => lease,
            _ => unreachable!(),
        };

        if !lease.valid(id) {
            return Err(Error::InvalidOrExpiredLease);
        }

        let value = std::mem::take(self).unwrap();
        *self = Slot::Filled(value);
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

        let res = slot.lock(Duration::from_secs(1));
        assert!(res.is_err());

        let res = slot.ack(0);
        assert!(res.is_err());

        let res = slot.nack(0);
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
        let res = slot.lock(Duration::from_secs(10));
        assert!(res.is_ok());

        let (orig_lease_id, actual) = res.unwrap();
        assert_eq!(val, actual);

        // Nack the slot which should mean we have a filled slot again.
        let res = slot.nack(orig_lease_id);
        assert!(res.is_ok());
        assert!(slot.is_filled());

        // Lock the slot again.
        let res = slot.lock(Duration::from_secs(10));
        assert!(res.is_ok());

        let (new_lease_id, actual) = res.unwrap();
        assert_eq!(val, actual);
        assert_ne!(orig_lease_id, new_lease_id);

        // Now ack the slot which should mean we have a empty slot.
        let res = slot.ack(new_lease_id);
        assert!(res.is_ok());
        assert!(slot.is_empty());
    }
}
