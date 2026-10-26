# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Metric families can expose optional unit metadata through derived `#[metric(unit = "...")]` fields.
- Added non-exhaustive `FamilyMeta` for family help and unit metadata, so future family metadata can be added without changing encoder method parameters. Its public fields can be inspected, and builder methods set their values.

### Breaking

- The minimum supported Rust version is now 1.98, for `core::fmt::NumBuffer`.
- Replaced the `itoa` dependency with `core::fmt::NumBuffer`, and `ryu` with `zmij`. Floats of 1e16 or more are now formatted with an explicit exponent sign, eg `1e+16` instead of `1e16`. This changes the value of `le` labels for histogram buckets of 1e16 or more.
- The encoding traits are redesigned, so each encoder and each metric type is implemented once:
  - `Encoding` has `write_family(name, kind, meta)` followed by `write_counter`, `write_gauge`, `write_float_gauge` and `write_histogram`. `FamilyMeta::new().with_help(...).with_unit(...)` supplies optional family metadata. It replaces `Encoding::write_help` and `MetricEncoding::write_type`.
  - `MetricEncoding` is no longer generic over the encoder. It has a `KIND` const and a generic `collect_into<E: Encoding>` method.
  - `MetricFamilyEncoding` is no longer generic over the encoder, and `collect_family_into` takes metadata: `collect_family_into(name, meta, enc)`.
  - Generic code no longer needs bounds like `CounterState: MetricEncoding<Enc>`. `E: Encoding` is enough.
  - `write_counter`, `write_float_counter` and `write_summary` take `SampleExtras`, and `write_histogram` takes `HistogramExtras`. These carry created timestamps and exemplars. Pass `SampleExtras::NONE` or `HistogramExtras::NONE` if there are none.
  - The `write_counter`, `write_gauge` and `write_float_gauge` free functions are removed. Call the methods on the encoder instead, after `write_family`.
  - `MetricKind` replaces the text encoder's `MetricType` enum, which is now private. External metric implementations can use the built-in kinds, including `Summary`, or `Custom(&'static str)` for encoder-specific types. The bundled encoders fall back to `untyped` in classic text and protobuf, or `unknown` in OpenMetrics, for custom kinds.
  - `write_histogram`, `write_summary`, `write_gauge_histogram` and `write_state_set` have default implementations in terms of the counter and gauge methods, so an encoder only needs to implement `write_family`, `write_counter`, `write_float_counter`, `write_gauge` and `write_float_gauge`.
- The prometheus text encoder moved to the new `measured-prometheus-text` crate. Replace `measured::text::` with `measured_prometheus_text::`.
  - `TextEncoder::new(writer)` is now `TextEncoder::from_writer(writer)`. `BufferedTextEncoder::new()` is unchanged.
  - `BufferedTextEncoder` is now a type alias for `TextEncoder<BytesWriter>`, and its error type is `std::io::Error` instead of `Infallible`. A single `MetricEncoding` implementation works with both buffered and writer-based encoders.
  - `MetricValue` is no longer public.
- `Encoding::MIME_TYPE` is a required associated const (added in 0.0.25, but not noted then). Encoder wrappers should forward it, eg `const MIME_TYPE: &'static str = T::MIME_TYPE;`.
- Metric internals are private. Use the new accessors instead:
  - `CounterState`, `GaugeState` and `FloatGaugeState` no longer expose `count`. Use `get()`.
  - `HistogramState::inner` and `HistogramStateInner` are private. Use `HistogramState::sample()`, which returns a `HistogramSample`.
  - `AtomicF64` is private.
  - `TextEncoder::writer` is private. Use `writer()`, `writer_mut()` or `into_inner()`.
- `MetricName::try_from_str` rejects `-`, matching `MetricName::from_str`.
- `Thresholds::with_buckets`, `linear_buckets` and `exponential_buckets` panic if the buckets contain NaN or +Inf, or are not strictly increasing.
- The `*_mut` counter and gauge methods wrap on overflow instead of panicking in debug builds, matching the atomic methods.
- Derive macros:
  - `LabelGroup` set constructors take arguments in field declaration order. Previously `dynamic_with` fields came first.
  - `FixedCardinalityLabel` rejects duplicate label values and enums with no variants.
  - `#[metric(init)]`, `#[metric(metadata)]` and `#[metric(label_set)]` require `#[metric(new(...))]` on the struct, instead of being silently ignored.
  - `LabelGroup` rejects duplicate label names, including renames.
