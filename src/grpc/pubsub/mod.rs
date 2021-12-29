// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

mod proto {
    tonic::include_proto!("pubsub");
}
mod handler;

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("pubsub_descriptor");

pub use handler::Handler;
pub use proto::pub_sub_service_client::PubSubServiceClient;
pub use proto::pub_sub_service_server::PubSubServiceServer;
pub use proto::{ConfimrationStatus, Confirmation, Lease, LeasedMessage, Message, Subscription};
