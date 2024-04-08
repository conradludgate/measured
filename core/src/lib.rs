//! # Measured. A low-overhead prometheus/metrics crate for measuring your application statistics.
//!
//! Getting started? See [`docs`]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;

use metric::{
    counter::CounterState, gauge::GaugeState, histogram::HistogramState, Metric, MetricVec,
};

#[cfg(any(doc, test))]
pub mod docs;
pub mod label;
pub mod metric;
pub mod text;

/// Implement [`FixedCardinalityLabel`] on an `enum`
///
/// # Container attributes
///
/// * `rename_all = "..."` - rename all variants based on their variant name, supporting:
///     * `"UpperCamelCase"`
///     * `"lowerCamelCase"`
///     * `"snake_case"`
///     * `"kebab-case"`
///     * `"SHOUTY_SNAKE_CASE"`
///     * `"SHOUTY-KEBAB-CASE"`
///     * `"Title Case"`
///     * `"Train-Case"`
/// * `singleton = "..."` - This `FixedCardinalityLabel` on it's own represents a [`LabelGroup`]
///
/// # Variant attributes
///
/// * `rename = "..."` - Rename this variant.
///
/// # Outputs
///
/// * `impl FixedCardinalityLabel for T { ... }`
/// * `impl LabelValue for T { ... }`
/// * `impl LabelGroup for T { ... }`
///     - If `singleton` is specified
///
/// # Example
///
/// ## Basic
///
/// ```
/// #[derive(measured::FixedCardinalityLabel)]
/// #[derive(Debug, Copy, Clone, PartialEq)]
/// enum Operation {
///     Create,
///     Update,
///     Delete,
///     DeleteAll,
/// }
///
/// use measured::label::FixedCardinalityLabel as _;
///
/// assert_eq!(Operation::cardinality(), 4, "Operation has 4 variants");
///
/// assert_eq!(Operation::Create.encode(), 0);
/// assert_eq!(Operation::Update.encode(), 1);
/// assert_eq!(Operation::Delete.encode(), 2);
/// assert_eq!(Operation::DeleteAll.encode(), 3);
///
/// assert_eq!(Operation::decode(0), Operation::Create);
/// assert_eq!(Operation::decode(1), Operation::Update);
/// assert_eq!(Operation::decode(2), Operation::Delete);
/// assert_eq!(Operation::decode(3), Operation::DeleteAll);
///
/// use measured::label::LabelValue as _;
/// use measured::label::LabelTestVisitor;
///
/// assert_eq!(Operation::Create.visit(LabelTestVisitor), "create");
/// assert_eq!(Operation::Update.visit(LabelTestVisitor), "update");
/// assert_eq!(Operation::Delete.visit(LabelTestVisitor), "delete");
/// assert_eq!(Operation::DeleteAll.visit(LabelTestVisitor), "delete_all");
/// ```
///
/// ## Integer values
///
/// ```
/// #[derive(measured::FixedCardinalityLabel)]
/// #[derive(Debug, Copy, Clone, PartialEq)]
/// enum StatusCode {
///     Ok = 200,
///     ImATeapot = 418,
///     InternalServerError = 500,
/// }
///
/// use measured::label::FixedCardinalityLabel as _;
///
/// assert_eq!(StatusCode::cardinality(), 3, "StatusCode has 3 variants");
///
/// assert_eq!(StatusCode::Ok.encode(), 0);
/// assert_eq!(StatusCode::ImATeapot.encode(), 1);
/// assert_eq!(StatusCode::InternalServerError.encode(), 2);
///
/// assert_eq!(StatusCode::decode(0), StatusCode::Ok);
/// assert_eq!(StatusCode::decode(1), StatusCode::ImATeapot);
/// assert_eq!(StatusCode::decode(2), StatusCode::InternalServerError);
///
/// use measured::label::LabelValue as _;
/// use measured::label::LabelTestVisitor;
///
/// assert_eq!(StatusCode::Ok.visit(LabelTestVisitor), "200");
/// assert_eq!(StatusCode::ImATeapot.visit(LabelTestVisitor), "418");
/// assert_eq!(StatusCode::InternalServerError.visit(LabelTestVisitor), "500");
/// ```
///
/// ## Custom values
///
/// ```
/// #[derive(measured::FixedCardinalityLabel)]
/// #[derive(Debug, Copy, Clone, PartialEq)]
/// #[label(rename_all = "SHOUTY-KEBAB-CASE")]
/// enum StatusCode {
///     #[label(rename = "a-okay")]
///     Ok,
///     ImATeapot,
///     InternalServerError,
/// }
///
/// use measured::label::LabelValue as _;
/// use measured::label::LabelTestVisitor;
///
/// assert_eq!(StatusCode::Ok.visit(LabelTestVisitor), "a-okay");
/// assert_eq!(StatusCode::ImATeapot.visit(LabelTestVisitor), "IM-A-TEAPOT");
/// assert_eq!(StatusCode::InternalServerError.visit(LabelTestVisitor), "INTERNAL-SERVER-ERROR");
/// ```
pub use measured_derive::FixedCardinalityLabel;

