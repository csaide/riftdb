// (c) Copyright 2021-2022 Christian Saide
// SPDX-License-Identifier: GPL-3.0-only

use prometheus::{
    Counter, CounterVec, Gauge, GaugeVec, Histogram, HistogramVec, IntCounter, IntCounterVec,
    IntGauge, IntGaugeVec,
};

use super::{
    opt::{to_common_opts, to_histogram_opts},
    Error, Opt, Result,
};

/// A Manager handles creating and returning fully qualified metric collectors based on the supplied const labels.
/// This should be created as needed on a per subsystem basis.
pub struct Manager {
    /// namespace represents the overall namespace to store metrics within. i.e. `rift`.
    pub namespace: String,
    /// subsystem represents the specific subsystem within a given application the metric is based on. i.e. `rest`.
    pub subsystem: String,
    /// version represents the specific version of the binary the metrics are from.
    pub version: String,
}

impl Manager {
    fn opts(&self, name: &str, help: &str, user_opts: Option<Vec<Opt>>) -> prometheus::Opts {
        let mut opts = to_common_opts(name, help, user_opts);
        opts.const_labels
            .insert(String::from("version"), self.version.clone());
        if opts.namespace.is_empty() {
            opts.namespace = self.namespace.clone();
        }
        if opts.subsystem.is_empty() {
            opts.subsystem = self.subsystem.clone();
        }
        opts
    }

