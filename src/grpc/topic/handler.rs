// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use super::proto::topic_service_server::TopicService;
use super::proto::{
    CreateTopicRequest, DeleteTopicRequest, GetTopicRequest, ListTopicRequest, Topic,
    UpdateTopicRequest,
};

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::RwLock;
use std::task::{Context, Poll};
use std::time::SystemTime;

use futures_core::Stream;
use prost_types::Timestamp;
use tonic::{Request, Response, Status};

pub struct TopicStream(Vec<Topic>);

impl Stream for TopicStream {
    type Item = Result<Topic, Status>;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let item = self.0.pop().map(Ok);
        Poll::Ready(item)
    }
}

/// The Topic service implementation.
#[derive(Debug)]
pub struct Handler {
    topics: RwLock<HashMap<String, Topic>>,
}

impl Handler {
    fn new() -> Self {
        let map = HashMap::default();
        let topics = RwLock::new(map);
        Handler { topics }
    }

    async fn _create(
        &self,
        request: Request<CreateTopicRequest>,
    ) -> Result<Response<Topic>, Status> {
        let request = request.into_inner();

        let mut topics = self.topics.write().unwrap();

        if topics.contains_key(&request.name) {
            return Err(Status::already_exists(format!(
                "specified topic '{}' already exists",
                request.name
            )));
        }

        let mut topic = Topic::default();
        let now = std::time::SystemTime::now();
        let now = Timestamp::from(now);

        topic.created = Some(now.clone());
        topic.name = request.name.clone();
        topic.ttl_ms = request.ttl_ms;
        topic.updated = Some(now);

        topics.insert(request.name, topic.clone());
        Ok(Response::new(topic))
    }

    async fn _get(&self, request: Request<GetTopicRequest>) -> Result<Response<Topic>, Status> {
        let request = request.into_inner();

        let topics = self.topics.read().unwrap();
        match topics.get(&request.name) {
            Some(topic) => Ok(Response::new(topic.clone())),
            None => Err(Status::not_found("not found")),
        }
    }

    async fn _list(
        &self,
        _request: Request<ListTopicRequest>,
    ) -> Result<Response<TopicStream>, Status> {
        let topics = self.topics.read().unwrap();
        let topics = topics
            .iter()
            .map(|(_, topic)| topic.clone())
            .collect::<Vec<Topic>>();

        let stream = TopicStream(topics);
        Ok(Response::new(stream))
    }

    async fn _update(
        &self,
        request: Request<UpdateTopicRequest>,
    ) -> Result<Response<Topic>, Status> {
        let request = request.into_inner();

        let mut topics = self.topics.write().unwrap();

        let now = SystemTime::now().into();
        match topics.get_mut(&request.name) {
            Some(topic) => {
                topic.updated = Some(now);
                topic.ttl_ms = request.ttl_ms;
                Ok(Response::new(topic.clone()))
            }
            None => Err(Status::not_found("not found")),
        }
    }

    async fn _delete(
        &self,
        request: Request<DeleteTopicRequest>,
    ) -> Result<Response<Topic>, Status> {
        let request = request.into_inner();

        let mut topics = self.topics.write().unwrap();
        match topics.remove(&request.name) {
            Some(topic) => Ok(Response::new(topic)),
            None => Err(Status::not_found("not found")),
        }
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}

#[tonic::async_trait]
impl TopicService for Handler {
    #[inline]
    async fn create(
        &self,
        request: Request<CreateTopicRequest>,
    ) -> Result<Response<Topic>, Status> {
        self._create(request).await
    }

    #[inline]
    async fn get(
        &self,
        request: tonic::Request<GetTopicRequest>,
    ) -> Result<tonic::Response<Topic>, tonic::Status> {
        self._get(request).await
    }

    type ListStream = TopicStream;

    #[inline]
    async fn list(
        &self,
        request: tonic::Request<ListTopicRequest>,
    ) -> Result<tonic::Response<Self::ListStream>, tonic::Status> {
        self._list(request).await
    }

    #[inline]
    async fn update(
        &self,
        request: Request<UpdateTopicRequest>,
    ) -> Result<Response<Topic>, Status> {
        self._update(request).await
    }

    #[inline]
    async fn delete(
        &self,
        request: Request<DeleteTopicRequest>,
    ) -> Result<Response<Topic>, Status> {
        self._delete(request).await
    }
}
