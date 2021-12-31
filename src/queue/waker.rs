// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::collections::{HashMap, VecDeque};
use std::task;

use uuid::Uuid;

/// A waker instance is responsible for tracking inflight future wakers waiting for new messages.
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
        if self.wakers.insert(id, waker).is_none() {
            self.ids.push_back(id);
        }
    }

    /// Wake the oldest known waker in this instance, if no wakers are registered
    /// this is effectively a no-op.
    pub fn wake(&mut self) {
        let id = match self.ids.pop_front() {
            Some(id) => id,
            None => return,
        };
        self.wakers.remove(&id).map(|waker| waker.wake());
    }
}
