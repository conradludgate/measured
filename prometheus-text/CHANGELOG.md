# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/conradludgate/measured/compare/measured-prometheus-text-v0.1.0...measured-prometheus-text-v0.1.1) - 2026-10-31

### Other

- updated the following local packages: measured

## [0.1.0](https://github.com/conradludgate/measured/releases/tag/measured-prometheus-text-v0.1.0) - 2026-10-26

### Other

- replace weak test coverage with focused behavioral checks
- make each crate documentation self-contained and clarify API contracts
- add metric units through non-exhaustive FamilyMeta
- document the 0.1 release with tutorials, guides and migration notes
- use NumBuffer and zmij and require Rust 1.98
- add OpenMetrics, exemplars and metric creation timestamps
- add float counters, callbacks, info, state sets and gauge histograms
- redesign encoding around metric families and sample kinds
- extract the Prometheus text encoder into its own crate

### Breaking

- The minimum supported Rust version is now 1.98. Uses `core::fmt::NumBuffer` and `zmij` instead of `itoa` and `ryu`.

The prometheus text encoder, moved out of `measured::text`. See the `measured` changelog for the changes made in the move.

### Added

- `OpenMetricsEncoder` and `BufferedOpenMetricsEncoder`, for the OpenMetrics text format. These support the built-in metric kinds, exemplars, and created timestamps (as `_created` samples). Untyped and custom kinds are written as `unknown`. Exemplars with labels longer than the OpenMetrics limit of 128 characters are not written.
- OpenMetrics output includes metric-family units declared with `#[metric(unit = "...")]`.

### Changed

- The prometheus text format does not support exemplars or created timestamps, so they are not written.
- The prometheus text format has no gauge histograms, state sets or info metrics. Gauge histograms are written as histograms, and state sets and info metrics are written as gauges.
