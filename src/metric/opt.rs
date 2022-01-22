// (c) Copyright 2021-2022 Christian Saide <supernomad>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

/// A metric option to use during registration.
pub enum Opt {
    /// A set of constant key/values for this metric.
    ConstLabels(HashMap<String, String>),
    /// A list of variable keys for this metric.
    Labels(Vec<String>),
    /// A list of buckets to use with histograms. Note this [Opt]
    /// is ignored in all cases other than histograms and summaries.
    Buckets(Vec<f64>),
    /// The namespace this metric belongs to.
    Namespace(String),
    /// The subsystem this metric belongs to.
    Subsystem(String),
}

pub(super) fn to_common_opts<N, H>(
    name: N,
    help: H,
    user_opts: Option<Vec<Opt>>,
) -> prometheus::Opts
where
    N: Into<String>,
    H: Into<String>,
{
    let mut opts = prometheus::Opts::new(name, help);
    let mut user_opts = match user_opts {
        Some(user_opts) => user_opts,
        None => return opts,
    };
    for opt in user_opts.drain(..) {
        use Opt::*;
        match opt {
            ConstLabels(const_labels) => opts.const_labels = const_labels,
            Labels(labels) => opts.variable_labels = labels,
            Buckets(_) => continue,
            Namespace(namespace) => opts.namespace = namespace,
            Subsystem(subsystem) => opts.subsystem = subsystem,
        };
    }
    opts
}

pub(super) fn to_histogram_opts<N, H>(
    name: N,
    help: H,
    user_opts: Option<Vec<Opt>>,
) -> prometheus::HistogramOpts
where
    N: Into<String>,
    H: Into<String>,
{
    let mut opts = prometheus::HistogramOpts::new(name, help);
    let mut user_opts = match user_opts {
        Some(user_opts) => user_opts,
        None => return opts,
    };
    for opt in user_opts.drain(..) {
        use Opt::*;
        match opt {
            ConstLabels(const_labels) => opts.common_opts.const_labels = const_labels,
            Labels(labels) => opts.common_opts.variable_labels = labels,
            Buckets(buckets) => opts.buckets = buckets,
            Namespace(namespace) => opts.common_opts.namespace = namespace,
            Subsystem(subsystem) => opts.common_opts.subsystem = subsystem,
        };
    }
    opts
}
