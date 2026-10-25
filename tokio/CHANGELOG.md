# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Breaking
- The minimum supported Rust version is now 1.98.

- Renamed metrics:
  - `threads_total` is now `threads`.
  - `workers_park_unpark_total` is now `worker_park_unpark_total`.
- `queued_tasks` labels worker-local queues as `kind="worker",worker="N"` instead of `kind="N"`.
- `scheduled_tasks_total` for tasks scheduled from outside the runtime is labelled `overflow="false"` instead of `overflow="true"`.

### Added

- With `--cfg tokio_unstable`, `worker_poll_time_seconds` is typed as a histogram.
- The `MetricGroup` impls work with any `Encoding`, with no extra bounds.
- `# TYPE` lines are written for counters and gauges.
- `worker_schedule_latency_seconds`, behind the hidden `__tokio-unstable-schedule-latency` feature. It requires `--cfg tokio_unstable`, and is only written when the runtime enables the schedule latency histogram.

## [0.0.25](https://github.com/conradludgate/measured/compare/measured-tokio-v0.0.24...measured-tokio-v0.0.25) - 2026-04-14

### Other

- bump versions to 0.0.25
