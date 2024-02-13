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
├─ measured           697 ns        │ 814.7 ns      │ 807.4 ns      │ 803.8 ns      │ 504     │ 50400000
├─ measured_sparse    177 ns        │ 449.8 ns      │ 414.4 ns      │ 408.6 ns      │ 504     │ 50400000
├─ metrics            881.8 ns      │ 1.067 µs      │ 999.9 ns      │ 995.9 ns      │ 504     │ 50400000
├─ prometheus         3.883 µs      │ 4.137 µs      │ 4.099 µs      │ 4.087 µs      │ 504     │ 50400000
╰─ prometheus_client  3.179 µs      │ 3.854 µs      │ 3.818 µs      │ 3.793 µs      │ 504     │ 50400000
```

### Histograms

* `fixed_cardinality` - Observe a value into a histogram. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads.
* `no_cardinality` - Start a timer and immediately stop it, record that time into a single histogram (no labels). Runs concurrently among multiple threads.

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           442 ns        │ 552.6 ns      │ 542.9 ns      │ 539.3 ns      │ 504     │ 50400000
│  ├─ measured_sparse    415 ns        │ 535.4 ns      │ 489.2 ns      │ 485.6 ns      │ 504     │ 50400000
│  ├─ metrics            747.1 ns      │ 1.353 µs      │ 1.216 µs      │ 1.212 µs      │ 504     │ 50400000
│  ├─ prometheus         3.115 µs      │ 3.702 µs      │ 3.611 µs      │ 3.597 µs      │ 504     │ 50400000
│  ╰─ prometheus_client  2.152 µs      │ 3.752 µs      │ 3.692 µs      │ 3.669 µs      │ 504     │ 50400000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           3.897 µs      │ 4.541 µs      │ 4.426 µs      │ 4.396 µs      │ 504     │ 50400000
   ├─ metrics            3.172 µs      │ 7.499 µs      │ 6.905 µs      │ 6.832 µs      │ 504     │ 50400000
   ├─ prometheus         3.179 µs      │ 4.129 µs      │ 3.889 µs      │ 3.881 µs      │ 504     │ 50400000
   ╰─ prometheus_client  1.407 µs      │ 1.628 µs      │ 1.583 µs      │ 1.574 µs      │ 504     │ 50400000
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
├─ measured           0 ns          │ 743.4 µs      │ 290.7 ns      │ 316.1 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 3.276 MB      │ 0 B           │ 132.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 1.638 MB      │ 0 B           │ 62.91 B       │         │
├─ metrics            208.7 ns      │ 58.94 ms      │ 499.7 ns      │ 919.3 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 7             │ 6.635         │         │
│                       188 B       │ 42.46 MB      │ 398 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       204 B       │ 21.23 MB      │ 209 B         │ 341.8 B       │         │
├─ prometheus         40.7 ns       │ 54.41 ms      │ 666.7 ns      │ 793.3 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 828 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.453         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  40.7 ns       │ 323.5 ms      │ 249.7 ns      │ 383 ns        │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        0           │ 3             │ 2             │ 1.817         │         │
                        0 B         │ 478.1 MB      │ 36 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        17 B        │ 239 MB        │ 17 B          │ 112 B         │         │
```