pub use label::FixedCardinalityLabel;

/// Implement [`LabelGroup`] on a `struct`
///
/// A [`LabelGroup`] is a collection of named [`LabelValue`](label::LabelValue)s. Additonally to the label group,
/// there is also a [`LabelGroupSet`](label::LabelGroupSet) that is created by this macro.
/// The set provides additional information needed to encode the values in the group.
///
/// # Container attributes
///
/// * `set = Ident` - The name that the corresponding [`LabelGroupSet`](label::LabelGroupSet) should take on. (**required**)
///
/// # Field attributes
///
/// * `fixed` - The field type implements [`FixedCardinalityLabel`] (**implied**)
/// * `fixed_with = Type` - The field corresponds to a [`FixedCardinalitySet`](label::FixedCardinalitySet)
/// * `dynamic_with = Type` - The field corresponds to a [`DynamicLabelSet`](label::DynamicLabelSet)
/// * `default` - The generated [`LabelGroupSet`](label::LabelGroupSet) can default this field.
///
/// # Outputs
///
/// * `impl LabelGroup for T { ... }`
/// * `struct TSet { ... }`
/// * `impl LabelGroupSet for TSet { ... }`
/// * `impl TSet { pub fn new(...) -> Self {} }`
///     - `new` contains args for all the non-default fields.
/// * `impl Default for TSet { ... }`
///     - only implemented if all fields are default fields.
///
/// # Example
///
/// ```
/// use lasso::{RodeoReader, ThreadedRodeo};
///
/// #[derive(measured::LabelGroup)]
/// #[label(set = ResponseSet)]
/// struct Response<'a> {
///     kind: StatusCode,
///
///     /// route paths are known up front, and stored in a `RodeoReader`
///     #[label(fixed_with = RodeoReader)]
///     route: &'a str,
///
///     /// user names are not known up-front and are allocated on-demand in a ThreadedRodeo
///     #[label(dynamic_with = ThreadedRodeo, default)]
///     user_name: &'a str,
/// }
///
/// #[derive(measured::FixedCardinalityLabel, Copy, Clone)]
/// enum StatusCode {
///     Ok = 200,
///     BadRequest = 400,
///     InternalServerError = 500,
/// }
///
/// let set = ResponseSet::new(
///     ["/foo/bar", "/home"].into_iter().collect::<lasso::Rodeo>().into_reader(),
/// );
///
/// use measured::label::LabelGroupSet as _;
///
/// let response = Response {
///     kind: StatusCode::InternalServerError,
///     route: "/home",
///     user_name: "conradludgate",
/// };
/// assert_eq!(set.encode(response), Some((5, 0)));
///
/// let response = Response {
///     kind: StatusCode::Ok,
///     route: "/foo/bar",
///     user_name: "conradludgate",
/// };
/// assert_eq!(set.encode(response), Some((0, 0)));
///
/// // the dynamic value `"conradludgate"` was inserted into the set
/// assert_eq!(set.user_name.len(), 1);
/// ```
pub use measured_derive::LabelGroup;

