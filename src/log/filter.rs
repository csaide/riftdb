// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

// stdlib usings
use std::result;

// extern usings
use slog::Drain;

/// Wraps a standard slog Drain so that we can filter the messages
/// logged by the defined log handler.
pub struct LevelFilter<D> {
    pub drain: D,
    pub level: slog::Level,
}

impl<D> Drain for LevelFilter<D>
where
    D: Drain,
{
    type Err = Option<D::Err>;
    type Ok = Option<D::Ok>;

    /// Handles actually filtering the log messages.
    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> result::Result<Self::Ok, Self::Err> {
        if record.level().is_at_least(self.level) {
            self.drain.log(record, values).map(Some).map_err(Some)
        } else {
            Ok(None)
        }
    }
}
