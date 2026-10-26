# measured-prometheus-text

Prometheus text and OpenMetrics exposition formats for [`measured`](https://crates.io/crates/measured).

Add `measured` and `measured-prometheus-text` to your dependencies. Use
`BufferedTextEncoder` for classic Prometheus text or `BufferedOpenMetricsEncoder` for OpenMetrics
1.0:

```rust
use measured::metric::group::Encoding;
use measured::{Counter, MetricGroup};
use measured_prometheus_text::BufferedTextEncoder;

#[derive(Default, MetricGroup)]
struct Metrics {
    requests_total: Counter,
}

let metrics = Metrics::default();
metrics.requests_total.inc();
let mut encoder = BufferedTextEncoder::new();
metrics.collect_group_into(&mut encoder).unwrap();
let body = encoder.finish();
let content_type = BufferedTextEncoder::MIME_TYPE;
assert_eq!(content_type, "text/plain; version=0.0.4");
assert!(!body.is_empty());
```

Call `finish()` once per scrape and send its bytes with the encoder's `Encoding::MIME_TYPE` as the
HTTP Content-Type. Reuse the buffered encoder between scrapes to reuse its allocation. For a
writer-based encoder, use `TextEncoder::from_writer` or `OpenMetricsEncoder::from_writer` and call
`flush()` to finish the collection.

OpenMetrics preserves exemplars, creation timestamps, and metric units. Classic text omits these
fields.

- [API reference](https://docs.rs/measured-prometheus-text/latest/measured_prometheus_text/)
- [Tutorial](https://docs.rs/measured/latest/measured/docs/tutorial/index.html)
- [Export recipes](https://docs.rs/measured/latest/measured/docs/how_to/index.html)
