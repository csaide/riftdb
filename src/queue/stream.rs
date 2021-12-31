// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use uuid::Uuid;

use super::{LeaseTag, UnboundedQueue};

/// A wrapper around [UnboundedQueue] implementing [futures_core::Stream].
pub struct UnboundedStream<T> {
    id: Uuid,
    queue: UnboundedQueue<T>,
}

impl<T> Stream for UnboundedStream<T>
where
    T: Clone,
{
    type Item = (LeaseTag, usize, T);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let next = self.queue.next();
        if next.is_none() {
            self.queue
                .waker
                .lock()
                .unwrap()
                .register(self.id, cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(next)
        }
    }
}

impl<T> From<UnboundedQueue<T>> for UnboundedStream<T>
where
    T: Clone,
{
    fn from(queue: UnboundedQueue<T>) -> Self {
        Self {
            id: Uuid::new_v4(),
            queue,
        }
    }
}
