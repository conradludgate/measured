# measured-tokio

Tokio metrics support for measured.

The [`measured_tokio` API documentation](https://docs.rs/measured-tokio/latest/measured_tokio/)
describes the available collectors. The `net` feature enables I/O metrics. The internal
`__tokio-unstable-schedule-latency` feature requires `--cfg tokio_unstable`; its use of unstable
Tokio APIs is outside the collector's semver guarantees.
