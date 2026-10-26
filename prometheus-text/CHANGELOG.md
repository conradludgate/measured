# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking

- The minimum supported Rust version is now 1.98. Uses `core::fmt::NumBuffer` and `zmij` instead of `itoa` and `ryu`.

The prometheus text encoder, moved out of `measured::text`. See the `measured` changelog for the changes made in the move.

### Added

- `OpenMetricsEncoder` and `BufferedOpenMetricsEncoder`, for the OpenMetrics text format. These support every `MetricKind`, exemplars, and created timestamps (as `_created` samples). Exemplars with labels longer than the OpenMetrics limit of 128 characters are not written.
- OpenMetrics output includes metric-family units declared with `#[metric(unit = "...")]`.

### Changed

- The prometheus text format does not support exemplars or created timestamps, so they are not written.
- The prometheus text format has no gauge histograms, state sets or info metrics. Gauge histograms are written as histograms, and state sets and info metrics are written as gauges.
