// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use crate::grpc::error::{sub_not_found, topic_not_found};
use crate::grpc::pubsub::Message;
use crate::pubsub::Registry;

use super::proto::subscription_service_server::SubscriptionService;
use super::proto::{
    CreateRequest, DeleteRequest, GetRequest, ListRequest, Subscription, UpdateRequest,
};

use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
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
        Handler::with_registry(topic_registry)
    }

    /// Create a new handler with a predefined registry.
    pub fn with_registry(topic_registry: Registry<Message>) -> Self {
        Handler { topic_registry }
    }

    #[cfg(test)]
    fn get_registry(&self) -> &Registry<Message> {
        &self.topic_registry
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
            let mut subs = iter
                .map(|(name, subscription)| {
                    Subscription::from_inner(
                        name.clone(),
                        request.topic.clone(),
                        subscription.clone(),
                    )
                })
                .collect::<Vec<Subscription>>();
            subs.sort_by_key(|k| k.name.clone());
            subs
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
    fn test_create() {
        let handler = Handler::default();
        let topic_name = String::from("topic");
        let sub_name = String::from("first");
        let second_sub_name = String::from("second");

        let reg = handler.get_registry();
        reg.create(topic_name.clone());

        let create_req = CreateRequest {
            topic: topic_name.clone(),
            name: sub_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        let res = res.get_ref();
        assert_eq!(res.name, sub_name);
        assert_eq!(res.topic, topic_name);

        let create_req = CreateRequest {
            topic: String::from("nope"),
            name: sub_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_err());

        let create_req = CreateRequest {
            topic: topic_name.clone(),
            name: second_sub_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        let res = res.get_ref();
        assert_eq!(res.name, second_sub_name);
        assert_eq!(res.topic, topic_name);
    }

    #[test]
    fn test_delete() {
        let topic_name = String::from("topic");
        let sub_name = String::from("first");

        let handler = Handler::default();

        let reg = handler.get_registry();
        reg.create(topic_name.clone());

        let create_req = CreateRequest {
            topic: topic_name.clone(),
            name: sub_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_ok());

        let delete_req = DeleteRequest {
            topic: String::from("nope"),
            name: sub_name.clone(),
        };
        let req = Request::new(delete_req);
        let res = aw!(handler.delete(req));
        assert!(res.is_err());

        let delete_req = DeleteRequest {
            topic: topic_name.clone(),
            name: String::from("nope"),
        };
        let req = Request::new(delete_req);
        let res = aw!(handler.delete(req));
        assert!(res.is_err());

        let delete_req = DeleteRequest {
            topic: topic_name.clone(),
            name: sub_name.clone(),
        };
        let req = Request::new(delete_req);
        let res = aw!(handler.delete(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        let res = res.get_ref();
        assert_eq!(res.name, sub_name);
        assert_eq!(res.topic, topic_name);
    }

    #[test]
    fn test_get_request() {
        let topic_name = String::from("topic");
        let sub_name = String::from("first");

        let handler = Handler::default();

        let reg = handler.get_registry();
        reg.create(topic_name.clone());

        let create_req = CreateRequest {
            topic: topic_name.clone(),
            name: sub_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_ok());

        let get_req = GetRequest {
            topic: topic_name.clone(),
            name: sub_name.clone(),
        };
        let req = Request::new(get_req);
        let res = aw!(handler.get(req));
        assert!(res.is_ok());
        let res = res.unwrap();
        let res = res.get_ref();
        assert_eq!(res.name, sub_name);
        assert_eq!(res.topic, topic_name);

        let get_req = GetRequest {
            topic: String::from("nope"),
            name: sub_name.clone(),
        };
        let req = Request::new(get_req);
        let res = aw!(handler.get(req));
        assert!(res.is_err());

        let get_req = GetRequest {
            topic: topic_name.clone(),
            name: String::from("nope"),
        };
        let req = Request::new(get_req);
        let res = aw!(handler.get(req));
        assert!(res.is_err());
    }

    #[test]
    fn test_list() {
        let topic_name = String::from("topic");
        let sub_name = String::from("first");
        let second_sub_name = String::from("second");

        let handler = Handler::default();

        let reg = handler.get_registry();
        reg.create(topic_name.clone());

        let create_req = CreateRequest {
            topic: topic_name.clone(),
            name: sub_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_ok());

        let create_req = CreateRequest {
            topic: topic_name.clone(),
            name: second_sub_name.clone(),
        };
        let req = Request::new(create_req);
        let res = aw!(handler.create(req));
        assert!(res.is_ok());

        let list_req = ListRequest {
            topic: String::from("nope"),
        };
        let req = Request::new(list_req);
        let stream = aw!(handler.list(req));
        assert!(stream.is_err());

        let list_req = ListRequest {
            topic: topic_name.clone(),
        };
        let req = Request::new(list_req);
        let stream = aw!(handler.list(req));
        assert!(stream.is_ok());
        let mut stream = stream.unwrap();
        let mut stream = stream.get_mut();

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
        assert_eq!(actual.name, second_sub_name);
        assert_eq!(actual.topic, topic_name);

        let actual = match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(actual) => actual,
            _ => unimplemented!(),
        };
        assert!(actual.is_some());
        let actual = actual.unwrap();
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual.name, sub_name);
        assert_eq!(actual.topic, topic_name);

        let actual = match Pin::new(&mut stream).poll_next(&mut cx) {
            Poll::Ready(actual) => actual,
            _ => unimplemented!(),
        };
        assert!(actual.is_none());
    }
}
