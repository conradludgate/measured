//! Quick examples to get you started
//!
//! ## Basic Usage
//!
//! The most basic usage is defining a single counter. This is very easy.
//!
//! ```
//! use measured::{Counter, MetricGroup};
//! use measured::metric::name::MetricName;
//! use measured::metric::MetricFamilyEncoding;
//! use measured::text::BufferedTextEncoder;
//!
//! // Define a metric group, consisting of 1 or more metrics
//! #[derive(MetricGroup)]
//! #[metric(new())]
//! struct MyMetricGroup {
//!     /// counts things
//!     my_first_counter: Counter,
//! }
//!
//! // create the metrics
//! let metrics = MyMetricGroup::new();
//!
//! // increment the counter value
//! metrics.my_first_counter.inc();
//!
//! // sample the metrics and encode the values to a textual format.
//! let mut text_encoder = BufferedTextEncoder::new();
//! metrics.collect_group_into(&mut text_encoder);
//! let bytes = text_encoder.finish();
//!
//! assert_eq!(
//!     bytes,
//!     r#"# HELP my_first_counter counts things
//! ## TYPE my_first_counter counter
//! my_first_counter 1
//! "#);
//! ```
//!
//! ## With labels
//!
//! It's common to have labels added to your metrics, such as adding an operation type. When all possible values
//! can be determined at compile time, you can define the label value as a [`FixedCardinalityLabel`](crate::FixedCardinalityLabel) enum.
//!
//! Multiple label pairs are collected into a [`LabelGroup`](crate::LabelGroup).
//!
//! ```
//! use measured::{CounterVec, LabelGroup, MetricGroup, FixedCardinalityLabel};
//! use measured::label::StaticLabelSet;
//! use measured::metric::name::MetricName;
//! use measured::metric::MetricFamilyEncoding;
//! use measured::text::BufferedTextEncoder;
//!
//! // Define a fixed cardinality label
//!
//! #[derive(FixedCardinalityLabel, Copy, Clone)]
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
//!     operation: Operation,
//! }
//!
//! // Define a metric group, consisting of 1 or more metrics
//! #[derive(MetricGroup)]
//! #[metric(new())]
//! struct MyMetricGroup {
//!     /// counts things
//!     my_first_counter: CounterVec<MyLabelGroupSet>,
//! }
//!
//! // create the metrics
//! let metrics = MyMetricGroup::new();
//!
//! // increment the counter at a given label
//! metrics.my_first_counter.inc(MyLabelGroup { operation: Operation::Create });
//! metrics.my_first_counter.inc(MyLabelGroup { operation: Operation::Delete });
//!
//! // sample the metrics and encode the values to a textual format.
//! let mut text_encoder = BufferedTextEncoder::new();
//! metrics.collect_group_into(&mut text_encoder);
//! let bytes = text_encoder.finish();
//!
//! assert_eq!(
//!     bytes,
//!     r#"# HELP my_first_counter counts things
//! ## TYPE my_first_counter counter
//! my_first_counter{operation="create"} 1
//! my_first_counter{operation="delete"} 1
//! "#);
//! ```
//!
//! ## With dynamic labels and label sets
//!
//! Sometimes, the labels cannot be determined at compile time, but they can be determine at the start of the program.
//! This might be the paths of a RESTful API. For efficiency,
//! `measured` offers a trait called [`FixedCardinalitySet`](crate::label::FixedCardinalitySet) that allows for compact encoding.
//!
//! Implementations of [`FixedCardinalitySet`](crate::label::FixedCardinalitySet) are provided for you,
//! notably [`phf::OrderedSet`], [`indexmap::IndexSet`], and [`lasso::RodeoReader`].
//! I recommend the latter for string-based labels that are not `&'static` as it will offer the most efficient use of memory.
//!
//! ```
//! use measured::{CounterVec, LabelGroup, MetricGroup};
//! use measured::metric::name::MetricName;
//! use measured::metric::MetricFamilyEncoding;
//! use measured::text::BufferedTextEncoder;
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
//! // Define a metric group, consisting of 1 or more metrics
//! #[derive(MetricGroup)]
//! #[metric(new(path: lasso::RodeoReader))]
//! struct MyMetricGroup {
//!     /// counts things
//!     #[metric(label_set = MyLabelGroupSet::new(path))]
//!     my_first_counter: CounterVec<MyLabelGroupSet>,
//! }
//!
//! // create the metrics
//! let paths = lasso::Rodeo::from_iter([
//!     "/api/v1/products",
//!     "/api/v1/users",
//! ])
//! .into_reader();
//! let metrics = MyMetricGroup::new(paths);
//!
//! // increment the counter at a given label
//! metrics.my_first_counter.inc(MyLabelGroup { path: "/api/v1/products" });
//! metrics.my_first_counter.inc(MyLabelGroup { path: "/api/v1/users" });
//!
//! // sample the metrics and encode the values to a textual format.
//! let mut text_encoder = BufferedTextEncoder::new();
//! metrics.collect_group_into(&mut text_encoder);
//! let bytes = text_encoder.finish();
//!
//! assert_eq!(
//!     bytes,
//!     r#"# HELP my_first_counter counts things
//! ## TYPE my_first_counter counter
//! my_first_counter{path="/api/v1/products"} 1
//! my_first_counter{path="/api/v1/users"} 1
//! "#);
//! ```
//!
//! In the rare case that the label cannot be determined even at startup, you can still use them. You will have to make use of the
//! [`DynamicLabelSet`](crate::label::DynamicLabelSet) trait. One implementation for string data is provided in the form of [`lasso::ThreadedRodeo`].
//!
//! It's not advised to use this for high cardinality labels, but if you must, this still offers good performance.
//!
//! ```
//! use measured::{CounterVec, LabelGroup, MetricGroup};
//! use measured::metric::name::MetricName;
//! use measured::metric::MetricFamilyEncoding;
//! use measured::text::BufferedTextEncoder;
//!
//! // Define a label group, consisting of 1 or more label values
//!
//! #[derive(LabelGroup)]
//! #[label(set = MyLabelGroupSet)]
//! struct MyLabelGroup<'a> {
//!     #[label(dynamic_with = lasso::ThreadedRodeo, default)]
//!     path: &'a str,
//! }
//!
//! // Define a metric group, consisting of 1 or more metrics
//! #[derive(MetricGroup)]
//! #[metric(new())]
//! struct MyMetricGroup {
//!     /// counts things
//!     #[metric(label_set = MyLabelGroupSet::new())]
//!     my_first_counter: CounterVec<MyLabelGroupSet>,
//! }
//!
//! // create the metrics
//! let metrics = MyMetricGroup::new();
//!
//! // increment the counter at a given label
//! metrics.my_first_counter.inc(MyLabelGroup { path: "/api/v1/products" });
//! metrics.my_first_counter.inc(MyLabelGroup { path: "/api/v1/users" });
//!
//! // sample the metrics and encode the values to a textual format.
//! let mut text_encoder = BufferedTextEncoder::new();
//! metrics.collect_group_into(&mut text_encoder);
//! let bytes = text_encoder.finish();
//!
//! assert_eq!(
//!     bytes,
//!     r#"# HELP my_first_counter counts things
//! ## TYPE my_first_counter counter
//! my_first_counter{path="/api/v1/products"} 1
//! my_first_counter{path="/api/v1/users"} 1
//! "#);
//! ```
