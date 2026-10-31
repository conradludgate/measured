# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.1](https://github.com/conradludgate/measured/compare/measured-prometheus-protobuf-v0.1.0...measured-prometheus-protobuf-v0.1.1) - 2026-10-31

### Other

- updated the following local packages: measured

## [0.1.0](https://github.com/conradludgate/measured/compare/measured-prometheus-protobuf-v0.0.25...measured-prometheus-protobuf-v0.1.0) - 2026-10-26

### Fixed

- fix histogram validation and metric exposition edge cases

### Other

- replace weak test coverage with focused behavioral checks
- refresh dependencies, remove unused features and pin bytes 1.10.1
- make each crate documentation self-contained and clarify API contracts
- add metric units through non-exhaustive FamilyMeta
- document the 0.1 release with tutorials, guides and migration notes
- use NumBuffer and zmij and require Rust 1.98
- add OpenMetrics, exemplars and metric creation timestamps
- redesign encoding around metric families and sample kinds
- establish nightly rustfmt configuration and format the workspace

### Breaking
- The minimum supported Rust version is now 1.98. Uses `core::fmt::NumBuffer` and `zmij` instead of `itoa` and `ryu`.

- Implements the redesigned `Encoding` trait. See the `measured` changelog.
- `ProtoEncoder::writer` is private. Use `writer()`, `writer_mut()` or `into_inner()`.
- Removed the unused `MetricType` enum.
- Metric families with no samples are dropped.

### Added

- Histograms, summaries, gauge histograms and float counters can be encoded.
- Metric-family unit metadata is encoded in the protobuf `MetricFamily`.
- State sets and info metrics are written as gauges, as protobuf has no type for them.
- Exemplars on counters and histogram buckets, including `+Inf`, and created timestamps on counters, histograms and summaries.

### Fixed

- Untyped and unsupported metric kinds use untyped protobuf sample payloads.
- Histogram exemplars above the largest finite bucket are preserved in an explicit `+Inf` bucket.
- Empty metric families were merged into the next metric family.
- A failed write left stale data in the encoder, which corrupted the next collection.

## [0.0.25](https://github.com/conradludgate/measured/compare/measured-prometheus-protobuf-v0.0.24...measured-prometheus-protobuf-v0.0.25) - 2026-04-14

### Other

- bump versions to 0.0.25
- Merge pull request #13 from ultrabear/expose_lasso
- re-export lasso when enabled so crate users do not have to add it themselves
