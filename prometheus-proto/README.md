# measured-prometheus-protobuf

Prometheus protobuf exposition format for measured.

Create a `ProtoEncoder::new(writer)`, collect a metric group into it, and call `flush()` to write
the final buffered family. Calling `into_inner()` without flushing loses that family.
Use `ProtoEncoder::<W>::MIME_TYPE` through the `Encoding` trait as the HTTP Content-Type.

The encoder supports histograms, summaries, gauge histograms, exemplars, and creation timestamps.

- [First metrics tutorial](../core/src/docs/tutorial.rs)
- [Format reference](../core/src/docs/reference.rs)
- [API reference](https://docs.rs/measured-prometheus-protobuf/latest/measured_prometheus_protobuf/)

## Protobuf

The current protobuf definition file was sourced from <https://raw.githubusercontent.com/prometheus/client_model/5f5f1b1fbb510ce158f311f4eec21086e7e61dac/io/prometheus/client/metrics.proto>
