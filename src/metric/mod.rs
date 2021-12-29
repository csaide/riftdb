// (c) Copyright 2021 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

// std usings
use std::collections::HashMap;

// extern usings
use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec,
};

mod error;

pub use error::{Error, Result};

/// A Manager handles creating and returning fully qualified metric collectors based on the supplied const labels.
/// This should be created as needed on a per subsystem basis.
pub struct Manager {
    /// namespace represents the overall namespace to store metrics within. i.e. `rift`.
    pub namespace: String,
    /// subsystem represents the specific subsystem within a given application the metric is based on. i.e. `rest`.
    pub subsystem: String,
    /// app represents the specific binary the metrics are from.
    pub bin: String,
    /// version represents the specific version of the binary the metrics are from.
    pub version: String,
}

impl Manager {
    fn opts(&self, name: &str, help: &str, raw_labels: &[&str]) -> prometheus::Opts {
        let mut const_labels = HashMap::new();
        const_labels.insert(String::from("binary"), self.bin.clone());
        const_labels.insert(String::from("version"), self.version.clone());

        let variable_labels = raw_labels.iter().map(|s| String::from(*s)).collect();
        prometheus::Opts {
            namespace: self.namespace.clone(),
            subsystem: self.subsystem.clone(),
            name: name.to_owned(),
            help: help.to_owned(),
            const_labels,
            variable_labels,
        }
    }

    fn histogram_opts(
        &self,
        name: &str,
        help: &str,
        buckets: &[f64],
        raw_labels: &[&str],
    ) -> prometheus::HistogramOpts {
        let buckets = if buckets.is_empty() {
            prometheus::DEFAULT_BUCKETS
        } else {
            buckets
        };
        prometheus::HistogramOpts {
            buckets: buckets.to_vec(),
            common_opts: self.opts(name, help, raw_labels),
        }
    }

    /// Create a new metrics manager instance, based on the supplied naming information.
    pub fn new(namespace: String, subsystem: String, bin: String, version: String) -> Manager {
        Manager {
            namespace,
            subsystem,
            bin,
            version,
        }
    }

