// (c) Copyright 2021-2022 Christian Saide
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

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use tonic::Code;

    use super::*;

    #[test]
    fn test_topic_not_found() {
        let err = topic_not_found::<usize>("woot");
        assert!(err.is_err());
        let err = err.unwrap_err();
        assert_eq!(err.message(), "the supplied topic 'woot' does not exist");
        assert_eq!(err.code(), Code::NotFound);
    }

    #[test]
    fn test_sub_not_found() {
        let err = sub_not_found::<usize>("woot", "testing");
        assert!(err.is_err());
        let err = err.unwrap_err();
        assert_eq!(
            err.message(),
            "the supplied subscription 'woot' is not assoicated with the given topic 'testing'"
        );
        assert_eq!(err.code(), Code::NotFound);
    }
}
