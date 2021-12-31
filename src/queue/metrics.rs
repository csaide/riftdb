// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

use std::time::Duration;

use lazy_static::lazy_static;
use prometheus::{register_int_counter, IntCounter, IntCounterVec, IntGauge};

pub const ACK_VALUE: &str = "ack";
pub const NACK_VALUE: &str = "nack";
pub const DEFAULT_TTL: Duration = Duration::from_secs(10);
pub const NO_CAPACITY: usize = 0;

lazy_static! {
    pub static ref TOTAL_MESSAGES_RECEIVED: IntCounter = register_int_counter!(
        "rift_const_queue_received_messages",
        "The total number of messages received by all const_queues."
    )
    .unwrap();
    pub static ref MESSAGE_RESULTS: IntCounterVec = register_int_counter_vec!(
        "rift_const_queue_message_results",
        "The number of handled messages by result type across all const_queues.",
        &["result"],
    )
    .unwrap();
    pub static ref MESSAGE_LEASE_EXPIRES: IntCounter = register_int_counter!(
        "rift_const_queue_message_lease_expires",
        "The number of message leases that have expired across all const_queues."
    )
    .unwrap();
    pub static ref MESSAGES_OUTSTANDING: IntGauge = register_int_gauge!(
        "rift_const_queue_outstanding_messages",
        "The totall number of messages currently locked across all const_queues."
    )
    .unwrap();
    pub static ref MESSAGES_PENDING: IntGauge = register_int_gauge!(
        "rift_const_queue_pending_messages",
        "The total number of messages currently pending across all const_queues."
    )
    .unwrap();
}
