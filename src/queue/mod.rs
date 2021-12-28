// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0

mod error;
mod queue;
mod slot;

pub use error::{Error, Result};
pub use queue::Queue;
pub use slot::{Slot, SlotLease};
