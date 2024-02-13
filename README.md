# measured

A better metrics crate

[![docs](https://img.shields.io/docsrs/measured?style=flat-square)](https://docs.rs/measured/latest/measured/)

## Benchmark results

### Counters

Increment a counter. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads. Medium contention.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 2.4 µs
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           133.6 ns      │ 278.8 ns      │ 265.6 ns      │ 263.1 ns      │ 512     │ 51200000
├─ measured_sparse    193 ns        │ 353.5 ns      │ 340.6 ns      │ 340.1 ns      │ 512     │ 51200000
├─ metrics            549.9 ns      │ 676 ns        │ 580.5 ns      │ 580.3 ns      │ 512     │ 51200000
├─ prometheus         2.068 µs      │ 2.295 µs      │ 2.239 µs      │ 2.225 µs      │ 512     │ 51200000
╰─ prometheus_client  2.676 µs      │ 3.137 µs      │ 3.07 µs       │ 3.026 µs      │ 512     │ 51200000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           29.99 ns      │ 805.1 ns      │ 791.5 ns      │ 777.8 ns      │ 504     │ 50400000
├─ measured_sparse    322.3 ns      │ 451.5 ns      │ 408.4 ns      │ 405.5 ns      │ 504     │ 50400000
├─ metrics            887.6 ns      │ 1.079 µs      │ 1.005 µs      │ 1 µs          │ 504     │ 50400000
├─ prometheus         3.37 µs       │ 4.222 µs      │ 4.156 µs      │ 4.126 µs      │ 504     │ 50400000
╰─ prometheus_client  2.872 µs      │ 3.916 µs      │ 3.86 µs       │ 3.843 µs      │ 504     │ 50400000
```

### Histograms

* `fixed_cardinality` - Observe a value into a histogram. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads. Medium contention.
* `no_cardinality` - Start a timer and immediately stop it, record that time into a single histogram (no labels). Runs concurrently among multiple threads. Very high contention.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 2.37 µs
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           434.1 ns      │ 569.2 ns      │ 510.3 ns      │ 504.5 ns      │ 512     │ 51200000
│  ├─ measured_sparse    445.1 ns      │ 612.9 ns      │ 528.5 ns      │ 523 ns        │ 512     │ 51200000
│  ├─ metrics            4.044 µs      │ 5.269 µs      │ 4.22 µs       │ 4.238 µs      │ 512     │ 51200000
│  ├─ prometheus         2.188 µs      │ 2.733 µs      │ 2.46 µs       │ 2.451 µs      │ 512     │ 51200000
│  ╰─ prometheus_client  1.999 µs      │ 2.867 µs      │ 2.66 µs       │ 2.633 µs      │ 512     │ 51200000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           7.08 µs       │ 8.266 µs      │ 7.551 µs      │ 7.493 µs      │ 512     │ 51200000
   ├─ metrics            11.57 µs      │ 12.29 µs      │ 11.66 µs      │ 11.68 µs      │ 512     │ 51200000
   ├─ prometheus         7.213 µs      │ 7.899 µs      │ 7.337 µs      │ 7.369 µs      │ 512     │ 51200000
   ╰─ prometheus_client  96.84 µs      │ 100.7 µs      │ 99.53 µs      │ 99.26 µs      │ 512     │ 51200000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           444.7 ns      │ 556.9 ns      │ 546 ns        │ 542.7 ns      │ 504     │ 50400000
│  ├─ measured_sparse    403.7 ns      │ 514.4 ns      │ 482.9 ns      │ 479.3 ns      │ 504     │ 50400000
│  ├─ metrics            998.4 ns      │ 1.46 µs       │ 1.133 µs      │ 1.136 µs      │ 504     │ 50400000
│  ├─ prometheus         2.873 µs      │ 3.754 µs      │ 3.61 µs       │ 3.568 µs      │ 504     │ 50400000
│  ╰─ prometheus_client  2.372 µs      │ 3.98 µs       │ 3.842 µs      │ 3.735 µs      │ 504     │ 50400000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           4.151 µs      │ 4.598 µs      │ 4.524 µs      │ 4.493 µs      │ 504     │ 50400000
   ├─ metrics            4.921 µs      │ 6.678 µs      │ 5.915 µs      │ 5.938 µs      │ 504     │ 50400000
   ├─ prometheus         2.279 µs      │ 4.347 µs      │ 4.154 µs      │ 4.088 µs      │ 504     │ 50400000
   ╰─ prometheus_client  1.399 µs      │ 1.64 µs       │ 1.587 µs      │ 1.572 µs      │ 504     │ 50400000
```

### Memory

This benchmark tests a high-cardinality scenario. Each iteration inserts a unique label group into a Counter. Each benchmark uses the same
deterministic random set of labels. This test runs single-threaded.

The first block of benchmark outputs runs a single iteration per sample, so the timer imprecision becomes a limitation.
The second block removes the memory tracking and runs 100 iterators per sample. This makes the fast/mean times more accurate but makes latency spikes less accurate as they end up diluted.

* `measured` sweeps the floor in this benchmark.
* `prometheus_client` is fast and uses quite little memory, but reallocs are extremely expensive and will introduce latency spikes.
* `metrics`/`prometheus` both use lots of memory, with the majority of inserts needing several allocations.

#### Linux Ryzen 9 7950x


```
Timer precision: 2.48 µs
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           2.089 µs      │ 632 µs        │ 2.709 µs      │ 2.796 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0.001         │         │
│                       0 B         │ 1.638 MB      │ 0 B           │ 132.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 819.2 KB      │ 0 B           │ 62.92 B       │         │
├─ metrics            2.129 µs      │ 18.11 ms      │ 2.879 µs      │ 3.194 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       7           │ 8             │ 7             │ 6.635         │         │
│                       398 B       │ 21.23 MB      │ 391 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       207 B       │ 10.61 MB      │ 203 B         │ 341.8 B       │         │
├─ prometheus         2.189 µs      │ 44.3 ms       │ 3.259 µs      │ 3.408 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       20          │ 21            │ 20            │ 18.17         │         │
│                       824 B       │ 142.6 MB      │ 838 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       6           │ 7             │ 6             │ 5.453         │         │
│                       355 B       │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       3           │ 3             │ 3             │ 2.726         │         │
│                       20 B        │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  2.119 µs      │ 448.2 ms      │ 2.749 µs      │ 2.968 µs      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        2           │ 3             │ 2             │ 1.817         │         │
                        36 B        │ 478.1 MB      │ 39 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        16 B        │ 239 MB        │ 19 B          │ 112 B         │         │
```

```
Timer precision: 2.35 µs
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           79.24 ns      │ 20.43 µs      │ 302.2 ns      │ 343.4 ns      │ 50000   │ 5000000
├─ metrics            215.7 ns      │ 180.9 µs      │ 433.3 ns      │ 658.5 ns      │ 50000   │ 5000000
├─ prometheus         465.6 ns      │ 422.2 µs      │ 719.2 ns      │ 693 ns        │ 50000   │ 5000000
╰─ prometheus_client  119.9 ns      │ 4.27 ms       │ 259.8 ns      │ 412.9 ns      │ 50000   │ 5000000
```

#### Macbook Pro M2 Max

```
Timer precision: 41 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           0 ns          │ 326.7 µs      │ 291.7 ns      │ 329.1 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 819.2 KB      │ 0 B           │ 48.64 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 409.6 KB      │ 0 B           │ 20.97 B       │         │
├─ metrics            207.7 ns      │ 55.37 ms      │ 540.7 ns      │ 930.8 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 5             │ 6.635         │         │
│                       183 B       │ 42.46 MB      │ 290 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       191 B       │ 21.23 MB      │ 204 B         │ 341.8 B       │         │
├─ prometheus         82.7 ns       │ 58.11 ms      │ 707.7 ns      │ 802.6 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 832 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.453         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  41.7 ns       │ 326.9 ms      │ 249.7 ns      │ 398 ns        │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        0           │ 3             │ 2             │ 1.817         │         │
                        0 B         │ 478.1 MB      │ 36 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        16 B        │ 239 MB        │ 16 B          │ 112 B         │         │
```

```
Timer precision: 41 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           162.6 ns      │ 18.38 µs      │ 204.2 ns      │ 225.2 ns      │ 50000   │ 5000000
├─ metrics            391.3 ns      │ 974.9 µs      │ 564.7 ns      │ 882 ns        │ 50000   │ 5000000
├─ prometheus         551.3 ns      │ 545.4 µs      │ 670.9 ns      │ 715.4 ns      │ 50000   │ 5000000
╰─ prometheus_client  115.1 ns      │ 3.121 ms      │ 256.7 ns      │ 374.7 ns      │ 50000   │ 5000000
```
