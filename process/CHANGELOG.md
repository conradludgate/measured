# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
