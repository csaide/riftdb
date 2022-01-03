// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::SystemTime;

use futures_core::Stream;
use prost_types::Timestamp;
use tonic::{Request, Response, Status};

use crate::grpc::error::{sub_not_found, topic_not_found};
use crate::pubsub::{Registry, UnboundedStream};

use super::proto::pub_sub_service_server::PubSubService;
use super::{ConfimrationStatus, Confirmation, Lease, LeasedMessage, Message, Subscription};

pub struct SubscribeStream {
    inner: UnboundedStream<Message>,
    subscription: String,
}

impl Stream for SubscribeStream {
    type Item = Result<LeasedMessage, Status>;
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let pinned = Pin::new(&mut self.inner);
        let (tag, index, msg) = match pinned.poll_next(cx) {
            Poll::Ready(opt) if opt.is_some() => opt.unwrap(),
            _ => return Poll::Pending,
        };
        let lease = Lease::from_tag(tag, msg.topic.clone(), self.subscription.clone(), index);
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
    topic_registry: Registry<Message>,
}

impl Handler {
    /// Create a new handler with no defined capacity. This is synonymous with `default()`.
    pub fn new() -> Self {
        let topic_registry = Registry::default();
        Self { topic_registry }
    }

    /// Create a new handler with the supplied topic registry.
    pub fn with_registry(topic_registry: Registry<Message>) -> Self {
        Self { topic_registry }
    }

    async fn _publish(&self, request: Request<Message>) -> Result<Response<Confirmation>, Status> {
        let mut msg = request.into_inner();
        if msg.data.is_empty() {
            return Err(Status::invalid_argument("data payload must be non-empty."));
        }
        if msg.topic.is_empty() {
            return Err(Status::invalid_argument("topic name must be non-empty"));
        }

        let topic = match self.topic_registry.get(&msg.topic) {
            Some(topic) => topic,
            None => return topic_not_found(&msg.topic),
        };

        msg.published = Some(Timestamp::from(SystemTime::now()));

        match topic.push(msg) {
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

        let topic = match self.topic_registry.get(&lease.topic) {
            Some(topic) => topic,
            None => return topic_not_found(&lease.topic),
        };
        let sub = match topic.get(&lease.subscription) {
            Some(sub) => sub,
            None => return sub_not_found(&lease.subscription, &lease.topic),
        };

        match sub.queue.ack(lease.id, lease.index as usize) {
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

        let topic = match self.topic_registry.get(&lease.topic) {
            Some(topic) => topic,
            None => return topic_not_found(&lease.topic),
        };
        let sub = match topic.get(&lease.subscription) {
            Some(sub) => sub,
            None => return sub_not_found(&lease.subscription, &lease.topic),
        };

        match sub.queue.nack(lease.id, lease.index as usize) {
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
        let subscription = request.into_inner();

        let topic = match self.topic_registry.get(&subscription.topic) {
            Some(topic) => topic,
            None => return topic_not_found(&subscription.topic),
        };
        let sub = match topic.get(&subscription.name) {
            Some(sub) => sub,
            None => return sub_not_found(&subscription.name, &subscription.topic),
        };

        let stream = SubscribeStream {
            inner: sub.queue.into(),
            subscription: subscription.name,
        };
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
