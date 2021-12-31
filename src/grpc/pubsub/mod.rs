// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

mod proto {
    use prost_types::Timestamp;

    use crate::queue::LeaseTag;

    tonic::include_proto!("pubsub");

    impl Lease {
        /// Generate a new lease from a [LeaseTag].
        pub fn from_tag(tag: LeaseTag, topic: String, index: usize) -> Self {
            Lease {
                topic,
                id: tag.id,
                index: index as u64,
                ttl_ms: tag.ttl.as_millis() as u64,
                deadline: Some(Timestamp::from(tag.deadline)),
                leased: Some(Timestamp::from(tag.leased_at)),
            }
        }
    }
}
mod handler;

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("pubsub_descriptor");

pub use handler::Handler;
pub use proto::pub_sub_service_client::PubSubServiceClient;
pub use proto::pub_sub_service_server::PubSubServiceServer;
pub use proto::{ConfimrationStatus, Confirmation, Lease, LeasedMessage, Message, Subscription};
