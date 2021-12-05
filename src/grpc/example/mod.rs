// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

mod proto {
    tonic::include_proto!("example");
}

mod server;

pub use proto::greeter_client::GreeterClient;
pub use proto::greeter_server::GreeterServer;
pub use proto::{HelloRequest, HelloResponse};
pub use server::GreeterImpl;
