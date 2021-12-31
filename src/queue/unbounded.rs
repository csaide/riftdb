// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::time::Duration;

use futures_core::Stream;
use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter, IntCounterVec, IntGauge};

use super::{LeaseTag, Result, Slot};

const ACK_VALUE: &str = "ack";
const NACK_VALUE: &str = "nack";
const DEFAULT_TTL: Duration = Duration::from_secs(10);
const NO_CAPACITY: usize = 0;

lazy_static! {
    static ref TOTAL_MESSAGES_RECEIVED: IntCounter = register_int_counter!(
        "rift_const_queue_received_messages",
        "The total number of messages received by all const_queues."
    )
    .unwrap();
    static ref MESSAGE_RESULTS: IntCounterVec = register_int_counter_vec!(
        "rift_const_queue_message_results",
        "The number of handled messages by result type across all const_queues.",
        &["result"],
    )
    .unwrap();
    static ref MESSAGE_LEASE_EXPIRES: IntCounter = register_int_counter!(
        "rift_const_queue_message_lease_expires",
        "The number of message leases that have expired across all const_queues."
    )
    .unwrap();
    static ref MESSAGES_OUTSTANDING: IntGauge = register_int_gauge!(
        "rift_const_queue_outstanding_messages",
        "The totall number of messages currently locked across all const_queues."
    )
    .unwrap();
    static ref MESSAGES_PENDING: IntGauge = register_int_gauge!(
        "rift_const_queue_pending_messages",
        "The total number of messages currently pending across all const_queues."
    )
    .unwrap();
}

#[derive(Debug, Default)]
pub struct Builder {
    cap: Option<usize>,
    ttl: Option<Duration>,
}

impl Builder {
    pub fn with_capacity(mut self, cap: usize) -> Self {
        self.cap = Some(cap);
        self
    }

    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn build<T>(self) -> UnboundedQueue<T> {
        UnboundedQueue::build(self)
    }
}

/// A basic queue implementation based on a const sized backing buffer.
#[derive(Debug, Clone)]
pub struct UnboundedQueue<T> {
    ttl: Duration,
    slots: Arc<Mutex<Vec<Slot<T>>>>,
    wakers: Arc<Mutex<Vec<Waker>>>,
}

impl<T> Default for UnboundedQueue<T> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<T> UnboundedQueue<T> {
    fn build(builder: Builder) -> Self {
        let slots = Vec::with_capacity(builder.cap.unwrap_or(NO_CAPACITY));
        let slots = Arc::new(Mutex::new(slots));

        //TODO(csaide): Fix this....
        let wakers = Vec::with_capacity(1024);
        let wakers = Arc::new(Mutex::new(wakers));
        Self {
            ttl: builder.ttl.unwrap_or(DEFAULT_TTL),
            slots,
            wakers,
        }
    }

    /// Create a new builder to define the various options for the unbounded queue instance.
    pub fn builder() -> Builder {
        Builder::default()
    }

    /// Create a new unbounded queue with no defined capacity and a default lease TTL of 10s.
    pub fn new() -> Self {
        // Create backing store for messages.
        let slots = Arc::new(Mutex::new(Vec::new()));
        let wakers = Arc::new(Mutex::new(Vec::new()));
        // Return a new queue.
        Self {
            ttl: DEFAULT_TTL,
            slots,
            wakers,
        }
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
            MESSAGE_RESULTS.with_label_values(&[ACK_VALUE]).inc();
            MESSAGES_OUTSTANDING.dec();
        }
        res
    }

    /// Nack the given message index.
    pub fn nack(&self, lease_id: u64, index: usize) -> Result<()> {
        let mut slots = self.slots.lock().unwrap();
        let res = slots[index].nack(lease_id);
        if res.is_ok() {
            MESSAGE_RESULTS.with_label_values(&[NACK_VALUE]).inc();
            MESSAGES_PENDING.inc();
            MESSAGES_OUTSTANDING.dec();
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
            MESSAGES_PENDING.inc();
            self.wakers
                .lock()
                .unwrap()
                .drain(..)
                .for_each(|waker| waker.wake());
        }
        res
    }

    /// Get the next available message from the front of the queue.
    pub fn next(&self) -> Option<(LeaseTag, usize, T)> {
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
                MESSAGE_LEASE_EXPIRES.inc();
                MESSAGES_OUTSTANDING.dec();
                MESSAGES_PENDING.inc();
                (idx, next)
            }
            _ => return None,
        };

        let res = next.lock(self.ttl).ok().map(|(tag, val)| (tag, idx, val));
        if res.is_some() {
            MESSAGES_PENDING.dec();
            MESSAGES_OUTSTANDING.inc();
        }
        res
    }
}

impl<T> Stream for UnboundedQueue<T>
where
    T: Clone,
{
    type Item = (LeaseTag, usize, T);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut slots = self.slots.lock().unwrap();
        let (index, slot) = match slots
            .iter_mut()
            .enumerate()
            .find(|(_, slot)| slot.is_filled())
        {
            Some(res) => res,
            None => {
                self.wakers.lock().unwrap().push(cx.waker().clone());
                return Poll::Pending;
            }
        };
        let next = slot.lock(self.ttl).ok().map(|(tag, msg)| (tag, index, msg));
        Poll::Ready(next)
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
