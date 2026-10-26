# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/conradludgate/measured/compare/measured-process-v0.0.25...measured-process-v0.1.0) - 2026-10-26

### Other

- replace weak test coverage with focused behavioral checks
- make each crate documentation self-contained and clarify API contracts
- add metric units through non-exhaustive FamilyMeta
- export process I/O, memory limits and page faults
- document the 0.1 release with tutorials, guides and migration notes
- use NumBuffer and zmij and require Rust 1.98
- add Tokio schedule latency metrics and fix process collection
- redesign encoding around metric families and sample kinds
- establish nightly rustfmt configuration and format the workspace

### Added

- Export the soft virtual-memory limit, process storage I/O, and minor/major page faults.

### Breaking
- The minimum supported Rust version is now 1.98.

- `cpu_seconds_total` is a counter, reported with fractional seconds instead of being a gauge truncated to whole seconds.

### Changed

- The `MetricGroup` impl works with any `Encoding`, with no extra bounds.

### Fixed

- `# TYPE` lines are written.
- `ProcessCollector::new(pid)` reports the start time of `pid` instead of the current process.
- Builds without warnings on platforms other than Linux, where it collects no metrics.

## [0.0.25](https://github.com/conradludgate/measured/compare/measured-process-v0.0.24...measured-process-v0.0.25) - 2026-04-14

### Other

- bump versions to 0.0.25
