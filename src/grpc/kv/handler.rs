// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use crate::defer;
use crate::grpc::interceptor::{LoggerExt, ResponseTimeExt};

use super::proto::kv_server::Kv;
use super::proto::{Key, KeyValue, Value};

use bytes::Bytes;
use tonic::{Request, Response, Status};

/// The concrete implementation of the [Greeter] gRPC Server trait.
#[derive(Debug, Default)]
pub struct Handler<T>
where
    T: crate::store::Store,
    T: Send + Sync,
{
    store: T,
}

impl<T> Handler<T>
where
    T: crate::store::Store,
    T: Send + Sync,
    T: 'static,
{
    /// Create a new KV gRPC server.
    pub fn new(store: T) -> Handler<T> {
        Handler { store }
    }

    async fn _set(&self, req: Request<KeyValue>) -> Result<Response<Value>, Status> {
        let logger = req.extensions().get::<LoggerExt>().unwrap();
        let resp_time = req.extensions().get::<ResponseTimeExt>().unwrap();
        defer::defer! {
            resp_time.observe()
        };

        info!(logger.logger, "Got set request!");

        let req = req.get_ref();
        let key = Bytes::copy_from_slice(&req.key);
        let value = Bytes::copy_from_slice(&req.value);
        let ttl = std::time::Duration::from_nanos(req.ttl);

        match self.store.set(key.clone(), value, ttl).await {
            Ok(out) => match out {
                None => Ok(Response::new(Value::default())),
                Some(value) => Ok(Response::new(Value {
                    key: key.to_vec(),
                    value: value.to_vec(),
                    created: None,
                    updated: None,
                })),
            },
            _ => unimplemented!(),
        }
    }

    async fn _get(&self, req: Request<Key>) -> Result<Response<Value>, Status> {
        let logger = req.extensions().get::<LoggerExt>().unwrap();
        let resp_time = req.extensions().get::<ResponseTimeExt>().unwrap();
        defer::defer! {
            resp_time.observe()
        };

        info!(logger.logger, "Got get request!");

        let req = req.get_ref();
        let key = Bytes::copy_from_slice(&req.key);

        match self.store.get(&key).await {
            Ok(out) => match out {
                None => Ok(Response::new(Value::default())),
                Some(value) => Ok(Response::new(Value {
                    key: key.to_vec(),
                    value: value.to_vec(),
                    created: None,
                    updated: None,
                })),
            },
            _ => unimplemented!(),
        }
    }

    async fn _del(&self, req: Request<Key>) -> Result<Response<Value>, Status> {
        let logger = req.extensions().get::<LoggerExt>().unwrap();
        let resp_time = req.extensions().get::<ResponseTimeExt>().unwrap();
        defer::defer! {
            resp_time.observe()
        };

        info!(logger.logger, "Got delete request!");

        let req = req.get_ref();
        let key = Bytes::copy_from_slice(&req.key);

        match self.store.delete(&key).await {
            Ok(out) => match out {
                None => Ok(Response::new(Value::default())),
                Some(value) => Ok(Response::new(Value {
                    key: key.to_vec(),
                    value: value.to_vec(),
                    created: None,
                    updated: None,
                })),
            },
            _ => unimplemented!(),
        }
    }
}

#[tonic::async_trait]
impl<T> Kv for Handler<T>
where
    T: crate::store::Store,
    T: Send + Sync,
    T: 'static,
{
    async fn set(&self, req: Request<KeyValue>) -> Result<Response<Value>, Status> {
        self._set(req).await
    }

    async fn get(&self, req: Request<Key>) -> Result<Response<Value>, Status> {
        self._get(req).await
    }

    async fn delete(&self, req: Request<Key>) -> Result<Response<Value>, Status> {
        self._del(req).await
    }
}
