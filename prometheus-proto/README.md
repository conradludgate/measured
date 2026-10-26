# measured-prometheus-protobuf

Prometheus protobuf exposition format for measured.

Add `measured` and `measured-prometheus-protobuf` to your dependencies. Create a
`ProtoEncoder::new(writer)`, collect a metric group into it, and call `flush()` to
write the final buffered family. Calling `into_inner()` without flushing loses that family.
Use `ProtoEncoder::<W>::MIME_TYPE` through the `Encoding` trait as the HTTP Content-Type.

```rust
use measured::metric::group::Encoding;
use measured::{Counter, MetricGroup};
use measured_prometheus_protobuf::ProtoEncoder;

#[derive(Default, MetricGroup)]
struct Metrics {
    requests_total: Counter,
}

let metrics = Metrics::default();
let mut encoder = ProtoEncoder::new(Vec::new());
metrics.collect_group_into(&mut encoder).unwrap();
encoder.flush().unwrap();
let bytes = encoder.into_inner();
assert!(!bytes.is_empty());
```

The encoder supports histograms, summaries, gauge histograms, exemplars, and creation timestamps.

- [First metrics tutorial](https://docs.rs/measured/latest/measured/docs/tutorial/index.html)
- [Format reference](https://docs.rs/measured/latest/measured/docs/reference/index.html)
- [API reference](https://docs.rs/measured-prometheus-protobuf/latest/measured_prometheus_protobuf/)

## Protobuf

The current protobuf definition file was sourced from <https://raw.githubusercontent.com/prometheus/client_model/5f5f1b1fbb510ce158f311f4eec21086e7e61dac/io/prometheus/client/metrics.proto>
