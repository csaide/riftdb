// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use crate::grpc::error::{sub_not_found, topic_not_found};
use crate::grpc::pubsub::Message;
use crate::topic::Registry;

use super::proto::subscription_service_server::SubscriptionService;
use super::proto::{
    CreateRequest, DeleteRequest, GetRequest, ListRequest, Subscription, UpdateRequest,
};

use std::pin::Pin;
use std::task::{Context, Poll};

use futures_core::Stream;
use tonic::{Request, Response, Status};

pub struct SubscriptionStream(Vec<Subscription>);

impl Stream for SubscriptionStream {
    type Item = Result<Subscription, Status>;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item = self.0.pop().map(Ok);
        Poll::Ready(item)
    }
}

/// The Subscription service implementation.
#[derive(Debug)]
pub struct Handler {
    topic_registry: Registry<Message>,
}

impl Handler {
    /// Create a new handler with a default registry.
    pub fn new() -> Self {
        let topic_registry = Registry::default();
        Handler { topic_registry }
    }

    /// Create a new handler with a predefined registry.
    pub fn with_registry(topic_registry: Registry<Message>) -> Self {
        Handler { topic_registry }
    }

    async fn _create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<Subscription>, Status> {
        let request = request.into_inner();
        let topic = match self.topic_registry.get(&request.topic) {
            Some(topic) => topic,
            None => return topic_not_found(&request.topic),
        };
        let sub = topic.create(request.name.clone());
        let sub = Subscription::from_inner(request.name, request.topic, sub);
        Ok(Response::new(sub))
    }

    async fn _get(&self, request: Request<GetRequest>) -> Result<Response<Subscription>, Status> {
        let request = request.into_inner();

        let topic = match self.topic_registry.get(&request.topic) {
            Some(topic) => topic,
            None => return topic_not_found(&request.topic),
        };
        let sub = match topic.get(&request.name) {
            Some(sub) => sub,
            None => return sub_not_found(&request.name, &request.topic),
        };
        let sub = Subscription::from_inner(request.name, request.topic, sub);
        Ok(Response::new(sub))
    }

    async fn _list(
        &self,
        request: Request<ListRequest>,
    ) -> Result<Response<SubscriptionStream>, Status> {
        let request = request.into_inner();

        let topic = match self.topic_registry.get(&request.topic) {
            Some(topic) => topic,
            None => return topic_not_found(&request.topic),
        };

        let subscriptions = topic.iter(|iter| {
            iter.map(|(name, subscription)| {
                Subscription::from_inner(name.clone(), request.topic.clone(), subscription.clone())
            })
            .collect::<Vec<Subscription>>()
        });

        let stream = SubscriptionStream(subscriptions);
        Ok(Response::new(stream))
    }

    async fn _update(
        &self,
        _request: Request<UpdateRequest>,
    ) -> Result<Response<Subscription>, Status> {
        unimplemented!()
    }

    async fn _delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<Subscription>, Status> {
        let request = request.into_inner();
        let topic = match self.topic_registry.get(&request.topic) {
            Some(topic) => topic,
            None => return topic_not_found(&request.topic),
        };

        match topic.remove(&request.name) {
            Some(subscription) => Ok(Response::new(Subscription::from_inner(
                request.name,
                request.topic,
                subscription,
            ))),
            None => sub_not_found(&request.name, &request.topic),
        }
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl SubscriptionService for Handler {
    #[inline]
    async fn create(
        &self,
        request: Request<CreateRequest>,
    ) -> Result<Response<Subscription>, Status> {
        self._create(request).await
    }

    #[inline]
    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> Result<tonic::Response<Subscription>, tonic::Status> {
        self._get(request).await
    }

    type ListStream = SubscriptionStream;

    #[inline]
    async fn list(
        &self,
        request: tonic::Request<ListRequest>,
    ) -> Result<tonic::Response<Self::ListStream>, tonic::Status> {
        self._list(request).await
    }

    #[inline]
    async fn update(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<Subscription>, Status> {
        self._update(request).await
    }

    #[inline]
    async fn delete(
        &self,
        request: Request<DeleteRequest>,
    ) -> Result<Response<Subscription>, Status> {
        self._delete(request).await
    }
}
