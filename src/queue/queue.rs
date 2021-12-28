// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::{collections::HashMap, sync::Mutex, time::Duration};

use super::{Error, Result, Slot, SlotLease};

/// A basic queue implementation for testing.
#[derive(Debug)]
pub struct Queue<T, const N: usize>
where
    T: Copy + Clone + Send + Sync,
{
    slots: Mutex<[Slot<T>; N]>,
    leases: Mutex<HashMap<u64, SlotLease>>,
}

impl<T, const N: usize> Default for Queue<T, N>
where
    T: Copy + Clone + Send + Sync,
{
    fn default() -> Self {
        // Create backing store for messages.
        let array = [Slot::default(); N];
        let slots = Mutex::new(array);

        // Create lease tracker.
        let map = HashMap::with_capacity(N);
        let leases = Mutex::new(map);

        // Return a new queue.
        Self { slots, leases }
    }
}

impl<T, const N: usize> Queue<T, N>
where
    T: Copy + Clone + Send + Sync,
{
    /// Ack the given message index.
    pub fn ack(&mut self, lease_id: u64) -> Result<()> {
        let index = {
            let mut leases = self.leases.lock().unwrap();

            let lease = match leases.remove(&lease_id) {
                Some(lease) if !lease.expired() => lease,
                _ => return Err(Error::InvalidOrExpiredLease),
            };
            lease.index()
        };

        let mut slots = self.slots.lock().unwrap();
        slots[index].ack()
    }

    /// Nack the given message index.
    pub fn nack(&mut self, lease_id: u64) -> Result<()> {
        let index = {
            let mut leases = self.leases.lock().unwrap();

            let lease = match leases.remove(&lease_id) {
                Some(lease) if !lease.expired() => lease,
                _ => return Err(Error::InvalidOrExpiredLease),
            };
            lease.index()
        };

        let mut slots = self.slots.lock().unwrap();
        slots[index].nack()
    }

    /// Push a new message into the queue.
    pub fn push(&mut self, msg: T) -> Result<()> {
        let mut slots = self.slots.lock().unwrap();
        let empty = match slots.iter_mut().filter(|slot| slot.is_empty()).next() {
            Some(empty) => empty,
            None => return Err(Error::QueueFull),
        };
        empty.fill(msg)
    }

    /// Get the next available message from the front of the queue.
    pub fn next(&self) -> Option<(u64, T)> {
        let mut slots = self.slots.lock().unwrap();
        let (idx, next) = match slots
            .iter_mut()
            .enumerate()
            .filter(|(_, slot)| slot.is_filled())
            .next()
        {
            Some(next) => next,
            None => return None,
        };

        let mut leases = self.leases.lock().unwrap();
        match next.lock() {
            Ok(val) => {
                let lease = SlotLease::new(idx, Duration::from_secs(10));
                let lease_id = lease.id();
                leases.insert(lease_id, lease);
                Some((lease_id, val))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_queue() {
        let mut queue = Queue::<usize, 1>::default();

        let msg = 1000 as usize;
        queue.push(msg).unwrap();
        let actual = queue.next();
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert_eq!(actual.1, msg);

        let res = queue.nack(actual.0);
        assert!(res.is_ok());

        let actual = queue.next();
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert_eq!(actual.1, msg);

        let res = queue.ack(actual.0);
        assert!(res.is_ok());

        let actual = queue.next();
        assert!(actual.is_none());
    }
}
