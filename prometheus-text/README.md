# measured-prometheus-text

Prometheus text and OpenMetrics exposition formats for measured.

Use `BufferedTextEncoder` for classic Prometheus text or `BufferedOpenMetricsEncoder` for OpenMetrics 1.0.
Collect a metric group, call `finish()` once per scrape, and send those bytes with the encoder's `Encoding::MIME_TYPE` as the HTTP Content-Type.
For a writer-based encoder, use `TextEncoder::from_writer` or `OpenMetricsEncoder::from_writer` and call `flush()` to finish the collection.

OpenMetrics preserves exemplars and creation timestamps; classic text omits them.

- [First metrics tutorial](../core/src/docs/tutorial.rs)
- [Export recipes](../core/src/docs/how_to.rs)
- [API reference](https://docs.rs/measured-prometheus-text/latest/measured_prometheus_text/)