    fn histogram_opts(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> prometheus::HistogramOpts {
        let mut opts = to_histogram_opts(name, help, user_opts);
        opts.common_opts
            .const_labels
            .insert(String::from("version"), self.version.clone());
        if opts.common_opts.namespace.is_empty() {
            opts.common_opts.namespace = self.namespace.clone();
        }
        if opts.common_opts.subsystem.is_empty() {
            opts.common_opts.subsystem = self.subsystem.clone();
        }
        opts
    }

    /// Create a new metrics manager instance, based on the supplied naming information.
    pub fn new(namespace: String, subsystem: String, version: String) -> Manager {
        Manager {
            namespace,
            subsystem,
            version,
        }
    }

    /// Register a new generic atomic f64 based counter. This is best used when you need
    /// to track fractional increments as opposed to whole number increments which you should use
    /// an IntCounter for.
    pub fn register_counter(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<Counter> {
        let opts = self.opts(name, help, user_opts);
        register_counter!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atomic f64 counter vec. This is best used when you need to track fractional
    /// increments over a multitude of different dimensions. Otherwise you should use an IntCounterVec.
    pub fn register_counter_vec(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<CounterVec> {
        let opts = self.opts(name, help, user_opts);
        let labels = opts.variable_labels.clone();
        let labels = labels.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        register_counter_vec!(opts, labels.as_ref())
            .map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new atomic u64 based counter. This is best used when you need to track whole number
    /// increments as opposed to fractional increments which you should use a Counter for.
    pub fn register_int_counter(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<IntCounter> {
        let opts = self.opts(name, help, user_opts);
        register_int_counter!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new atomic u64 counter vec. This is best used when you need to track  whole number
    /// increments over a multitude of different dimensions. Otherwise you should use an CounterVec.
    pub fn register_int_counter_vec(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<IntCounterVec> {
        let opts = self.opts(name, help, user_opts);
        let labels = opts.variable_labels.clone();
        let labels = labels.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        register_int_counter_vec!(opts, labels.as_ref())
            .map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atomic f64 based gauge. This is best used when you need to track
    /// fractional increments and decrements, as opposed to whole number increments and decrements,
    /// which you should use an IntGauge for.
    pub fn register_gauge(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<Gauge> {
        let opts = self.opts(name, help, user_opts);
        register_gauge!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atomic f64 gauge vec. This is best used when you need to track fractional
    /// increments and decrements over a multitude of different dimensions. Otherwise you should use an
    /// IntGaugeVec.
    pub fn register_gauge_vec(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<GaugeVec> {
        let opts = self.opts(name, help, user_opts);
        let labels = opts.variable_labels.clone();
        let labels = labels.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        register_gauge_vec!(opts, labels.as_ref()).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new atomic u64 based gauge. This is best used when you need to track whole increments
    /// and decrements, as opposed to fractional number increments and decrements, which you should use an
    /// Gauge for.
    pub fn register_int_gauge(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<IntGauge> {
        let opts = self.opts(name, help, user_opts);
        register_int_gauge!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new atomic u64 gauge vec. This is best used when you need to track whole
    /// increments and decrements over a multitude of different dimensions. Otherwise you should use an
    /// GaugeVec.
    pub fn register_int_gauge_vec(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<IntGaugeVec> {
        let opts = self.opts(name, help, user_opts);
        let labels = opts.variable_labels.clone();
        let labels = labels.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        register_int_gauge_vec!(opts, labels.as_ref())
            .map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atmoic f64 based bucketed histogram. This is best when you need to track
    /// individual observations.
    pub fn register_histogram(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<Histogram> {
        let opts = self.histogram_opts(name, help, user_opts);
        register_histogram!(opts).map_err(|err| Error::from(name.to_owned(), err))
    }

    /// Register a new generic atmoic f64 based bucketed histogram vec. This is best when you need to
    /// track individual observations over a multitude of different dimensions.
    pub fn register_histogram_vec(
        &self,
        name: &str,
        help: &str,
        user_opts: Option<Vec<Opt>>,
    ) -> Result<HistogramVec> {
        let opts = self.histogram_opts(name, help, user_opts);
        let labels = opts.common_opts.variable_labels.clone();
        let labels = labels.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        register_histogram_vec!(opts, labels.as_ref())
            .map_err(|err| Error::from(name.to_owned(), err))
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
            String::from("0.1.0"),
        )
    }

    #[test]
    fn test_counter() {
        let mm = manager();
        let cnt = match mm.register_counter("counter", "A test counter!", None) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.inc();
        assert_eq!(1.0, cnt.get());
    }

    #[test]
    fn test_counter_vec() {
        let mm = manager();
        let opts = vec![Opt::Labels(vec![String::from("testing")])];
        let cnt = match mm.register_counter_vec("counter_vec", "A test counter vec!", Some(opts)) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.with_label_values(&["woot"]).inc();
        assert_eq!(1.0, cnt.with_label_values(&["woot"]).get());
    }

    #[test]
    fn test_int_counter() {
        let mm = manager();
        let cnt = match mm.register_int_counter("int_counter", "A test int counter!", None) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.inc();
        assert_eq!(1, cnt.get());
    }

    #[test]
    fn test_int_counter_vec() {
        let mm = manager();
        let opts = vec![Opt::Labels(vec![String::from("testing")])];
        let cnt = match mm.register_int_counter_vec(
            "int_counter_vec",
            "A test int counter vec!",
            Some(opts),
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
        let cnt = match mm.register_gauge("gauge", "A test gauge!", None) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.inc();
        assert_eq!(1.0, cnt.get());
    }

    #[test]
    fn test_gauge_vec() {
        let mm = manager();
        let opts = vec![Opt::Labels(vec![String::from("testing")])];
        let cnt = match mm.register_gauge_vec("gauge_vec", "A test gauge vec!", Some(opts)) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.with_label_values(&["woot"]).inc();
        assert_eq!(1.0, cnt.with_label_values(&["woot"]).get());
    }

    #[test]
    fn test_int_gauge() {
        let mm = manager();
        let cnt = match mm.register_int_gauge("int_gauge", "A test int gauge!", None) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        cnt.inc();
        assert_eq!(1, cnt.get());
    }

    #[test]
    fn test_int_gauge_vec() {
        let mm = manager();
        let opts = vec![Opt::Labels(vec![String::from("testing")])];
        let cnt =
            match mm.register_int_gauge_vec("int_gauge_vec", "A test int gauge vec!", Some(opts)) {
                Ok(metric) => metric,
                Err(_) => unimplemented!(),
            };
        cnt.with_label_values(&["woot"]).inc();
        assert_eq!(1, cnt.with_label_values(&["woot"]).get());
    }

    #[test]
    fn test_histogram() {
        let mm = manager();
        let hist = match mm.register_histogram("histogram", "A test histogram!", None) {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        hist.observe(0.1);
        assert_eq!(0.1, hist.get_sample_sum());
    }

    #[test]
    fn test_histogram_vec() {
        let mm = manager();
        let opts = vec![Opt::Labels(vec![String::from("testing")])];
        let hist = match mm.register_histogram_vec("histogram_vec", "A test histogram!", Some(opts))
        {
            Ok(metric) => metric,
            Err(_) => unimplemented!(),
        };
        hist.with_label_values(&["woot"]).observe(0.1);
        assert_eq!(0.1, hist.with_label_values(&["woot"]).get_sample_sum());
    }
}
