// (c) Copyright 2021-2022 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::collections::{HashMap, VecDeque};
use std::task;

use uuid::Uuid;

/// A waker instance is responsible for tracking inflight stream tasks waiting for new messages. The ordering of wake events
/// is a round robin FIFO implementation.
#[derive(Debug, Default)]
pub struct Waker {
    wakers: HashMap<Uuid, task::Waker>,
    ids: VecDeque<Uuid>,
}

impl Waker {
    /// Generate a new waker with a predefined initial capacity for wakers.
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            wakers: HashMap::with_capacity(cap),
            ids: VecDeque::with_capacity(cap),
        }
    }

    /// Register the given [Uuid]/[Waker] combination with this waker instance.
    /// If the given ID is already registered the original waker is overwritten
    /// with the new waker.
    pub fn register(&mut self, id: Uuid, waker: task::Waker) {
        // First insert the new waker, in all cases we want the newest waker instance by ID.
        if self.wakers.insert(id, waker).is_none() {
            // If we have never seen this ID before, store it in the ids queue for use later.
            self.ids.push_back(id);
        }
    }

    /// Wake the oldest known waker in this instance, if no wakers are registered
    /// this is effectively a no-op.
    pub fn wake(&mut self) -> bool {
        // Grab the oldest id or short circuit if we don't have any wakers currently.
        let id = match self.ids.pop_front() {
            Some(id) => id,
            None => return false,
        };
        // Pop the waker off the map and then consume it immediately by calling `wake()`.
        if let Some(waker) = self.wakers.remove(&id) {
            waker.wake();
            return true;
        }
        unreachable!()
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_waker() {
        let mut waker = Waker::default();
        assert_eq!(0, waker.wakers.len());
        assert_eq!(0, waker.ids.len());

        let first = Uuid::new_v4();
        let second = Uuid::new_v4();
        waker.register(first, futures::task::noop_waker());
        waker.register(first, futures::task::noop_waker());
        waker.register(second, futures::task::noop_waker());

        assert!(waker.wake());
        assert!(waker.wake());
        assert!(!waker.wake());
    }
}