- `InvalidMetricName` is `#[non_exhaustive]`.
- `metric::LabelIdInner` is private.

### Added

- `get()` on `Counter`, `Gauge` and `FloatGauge`, and `get(labels)` on `CounterVec`, `GaugeVec` and `FloatGaugeVec`.
- `Histogram::sample()` and `HistogramVec::sample(labels)`.
- `MetricVec::find_metric(id)`, which looks up a metric without creating it.
- `MetricVec::find_labels(labels)`, `LabelGroupSet::find` and `LabelSet::find`, which look up labels without adding new values to dynamic label sets like `ThreadedRodeo`. Label sets that insert values should override `find`.
- The new `get(labels)` and `sample(labels)` methods use `find_labels` and `find_metric`, so reading a metric does not create it or grow its label sets. They return 0, or an empty sample, for labels that are not in the label set.
- `MetricVec::supports_removal()`.
- `MetricVec::try_for_each`, which visits every metric in a vec with its labels.
- `MetricVec::project`, which encodes part of each metric in a vec as its own metric family, eg the two halves of a pair of counters.
- `CallbackMetric`, with `CallbackGauge`, `CallbackFloatGauge` and `CallbackCounter`, for metrics computed when they are collected.
- `InfoMetric`, an info metric with replaceable labels (`set_labels` and `get_labels`).
- `FloatCounter`, `FloatCounterVec` and `CallbackFloatCounter`, for counters of non-integer values like seconds. Negative and NaN increments are ignored, and panic in debug builds.
- `GaugeHistogram` and `GaugeHistogramVec`, for distributions of values that can be removed, eg the ages of queued items.
- `StateSet`, which exposes which state a `FixedCardinalityLabel` enum is in.
- `MetricKind::GaugeHistogram`, `MetricKind::StateSet` and `MetricKind::Info`, from OpenMetrics.
- Exemplars: `Exemplar` (also exported from the crate root), and `ExemplarCounter`, `ExemplarCounterVec`, `ExemplarHistogram` and `ExemplarHistogramVec`, which record the most recent exemplar per series, or per bucket.
- Created timestamps: `Created<M>`, and `CreatedCounter`, `CreatedFloatCounter` and `CreatedHistogram` (and their `Vec`s), which record when each series was created.
- `#[derive(LabelGroup)]` no longer requires `#[label(set = ...)]`.
- `LabelValue` impls for integers, `f64`, `bool` and `Cow<str>`.
- `Range<usize>` is a `FixedCardinalitySet`, eg for worker IDs.
- `Histogram::observe_duration` and `Histogram::observe_duration_since`.
- `*_mut` methods on `Gauge`, `GaugeVec`, `FloatGauge` and `FloatGaugeVec`.
- `Debug`, `Clone` and `Copy` impls on `NoLabels`, `StaticLabelSet` and the metric name suffixes, and `Debug`, `Clone` and `PartialEq` on `Thresholds`.

### Fixed

- NaN histogram observations are counted only in the +Inf bucket.
- Text encoding escapes `\` and newlines in HELP text, writes infinite floats as `+Inf` and `-Inf`, and writes counters above `i64::MAX` correctly.
- Generated derive code uses fully qualified paths, so it compiles when `Result` or `Option` are shadowed.
- `#[doc(hidden)]` and non-literal doc attributes are accepted on `MetricGroup` fields.
- Metric and label renames that differ only in case no longer collide, and invalid renames no longer panic the derive macro.
- The derive macros accept raw identifiers, eg `r#type`, and use the name without `r#`.

## [0.0.25](https://github.com/conradludgate/measured/compare/measured-v0.0.24...measured-v0.0.25) - 2026-04-14

### Other

- bump versions to 0.0.25
- Merge pull request #13 from ultrabear/expose_lasso
- re-export lasso when enabled so crate users do not have to add it themselves
- enable auto_cfg
- make measured docs build again
