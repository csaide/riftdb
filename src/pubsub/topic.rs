// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::collections::hash_map::Iter;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::SystemTime,
};

use super::{Queue, Sub};

/// A topic represents a configured data flow through the rift system.
#[derive(Debug, Clone)]
pub struct Topic<T> {
    /// The last time this particular topic was updated.
    pub updated: Option<SystemTime>,
    /// The datetime when this Topic was created.
    pub created: SystemTime,
    subscriptions: Arc<RwLock<HashMap<String, Sub<T>>>>,
}

impl<T> Topic<T>
where
    T: Clone,
{
    /// Create a new default topic.
    pub fn new() -> Self {
        let subscriptions = Arc::new(RwLock::new(HashMap::new()));
        Self {
            updated: None,
            created: SystemTime::now(),
            subscriptions,
        }
    }

    /// Create a new topic with a predefined capacity for subscriber subscriptions.
    pub fn with_capacity(cap: usize) -> Self {
        let subscriptions = HashMap::with_capacity(cap);
        let subscriptions = Arc::new(RwLock::new(subscriptions));
        Self {
            updated: None,
            created: SystemTime::now(),
            subscriptions,
        }
    }

    /// Create a new subscription within this topic.
    pub fn create(&self, name: String) -> Sub<T> {
        let mut subs = self.subscriptions.write().unwrap();

        if let Some(sub) = subs.get(&name) {
            return sub.clone();
        }

        let queue = Queue::<T>::builder().build();
        let sub = Sub::with_queue(queue);
        subs.insert(name, sub.clone());
        sub
    }

    /// Remove the supplied subscription if it exists.
    pub fn remove(&self, name: &str) -> Option<Sub<T>> {
        let mut subs = self.subscriptions.write().unwrap();
        subs.remove(name)
    }

    /// Retrieve the specified subscription if it exists, otherwise returning
    /// [None].
    pub fn get(&self, name: &str) -> Option<Sub<T>> {
        let subs = self.subscriptions.read().unwrap();
        subs.get(name).cloned()
    }

    /// Handle the supplied message.
    pub fn push(&self, msg: T) -> Result<(), String> {
        let subs = self.subscriptions.read().unwrap();
        let (_, sub) = match subs.iter().next() {
            Some(sub) => sub,
            None => return Err(String::from("no subscriptions....")),
        };

        sub.queue.push(msg).map_err(|err| err.to_string())
    }

    /// Iterate over the topics contained in this registry. The supplied FnOnce is used to ensure
    /// the inner state is not mutated while iterating.
    pub fn iter<R>(&self, func: impl FnOnce(Iter<'_, String, Sub<T>>) -> R) -> R {
        let guard = self.subscriptions.read().unwrap();
        func(guard.iter())
    }
}

impl<T> Default for Topic<T> {
    fn default() -> Self {
        Self {
            created: SystemTime::now(),
            ..Default::default()
        }
    }
}
