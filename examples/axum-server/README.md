# Axum server example

This example shows a complete `measured` integration with Axum. It records request counts,
response counts, and request-duration histograms using typed labels, then exposes them from a
`/metrics` endpoint in Prometheus text format.

Run it from the workspace root:

```sh
cargo run -p axum-example
```

The server listens on `http://localhost:3000`. Make requests to the example routes, then fetch
`http://localhost:3000/metrics` to inspect the resulting series. The label set uses matched route
templates (such as `/users/:id`) rather than concrete IDs to keep cardinality bounded.

The example enables the `lasso` feature because route labels are discovered at runtime. For the
basic instrumentation path, start with the
[measured tutorial](https://docs.rs/measured/latest/measured/docs/tutorial/index.html).
