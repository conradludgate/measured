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
//! `measured-tokio` has a `net` feature for IO metrics and an internal
//! `__tokio-unstable-schedule-latency` feature requiring `--cfg tokio_unstable`.
//! Unstable Tokio APIs are outside the collector's semver guarantees.
//!
//! # Export formats
//!
//! | Crate | Encoders | Extras |
//! | --- | --- | --- |
//! | `measured-prometheus-text` | `TextEncoder`, `BufferedTextEncoder` | Classic Prometheus text; omits exemplars and creation timestamps |
//! | `measured-prometheus-text` | `OpenMetricsEncoder`, `BufferedOpenMetricsEncoder` | OpenMetrics 1.0; preserves exemplars and creation timestamps |
//! | `measured-prometheus-protobuf` | `ProtoEncoder` | Delimited Prometheus protobuf; preserves exemplars and creation timestamps |
//!
//! Classic text represents info metrics and state sets as gauges, and gauge histograms as histograms.
//! Protobuf represents info metrics and state sets as gauges and has a native gauge histogram type.
//! See [`MetricKind`](crate::metric::group::MetricKind) and each encoder's API for fallback behavior.
//! Use `Encoding::MIME_TYPE` for the chosen encoder's HTTP content type.
//!
//! # Version requirements and migration
//!
//! Version 0.1 requires Rust 1.98 or newer. The text encoder lives in its own crate.
//! Custom metric implementations use `MetricEncoding` without an encoder type parameter;
//! `collect_into` is generic over `E: Encoding`.
//! The repository's `core/CHANGELOG.md` lists all breaking changes from 0.0.25.