pub use label::group::LabelGroup;

/// Implement [`MetricGroup`] on a `struct`
///
/// A [`MetricGroup`] is a collection of named [`Metric`]s, [`MetricVec`]s, or nested [`MetricGroup`]s.
///
/// # Container attributes
///
/// * `new(..args)` - The arguments the generated `fn new() -> Self` should take. If not provided, no new function is generated.
///
/// # Field attributes
///
/// ## Nested groups
/// These are for fields that also implement [`MetricGroup`]
///
/// * `namespace = "..."` - The field represents a nested group with the given namespace.
/// * `flatten` - The field represents a nested group with no namespacing.
/// * `init` - The expression needed to initialise the nested metric group.
///
/// ## Metrics
/// These are for fields that implement [`MetricFamilyEncoding`](metric::MetricFamilyEncoding)
///
/// * `rename = "..."` - By default, metrics take on the field name in snake case. rename allows renaming them.
/// * `metadata = expr` - The metadata to initialise a [`Metric`] or [`MetricVec`] with.
/// * `label_set = expr` - The [`LabelGroupSet`](label::LabelGroupSet) to initialise a [`MetricVec`] with.
/// * `init = expr` - The expression needed to initialise the metric, if it cannot be defaulted.
///
/// # Outputs
///
/// * `impl MetricGroup for T { ... }`
/// * `impl MetricGroup { pub fn new(...) -> Self { ... } }`
pub use measured_derive::MetricGroup;

pub use metric::group::MetricGroup;

/// A [`Metric`] that counts individual observations from an event or sample stream in configurable buckets.
/// Similar to a Summary, it also provides a sum of observations and an observation count.
///
/// ```
/// use measured::Histogram;
/// use measured::metric::histogram::Thresholds;
/// use measured::metric::name::MetricName;
/// use measured::metric::MetricFamilyEncoding;
/// use measured::text::BufferedTextEncoder;
///
/// // create a histogram with 8 buckets starting at 0.01, increasing by 2x each time up to 2.56
/// let histogram = Histogram::with_metadata(Thresholds::<8>::exponential_buckets(0.01, 2.0));
/// // observe a value
/// histogram.get_metric().observe(1.0);
///
/// // sample the histogram and encode the value to a textual format.
/// let mut text_encoder = BufferedTextEncoder::new();
/// let name = MetricName::from_str("my_first_histogram");
/// histogram.collect_family_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type Histogram<const N: usize> = Metric<HistogramState<N>>;

/// A collection of multiple [`Histogram`]s, keyed by [`LabelGroup`]s
///
/// ```
/// use measured::{HistogramVec, LabelGroup, FixedCardinalityLabel};
/// use measured::label::StaticLabelSet;
/// use measured::metric::histogram::Thresholds;
/// use measured::metric::name::MetricName;
/// use measured::metric::MetricFamilyEncoding;
/// use measured::text::BufferedTextEncoder;
///
/// // Define a fixed cardinality label
///
/// #[derive(FixedCardinalityLabel, Copy, Clone)]
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
///     operation: Operation,
/// }
///
/// // create a histogram vec
/// let histograms = HistogramVec::with_label_set_and_metadata(
///     MyLabelGroupSet::new(),
///     Thresholds::<8>::exponential_buckets(0.01, 2.0),
/// );
/// // observe a value
/// histograms.observe(MyLabelGroup { operation: Operation::Create }, 0.5);
/// histograms.observe(MyLabelGroup { operation: Operation::Delete }, 2.0);
///
/// // sample the histograms and encode the values to a textual format.
/// let mut text_encoder = BufferedTextEncoder::new();
/// let name = MetricName::from_str("my_first_histogram");
/// histograms.collect_family_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type HistogramVec<L, const N: usize> = MetricVec<HistogramState<N>, L>;

