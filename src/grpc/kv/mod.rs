// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

mod proto {
    tonic::include_proto!("kv");
}

mod server;

pub use proto::kv_client::KvClient;
pub use proto::kv_server::KvServer;
pub use proto::{Key, KeyValue, Value};
pub use server::KVImpl;
