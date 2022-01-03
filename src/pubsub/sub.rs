// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::time::SystemTime;

use super::Queue;

/// A subscription represents a single consumer of a given topic.
#[derive(Debug, Clone)]
pub struct Sub<T> {
    /// The last time this particular topic was updated.
    pub updated: Option<SystemTime>,
    /// The datetime when this Topic was created.
    pub created: SystemTime,
    /// The backing persistent queue for this subscription.
    pub queue: Queue<T>,
}

impl<T> Sub<T> {
    /// Create a new subscription with a predefined backing queue.
    pub fn with_queue(queue: Queue<T>) -> Self {
        Self {
            updated: None,
            created: SystemTime::now(),
            queue,
        }
    }
}

impl<T> Default for Sub<T> {
    fn default() -> Self {
        Self {
            updated: None,
            created: SystemTime::now(),
            queue: Queue::default(),
        }
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_subscription() {
        let first = Sub::<u32>::default();
        assert!(SystemTime::now().ge(&first.created));
        let queue = Queue::default();
        let second = Sub::<u32>::with_queue(queue);
        assert_ne!(first.created, second.created);
    }
}
