// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

mod proto {
    tonic::include_proto!("kv");
}
mod handler;

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("kv_descriptor");

pub use handler::Handler;
pub use proto::kv_client::KvClient;
pub use proto::kv_server::KvServer;
pub use proto::{Key, KeyValue, Value};
