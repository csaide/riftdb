// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::sync::{Arc, Mutex};
use std::task;
use std::time::Duration;

use uuid::Uuid;

use super::{Error, LeaseTag, Result, Slot, Waker};

pub const DEFAULT_TTL: Duration = Duration::from_secs(10);
pub const NO_CAPACITY: usize = 0;

/// The queue builder enables simple setting of various configuraiton options
/// on a [Queue] instance.
#[derive(Debug, Default)]
pub struct QueueBuilder {
    message_cap: Option<usize>,
    subscription_cap: Option<usize>,
    ttl: Option<Duration>,
}

impl QueueBuilder {
    /// Set the initial message capacity of the [Queue].
    pub fn with_message_capacity(mut self, cap: usize) -> Self {
        self.message_cap = Some(cap);
        self
    }

    /// Set the initial subscription capacity of the [Queue].
    pub fn with_subscription_capacity(mut self, cap: usize) -> Self {
        self.subscription_cap = Some(cap);
        self
    }

    /// Set the message ttl of the [Queue].
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    /// Build the resulting [Queue].
    pub fn build<T>(self) -> Queue<T> {
        Queue::build(self)
    }
}

/// A basic queue implementation.
#[derive(Debug, Clone)]
pub struct Queue<T> {
    ttl: Duration,
    slots: Arc<Mutex<Vec<Slot<T>>>>,
    pub(crate) waker: Arc<Mutex<Waker>>,
}

impl<T> Queue<T> {
    fn build(builder: QueueBuilder) -> Self {
        let slots = Vec::with_capacity(builder.message_cap.unwrap_or(NO_CAPACITY));
        let slots = Arc::new(Mutex::new(slots));

        let waker = Waker::with_capacity(builder.subscription_cap.unwrap_or(NO_CAPACITY));
        let waker = Arc::new(Mutex::new(waker));
        Self {
            ttl: builder.ttl.unwrap_or(DEFAULT_TTL),
            slots,
            waker,
        }
    }

    /// Create a new builder to define the various options for the unbounded queue instance.
    pub fn builder() -> QueueBuilder {
        QueueBuilder::default()
    }

    /// Create a new unbounded queue with no defined capacity and a default lease TTL of 10s.
    pub fn new() -> Self {
        // Create backing store for messages.
        let slots = Arc::new(Mutex::new(Vec::new()));
        let waker = Arc::new(Mutex::new(Waker::default()));
        // Return a new queue.
        Self {
            ttl: DEFAULT_TTL,
            slots,
            waker,
        }
    }
}

impl<T> Queue<T>
where
    T: Clone,
{
    #[doc(hidden)]
    pub fn register_task_waker(&self, id: Uuid, waker: task::Waker) {
        self.waker.lock().unwrap().register(id, waker)
    }

    /// Ack the given message index.
    pub fn ack(&self, lease_id: u64, index: usize) -> Result<()> {
        let mut slots = self.slots.lock().unwrap();
        if index >= slots.len() {
            return Err(Error::IndexOutOfRange);
        }
        let res = slots[index].ack(lease_id);
        if res.is_ok() {
            // MESSAGE_RESULTS.with_label_values(&[ACK_VALUE]).inc();
            // MESSAGES_OUTSTANDING.dec();
        }
        res
    }

    /// Nack the given message index.
    pub fn nack(&self, lease_id: u64, index: usize) -> Result<()> {
        let mut slots = self.slots.lock().unwrap();
        if index >= slots.len() {
            return Err(Error::IndexOutOfRange);
        }
        let res = slots[index].nack(lease_id);
        if res.is_ok() {
            // MESSAGE_RESULTS.with_label_values(&[NACK_VALUE]).inc();
            // MESSAGES_PENDING.inc();
            // MESSAGES_OUTSTANDING.dec();
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
            // TOTAL_MESSAGES_RECEIVED.inc();
            // MESSAGES_PENDING.inc();

            // Lets wake the oldest waker, if it exists, so that it can consume
            // this new message on the next poll.
            self.waker.lock().unwrap().wake();
        }
        res
    }

    /// Get the next available message from the front of the queue.
    pub fn next(&self) -> Option<(LeaseTag, usize, T)> {
        let mut slots = self.slots.lock().unwrap();
        let (idx, next) = match slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_filled())
        {
            Some(res) => res,
            _ => return None,
        };

        let res = next.lock(self.ttl).ok().map(|(tag, val)| (tag, idx, val));
        if res.is_some() {
            // MESSAGES_PENDING.dec();
            // MESSAGES_OUTSTANDING.inc();
        }
        res
    }
}

impl<T> Default for Queue<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_builder() {
        Queue::<usize>::builder()
            .with_message_capacity(1024)
            .with_subscription_capacity(1023)
            .with_ttl(Duration::from_millis(100))
            .build::<usize>();
    }

    #[test]
    fn test_queue() {
        let queue = Queue::<usize>::default();

        let msg = 1000 as usize;
        queue.push(msg).unwrap();
        let actual = queue.next();
        assert!(actual.is_some());
        let (first_lease_tag, first_idx, actual) = actual.unwrap();
        assert_eq!(actual, msg);

        let res = queue.nack(first_lease_tag.id, first_idx);
        assert!(res.is_ok());

        let actual = queue.next();
        assert!(actual.is_some());
        let (second_lease_tag, second_idx, actual) = actual.unwrap();
        assert_eq!(actual, msg);

        let res = queue.ack(second_lease_tag.id, second_idx);
        assert!(res.is_ok());

        let actual = queue.next();
        assert!(actual.is_none());
    }
}
