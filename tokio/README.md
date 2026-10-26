# measured-tokio

Export Tokio runtime metrics through [`measured`](https://crates.io/crates/measured).
`RuntimeCollector` samples one runtime; `NamedRuntimesCollector` can sample several runtimes and
label their metrics by name.

## Use

```toml
[dependencies]
measured = "0.1"
measured-tokio = "0.1"
measured-prometheus-text = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Add a `RuntimeCollector` to a `MetricGroup` and collect it when serving a scrape. The crate-level
documentation has a complete example.

## Optional metrics

| Requirement | Additional metrics |
| --- | --- |
| Default | Stable Tokio runtime metrics |
| `net` feature **and** `--cfg tokio_unstable` | I/O driver file descriptor and ready-event counters |
| `__tokio-unstable-schedule-latency` **and** `--cfg tokio_unstable` | Schedule-latency histogram data (unstable Tokio API; outside semver guarantees) |

`--cfg tokio_unstable` opts into Tokio APIs that may change between Tokio releases. See the
[API reference](https://docs.rs/measured-tokio/latest/measured_tokio/) and the
[measured tutorial](https://docs.rs/measured/latest/measured/docs/tutorial/index.html).
