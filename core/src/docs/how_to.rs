//! Complete common instrumentation tasks.
//!
//! These recipes assume you can already define and collect a metric group. Start with the
//! [tutorial](super::tutorial) if you need a complete first example.
//!
//! # Export a scrape response
//!
//! Add `measured-prometheus-text = "0.1"`. Collect your group into a `BufferedTextEncoder`,
//! call `finish()` once, and use its bytes as the HTTP response body. Set `Content-Type` to
//! the encoder's `Encoding::MIME_TYPE`. Reuse the encoder between scrapes to reuse its allocation.
//!
//! ```
//! use measured::{Counter, MetricGroup, metric::group::Encoding};
//! use measured_prometheus_text::BufferedTextEncoder;
//! #[derive(MetricGroup, Default)]
//! struct Metrics { requests_total: Counter }
//! let metrics = Metrics::default();
//! let mut encoder = BufferedTextEncoder::new();
//! metrics.collect_group_into(&mut encoder).unwrap();
//! let body = encoder.finish();
//! let content_type = BufferedTextEncoder::MIME_TYPE;
//! assert_eq!(content_type, "text/plain; version=0.0.4");
//! assert!(!body.is_empty());
//! ```
//!
//! If several handlers share an encoder, lock it for the entire collection and `finish()` operation.
//! The repository's `examples/axum-server` shows an HTTP integration.
//!
//! # Export OpenMetrics
//!
//! Use `BufferedOpenMetricsEncoder` from the same crate when the consumer accepts OpenMetrics.
//! Use its MIME type in the response. `finish()` writes the required `# EOF` terminator, so call it
//! exactly once per collection. Writer-based encoders use `flush()` instead.
//!
//! ```
//! use measured::{Counter, MetricGroup, metric::group::Encoding};
//! use measured_prometheus_text::BufferedOpenMetricsEncoder;
//! #[derive(MetricGroup, Default)]
//! struct Metrics { requests_total: Counter }
//! let metrics = Metrics::default();
//! let mut encoder = BufferedOpenMetricsEncoder::new();
//! metrics.collect_group_into(&mut encoder).unwrap();
//! let body = encoder.finish();
//! assert!(body.ends_with(b"# EOF\n"));
//! assert!(BufferedOpenMetricsEncoder::MIME_TYPE.starts_with("application/openmetrics-text"));
//! ```
//!
//! # Record a duration
//!
//! Choose histogram bounds in seconds and pass a `Duration` to `observe_duration`.
//! Keep the bounds stable across instances whose histograms you intend to aggregate.
//!
//! ```
//! use measured::{Histogram, metric::histogram::Thresholds};
//! use std::time::Duration;
//! let latency = Histogram::with_metadata(Thresholds::with_buckets([0.01, 0.1, 1.0]));
//! latency.observe_duration(Duration::from_millis(50));
//! let sample = latency.sample();
//! assert_eq!(sample.buckets, [0, 1, 0]);
//! assert_eq!(sample.count(), 1);
//! ```
//!
//! # Use labels discovered at runtime
//!
//! Enable `measured = { version = "0.1", features = ["lasso"] }`.
//! If all values are known at startup, build a `lasso::RodeoReader` and use `label(fixed_with = ...)`.
//!
//! ```
//! # #[cfg(feature = "lasso")]
//! # {
//! use measured::{CounterVec, LabelGroup, lasso::{Rodeo, RodeoReader}};
//! #[derive(LabelGroup)]
//! #[label(set = RouteSet)]
//! struct Route<'a> {
//!     #[label(fixed_with = RodeoReader)]
//!     route: &'a str,
//! }
//! let routes = Rodeo::from_iter(["/users", "/products"]).into_reader();
//! let requests = CounterVec::with_label_set(RouteSet::new(routes));
//! requests.inc(Route { route: "/users" });
//! assert_eq!(requests.get(Route { route: "/users" }), 1);
//! assert!(requests.try_with_labels(Route { route: "/unknown" }).is_none());
//! # }
//! ```
//!
//! If values arrive later, use `lasso::ThreadedRodeo` with `label(dynamic_with = ...)`:
//!
//! ```
//! # #[cfg(feature = "lasso")]
//! # {
//! use measured::{CounterVec, LabelGroup, lasso::ThreadedRodeo};
//! #[derive(LabelGroup)]
//! #[label(set = RouteSet)]
//! struct Route<'a> {
//!     #[label(dynamic_with = ThreadedRodeo, default)]
//!     route: &'a str,
//! }
//! let requests = CounterVec::<RouteSet>::new();
//! requests.inc(Route { route: "/users/:id" });
//! assert_eq!(requests.get(Route { route: "/users/:id" }), 1);
//! assert_eq!(requests.get(Route { route: "/unknown" }), 0);
//! assert!(requests.find_labels(Route { route: "/unknown" }).is_none());
//! # }
//! ```
//!
//! Use route templates such as `/users/:id`; recording each user's full URL creates a series per value.
//! Reading with `get`, `sample`, or `find_labels` does not intern new values in the built-in dynamic sets.
//! Custom sets that insert values in `encode` must implement a non-inserting `find`.
//!
//! # Remove a series
//!
//! Construct a sparse vector if removal is required; dense vectors do not support removal.
//! Look up the ID with `find_labels` so removal cannot insert new labels.
//!
//! ```
//! use measured::{CounterVec, FixedCardinalityLabel, label::StaticLabelSet};
//! #[derive(FixedCardinalityLabel, Clone, Copy)]
//! #[label(singleton = "worker")]
//! enum Worker { Background }
//! let requests = CounterVec::<StaticLabelSet<Worker>>::sparse();
//! requests.inc(Worker::Background);
//! if let Some(id) = requests.find_labels(Worker::Background) {
//!     assert!(requests.remove_metric(id).is_some());
//! }
//! assert_eq!(requests.get(Worker::Background), 0);
//! ```
//!
//! Drop any metric guards before removal. A sparse vector holds shard locks during access;
//! see [`MetricLockGuard`](crate::metric::MetricLockGuard) for the deadlock contract.
//!
//! # Add trace information or creation timestamps
//!
//! Use [`ExemplarCounter`](crate::ExemplarCounter) or [`ExemplarHistogram`](crate::ExemplarHistogram)
//! with [`Exemplar`](crate::Exemplar) to attach labels such as a trace ID to an observation.
//! Choose [`CreatedCounter`](crate::CreatedCounter), [`CreatedFloatCounter`](crate::CreatedFloatCounter),
//! or [`CreatedHistogram`](crate::CreatedHistogram) when consumers need series creation timestamps.
//! Their API pages contain recording examples. OpenMetrics and protobuf preserve these extras;
//! classic Prometheus text omits them.