    /// Register a new generic atomic f64 based counter. This is best used when you need
    /// to track fractional increments as opposed to whole number increments which you should use
    /// an IntCounter for.
    pub fn register_counter(&self, name: &str, help: &str) -> Result<Counter> {
        let opts = self.opts(name, help, &[]);
        register_counter!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atomic f64 counter vec. This is best used when you need to track fractional
    /// increments over a multitude of different dimensions. Otherwise you should use an IntCounterVec.
    pub fn register_counter_vec(
        &self,
        name: &str,
        help: &str,
        keys: &[&str],
    ) -> Result<CounterVec> {
        let opts = self.opts(name, help, keys);
        register_counter_vec!(opts, keys).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new atomic u64 based counter. This is best used when you need to track whole number
    /// increments as opposed to fractional increments which you should use a Counter for.
    pub fn register_int_counter(&self, name: &str, help: &str) -> Result<IntCounter> {
        let opts = self.opts(name, help, &[]);
        register_int_counter!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new atomic u64 counter vec. This is best used when you need to track  whole number
    /// increments over a multitude of different dimensions. Otherwise you should use an CounterVec.
    pub fn register_int_counter_vec(
        &self,
        name: &str,
        help: &str,
        keys: &[&str],
    ) -> Result<IntCounterVec> {
        let opts = self.opts(name, help, keys);
        register_int_counter_vec!(opts, keys).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atomic f64 based gauge. This is best used when you need to track
    /// fractional increments and decrements, as opposed to whole number increments and decrements,
    /// which you should use an IntGauge for.
    pub fn register_gauge(&self, name: &str, help: &str) -> Result<Gauge> {
        let opts = self.opts(name, help, &[]);
        register_gauge!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atomic f64 gauge vec. This is best used when you need to track fractional
    /// increments and decrements over a multitude of different dimensions. Otherwise you should use an
    /// IntGaugeVec.
    pub fn register_gauge_vec(&self, name: &str, help: &str, keys: &[&str]) -> Result<GaugeVec> {
        let opts = self.opts(name, help, keys);
        register_gauge_vec!(opts, keys).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new atomic u64 based gauge. This is best used when you need to track whole increments
    /// and decrements, as opposed to fractional number increments and decrements, which you should use an
    /// Gauge for.
    pub fn register_int_gauge(&self, name: &str, help: &str) -> Result<IntGauge> {
        let opts = self.opts(name, help, &[]);
        register_int_gauge!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new atomic u64 gauge vec. This is best used when you need to track whole
    /// increments and decrements over a multitude of different dimensions. Otherwise you should use an
    /// GaugeVec.
    pub fn register_int_gauge_vec(
        &self,
        name: &str,
        help: &str,
        keys: &[&str],
    ) -> Result<IntGaugeVec> {
        let opts = self.opts(name, help, keys);
        register_int_gauge_vec!(opts, keys).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atmoic f64 based bucketed histogram. This is best when you need to track
    /// individual observations.
    pub fn register_histogram(&self, name: &str, help: &str, buckets: &[f64]) -> Result<Histogram> {
        let opt = self.histogram_opts(name, help, buckets, &[]);
        register_histogram!(opt).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atmoic f64 based bucketed histogram vec. This is best when you need to
    /// track individual observations over a multitude of different dimensions.
    pub fn register_histogram_vec(
        &self,
        name: &str,
        help: &str,
        buckets: &[f64],
        keys: &[&str],
    ) -> Result<HistogramVec> {
        let opt = self.histogram_opts(name, help, buckets, keys);
        register_histogram_vec!(opt, keys).map_err(|err| Error::from(name.to_owned(), err))
    }
}

#[cfg(test)]
mod tests {
    use std::unimplemented;

    use super::*;

    fn manager() -> Manager {
        Manager::new(
            String::from("testing"),
            String::from("test"),
            String::from("test"),
            String::from("0.1.0"),
        )
    }

    #[test]
    fn test_counter() {
        let mm = manager();
        let cnt = match mm.register_counter("counter", "A test counter!") {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.inc();
        assert_eq!(1.0, cnt.get());
    }

    #[test]
    fn test_counter_vec() {
        let mm = manager();
        let cnt = match mm.register_counter_vec("counter_vec", "A test counter vec!", &["testing"])
        {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.with_label_values(&["woot"]).inc();
        assert_eq!(1.0, cnt.with_label_values(&["woot"]).get());
    }

    #[test]
    fn test_int_counter() {
        let mm = manager();
        let cnt = match mm.register_int_counter("int_counter", "A test int counter!") {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.inc();
        assert_eq!(1, cnt.get());
    }

    #[test]
    fn test_int_counter_vec() {
        let mm = manager();
        let cnt = match mm.register_int_counter_vec(
            "int_counter_vec",
            "A test int counter vec!",
            &["testing"],
        ) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.with_label_values(&["woot"]).inc();
        assert_eq!(1, cnt.with_label_values(&["woot"]).get());
    }

    #[test]
    fn test_gauge() {
        let mm = manager();
        let cnt = match mm.register_gauge("gauge", "A test gauge!") {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.inc();
        assert_eq!(1.0, cnt.get());
    }

    #[test]
    fn test_gauge_vec() {
        let mm = manager();
        let cnt = match mm.register_gauge_vec("gauge_vec", "A test gauge vec!", &["testing"]) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.with_label_values(&["woot"]).inc();
        assert_eq!(1.0, cnt.with_label_values(&["woot"]).get());
    }

    #[test]
    fn test_int_gauge() {
        let mm = manager();
        let cnt = match mm.register_int_gauge("int_gauge", "A test int gauge!") {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.inc();
        assert_eq!(1, cnt.get());
    }

    #[test]
    fn test_int_gauge_vec() {
        let mm = manager();
        let cnt =
            match mm.register_int_gauge_vec("int_gauge_vec", "A test int gauge vec!", &["testing"])
            {
                Ok(metric) => metric,
                Err(_) => unimplemented!(),
            };
        cnt.with_label_values(&["woot"]).inc();
        assert_eq!(1, cnt.with_label_values(&["woot"]).get());
    }

    #[test]
    fn test_histogram() {
        let mm = manager();
        let hist = match mm.register_histogram(
            "histogram",
            "A test histogram!",
            prometheus::DEFAULT_BUCKETS,
        ) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        hist.observe(0.1);
        assert_eq!(0.1, hist.get_sample_sum());
    }

    #[test]
    fn test_histogram_no_buckets() {
        let mm = manager();
        let hist = match mm.register_histogram("histogram_no_buckets", "A test histogram!", &[]) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        hist.observe(0.1);
        assert_eq!(0.1, hist.get_sample_sum());
    }

    #[test]
    fn test_histogram_vec() {
        let mm = manager();
        let hist = match mm.register_histogram_vec(
            "histogram_vec",
            "A test histogram!",
            prometheus::DEFAULT_BUCKETS,
            &["testing"],
        ) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        hist.with_label_values(&["woot"]).observe(0.1);
        assert_eq!(0.1, hist.with_label_values(&["woot"]).get_sample_sum());
    }
}
