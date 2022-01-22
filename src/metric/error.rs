// (c) Copyright 2021-2022 Christian Saide <supernomad>
// SPDX-License-Identifier: GPL-3.0-only

// stdlib usings
use std::result;

// extern usings
use thiserror::Error;

/// Custom Result wrapper to simplify usage.
pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Error)]
/// Represents metric generation and collection errors.
pub enum Error {
    /// Handles the case where a single metric name is used to register multiple
    /// different metrics.
    #[error("the provided metric has already been registered: {name}")]
    AlreadyRegistered {
        /// The duplicate metric name used.
        name: String,
    },
    /// Handles the case where the number of labels during write differs from the
    /// registered number of labels.
    #[error("the provided label count is incorrect for metric '{name}': got '{got}' but expected '{expected}'")]
    IncorrectLabelCount {
        /// The name of the metric.
        name: String,
        /// The expected number of labels for this metric.
        expected: usize,
        /// The actual number of labels received during write to this metric.
        got: usize,
    },
    /// Handles unknown error cases.
    #[error("an internal prometheus error occured when handling metric '{name}': {source}")]
    Unknown {
        /// The name of the metric.
        name: String,
        /// The initial error cause.
        source: prometheus::Error,
    },
}

impl Error {
    /// Translates a given prometheus Error into a local Error.
    ///
    /// ```rust
    /// use librift::metric;
    ///
    /// let x = prometheus::Error::AlreadyReg;
    /// match metric::Error::from(String::from("testing"), x) {
    ///     metric::Error::AlreadyRegistered { name } => assert_eq!(String::from("testing"), name),
    ///     _ => unimplemented!(),
    /// };
    /// ```
    pub fn from(name: String, e: prometheus::Error) -> Error {
        match e {
            prometheus::Error::AlreadyReg => Error::AlreadyRegistered { name },
            prometheus::Error::InconsistentCardinality { expect, got } => {
                Error::IncorrectLabelCount {
                    name,
                    expected: expect,
                    got,
                }
            }
            _ => Error::Unknown { name, source: e },
        }
    }
}

#[cfg(test)]
#[cfg(not(tarpaulin_include))]
mod tests {
    use super::*;

    #[test]
    fn test_from_prometheus() {
        let input = prometheus::Error::AlreadyReg;
        let actual = Error::from("name".to_string(), input);
        assert!(matches!(actual, Error::AlreadyRegistered {name} if name == "name"));

        let input = prometheus::Error::InconsistentCardinality { expect: 1, got: 0 };
        let actual = Error::from("name".to_string(), input);
        assert!(
            matches!(actual, Error::IncorrectLabelCount {name,expected,got} if name == "name" && expected == 1 && got == 0)
        );

        let input = prometheus::Error::Msg(String::from("hello"));
        let actual = Error::from("name".to_string(), input);
        assert!(matches!(actual, Error::Unknown {name, ..} if name == "name"))
    }
}
