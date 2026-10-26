//! Understand the model and its tradeoffs.
//!
//! Read the [terminology guide](super::guide) for how metrics, vectors, groups, and label sets fit
//! together, and the [design discussion](super::details) for storage and allocation decisions.
//! For steps to complete a task, use the [how-to guides](super::how_to).
//!
//! # Choosing a metric
//!
//! Counters accumulate events; gauges describe a current quantity. Histograms retain a distribution
//! in fixed buckets, allowing consumers to aggregate counts and estimate quantiles. Use a float
//! counter for cumulative fractional quantities such as CPU seconds.
//!
//! An info metric exposes descriptive labels on a sample that is always 1. A state set exposes
//! which enum state is active. Callback metrics read a value that another subsystem already owns
//! when a scrape occurs, avoiding a second copy that must be kept in sync.
//!
//! Gauge histograms describe a distribution whose values can be added and removed. Their bucket
//! counts can decrease, so counter rate calculations are inappropriate for them, even when classic
//! text exports them with a histogram TYPE line.
//!
//! # Why label sets are explicit
//!
//! Each label combination identifies a separate series. A label set maps values to compact IDs,
//! so repeated recording does not require storing a new copy of every string in every series.
//! Fixed cardinality can also let a vector address a series directly in an array.
//!
//! Dynamic sets allow values that are not known at startup, but both the interner and the metric
//! vector grow as new combinations appear. Efficient storage does not make unbounded cardinality
//! inexpensive for the application or its monitoring system. Prefer bounded values such as route
//! templates and operation names over user IDs or arbitrary URLs.
//!
//! # Recording and encoding are separate
//!
//! Metric state defines what is recorded; an encoder determines its wire representation.
//! `MetricEncoding` collects a state into any `Encoding`, so adding an encoder does not require
//! implementing every metric again. `MetricGroup` supplies names, help text, and grouping.
//! Encoders can translate unsupported metric kinds into simpler representations; the
//! [format reference](super::reference#export-formats) describes those differences.
//!
//! Exemplars attach an example observation, such as a trace ID, to a series rather than creating
//! a new series for each trace. Creation timestamps let a consumer distinguish a counter reset
//! between scrapes. Both require extra storage, which is why their metric wrappers are opt-in.
