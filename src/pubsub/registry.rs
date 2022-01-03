// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::hash_map::Iter;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use super::Topic;

/// Handles managing and tracking the lifecycle of a set of topics.
#[derive(Debug, Default, Clone)]
pub struct Registry<T> {
    topics: Arc<RwLock<HashMap<String, Topic<T>>>>,
}

impl<T> Registry<T> {
    /// Create a new topic manager with an initial capacity.
    pub fn with_capacity(cap: usize) -> Self {
        let topics = HashMap::with_capacity(cap);
        let topics = Arc::new(RwLock::new(topics));
        Self { topics }
    }
}

impl<T> Registry<T>
where
    T: Clone,
{
    /// Create a new topic, store it, and return it for use.
    pub fn create(&self, name: String) -> Topic<T> {
        let mut topics = self.topics.write().unwrap();

        if let Some(topic) = topics.get(&name).cloned() {
            return topic;
        }

        let topic = Topic::with_capacity(0);
        topics.insert(name, topic.clone());
        topic
    }

    /// Delete the specified topic if it exists.
    pub fn delete(&self, name: &str) -> Option<Topic<T>> {
        let mut topics = self.topics.write().unwrap();
        topics.remove(name)
    }

    /// Retrieve the specified topic if it exists, otherwise returning
    /// [None].
    pub fn get(&self, name: &str) -> Option<Topic<T>> {
        let topics = self.topics.read().unwrap();
        topics.get(name).cloned()
    }

    /// Iterate over the topics contained in this registry. The supplied FnOnce is used to ensure
    /// the inner state is not mutated while iterating.
    pub fn iter<R>(&self, func: impl FnOnce(Iter<'_, String, Topic<T>>) -> R) -> R {
        let guard = self.topics.read().unwrap();
        func(guard.iter())
    }
}
