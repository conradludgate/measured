# measured-process

Export metrics for a Linux process from `/proc`. On non-Linux targets the collector emits no
metrics. Individual metrics are omitted when the corresponding procfs value cannot be read.

## Use

```toml
[dependencies]
measured = "0.1"
measured-process = "0.1"
measured-prometheus-text = "0.1"
```

```rust
use measured::MetricGroup;
use measured_process::ProcessCollector;
use measured_prometheus_text::BufferedTextEncoder;

let process = ProcessCollector::for_self();
let mut encoder = BufferedTextEncoder::new();
process.collect_group_into(&mut encoder).unwrap();
let body = encoder.finish();
```

`ProcessCollector::new(pid)` selects a process by PID; `ProcessCollector::for_self()` selects the
current process. Collect the group on each scrape to sample current process data.

The collector may export `open_fds`, `max_fds`, `virtual_memory_bytes`,
`virtual_memory_max_bytes`, `resident_memory_bytes`, `cpu_seconds_total`, `threads`,
`minor_page_faults_total`, `major_page_faults_total`, `io_read_bytes_total`,
`io_write_bytes_total`, and `start_time_seconds`. Some values depend on kernel procfs data and
permissions.

- [API reference](https://docs.rs/measured-process/latest/measured_process/)
- [measured tutorial](https://docs.rs/measured/latest/measured/docs/tutorial/index.html)
