//! # Measured. A metrics crate.
//!
//! This crate was born out of a desire for better ergonomics dealing with prometheus,
//! with the added extra goal of minimizing small allocations to reduce memory fragmentation.
//!
//! ## Basic Usage
//!
//! The most basic usage is defining a single counter. This is very easy.
//!
//! ```
//! use measured::Counter;
//! use measured::text::TextEncoder;
//!
//! // create a counter
//! let counter = Counter::new();
//! // incremenet the counter value
//! counter.get_metric().inc();
//!
//! // sample the counter and encode the value to a textual format.
//! let mut text_encoder = TextEncoder::new();
//! counter.collect_into("my_first_counter", &mut text_encoder);
//! let bytes = text_encoder.finish();
//! ```
//!
//! ## With labels
//!
//! It's common to have labels added to your metrics, such as adding an operation type. When all possible values
//! can be determined at compile time, you can define the label value as a [`FixedCardinalityLabel`] enum.
//!
//! Multiple label pairs are collected into a [`LabelGroup`](label::LabelGroup).
//!
//! ```
//! use measured::CounterVec;
//! use measured::text::TextEncoder;
//! use measured::{LabelGroup, FixedCardinalityLabel};
//!
//! // Define a fixed cardinality label
//!
//! #[derive(FixedCardinalityLabel)]
//! #[label(rename_all = "snake_case")]
//! enum Operation {
//!     Create,
//!     Update,
//!     Delete,
//! }
//!
//! // Define a label group, consisting of 1 or more label values
//!
//! #[derive(LabelGroup)]
//! #[label(set = MyLabelGroupSet)]
//! struct MyLabelGroup {
//!     #[label(fixed)]
//!     operation: Operation,
//! }
//!
//! // create a counter vec
//! let counter = CounterVec::new(MyLabelGroupSet {});
//! // increment the counter at a given label
//! counter.inc(MyLabelGroup { operation: Operation::Create });
//! counter.inc(MyLabelGroup { operation: Operation::Delete });
//!
//! // sample the counters and encode the values to a textual format.
//! let mut text_encoder = TextEncoder::new();
//! counter.collect_into("my_first_counter", &mut text_encoder);
//! let bytes = text_encoder.finish();
//! ```
//!
//! ## With dynamic labels
//!
//! Sometimes, the labels cannot be determined at compile time, but they can be determine at the start of the program.
//! This might be the paths of a RESTful API. For efficiency,
//! `measured` offers a trait called [`FixedCardinalityDynamicLabel`](label::FixedCardinalityDynamicLabel) that allows for compact encoding.
//!
//! Implementations of [`FixedCardinalityDynamicLabel`](label::FixedCardinalityDynamicLabel) are provided for you,
//! notably [`indexmap::IndexSet`] and [`lasso::RodeoReader`].
//! I recommend the latter for string-based labels that are not `&'static` as it will offer the most efficient use of memory.
//!
//! ```
//! use measured::CounterVec;
//! use measured::text::TextEncoder;
//! use measured::{LabelGroup, FixedCardinalityLabel};
//!
//! // Define a label group, consisting of 1 or more label values
//!
//! #[derive(LabelGroup)]
//! #[label(set = MyLabelGroupSet)]
//! struct MyLabelGroup<'a> {
//!     #[label(fixed_with = lasso::RodeoReader)]
//!     path: &'a str,
//! }
//!
//! // initialise your fixed cardinality set
//! let set = MyLabelGroupSet {
//!     path: lasso::Rodeo::from_iter([
//!         "/api/v1/products",
//!         "/api/v1/users",
//!     ])
//!     .into_reader(),
//! };
//!
//! // create a counter vec
//! let counter = CounterVec::new(set);
//! // increment the counter at a given label
//! counter.inc(MyLabelGroup { path: "/api/v1/products" });
//! counter.inc(MyLabelGroup { path: "/api/v1/users" });
//!
//! // sample the counters and encode the values to a textual format.
//! let mut text_encoder = TextEncoder::new();
//! counter.collect_into("my_first_counter", &mut text_encoder);
//! let bytes = text_encoder.finish();
//! ```
//!
//! In the rare case that the label cannot be determined even at startup, you can still use them. You will have to make use of the
//! [`DynamicLabel`](label::DynamicLabel) trait. One implementation for string data is provided in the form of [`lasso::ThreadedRodeo`].
//!
//! It's not advised to use this for high cardinality labels, but if you must, this still offers good performance.
//!
//! ```
//! use measured::CounterVec;
//! use measured::text::TextEncoder;
//! use measured::{LabelGroup, FixedCardinalityLabel};
//!
//! // Define a label group, consisting of 1 or more label values
//!
//! #[derive(LabelGroup)]
//! #[label(set = MyLabelGroupSet)]
//! struct MyLabelGroup<'a> {
//!     #[label(dynamic_with = lasso::ThreadedRodeo)]
//!     path: &'a str,
//! }
//!
//! // initialise your dynamic cardinality set
//! let set = MyLabelGroupSet {
//!     path: lasso::ThreadedRodeo::new(),
//! };
//!
//! // create a counter vec
//! let counter = CounterVec::new(set);
//! // increment the counter at a given label
//! counter.inc(MyLabelGroup { path: "/api/v1/products" });
//! counter.inc(MyLabelGroup { path: "/api/v1/users" });
//!
//! // sample the counters and encode the values to a textual format.
//! let mut text_encoder = TextEncoder::new();
//! counter.collect_into("my_first_counter", &mut text_encoder);
//! let bytes = text_encoder.finish();
//! ```
//!
//! ## Prometheus vs Memory Fragmentation
//!
//! The [`prometheus`](https://docs.rs/prometheus/0.13.3/prometheus/index.html) crate allows you to very quickly
//! start recording metrics for your application and expose a text-based scrape endpoint. However, the implementation
//! can quickly lead to memory fragmentation issues.
//!
//! For example, let's look at `IntCounterVec`. It's an alias for `MetricVec<CounterVecBuilder<AtomicU64>>`. `MetricVec` has the following definition:
//!
//! ```ignore
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
//! ```ignore
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
