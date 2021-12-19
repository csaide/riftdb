// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::collections::LinkedList;
use std::sync::Mutex;

/// A basic queue implementation for testing.
#[derive(Debug, Default)]
pub struct Queue<T> {
    backing: Mutex<LinkedList<T>>,
}

impl<T> Queue<T>
where
    T: Sized + Send + Sync,
{
    /// Prepend the given message to the beginning of the queue.
    pub fn prepend(&self, msg: T) {
        let mut backing = self.backing.lock().unwrap();
        backing.push_front(msg)
    }

    /// Append the given message to the end of the queue.
    pub fn append(&self, msg: T) {
        let mut backing = self.backing.lock().unwrap();
        backing.push_back(msg)
    }

    /// Get the next available message from the front of the queue.
    pub fn next(&self) -> Option<T> {
        let mut backing = self.backing.lock().unwrap();
        backing.pop_front()
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_queue() {
        let queue = Queue::default();

        let msg = String::from("Hello world!");
        queue.append(msg.clone());
        let actual = queue.next();
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert_eq!(actual, msg);

        let msg2 = String::from("Woot Woot!");
        queue.append(msg.clone());
        queue.prepend(msg2.clone());
        let actual = queue.next();
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert_eq!(actual, msg2);
    }
}
