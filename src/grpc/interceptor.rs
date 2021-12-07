// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Instant;

use prometheus::{Histogram, IntCounter};
use tonic::service::Interceptor;
use tonic::{Request, Status};

use crate::metric::Manager;

/// The LoggerExt handles injecting a request specific logger into the gRPC execution
/// chain.
pub struct LoggerExt {
    /// The logger to use throughout this requests life cycle.
    pub logger: slog::Logger,
}

/// The ResponseTimeExt handles injecting a
pub struct ResponseTimeExt {
    /// The response time histogram to use for observing measurements for this gRPC
    /// request.
    pub histogram: Histogram,
    /// The start instant for this request to measure execution time against.
    pub start: Instant,
}

impl ResponseTimeExt {
    /// Observe the total response time generally used within a defer statement.
    pub fn observe(&self) {
        self.histogram
            .observe(self.start.elapsed().as_millis() as f64)
    }
}

/// The interceptor wrapper to have all gRPC requests pass through.
#[derive(Debug, Clone)]
pub struct RiftInterceptor {
    logger: slog::Logger,

    total_requests: IntCounter,
    response_time: Histogram,
}

impl RiftInterceptor {
    /// Create a new RiftInterceptor based on the supplied arguments.
    pub fn new(logger: &slog::Logger, mm: Manager) -> Self {
        Self {
            logger: logger.clone(),
            total_requests: mm
                .register_int_counter(
                    "total_requests",
                    "The total count of gRPC requests seen by this server.",
                )
                .unwrap(),
            response_time: mm
                .register_histogram(
                    "response_time",
                    "The response time over all received gRPC requests seen by this server.",
                    &[],
                )
                .unwrap(),
        }
    }
}

impl Interceptor for RiftInterceptor {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        self.total_requests.inc();

        let req_id = if let Some(req_id) = req.metadata().get("x-request-id") {
            req_id.to_str().unwrap().to_string()
        } else {
            uuid::Uuid::new_v4().to_string()
        };

        req.extensions_mut().insert(LoggerExt {
            logger: self.logger.new(o!("reqID" => req_id)),
        });
        req.extensions_mut().insert(ResponseTimeExt {
            histogram: self.response_time.clone(),
            start: Instant::now(),
        });

        Ok(req)
    }
}
