// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use tonic::{Request, Response, Status};

use crate::queue::UnboundedQueue;

use super::proto::pub_sub_service_server::PubSubService;
use super::{ConfimrationStatus, Confirmation, Lease, LeasedMessage, Message, Subscription};

pub struct SubscribeStream(UnboundedQueue<Message>);

impl Stream for SubscribeStream {
    type Item = Result<LeasedMessage, Status>;
    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.0.next() {
            Some((id, index, msg)) => {
                let lease = Lease {
                    id,
                    index: index as u64,
                };
                let leased = LeasedMessage {
                    lease: Some(lease),
                    message: Some(msg),
                };
                Poll::Ready(Some(Ok(leased)))
            }
            None => Poll::Ready(None),
        }
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

    /// Create a new handler with a defined initial message backlog capacity.
    pub fn with_capacity(cap: usize) -> Self {
        let queue = UnboundedQueue::with_capacity(cap);
        Self { queue }
    }

    async fn _publish(&self, request: Request<Message>) -> Result<Response<Confirmation>, Status> {
        let msg = request.into_inner();
        let mut conf = Confirmation::default();
        match self.queue.push(msg) {
            Ok(()) => conf.status = ConfimrationStatus::Committed as i32,
            Err(err) => {
                return Err(Status::internal(format!(
                    "queue is full or otherwise invalid: {}",
                    err
                )))
            }
        };
        Ok(Response::new(conf))
    }

    async fn _ack(&self, request: Request<Lease>) -> Result<Response<Confirmation>, Status> {
        let lease = request.into_inner();

        let mut conf = Confirmation::default();
        match self.queue.ack(lease.id, lease.index as usize) {
            Ok(()) => conf.status = ConfimrationStatus::Committed as i32,
            Err(err) => {
                return Err(Status::internal(format!(
                    "queue is full or otherwise invalid: {}",
                    err
                )))
            }
        }
        Ok(Response::new(conf))
    }

    async fn _nack(&self, request: Request<Lease>) -> Result<Response<Confirmation>, Status> {
        let lease = request.into_inner();

        let mut conf = Confirmation::default();
        match self.queue.nack(lease.id, lease.index as usize) {
            Ok(()) => conf.status = ConfimrationStatus::Committed as i32,
            Err(err) => {
                return Err(Status::internal(format!(
                    "queue is full or otherwise invalid: {}",
                    err
                )))
            }
        }
        Ok(Response::new(conf))
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
