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
├─ measured           77.36 ns      │ 190 ns        │ 145.6 ns      │ 147.6 ns      │ 504     │ 50400000
├─ measured_papaya    213 ns        │ 341.3 ns      │ 289.7 ns      │ 286.3 ns      │ 504     │ 50400000
├─ measured_sparse    226.5 ns      │ 539.8 ns      │ 421.1 ns      │ 416.9 ns      │ 504     │ 50400000
├─ metrics            865.5 ns      │ 1.359 µs      │ 1.13 µs       │ 1.13 µs       │ 504     │ 50400000
├─ prometheus         2.325 µs      │ 4.648 µs      │ 4.257 µs      │ 4.188 µs      │ 504     │ 50400000
╰─ prometheus_client  925.2 ns      │ 3.772 µs      │ 3.386 µs      │ 3.272 µs      │ 504     │ 50400000
```

### Histograms

* `fixed_cardinality` - Observe a value into a histogram. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads. Medium contention.
* `no_cardinality` - Start a timer and immediately stop it, record that time into a single histogram (no labels). Runs concurrently among multiple threads. Very high contention.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 10 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           212.4 ns      │ 402.9 ns      │ 346.6 ns      │ 342.5 ns      │ 512     │ 51200000
│  ├─ measured_sparse    382.5 ns      │ 586.6 ns      │ 510 ns        │ 506.3 ns      │ 512     │ 51200000
│  ├─ metrics            745.5 ns      │ 998.9 ns      │ 847.1 ns      │ 848.7 ns      │ 512     │ 51200000
│  ├─ prometheus         1.509 µs      │ 1.779 µs      │ 1.662 µs      │ 1.654 µs      │ 512     │ 51200000
│  ╰─ prometheus_client  1.701 µs      │ 2.556 µs      │ 2.442 µs      │ 2.413 µs      │ 512     │ 51200000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           2.8 µs        │ 3.532 µs      │ 3.331 µs      │ 3.264 µs      │ 512     │ 51200000
   ├─ metrics            1.072 µs      │ 1.328 µs      │ 1.204 µs      │ 1.201 µs      │ 512     │ 51200000
   ├─ prometheus         2.645 µs      │ 3.531 µs      │ 3.242 µs      │ 3.21 µs       │ 512     │ 51200000
   ╰─ prometheus_client  22.61 µs      │ 23.15 µs      │ 23 µs         │ 22.97 µs      │ 512     │ 51200000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           325 ns        │ 505.8 ns      │ 416.1 ns      │ 412.5 ns      │ 504     │ 50400000
│  ├─ measured_papaya    391.9 ns      │ 693.6 ns      │ 524 ns        │ 523.2 ns      │ 504     │ 50400000
│  ├─ measured_sparse    391.6 ns      │ 599.8 ns      │ 533.4 ns      │ 529.5 ns      │ 504     │ 50400000
│  ├─ metrics            1.037 µs      │ 1.39 µs       │ 1.275 µs      │ 1.267 µs      │ 504     │ 50400000
│  ├─ prometheus         2.724 µs      │ 4.268 µs      │ 4.052 µs      │ 4.026 µs      │ 504     │ 50400000
│  ╰─ prometheus_client  2.071 µs      │ 4.452 µs      │ 4.212 µs      │ 4.1 µs        │ 504     │ 50400000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           2.907 µs      │ 5.25 µs       │ 5.013 µs      │ 4.944 µs      │ 504     │ 50400000
   ├─ metrics            180.7 ns      │ 2.088 µs      │ 1.988 µs      │ 1.969 µs      │ 504     │ 50400000
   ├─ prometheus         2.619 µs      │ 5.183 µs      │ 4.563 µs      │ 4.564 µs      │ 504     │ 50400000
   ╰─ prometheus_client  1.116 µs      │ 2.228 µs      │ 2.126 µs      │ 2.109 µs      │ 504     │ 50400000
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
├─ measured           40.58 ns      │ 1.623 ms      │ 499.5 ns      │ 629.2 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 3.276 MB      │ 0 B           │ 132.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 1.638 MB      │ 0 B           │ 62.91 B       │         │
├─ measured_papaya    40.58 ns      │ 4.553 ms      │ 458.5 ns      │ 628.2 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       1           │ 3             │ 1             │ 1             │         │
│                       32 B        │ 75.49 MB      │ 32 B          │ 110.8 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 0             │ 0             │ 0.091         │         │
│                       0 B         │ 0 B           │ 0 B           │ 23.89 B       │         │
├─ metrics            290.5 ns      │ 84.47 ms      │ 624.5 ns      │ 1.129 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 7             │ 6.635         │         │
│                       188 B       │ 42.46 MB      │ 400 B         │ 648 B         │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       204 B       │ 21.23 MB      │ 208 B         │ 341.8 B       │         │
├─ prometheus         41.58 ns      │ 82.12 ms      │ 999.5 ns      │ 1.173 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 828 B         │ 810.4 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.452         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  40.58 ns      │ 388 ms        │ 374.5 ns      │ 542.6 ns      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        0           │ 3             │ 2             │ 1.817         │         │
                        0 B         │ 478.1 MB      │ 34 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        16 B        │ 239 MB        │ 16 B          │ 112 B         │         │
```

