// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::SystemTime;

use futures_core::Stream;
use prost_types::Timestamp;
use tonic::{Request, Response, Status};

use crate::queue::UnboundedQueue;

use super::proto::pub_sub_service_server::PubSubService;
use super::{ConfimrationStatus, Confirmation, Lease, LeasedMessage, Message, Subscription};

pub struct SubscribeStream(UnboundedQueue<Message>);

impl Stream for SubscribeStream {
    type Item = Result<LeasedMessage, Status>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let pinned = Pin::new(&mut self.0);
        let (tag, index, msg) = match pinned.poll_next(cx) {
            Poll::Ready(opt) if opt.is_some() => opt.unwrap(),
            Poll::Ready(_) => return Poll::Ready(None),
            Poll::Pending => return Poll::Pending,
        };
        let lease = Lease::from_tag(tag, msg.topic.clone(), index);
        let leased_msg = LeasedMessage {
            lease: Some(lease),
            message: Some(msg),
        };
        Poll::Ready(Some(Ok(leased_msg)))
    }
}

/// The concrete server handler for the pubsub service.
#[derive(Debug)]
pub struct Handler {
    queue: UnboundedQueue<Message>,
}

impl Handler {
    /// Create a new handler with no defined capacity. This is synonymous with `default()`.
    pub fn new() -> Self {
        let queue = UnboundedQueue::default();
        Self { queue }
    }

    /// Create a new handler with the predefined queue.
    pub fn with_queue(queue: UnboundedQueue<Message>) -> Self {
        Self { queue }
    }

    async fn _publish(&self, request: Request<Message>) -> Result<Response<Confirmation>, Status> {
        let mut msg = request.into_inner();
        if msg.data.is_empty() {
            return Err(Status::invalid_argument("data payload must be non-empty."));
        }

        msg.published = Some(Timestamp::from(SystemTime::now()));

        match self.queue.push(msg) {
            Ok(()) => Ok(Response::new(Confirmation {
                status: ConfimrationStatus::Committed as i32,
            })),
            Err(err) => Err(Status::internal(format!(
                "queue is full or otherwise invalid: {}",
                err
            ))),
        }
    }

    async fn _ack(&self, request: Request<Lease>) -> Result<Response<Confirmation>, Status> {
        let lease = request.into_inner();

        match self.queue.ack(lease.id, lease.index as usize) {
            Ok(()) => Ok(Response::new(Confirmation {
                status: ConfimrationStatus::Committed as i32,
            })),
            Err(err) => Err(Status::internal(format!(
                "queue is full or otherwise invalid: {}",
                err
            ))),
        }
    }

    async fn _nack(&self, request: Request<Lease>) -> Result<Response<Confirmation>, Status> {
        let lease = request.into_inner();

        match self.queue.nack(lease.id, lease.index as usize) {
            Ok(()) => Ok(Response::new(Confirmation {
                status: ConfimrationStatus::Committed as i32,
            })),
            Err(err) => Err(Status::internal(format!(
                "queue is full or otherwise invalid: {}",
                err
            ))),
        }
    }

    async fn _subscribe(
        &self,
        request: Request<Subscription>,
    ) -> Result<Response<SubscribeStream>, Status> {
        let _ = request.into_inner();

        let queue = self.queue.clone();
        let stream = SubscribeStream(queue);
        Ok(Response::new(stream))
    }
}

impl Default for Handler {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl PubSubService for Handler {
    type SubscribeStream = SubscribeStream;

    #[inline]
    async fn publish(&self, request: Request<Message>) -> Result<Response<Confirmation>, Status> {
        self._publish(request).await
    }

    #[inline]
    async fn ack(&self, request: Request<Lease>) -> Result<Response<Confirmation>, Status> {
        self._ack(request).await
    }

    #[inline]
    async fn nack(&self, request: Request<Lease>) -> Result<Response<Confirmation>, Status> {
        self._nack(request).await
    }

    #[inline]
    async fn subscribe(
        &self,
        request: Request<Subscription>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        self._subscribe(request).await
    }
}
