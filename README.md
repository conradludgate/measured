# measured

A better metrics crate

[![docs](https://img.shields.io/docsrs/measured?style=flat-square)](https://docs.rs/measured/latest/measured/)

## Benchmark results

### Counters

Increment a counter. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads. Medium contention.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 41 ns
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           60.55 ns      │ 190.3 ns      │ 147.5 ns      │ 149.2 ns      │ 504     │ 50400000
├─ measured_sparse    380.4 ns      │ 538.5 ns      │ 497.9 ns      │ 495.6 ns      │ 504     │ 50400000
├─ metrics            1.06 µs       │ 1.327 µs      │ 1.233 µs      │ 1.228 µs      │ 504     │ 50400000
├─ prometheus         4.332 µs      │ 4.595 µs      │ 4.543 µs      │ 4.532 µs      │ 504     │ 50400000
╰─ prometheus_client  4.074 µs      │ 4.391 µs      │ 4.332 µs      │ 4.323 µs      │ 504     │ 50400000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           22.31 ns      │ 46.01 ns      │ 23.04 ns      │ 26.7 ns       │ 504     │ 50400000
├─ measured_sparse    40.81 ns      │ 366.6 ns      │ 230.6 ns      │ 227 ns        │ 504     │ 50400000
├─ metrics            1.129 µs      │ 1.411 µs      │ 1.303 µs      │ 1.302 µs      │ 504     │ 50400000
├─ prometheus         2.109 µs      │ 4.681 µs      │ 4.586 µs      │ 4.413 µs      │ 504     │ 50400000
╰─ prometheus_client  1.397 µs      │ 4.264 µs      │ 4.092 µs      │ 3.929 µs      │ 504     │ 50400000
```

### Histograms

* `fixed_cardinality` - Observe a value into a histogram. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads. Medium contention.
* `no_cardinality` - Start a timer and immediately stop it, record that time into a single histogram (no labels). Runs concurrently among multiple threads. Very high contention.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 2.36 µs
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           296 ns        │ 464.6 ns      │ 380.6 ns      │ 381.1 ns      │ 512     │ 51200000
│  ├─ measured_sparse    457.1 ns      │ 621.2 ns      │ 523.4 ns      │ 522 ns        │ 512     │ 51200000
│  ├─ metrics            4.146 µs      │ 4.867 µs      │ 4.314 µs      │ 4.346 µs      │ 512     │ 51200000
│  ├─ prometheus         1.43 µs       │ 1.872 µs      │ 1.525 µs      │ 1.546 µs      │ 512     │ 51200000
│  ╰─ prometheus_client  2.196 µs      │ 2.753 µs      │ 2.551 µs      │ 2.549 µs      │ 512     │ 51200000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           7.211 µs      │ 12.88 µs      │ 7.283 µs      │ 7.685 µs      │ 512     │ 51200000
   ├─ metrics            11.68 µs      │ 12.67 µs      │ 11.81 µs      │ 11.89 µs      │ 512     │ 51200000
   ├─ prometheus         7.202 µs      │ 8.017 µs      │ 7.322 µs      │ 7.362 µs      │ 512     │ 51200000
   ╰─ prometheus_client  109.5 µs      │ 113.1 µs      │ 111.4 µs      │ 111.4 µs      │ 512     │ 51200000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           49.37 ns      │ 101.8 ns      │ 50.1 ns       │ 58.5 ns       │ 504     │ 50400000
│  ├─ measured_sparse    52.83 ns      │ 428.8 ns      │ 162.1 ns      │ 198.4 ns      │ 504     │ 50400000
│  ├─ metrics            1.097 µs      │ 1.416 µs      │ 1.203 µs      │ 1.208 µs      │ 504     │ 50400000
│  ├─ prometheus         3.185 µs      │ 4.323 µs      │ 4.278 µs      │ 4.248 µs      │ 504     │ 50400000
│  ╰─ prometheus_client  3.738 µs      │ 4.546 µs      │ 4.486 µs      │ 4.455 µs      │ 504     │ 50400000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           105.4 ns      │ 415.2 ns      │ 227.5 ns      │ 226 ns        │ 504     │ 50400000
   ├─ metrics            2.673 µs      │ 8.921 µs      │ 7.939 µs      │ 7.834 µs      │ 504     │ 50400000
   ├─ prometheus         3.456 µs      │ 5.314 µs      │ 5.036 µs      │ 4.935 µs      │ 504     │ 50400000
   ╰─ prometheus_client  2.03 µs       │ 2.36 µs       │ 2.309 µs      │ 2.3 µs        │ 504     │ 50400000
```

### Memory

This benchmark tests a high-cardinality scenario. Each iteration inserts a unique label group into a Counter. Each benchmark uses the same
deterministic random set of labels. This test runs single-threaded.

The first block of benchmark outputs runs a single iteration per sample, so the timer imprecision becomes a limitation.
The second block removes the memory tracking and runs 100 iterators per sample. This makes the fast/mean times more accurate but makes latency spikes less accurate as they end up diluted.

* `measured` sweeps the floor in this benchmark.
* `prometheus_client` is fast and uses quite little memory, but reallocs are extremely expensive and will introduce latency spikes.
* `metrics`/`prometheus` both use lots of memory, with the majority of inserts needing several allocations.

#### Linux Ryzen 9 7950x (32 Threads)


```
Timer precision: 2.39 µs
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           2.089 µs      │ 777.3 µs      │ 2.709 µs      │ 2.811 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0.001         │         │
│                       0 B         │ 1.638 MB      │ 0 B           │ 132.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 819.2 KB      │ 0 B           │ 62.92 B       │         │
├─ metrics            2.129 µs      │ 18.05 ms      │ 2.889 µs      │ 3.228 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       7           │ 8             │ 5             │ 6.635         │         │
│                       394 B       │ 21.23 MB      │ 289 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       205 B       │ 10.61 MB      │ 203 B         │ 341.8 B       │         │
├─ prometheus         2.119 µs      │ 41.79 ms      │ 3.229 µs      │ 3.357 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       20          │ 21            │ 20            │ 18.17         │         │
│                       821 B       │ 142.6 MB      │ 832 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       6           │ 7             │ 6             │ 5.453         │         │
│                       355 B       │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       3           │ 3             │ 3             │ 2.726         │         │
│                       20 B        │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  2.109 µs      │ 452.1 ms      │ 2.759 µs      │ 2.979 µs      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        2           │ 3             │ 2             │ 1.817         │         │
                        41 B        │ 478.1 MB      │ 41 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        17 B        │ 239 MB        │ 19 B          │ 112 B         │         │
```

```
Timer precision: 2.26 µs
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           81.44 ns      │ 20.1 µs       │ 271.6 ns      │ 305.1 ns      │ 50000   │ 5000000
├─ metrics            226.2 ns      │ 177 µs        │ 416.4 ns      │ 635.6 ns      │ 50000   │ 5000000
├─ prometheus         445.8 ns      │ 427.5 µs      │ 724.5 ns      │ 688.4 ns      │ 50000   │ 5000000
╰─ prometheus_client  113.1 ns      │ 4.231 ms      │ 260.4 ns      │ 416.8 ns      │ 50000   │ 5000000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           40.58 ns      │ 55.67 ms      │ 541.5 ns      │ 703.1 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 276.8 MB      │ 0 B           │ 159.3 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 138.4 MB      │ 0 B           │ 76.33 B       │         │
├─ metrics            249.5 ns      │ 99.55 ms      │ 707.5 ns      │ 1.403 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 7             │ 6.635         │         │
│                       183 B       │ 42.46 MB      │ 392 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       191 B       │ 21.23 MB      │ 204 B         │ 341.8 B       │         │
├─ prometheus         82.58 ns      │ 85.65 ms      │ 1.082 µs      │ 1.3 µs        │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 826 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.453         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  40.58 ns      │ 652.7 ms      │ 457.5 ns      │ 756.4 ns      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        0           │ 3             │ 2             │ 1.817         │         │
                        0 B         │ 478.1 MB      │ 37 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        16 B        │ 239 MB        │ 16 B          │ 112 B         │         │
```

```
Timer precision: 41 ns
high_cardinality      fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           98.33 ns      │ 703.6 µs      │ 397.9 ns      │ 447 ns        │ 50000   │ 5000000
├─ metrics            427.4 ns      │ 1.044 ms      │ 869.5 ns      │ 1.458 µs      │ 50000   │ 5000000
├─ prometheus         802 ns        │ 818.4 µs      │ 1.089 µs      │ 1.152 µs      │ 50000   │ 5000000
╰─ prometheus_client  161.6 ns      │ 6.559 ms      │ 499.1 ns      │ 721.2 ns      │ 50000   │ 5000000
```

### Encoding

Encode a counter family into a prometheus text format. With the extra dimension of number of counters in the counter family.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 2.24 µs
encoding              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured                         │               │               │               │         │
│  ├─ 100             2.945 µs      │ 5.269 µs      │ 3.119 µs      │ 3.195 µs      │ 100     │ 1000
│  ├─ 1000            24.12 µs      │ 39.75 µs      │ 25.02 µs      │ 25.43 µs      │ 100     │ 1000
│  ├─ 10000           247.8 µs      │ 411.6 µs      │ 257.3 µs      │ 263.4 µs      │ 100     │ 1000
│  ╰─ 100000          2.486 ms      │ 3.594 ms      │ 2.566 ms      │ 2.604 ms      │ 100     │ 1000
├─ metrics                          │               │               │               │         │
│  ├─ 100             40.14 µs      │ 44.47 µs      │ 40.64 µs      │ 40.91 µs      │ 100     │ 1000
│  ├─ 1000            395.4 µs      │ 404.2 µs      │ 397.7 µs      │ 398 µs        │ 100     │ 1000
│  ├─ 10000           4.084 ms      │ 4.741 ms      │ 4.298 ms      │ 4.281 ms      │ 100     │ 1000
│  ╰─ 100000          63.26 ms      │ 77.02 ms      │ 65.06 ms      │ 65.96 ms      │ 100     │ 1000
├─ prometheus                       │               │               │               │         │
│  ├─ 100             22.17 µs      │ 24.76 µs      │ 22.5 µs       │ 22.65 µs      │ 100     │ 1000
│  ├─ 1000            259.1 µs      │ 270 µs        │ 262.4 µs      │ 262.5 µs      │ 100     │ 1000
│  ├─ 10000           3.542 ms      │ 3.664 ms      │ 3.556 ms      │ 3.558 ms      │ 100     │ 1000
│  ╰─ 100000          62.62 ms      │ 67.59 ms      │ 65.59 ms      │ 65.49 ms      │ 100     │ 1000
╰─ prometheus_client                │               │               │               │         │
   ├─ 100             4.992 µs      │ 7.61 µs       │ 5.508 µs      │ 5.515 µs      │ 100     │ 1000
   ├─ 1000            50.56 µs      │ 57.72 µs      │ 53.1 µs       │ 53.11 µs      │ 100     │ 1000
   ├─ 10000           515.1 µs      │ 532.2 µs      │ 522.3 µs      │ 522.7 µs      │ 100     │ 1000
   ╰─ 100000          5.184 ms      │ 5.347 ms      │ 5.265 ms      │ 5.261 ms      │ 100     │ 1000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
encoding              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured                         │               │               │               │         │
│  ├─ 100             4.132 µs      │ 6.795 µs      │ 4.197 µs      │ 4.257 µs      │ 100     │ 1000
│  ├─ 1000            46.04 µs      │ 410.8 µs      │ 46.88 µs      │ 83.37 µs      │ 100     │ 1000
│  ├─ 10000           483.4 µs      │ 536.1 µs      │ 503.5 µs      │ 504.7 µs      │ 100     │ 1000
│  ╰─ 100000          5.458 ms      │ 7.584 ms      │ 5.65 ms       │ 5.738 ms      │ 100     │ 1000
├─ metrics                          │               │               │               │         │
│  ├─ 100             61.68 µs      │ 82.22 µs      │ 63.68 µs      │ 64.63 µs      │ 100     │ 1000
│  ├─ 1000            673.4 µs      │ 1.088 ms      │ 751.6 µs      │ 753.8 µs      │ 100     │ 1000
│  ├─ 10000           6.805 ms      │ 11.28 ms      │ 7.425 ms      │ 7.689 ms      │ 100     │ 1000
│  ╰─ 100000          196.7 ms      │ 217 ms        │ 205.9 ms      │ 206.2 ms      │ 100     │ 1000
├─ prometheus                       │               │               │               │         │
│  ├─ 100             28.24 µs      │ 32.71 µs      │ 29.07 µs      │ 29.31 µs      │ 100     │ 1000
│  ├─ 1000            387.6 µs      │ 764.6 µs      │ 390.1 µs      │ 401.6 µs      │ 100     │ 1000
│  ├─ 10000           4.635 ms      │ 5.676 ms      │ 4.668 ms      │ 4.723 ms      │ 100     │ 1000
│  ╰─ 100000          82.93 ms      │ 107.7 ms      │ 88.83 ms      │ 89.06 ms      │ 100     │ 1000
╰─ prometheus_client                │               │               │               │         │
   ├─ 100             6.699 µs      │ 8.299 µs      │ 6.716 µs      │ 6.823 µs      │ 100     │ 1000
   ├─ 1000            68.91 µs      │ 77.74 µs      │ 69.98 µs      │ 70.35 µs      │ 100     │ 1000
   ├─ 10000           692.9 µs      │ 739.8 µs      │ 694.8 µs      │ 698 µs        │ 100     │ 1000
   ╰─ 100000          6.943 ms      │ 8.266 ms      │ 6.961 ms      │ 7.005 ms      │ 100     │ 1000
```
