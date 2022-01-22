// (c) Copyright 2021-2022 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::pin::Pin;
use std::task::{Context, Poll};

use uuid::Uuid;

use super::{LeaseTag, Queue};

/// A wrapper around [Queue] implementing [futures_core::Stream].
pub struct Stream<T> {
    id: Uuid,
    queue: Queue<T>,
}

impl<T> futures::Stream for Stream<T>
where
    T: Clone,
{
    type Item = (LeaseTag, usize, T);
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let next = self.queue.next();
        if next.is_none() {
            self.queue.register_task_waker(self.id, cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(next)
        }
    }
}

impl<T> From<Queue<T>> for Stream<T>
where
    T: Clone,
{
    fn from(queue: Queue<T>) -> Self {
        Self {
            id: Uuid::new_v4(),
            queue,
        }
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;
    use futures::Stream as FuturesStream;

    #[test]
    fn test_stream_happy_path() {
        let msg1 = 0;
        let msg2 = 1;
        let msg3 = 2;
        let queue = Queue::default();
        queue.push(msg1).expect("failed to push message");
        queue.push(msg2).expect("failed to push message");
        queue.push(msg3).expect("failed to push message");

        let mut stream = Stream::from(queue);

        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        let first = match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(first) => first,
            _ => unimplemented!(),
        };
        assert!(first.is_some());
        let first = first.unwrap();
        assert_eq!(msg1, first.2);

        let second = match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(second) => second,
            _ => unimplemented!(),
        };
        assert!(second.is_some());
        let second = second.unwrap();
        assert_eq!(msg2, second.2);

        let third = match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(third) => third,
            _ => unimplemented!(),
        };
        assert!(third.is_some());
        let third = third.unwrap();
        assert_eq!(msg3, third.2);

        match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Pending => assert!(true),
            _ => unimplemented!(),
        };
    }
}
