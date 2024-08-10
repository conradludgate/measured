//! Why this crate exists, and how it works under the hood.
//!
//! This crate was born out of a desire for better ergonomics dealing with prometheus,
//! with the added extra goal of minimizing small allocations to reduce memory fragmentation.
//!
//! # Implementation details
//!
//! This crate makes several compromises in pursuit of performance.
//!
//! ## Minimal allocations
//!
//! Most other prometheus/metrics crates make use of many `String` and `Arc` allocations for labels and metrics respectively.
//! These all end up being small allocations and can make the allocator struggle, as well as cause a lot more cache misses.
//!
//! `measured` recommends you store string-based labels in large allocations based on [`lasso`], and all metrics are stored back-to-back
//! in the metric vec, providing much better cache locality. See <#prometheus-vs-memory-fragmentation> for a breakdown.
//!
//! ## Strongly typed labels
//!
//! A common paper-cut with `prometheus`/`metrics` crates is that you can provide the incorrect set of labels because they are based loosely on `Vec<(String, String)>`.
//!
//! In `prometheus`, the most performant mechanism for accessing a metric is using `vec.with_label_values(&["value1", "value2"])`.
//! It's easy to accidentally swap the order of the labels, or forget to include a label when you update the definition.
//!
//! In `metrics`, the only way to use a metric with a label is via `counter!("counter_name", "label1" => "value1")`. If you need to manipulate
//! that metric in multiple places, there is no 'source of truth' for what labels should be set.
//!
//! In `measured`, all metric vecs are typed to include their `LabelGroup` type. This consistently defines _all_ labels that are needed, and it is
//! impossible thanks to Rust to forget to init a field, or to init them in the wrong order.
//!
//! ## Labels as integers
//!
//! Most prometheus/metrics crates back metric vecs with a HashMap from labels to the individual metric value.
//! They all try to hash the list of label strings in such a way to reduce overhead of the hashmap.
//!
//! `measured` does not need any hashing step and instead encodes label values directly to integers in a one-to-one mapping.
//!
//! ```
//! use measured::FixedCardinalityLabel;
//! #[derive(FixedCardinalityLabel, Copy, Clone, Debug, PartialEq)]
//! enum Value {
//!     Success,
//!     Failure,
//! }
//!
//! assert_eq!(Value::Success.encode(), 0);
//! assert_eq!(Value::Failure.encode(), 1);
//!
//! assert_eq!(Value::decode(0), Value::Success);
//! assert_eq!(Value::decode(1), Value::Failure);
//! ```
//!
//! When possible, `measured` uses a `Vec` to store all metrics, falling back to a sharded `HashMap` if the total cardinality is too large or unknown.
//! Because we are hashing integers, `FxHash` can work fast and efficiently to provide very quick hashmap operations.
//!
//! # Prior art
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
//! So, if we have a counter vec with 3 different labels, and a total of 24 unique label groups, then we will have
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
