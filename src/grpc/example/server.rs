// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use super::proto::greeter_server::Greeter;
use super::proto::{HelloRequest, HelloResponse};

use tonic::{Request, Response, Status};

/// The concrete implementation of the [Greeter] gRPC Server trait.
#[derive(Debug, Default)]
pub struct GreeterImpl {}

#[tonic::async_trait]
impl Greeter for GreeterImpl {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloResponse>, Status> {
        println!("Received request from: {:?}", request);

        let response = HelloResponse {
            message: format!("Hello {}!", request.into_inner().name),
        };

        Ok(Response::new(response))
    }
}
