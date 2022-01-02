// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

mod proto {
    use prost_types::Timestamp;

    tonic::include_proto!("subscription");

    impl Subscription {
        /// Create a subscription based on the supplied name, topic association, and inner subscription.
        pub fn from_inner<T>(
            name: String,
            topic: String,
            i: crate::subscription::Subscription<T>,
        ) -> Self {
            Self {
                created: Some(Timestamp::from(i.created)),
                name,
                topic,
                updated: i.updated.map(Timestamp::from),
            }
        }
    }
}
mod handler;

pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
    tonic::include_file_descriptor_set!("subscription_descriptor");

pub use handler::Handler;
pub use proto::subscription_service_client::SubscriptionServiceClient;
pub use proto::subscription_service_server::SubscriptionServiceServer;
pub use proto::{
    CreateRequest, DeleteRequest, GetRequest, ListRequest, Subscription, UpdateRequest,
};
