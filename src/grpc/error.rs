// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

use tonic::{Response, Status};

/// Create and return a topic not found error.
pub fn topic_not_found<T>(topic: &str) -> Result<Response<T>, Status> {
    return Err(Status::not_found(format!(
        "the supplied topic '{}' does not exist",
        topic
    )));
}

/// Create and return a subscription not found error.
pub fn sub_not_found<T>(subscription: &str, topic: &str) -> Result<Response<T>, Status> {
    return Err(Status::not_found(format!(
        "the supplied subscription '{}' is not assoicated with the given topic '{}'",
        subscription, topic
    )));
}