```
Timer precision: 41 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           113.3 ns      │ 23.71 µs      │ 428.7 ns      │ 472.1 ns      │ 50000   │ 5000000
├─ measured_papaya    156.2 ns      │ 42.41 µs      │ 505.4 ns      │ 669.9 ns      │ 50000   │ 5000000
├─ metrics            406.2 ns      │ 1.036 ms      │ 741.2 ns      │ 1.208 µs      │ 50000   │ 5000000
├─ prometheus         769.5 ns      │ 786.5 µs      │ 1 µs          │ 1.076 µs      │ 50000   │ 5000000
╰─ prometheus_client  201.6 ns      │ 4.428 ms      │ 413.7 ns      │ 596.2 ns      │ 50000   │ 5000000
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
│  ├─ 100             4.278 µs      │ 5.957 µs      │ 4.37 µs       │ 4.569 µs      │ 100     │ 1000
│  ├─ 1000            42.69 µs      │ 51.12 µs      │ 43.52 µs      │ 44.25 µs      │ 100     │ 1000
│  ├─ 10000           489.1 µs      │ 627.9 µs      │ 509.8 µs      │ 512 µs        │ 100     │ 1000
│  ╰─ 100000          6.072 ms      │ 11.21 ms      │ 6.391 ms      │ 6.537 ms      │ 100     │ 1000
├─ measured_papaya                  │               │               │               │         │
│  ├─ 100             4.295 µs      │ 7.599 µs      │ 4.399 µs      │ 4.516 µs      │ 100     │ 1000
│  ├─ 1000            47.25 µs      │ 63.99 µs      │ 48.07 µs      │ 48.72 µs      │ 100     │ 1000
│  ├─ 10000           570.3 µs      │ 644.1 µs      │ 591.1 µs      │ 593.3 µs      │ 100     │ 1000
│  ╰─ 100000          7.718 ms      │ 11.21 ms      │ 8.14 ms       │ 8.524 ms      │ 100     │ 1000
├─ metrics                          │               │               │               │         │
│  ├─ 100             59.4 µs       │ 74.29 µs      │ 60.4 µs       │ 61.52 µs      │ 100     │ 1000
│  ├─ 1000            680 µs        │ 756.4 µs      │ 694.4 µs      │ 694.3 µs      │ 100     │ 1000
│  ├─ 10000           7.249 ms      │ 11.88 ms      │ 8.206 ms      │ 8.1 ms        │ 100     │ 1000
│  ╰─ 100000          180.8 ms      │ 223.4 ms      │ 207.9 ms      │ 206.3 ms      │ 100     │ 1000
├─ prometheus                       │               │               │               │         │
│  ├─ 100             25.7 µs       │ 29.99 µs      │ 26.17 µs      │ 26.4 µs       │ 100     │ 1000
│  ├─ 1000            378.8 µs      │ 623.7 µs      │ 476.4 µs      │ 476.3 µs      │ 100     │ 1000
│  ├─ 10000           4.529 ms      │ 5.829 ms      │ 4.782 ms      │ 4.888 ms      │ 100     │ 1000
│  ╰─ 100000          80.45 ms      │ 100.1 ms      │ 87.34 ms      │ 87.28 ms      │ 100     │ 1000
╰─ prometheus_client                │               │               │               │         │
   ├─ 100             6.128 µs      │ 8.649 µs      │ 6.162 µs      │ 6.28 µs       │ 100     │ 1000
   ├─ 1000            63.46 µs      │ 71.4 µs       │ 64.26 µs      │ 64.8 µs       │ 100     │ 1000
   ├─ 10000           633 µs        │ 663.1 µs      │ 645.1 µs      │ 645.1 µs      │ 100     │ 1000
   ╰─ 100000          6.391 ms      │ 7.673 ms      │ 6.514 ms      │ 6.606 ms      │ 100     │ 1000
```
