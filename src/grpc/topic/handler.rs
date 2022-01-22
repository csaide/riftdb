// (c) Copyright 2021-2022 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use crate::grpc::error::topic_not_found;
use crate::grpc::pubsub::Message;
use crate::pubsub::Registry;

use super::proto::topic_service_server::TopicService;
use super::proto::{CreateRequest, DeleteRequest, GetRequest, ListRequest, Topic, UpdateRequest};

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
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
    topic_registry: Registry<Message>,
}

impl Handler {
    /// Create a new handler with a default registry.
    pub fn new() -> Self {
        let topic_registry = Registry::default();
        Handler::with_registry(topic_registry)
    }

    /// Create a new handler with a predefined registry.
    pub fn with_registry(topic_registry: Registry<Message>) -> Self {
        Handler { topic_registry }
    }

    async fn _create(&self, request: Request<CreateRequest>) -> Result<Response<Topic>, Status> {
        let request = request.into_inner();

        let topic = self.topic_registry.create(request.name.clone());
        Ok(Response::new(Topic::from_inner(request.name, topic)))
    }

    async fn _get(&self, request: Request<GetRequest>) -> Result<Response<Topic>, Status> {
        let request = request.into_inner();

        match self.topic_registry.get(&request.name) {
            Some(topic) => Ok(Response::new(Topic::from_inner(request.name, topic))),
            None => topic_not_found(&request.name),
        }
    }

    async fn _list(&self, _request: Request<ListRequest>) -> Result<Response<TopicStream>, Status> {
        let topics = self.topic_registry.iter(|iter| {
            let mut topics = iter
                .map(|(name, topic)| Topic::from_inner(name.clone(), topic.clone()))
                .collect::<Vec<Topic>>();
            topics.sort_by_key(|topic| topic.name.clone());
            topics
        });

        let stream = TopicStream(topics);
        Ok(Response::new(stream))
    }

    async fn _update(&self, _request: Request<UpdateRequest>) -> Result<Response<Topic>, Status> {
        unimplemented!()
    }

    async fn _delete(&self, request: Request<DeleteRequest>) -> Result<Response<Topic>, Status> {
        let request = request.into_inner();

        match self.topic_registry.delete(&request.name) {
            Some(topic) => Ok(Response::new(Topic::from_inner(request.name, topic))),
            None => topic_not_found(&request.name),
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
    async fn create(&self, request: Request<CreateRequest>) -> Result<Response<Topic>, Status> {
        self._create(request).await
    }

    #[inline]
    async fn get(
        &self,
        request: tonic::Request<GetRequest>,
    ) -> Result<tonic::Response<Topic>, tonic::Status> {
        self._get(request).await
    }

    type ListStream = TopicStream;

    #[inline]
    async fn list(
        &self,
        request: tonic::Request<ListRequest>,
    ) -> Result<tonic::Response<Self::ListStream>, tonic::Status> {
        self._list(request).await
    }

    #[inline]
    async fn update(&self, request: Request<UpdateRequest>) -> Result<Response<Topic>, Status> {
        self._update(request).await
    }

    #[inline]
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Topic>, Status> {
        self._delete(request).await
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_happy_path() {
        let handler = Handler::default();
        let topic_name = String::from("topic");
        let second_topic_name = String::from("second");

        let create_req = CreateRequest {
            name: topic_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        let res = res.get_ref();
        assert_eq!(topic_name, res.name);

        let create_req = CreateRequest {
            name: second_topic_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        let res = res.get_ref();
        assert_eq!(second_topic_name, res.name);

        let get_req = GetRequest {
            name: topic_name.clone(),
        };
        let req = Request::new(get_req);
        let actual = aw!(handler.get(req));
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        let actual = actual.get_ref();
        assert_eq!(topic_name, actual.name);

        let list_req = ListRequest {};
        let req = Request::new(list_req);
        let res = aw!(handler.list(req));
        assert!(res.is_ok());
        let mut res = res.unwrap();
        let mut stream = res.get_mut();

        let waker = futures::task::noop_waker();
        let mut cx = std::task::Context::from_waker(&waker);

        let actual = match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(actual) => actual,
            _ => unimplemented!(),
        };
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual.name, topic_name);

        let actual = match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(actual) => actual,
            _ => unimplemented!(),
        };
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual.name, second_topic_name);

        let actual = match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(actual) => actual,
            _ => unimplemented!(),
        };
        assert!(actual.is_none());

        let del_req = DeleteRequest {
            name: topic_name.clone(),
        };
        let req = Request::new(del_req);
        let actual = aw!(handler.delete(req));
        assert!(actual.is_ok());
    }
}
