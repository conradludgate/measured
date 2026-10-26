# measured-derive

Derive macros for [`measured`](https://crates.io/crates/measured).

Most applications should depend on `measured`, which re-exports these derives:

```toml
[dependencies]
measured = "0.1"
```

```rust
use measured::{Counter, MetricGroup};

#[derive(MetricGroup, Default)]
struct Metrics {
    /// Number of completed requests.
    requests_total: Counter,
}
```

The macros are `FixedCardinalityLabel`, `LabelGroup`, and `MetricGroup`. Derive attributes configure
label names and storage, metric names and namespaces, constructors, initialization, help text, and
units. See the [measured documentation](https://docs.rs/measured/latest/measured/) for the full
attribute reference.
