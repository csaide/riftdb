// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

mod proto {
    tonic::include_proto!("topic");
}
mod handler;

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("topic_descriptor");

pub use handler::Handler;
pub use proto::topic_service_client::TopicServiceClient;
pub use proto::topic_service_server::TopicServiceServer;
pub use proto::{CreateTopicRequest, DeleteTopicRequest, Topic, UpdateTopicRequest};
