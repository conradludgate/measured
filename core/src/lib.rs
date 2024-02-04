//! # Measured. A metrics crate.
//!
//! This crate was born out of a desire for better ergonomics dealing with prometheus,
//! with the added extra goal of minimizing small allocations to reduce memory fragmentation.
//!
//! ## Prometheus vs Memory Fragmentation
//!
//! The [`prometheus`](https://docs.rs/prometheus/0.13.3/prometheus/index.html) crate allows you to very quickly
//! start recording metrics for your application and expose a text-based scrape endpoint. However, the implementation
//! can quickly lead to memory fragmentation issues.
//!
//! For example, let's look at `IntCounterVec`. It's an alias for `MetricVec<CounterVecBuilder<AtomicU64>>`. `MetricVec` has the following definition:
//!
//! ```no_compile
//! pub struct MetricVec<T: MetricVecBuilder> {
//!     pub(crate) v: Arc<MetricVecCore<T>>,
//! }
//! pub(crate) struct MetricVecCore<T: MetricVecBuilder> {
//!     pub children: RwLock<HashMap<u64, T::M>>,
//!     // ...
//! }
//! ```
//!
//! And for our int counter, `T::M` here is
//!
//! ```no_compile
//! pub struct GenericCounter<P: Atomic> {
//!     v: Arc<Value<P>>,
//! }
//!
//! pub struct Value<P: Atomic> {
//!     pub val: P,
//!     pub label_pairs: Vec<LabelPair>,
//!     // ...
//! }
//!
//! pub struct LabelPair {
//!     name: ::protobuf::SingularField<::std::string::String>,
//!     value: ::protobuf::SingularField<::std::string::String>,
//!     // ...
//! }
//! ```
//!
//! So, if we have a counter vec with 3 different labels, and a totel of 24 unique label groups, then we will have
//!
//! * 1 allocation for the `MetricVec` `Arc`
//! * 1 allocation for the `MetricVecCore` `HashMap`
//! * 24 allocations for the counter value `Arc`
//! * 24 allocations for the label pairs `Vec`
//! * 144 allocations for the `String`s in the `LabelPair`
//!
//! Totalling **194 small allocations**.
//!
//! There's nothing wrong with small allocations necessarily, but since these are long-lived allocations that are not allocated inside of
//! an arena, it can lead to fragmentation issues where each small alloc can occupy many different allocator pages and prevent them from being freed.
//!
//! Compared to this crate, `measured` **only needs 1 allocation** for the `HashMap`.
//! If you have semi-dynamic string labels (such as REST API path slugs) then that would add 4 allocations for
//! a [`RodeoReader`](lasso::RodeoReader) or 2 allocations for an [`IndexSet`](indexmap::IndexSet) to track them.
//!
//! And while it's bad form to have extremely high-cardinality metrics, this crate can easily handle
//! 100,000 unique label groups with just a few large allocations.
//!
//! ## Comparisons to the `metrics` family of crates
//!
//! The [`metrics`](https://docs.rs/metrics/latest/metrics/) facade crate and
//! [`metrics_exporter_prometheus`](https://docs.rs/metrics-exporter-prometheus/latest/metrics_exporter_prometheus/index.html)
//! implementation add a lot of complexity to exposing metrics. They also still alloc an `Arc<AtomicU64>` per individual counter
//! which does not solve the problem of memory fragmentation.

use metric::{counter::CounterState, histogram::HistogramState, Metric, MetricVec};

pub mod label;
pub mod metric;
pub mod text;

pub use measured_derive::{FixedCardinalityLabel, LabelGroup};

pub type Histogram<const N: usize> = Metric<HistogramState<N>>;
pub type HistogramVec<L, const N: usize> = MetricVec<HistogramState<N>, L>;
pub type Counter = Metric<CounterState>;
pub type CounterVec<L> = MetricVec<CounterState, L>;
