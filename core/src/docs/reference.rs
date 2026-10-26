//! Find types, configuration, and compatibility contracts.
//!
//! # API map
//!
//! | Area | Reference |
//! | --- | --- |
//! | Metric types and recording methods | [Crate exports](crate) |
//! | Derive attributes | [`MetricGroup`](crate::MetricGroup), [`LabelGroup`](crate::LabelGroup), [`FixedCardinalityLabel`](crate::FixedCardinalityLabel) |
//! | Storage, lookup, removal, projection | [`MetricVec`](crate::metric::MetricVec) |
//! | Label value and set contracts | [`label`](crate::label) |
//! | Custom metric implementations | [`MetricType`](crate::metric::MetricType), [`MetricEncoding`](crate::metric::MetricEncoding), [`MetricFamilyEncoding`](crate::metric::MetricFamilyEncoding) |
//! | Encoder implementations | [`Encoding`](crate::metric::group::Encoding) |
//! | Naming and namespaces | [`name`](crate::metric::name) |
//!
//! # Cargo features
//!
//! The `measured` crate has no default features. Derive macros are always available.
//!
//! | Feature | Label set support |
//! | --- | --- |
//! | `lasso` | `RodeoReader`, `ThreadedRodeo`; also reexports `measured::lasso` |
//! | `paracord` | `paracord::ParaCord` |
//! | `indexmap` | `indexmap::IndexSet` |
//! | `phf` | `phf::OrderedSet<&'static str>` |
//!
//! # Export formats
//!
//! | Crate | Encoders | Extras |
//! | --- | --- | --- |
//! | [measured-prometheus-text](https://docs.rs/measured-prometheus-text/latest/measured_prometheus_text/) | `TextEncoder`, `BufferedTextEncoder` | Classic Prometheus text; omits exemplars and creation timestamps |
//! | [measured-prometheus-text](https://docs.rs/measured-prometheus-text/latest/measured_prometheus_text/) | `OpenMetricsEncoder`, `BufferedOpenMetricsEncoder` | OpenMetrics 1.0; preserves exemplars and creation timestamps |
//! | [measured-prometheus-protobuf](https://docs.rs/measured-prometheus-protobuf/latest/measured_prometheus_protobuf/) | `ProtoEncoder` | Delimited Prometheus protobuf; preserves exemplars and creation timestamps |
//!
//! Classic text represents info metrics and state sets as gauges, and gauge histograms as histograms.
//! Protobuf represents info metrics and state sets as gauges and has a native gauge histogram type.
//! See [`MetricKind`](crate::metric::group::MetricKind) and each encoder's API for fallback behavior.
//! Use `Encoding::MIME_TYPE` for the chosen encoder's HTTP content type.
//!
//! # Metric units
//!
//! Put a known unit on a derived metric field with `#[metric(unit = "seconds")]`. The declared
//! metric name must end in `_<unit>`; counter names may have a trailing `_total`, as in
//! `request_duration_seconds_total`. The derive checks this naming rule at compile time.
//! OpenMetrics emits a `# UNIT` line, and protobuf preserves the unit metadata. Classic Prometheus
//! text has no unit field and ignores it. Manual family collection can pass a unit to
//! `collect_family_into` with `FamilyMeta::new().with_unit("seconds")`, but metric names are not
//! checked automatically.
//!
//! ```compile_fail
//! use measured::{Counter, MetricGroup};
//!
//! #[derive(MetricGroup)]
//! struct Metrics {
//!     #[metric(unit = "seconds")]
//!     request_duration: Counter,
//! }
//! ```
