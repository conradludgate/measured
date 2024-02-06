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
//! use measured::metric::name::MetricName;
//! use measured::text::TextEncoder;
//!
//! // create a counter
//! let counter = Counter::new();
//! // increment the counter value
//! counter.get_metric().inc();
//!
//! // sample the counter and encode the value to a textual format.
//! let mut text_encoder = TextEncoder::new();
//! let name = MetricName::from_static("my_first_counter");
//! counter.collect_into(name, &mut text_encoder);
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
//! use measured::{CounterVec, LabelGroup, FixedCardinalityLabel};
//! use measured::metric::name::MetricName;
//! use measured::text::TextEncoder;
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
//! let counters = CounterVec::new(MyLabelGroupSet {});
//! // increment the counter at a given label
//! counters.inc(MyLabelGroup { operation: Operation::Create });
//! counters.inc(MyLabelGroup { operation: Operation::Delete });
//!
//! // sample the counters and encode the values to a textual format.
//! let mut text_encoder = TextEncoder::new();
//! let name = MetricName::from_static("my_first_counter");
//! counters.collect_into(name, &mut text_encoder);
//! let bytes = text_encoder.finish();
//! ```
//!
//! ## With dynamic labels and label sets
//!
//! Sometimes, the labels cannot be determined at compile time, but they can be determine at the start of the program.
//! This might be the paths of a RESTful API. For efficiency,
//! `measured` offers a trait called [`FixedCardinalitySet`](label::FixedCardinalitySet) that allows for compact encoding.
//!
//! Implementations of [`FixedCardinalitySet`](label::FixedCardinalitySet) are provided for you,
//! notably [`indexmap::IndexSet`] and [`lasso::RodeoReader`].
//! I recommend the latter for string-based labels that are not `&'static` as it will offer the most efficient use of memory.
//!
//! ```
//! use measured::{CounterVec, LabelGroup, FixedCardinalityLabel};
//! use measured::metric::name::MetricName;
//! use measured::text::TextEncoder;
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
//! let counters = CounterVec::new(set);
//! // increment the counter at a given label
//! counters.inc(MyLabelGroup { path: "/api/v1/products" });
//! counters.inc(MyLabelGroup { path: "/api/v1/users" });
//!
//! // sample the counters and encode the values to a textual format.
//! let mut text_encoder = TextEncoder::new();
//! let name = MetricName::from_static("my_first_counter");
//! counters.collect_into(name, &mut text_encoder);
//! let bytes = text_encoder.finish();
//! ```
//!
//! In the rare case that the label cannot be determined even at startup, you can still use them. You will have to make use of the
//! [`DynamicLabelSet`](label::DynamicLabelSet) trait. One implementation for string data is provided in the form of [`lasso::ThreadedRodeo`].
//!
//! It's not advised to use this for high cardinality labels, but if you must, this still offers good performance.
//!
//! ```
//! use measured::{CounterVec, LabelGroup, FixedCardinalityLabel};
//! use measured::metric::name::MetricName;
//! use measured::text::TextEncoder;
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
//! let counters = CounterVec::new(set);
//! // increment the counter at a given label
//! counters.inc(MyLabelGroup { path: "/api/v1/products" });
//! counters.inc(MyLabelGroup { path: "/api/v1/users" });
//!
//! // sample the counters and encode the values to a textual format.
//! let mut text_encoder = TextEncoder::new();
//! let name = MetricName::from_static("my_first_counter");
//! counters.collect_into(name, &mut text_encoder);
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
//!     name: SingularField<String>,
//!     value: SingularField<String>,
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

extern crate alloc;

use metric::{counter::CounterState, histogram::HistogramState, Metric, MetricVec};

pub mod label;
pub mod metric;
pub mod text;

pub use measured_derive::{FixedCardinalityLabel, LabelGroup};

