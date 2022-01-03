// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

mod proto {
    use prost_types::Timestamp;

    tonic::include_proto!("topic");

    impl Topic {
        /// Create a new topic from the supplied topic name and inner topic.
        pub fn from_inner<T>(name: String, i: crate::pubsub::Topic<T>) -> Self {
            Self {
                updated: i.updated.map(Timestamp::from),
                created: Some(Timestamp::from(i.created)),
                name,
            }
        }
    }
}
mod handler;

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("topic_descriptor");

pub use handler::Handler;
pub use proto::topic_service_client::TopicServiceClient;
pub use proto::topic_service_server::TopicServiceServer;
pub use proto::{CreateRequest, DeleteRequest, GetRequest, ListRequest, Topic, UpdateRequest};
