//! The guide to terminology used in this crate, and how it all fits together.
//!
//! # `Metric`
//!
//! A [`Metric`](crate::Metric) represents a single aggregated quantity.
//!
//! ## `Counter`
//!
//! A [`Counter`](crate::Counter) is a `Metric` that typically represents how many events have been observed throughout the lifetime of the program.
//!
//! For instance:
//! * Total number of HTTP requests
//! * Total number of error reponses
//!
//! When sampled over time, the increase can be used to calculate the rate-per-second of each event.
//!
//! ## `Gauge`
//!
//! A [`Gauge`](crate::Gauge) is a `Metric` that typically represents the size of a resource as used by the program at the current point in time.
//!
//! For instance:
//! * Memory usage in bytes.
//! * Number of idle database connections in a connection pool.
//! * Number of active tasks
//!
//! ## `Histogram`
//!
//! A [`Histogram`](crate::Histogram) is a `Metric` that represents dynamically sized observations throughout the lifetime of the program.
//!
//! For instance:
//! * Latencies for DB queries
//! * Sizes of HTTP request bodies
//!
//! When sampled over time, the histogram bucket increases can be used to calculate quantiles, such as P50s, P99s, etc.
//!
//! # `MetricVec`
//!
//! A [`MetricVec`](crate::MetricVec) represents multiple `Metric`s, keyed by a group of labels.
//!
//! These labels are typically used to refine the metric to a more specific case. For instance:
//! * a [`CounterVec`](crate::CounterVec) could represent the total number of HTTP requests **by resource path**
//! * a [`GaugeVec`](crate::GaugeVec) could represent the number of currently active tasks **by task type**
//! * a [`HistogramVec`](crate::HistogramVec) could represent the latencies of DB queries **by logical query name**
//!
//! # `MetricGroup`
//!
//! As the name implies, a [`MetricGroup`](crate::MetricGroup) is a logical group of `Metric`s or `MetricVec`s.
//! These groups can be as small or as large as necessary for the needs of your library or application.
//!
//! Groups can be nested with other groups, with the ability to namespace the metrics within the group as well,
//! for better isolation of metrics exposed by a library.
//!
//! # Labels
//!
//! ## `LabelValue`
//!
//! A [`LabelValue`](crate::label::LabelValue) is a value that can be encoded in a label position.
//!
//! ### `FixedCardinalityLabel`
//!
//! A [`FixedCardinalityLabel`](crate::FixedCardinalityLabel) is a `LabelValue` that has a fixed 'cardinality', i.e. a fixed number of possible values.
//! This is typically represented via a unit `enum` where each variant represents one of the fixed possible values.
//!
//! ## `LabelSet`
//!
//! This is a unique quirk of this crate. A [`LabelSet`](crate::label::LabelSet) is needed in order to convert an arbitrary `LabelValue` into
//! a specific integer value. This is a necessary feature in order to unlock some of the impressive performance gains that this library unlocks.
//!
//! ### `FixedCardinalitySet`
//!
//! A [`FixedCardinalitySet`](crate::label::FixedCardinalitySet) is a `LabelSet` that can encode a fixed number of possible `LabelValue`s.
//! This is slightly different to `FixedCardinalityLabel` because a `FixedCardinalityLabel` must know all possible values at compile time,
//! whereas a `FixedCardinalitySet` must only know all possible values at startup time.
//!
//! ### `DynamicLabelSet`
//!
//! A [`DynamicLabelSet`](crate::label::DynamicLabelSet) is a `LabelSet` that can encode any number of possible `LabelValue`s.
//! Values do not need to be provided up front. Values will be inserted into the set with a consistent index value when they are discovered.
//! Because of this, they end up being the most expensive set option. They are necessary for use cases where values are only known post-startup,
//! or just useful for convenience where some performance loss is acceptable.
//!
//! # `LabelGroup`
//!
//! As the name implies, a [`LabelGroup`](crate::LabelGroup) is a group of labels which can be used as the key to a `MetricVec`.
//!
//! # `LabelGroupSet`
//!
//! A [`LabelGroupSet`](crate::label::LabelGroupSet) is to a `LabelGroup`, as a `LabelSet` is to a `LabelValue`.