/// A [`Metric`] that counts individual observations from an event or sample stream in configurable buckets.
/// Similar to a Summary, it also provides a sum of observations and an observation count.
///
/// ```
/// use measured::Histogram;
/// use measured::metric::histogram::Thresholds;
/// use measured::metric::name::MetricName;
/// use measured::text::TextEncoder;
///
/// // create a histogram with 8 buckets starting at 0.01, increasing by 2x each time up to 2.56
/// let histogram = Histogram::new_metric(Thresholds::<8>::exponential_buckets(0.01, 2.0));
/// // observe a value
/// histogram.get_metric().observe(1.0);
///
/// // sample the histogram and encode the value to a textual format.
/// let mut text_encoder = TextEncoder::new();
/// let name = MetricName::from_static("my_first_histogram");
/// histogram.collect_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type Histogram<const N: usize> = Metric<HistogramState<N>>;

/// A collection of multiple [`Histogram`]s, keyed by [`LabelGroup`](label::LabelGroup)s
///
/// ```
/// use measured::{HistogramVec, LabelGroup, FixedCardinalityLabel};
/// use measured::metric::histogram::Thresholds;
/// use measured::metric::name::MetricName;
/// use measured::text::TextEncoder;
///
/// // Define a fixed cardinality label
///
/// #[derive(FixedCardinalityLabel)]
/// #[label(rename_all = "snake_case")]
/// enum Operation {
///     Create,
///     Update,
///     Delete,
/// }
///
/// // Define a label group, consisting of 1 or more label values
///
/// #[derive(LabelGroup)]
/// #[label(set = MyLabelGroupSet)]
/// struct MyLabelGroup {
///     #[label(fixed)]
///     operation: Operation,
/// }
///
/// // create a histogram vec
/// let histograms = HistogramVec::new_metric_vec(
///     MyLabelGroupSet {},
///     Thresholds::<8>::exponential_buckets(0.01, 2.0),
/// );
/// // observe a value
/// histograms.observe(MyLabelGroup { operation: Operation::Create }, 0.5);
/// histograms.observe(MyLabelGroup { operation: Operation::Delete }, 2.0);
///
/// // sample the histograms and encode the values to a textual format.
/// let mut text_encoder = TextEncoder::new();
/// let name = MetricName::from_static("my_first_histogram");
/// histograms.collect_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type HistogramVec<L, const N: usize> = MetricVec<HistogramState<N>, L>;

/// A [`Metric]` that represents a single numerical value that only ever goes up.
///
/// ```
/// use measured::Counter;
/// use measured::metric::name::MetricName;
/// use measured::text::TextEncoder;
///
/// // create a counter
/// let counter = Counter::new();
/// // increment the counter value
/// counter.get_metric().inc();
///
/// // sample the counter and encode the value to a textual format.
/// let mut text_encoder = TextEncoder::new();
/// let name = MetricName::from_static("my_first_counter");
/// counter.collect_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type Counter = Metric<CounterState>;

/// A collection of multiple [`Counter`]s, keyed by [`LabelGroup`](label::LabelGroup)s
///
/// ```
/// use measured::{CounterVec, LabelGroup, FixedCardinalityLabel};
/// use measured::metric::name::MetricName;
/// use measured::text::TextEncoder;
///
/// // Define a fixed cardinality label
///
/// #[derive(FixedCardinalityLabel)]
/// #[label(rename_all = "snake_case")]
/// enum Operation {
///     Create,
///     Update,
///     Delete,
/// }
///
/// // Define a label group, consisting of 1 or more label values
///
/// #[derive(LabelGroup)]
/// #[label(set = MyLabelGroupSet)]
/// struct MyLabelGroup {
///     #[label(fixed)]
///     operation: Operation,
/// }
///
/// // create a counter vec
/// let counters = CounterVec::new(MyLabelGroupSet {});
/// // increment the counter at a given label
/// counters.inc(MyLabelGroup { operation: Operation::Create });
/// counters.inc(MyLabelGroup { operation: Operation::Delete });
///
/// // sample the counters and encode the values to a textual format.
/// let mut text_encoder = TextEncoder::new();
/// let name = MetricName::from_static("my_first_counter");
/// counters.collect_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type CounterVec<L> = MetricVec<CounterState, L>;
