// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::sync::{Arc, Mutex};
use std::time::Duration;

use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter, IntGauge};

use super::{Result, Slot};

lazy_static! {
    static ref TOTAL_MESSAGES_RECEIVED: IntCounter = register_int_counter!(
        "rift_const_queue_received_messages",
        "The total number of messages received by all const_queues."
    )
    .unwrap();
    static ref TOTAL_MESSAGES_ACKED: IntCounter = register_int_counter!(
        "rift_const_queue_acked_messages",
        "The total number of messages successfuly acked by all const_queues."
    )
    .unwrap();
    static ref TOTAL_MESSAGES_NACKED: IntCounter = register_int_counter!(
        "rift_const_queue_nacked_messages",
        "The total number of messages successfuly nacked by all const_queues."
    )
    .unwrap();
    static ref TOTAL_MESSAGES_OUTSTANDING: IntGauge = register_int_gauge!(
        "rift_const_queue_outstanding_messages",
        "The totall number of messages currently locked across all const_queues."
    )
    .unwrap();
    static ref TOTAL_MESSAGES_PENDING: IntGauge = register_int_gauge!(
        "rift_const_queue_pending_messages",
        "The total number of messages currently pending across all const_queues."
    )
    .unwrap();
}

/// A basic queue implementation based on a const sized backing buffer.
#[derive(Debug, Clone)]
pub struct UnboundedQueue<T> {
    slots: Arc<Mutex<Vec<Slot<T>>>>,
}

impl<T> Default for UnboundedQueue<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> UnboundedQueue<T> {
    /// Create a new unbounded queue with no defined capacity.
    pub fn new() -> Self {
        // Create backing store for messages.
        let slots = Arc::new(Mutex::new(Vec::new()));
        // Return a new queue.
        Self { slots }
    }

    /// Create a new unbounded queue with a defined initial message capacity.
    pub fn with_capacity(cap: usize) -> Self {
        let slots = Arc::new(Mutex::new(Vec::with_capacity(cap)));
        Self { slots }
    }
}

impl<T> UnboundedQueue<T>
where
    T: Clone,
{
    /// Ack the given message index.
    pub fn ack(&self, lease_id: u64, index: usize) -> Result<()> {
        let mut slots = self.slots.lock().unwrap();
        let res = slots[index].ack(lease_id);
        if res.is_ok() {
            TOTAL_MESSAGES_ACKED.inc();
            TOTAL_MESSAGES_OUTSTANDING.dec();
        }
        res
    }

    /// Nack the given message index.
    pub fn nack(&self, lease_id: u64, index: usize) -> Result<()> {
        let mut slots = self.slots.lock().unwrap();
        let res = slots[index].nack(lease_id);
        if res.is_ok() {
            TOTAL_MESSAGES_NACKED.inc();
            TOTAL_MESSAGES_PENDING.inc();
            TOTAL_MESSAGES_OUTSTANDING.dec();
        }
        res
    }

    /// Push a new message into the queue.
    pub fn push(&self, msg: T) -> Result<()> {
        let mut slots = self.slots.lock().unwrap();
        let empty = match slots.iter_mut().find(|slot| slot.is_empty()) {
            Some(empty) => empty,
            None => {
                slots.push(Slot::Empty);
                slots.last_mut().unwrap()
            }
        };

        let res = empty.fill(msg);
        if res.is_ok() {
            TOTAL_MESSAGES_RECEIVED.inc();
            TOTAL_MESSAGES_PENDING.inc();
        }
        res
    }

    /// Get the next available message from the front of the queue.
    pub fn next(&self) -> Option<(u64, usize, T)> {
        let mut slots = self.slots.lock().unwrap();
        let (idx, next) = match slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_filled() || slot.is_expired())
        {
            Some((idx, next)) if next.is_filled() => (idx, next),
            Some((idx, next)) if next.is_expired() => {
                if next.expired().is_err() {
                    return None;
                }
                (idx, next)
            }
            _ => return None,
        };

        let res = next
            .lock(Duration::from_secs(10))
            .ok()
            .map(|(lease_id, val)| (lease_id, idx, val));
        if res.is_some() {
            TOTAL_MESSAGES_PENDING.dec();
            TOTAL_MESSAGES_OUTSTANDING.inc();
        }
        res
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_queue() {
        let queue = UnboundedQueue::<usize>::default();

        let msg = 1000 as usize;
        queue.push(msg).unwrap();
        let actual = queue.next();
        assert!(actual.is_some());
        let (first_lease_id, first_idx, actual) = actual.unwrap();
        assert_eq!(actual, msg);

        let res = queue.nack(first_lease_id, first_idx);
        assert!(res.is_ok());

        let actual = queue.next();
        assert!(actual.is_some());
        let (second_lease_id, second_idx, actual) = actual.unwrap();
        assert_eq!(actual, msg);

        let res = queue.ack(second_lease_id, second_idx);
        assert!(res.is_ok());

        let actual = queue.next();
        assert!(actual.is_none());
    }
}
