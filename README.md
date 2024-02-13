# measured

A better metrics crate

[![docs](https://img.shields.io/docsrs/measured?style=flat-square)](https://docs.rs/measured/latest/measured/)

## Benchmark results

### Counters

Increment a counter. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads.

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           496.1 ns      │ 807.7 ns      │ 792.8 ns      │ 784.6 ns      │ 504     │ 50400000
├─ measured_flurry    132.6 ns      │ 235.8 ns      │ 197.2 ns      │ 193.3 ns      │ 504     │ 50400000
├─ measured_sparse    318.2 ns      │ 439.2 ns      │ 413.5 ns      │ 410.3 ns      │ 504     │ 50400000
├─ metrics            841.5 ns      │ 1.032 µs      │ 960.1 ns      │ 953.6 ns      │ 504     │ 50400000
├─ prometheus         2.728 µs      │ 4.178 µs      │ 4.051 µs      │ 3.963 µs      │ 504     │ 50400000
╰─ prometheus_client  2.822 µs      │ 3.747 µs      │ 3.618 µs      │ 3.598 µs      │ 504     │ 50400000
```

### Histograms

* `fixed_cardinality` - Observe a value into a histogram. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads.
* `no_cardinality` - Start a timer and immediately stop it, record that time into a single histogram (no labels). Runs concurrently among multiple threads.

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           444 ns        │ 553.8 ns      │ 539 ns        │ 535.6 ns      │ 504     │ 50400000
│  ├─ measured_flurry    281.7 ns      │ 366.4 ns      │ 346.6 ns      │ 343 ns        │ 504     │ 50400000
│  ├─ measured_sparse    412.9 ns      │ 520.3 ns      │ 491.7 ns      │ 487.7 ns      │ 504     │ 50400000
│  ├─ metrics            1.013 µs      │ 1.303 µs      │ 1.171 µs      │ 1.168 µs      │ 504     │ 50400000
│  ├─ prometheus         2.732 µs      │ 3.791 µs      │ 3.68 µs       │ 3.646 µs      │ 504     │ 50400000
│  ╰─ prometheus_client  1.406 µs      │ 3.893 µs      │ 3.776 µs      │ 3.715 µs      │ 504     │ 50400000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           3.824 µs      │ 4.577 µs      │ 4.479 µs      │ 4.428 µs      │ 504     │ 50400000
   ├─ metrics            4.859 µs      │ 8.274 µs      │ 7.961 µs      │ 7.831 µs      │ 504     │ 50400000
   ├─ prometheus         2.45 µs       │ 4.343 µs      │ 4.164 µs      │ 4.098 µs      │ 504     │ 50400000
   ╰─ prometheus_client  1.1 µs        │ 1.653 µs      │ 1.576 µs      │ 1.56 µs       │ 504     │ 50400000
```

### Memory

This benchmark tests a high-cardinality scenario. Each iteration inserts a unique label group into a Counter. Each benchmark uses the same
deterministic random set of labels.

* `measured` sweeps the floor in this benchmark.
* `prometheus_client` is fast and uses quite little memory, but reallocs are extremely expensive and will introduce latency spikes.
* `metrics`/`prometheus` both use lots of memory, with the majority of inserts needing several allocations.

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           40.7 ns       │ 622.6 ms      │ 457.7 ns      │ 729.6 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1026310       │ 2             │ 2.07          │         │
│                       0 B         │ 198.4 MB      │ 168 B         │ 260.4 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1026431       │ 0             │ 0.252         │         │
│                       0 B         │ 164.9 MB      │ 0 B           │ 66.64 B       │         │
├─ metrics            249.7 ns      │ 59.23 ms      │ 541.7 ns      │ 971.8 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 7             │ 6.635         │         │
│                       188 B       │ 42.46 MB      │ 397 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       204 B       │ 21.23 MB      │ 209 B         │ 341.8 B       │         │
├─ prometheus         41.7 ns       │ 54.18 ms      │ 708.7 ns      │ 839 ns        │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 827 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.453         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  40.7 ns       │ 311.9 ms      │ 249.7 ns      │ 385.3 ns      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        0           │ 3             │ 2             │ 1.817         │         │
                        0 B         │ 478.1 MB      │ 38 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        17 B        │ 239 MB        │ 17 B          │ 112 B         │         │
```