/// A [`Metric`] that represents a single numerical value that only ever goes up.
///
/// ```
/// use measured::Counter;
/// use measured::metric::name::MetricName;
/// use measured::metric::MetricFamilyEncoding;
/// use measured::text::BufferedTextEncoder;
///
/// // create a counter
/// let counter = Counter::new();
/// // increment the counter value
/// counter.inc();
///
/// // sample the counter and encode the value to a textual format.
/// let mut text_encoder = BufferedTextEncoder::new();
/// let name = MetricName::from_str("my_first_counter");
/// counter.collect_family_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type Counter = Metric<CounterState>;

/// A collection of multiple [`Counter`]s, keyed by [`LabelGroup`]s
///
/// ```
/// use measured::{CounterVec, LabelGroup, FixedCardinalityLabel};
/// use measured::label::StaticLabelSet;
/// use measured::metric::name::MetricName;
/// use measured::metric::MetricFamilyEncoding;
/// use measured::text::BufferedTextEncoder;
///
/// // Define a fixed cardinality label
///
/// #[derive(FixedCardinalityLabel, Copy, Clone)]
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
///     operation: Operation,
/// }
///
/// // create a counter vec
/// let counters = CounterVec::with_label_set(MyLabelGroupSet::new());
/// // increment the counter at a given label
/// counters.inc(MyLabelGroup { operation: Operation::Create });
/// counters.inc(MyLabelGroup { operation: Operation::Delete });
///
/// // sample the counters and encode the values to a textual format.
/// let mut text_encoder = BufferedTextEncoder::new();
/// let name = MetricName::from_str("my_first_counter");
/// counters.collect_family_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type CounterVec<L> = MetricVec<CounterState, L>;

/// A [`Metric`] that represents a single numerical value that can go up or down over time.
///
/// ```
/// use measured::Gauge;
/// use measured::metric::name::MetricName;
/// use measured::metric::MetricFamilyEncoding;
/// use measured::text::BufferedTextEncoder;
///
/// // create a gauge
/// let gauge = Gauge::new();
/// // increment the gauge value
/// gauge.get_metric().inc();
///
/// // sample the gauge and encode the value to a textual format.
/// let mut text_encoder = BufferedTextEncoder::new();
/// let name = MetricName::from_str("my_first_gauge");
/// gauge.collect_family_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type Gauge = Metric<GaugeState>;

/// A collection of multiple [`Gauge`]s, keyed by [`LabelGroup`]s
///
/// ```
/// use measured::{GaugeVec, LabelGroup, FixedCardinalityLabel};
/// use measured::label::StaticLabelSet;
/// use measured::metric::name::MetricName;
/// use measured::metric::MetricFamilyEncoding;
/// use measured::text::BufferedTextEncoder;
///
/// // Define a fixed cardinality label
///
/// #[derive(FixedCardinalityLabel, Copy, Clone)]
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
///     operation: Operation,
/// }
///
/// // create a gauge vec
/// let gauges = GaugeVec::with_label_set(MyLabelGroupSet::new());
/// // increment the gauge at a given label
/// gauges.inc(MyLabelGroup { operation: Operation::Create });
/// gauges.inc(MyLabelGroup { operation: Operation::Delete });
///
/// // sample the gauges and encode the values to a textual format.
/// let mut text_encoder = BufferedTextEncoder::new();
/// let name = MetricName::from_str("my_first_gauge");
/// gauges.collect_family_into(name, &mut text_encoder);
/// let bytes = text_encoder.finish();
/// ```
pub type GaugeVec<L> = MetricVec<GaugeState, L>;
