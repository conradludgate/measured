//! Record and export request metrics.
//!
//! In this tutorial you will count requests by outcome and export a Prometheus text response.
//! You need Rust 1.98 or newer. No optional measured features are needed.
//!
//! # 1. Create a project
//!
//! Run `cargo new measured-demo` and open the new directory. Add these dependencies to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! measured = "0.1"
//! measured-prometheus-text = "0.1"
//! ```
//!
//! # 2. Record two requests
//!
//! Replace `src/main.rs` with the following program. `Outcome` defines the two allowed label values.
//! `RequestLabels` gives the label its name, and `RequestLabelSet` is generated for the counter vector.
//! The field's doc comment becomes the exported help text.
//!
//! ```
//! use measured::{CounterVec, FixedCardinalityLabel, LabelGroup, MetricGroup};
//! use measured_prometheus_text::BufferedTextEncoder;
//!
//! #[derive(FixedCardinalityLabel, Clone, Copy)]
//! enum Outcome {
//!     Success,
//!     Failure,
//! }
//!
//! #[derive(LabelGroup)]
//! #[label(set = RequestLabelSet)]
//! struct RequestLabels {
//!     outcome: Outcome,
//! }
//!
//! #[derive(MetricGroup, Default)]
//! struct Metrics {
//!     /// Number of completed requests.
//!     requests_total: CounterVec<RequestLabelSet>,
//! }
//!
//! fn main() {
//!     let metrics = Metrics::default();
//!     metrics
//!         .requests_total
//!         .inc(RequestLabels { outcome: Outcome::Success });
//!     metrics
//!         .requests_total
//!         .inc(RequestLabels { outcome: Outcome::Failure });
//!
//!     let mut encoder = BufferedTextEncoder::new();
//!     metrics.collect_group_into(&mut encoder).unwrap();
//!     let body = encoder.finish();
//!     let text = std::str::from_utf8(&body).unwrap();
//!     print!("{text}");
//!
//!     assert_eq!(
//!         text,
//!         concat!(
//!             "# HELP requests_total Number of completed requests.\n",
//!             "# TYPE requests_total counter\n",
//!             "requests_total{outcome=\"success\"} 1\n",
//!             "requests_total{outcome=\"failure\"} 1\n",
//!         )
//!     );
//! }
//! ```
//!
//! # 3. Inspect the output
//!
//! Run `cargo run`. You should see a HELP line, a TYPE line, and two request counts, each equal to 1.
//! Add another increment with `Outcome::Success` and run the program again. Its count becomes 2;
//! the failure count stays at 1. Update the assertion's expected success count to match.
//!
//! You now have a metric group that records labelled events and encodes a complete scrape body.
//! In a server, keep the group alive across requests; constructing a new group resets the counters.
//!
//! Next, [export a scrape response](super::how_to#export-a-scrape-response) or
//! [record a duration](super::how_to#record-a-duration). For why label sets exist, read the
//! [explanation](super::explanation).
