# measured

A better metrics crate

[![docs](https://img.shields.io/docsrs/measured?style=flat-square)](https://docs.rs/measured/latest/measured/)

## Benchmark results

### Counters

Increment a counter. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads. Medium contention.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 2.31 µs
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           67.26 ns      │ 152.2 ns      │ 142.6 ns      │ 142.3 ns      │ 512     │ 51200000
├─ measured_sparse    479.5 ns      │ 678.3 ns      │ 520.4 ns      │ 533.5 ns      │ 512     │ 51200000
├─ metrics            514.5 ns      │ 658.9 ns      │ 596.3 ns      │ 596.9 ns      │ 512     │ 51200000
├─ prometheus         2.416 µs      │ 3.421 µs      │ 2.62 µs       │ 2.653 µs      │ 512     │ 51200000
╰─ prometheus_client  2.328 µs      │ 3.162 µs      │ 3.102 µs      │ 3.048 µs      │ 512     │ 51200000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           46.59 ns      │ 186.7 ns      │ 151.7 ns      │ 150.9 ns      │ 504     │ 50400000
├─ measured_sparse    400.7 ns      │ 506.8 ns      │ 486.4 ns      │ 485 ns        │ 504     │ 50400000
├─ metrics            957.3 ns      │ 1.208 µs      │ 1.071 µs      │ 1.073 µs      │ 504     │ 50400000
├─ prometheus         2.307 µs      │ 4.728 µs      │ 4.673 µs      │ 4.62 µs       │ 504     │ 50400000
╰─ prometheus_client  3.538 µs      │ 4.39 µs       │ 4.338 µs      │ 4.316 µs      │ 504     │ 50400000
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
│  ├─ measured           323.9 ns      │ 441.6 ns      │ 419.3 ns      │ 416 ns        │ 504     │ 50400000
│  ├─ measured_sparse    458.7 ns      │ 690.9 ns      │ 568.5 ns      │ 568.5 ns      │ 504     │ 50400000
│  ├─ metrics            1.303 µs      │ 1.635 µs      │ 1.449 µs      │ 1.457 µs      │ 504     │ 50400000
│  ├─ prometheus         3.711 µs      │ 4.303 µs      │ 4.26 µs       │ 4.239 µs      │ 504     │ 50400000
│  ╰─ prometheus_client  4.23 µs       │ 4.665 µs      │ 4.614 µs      │ 4.59 µs       │ 504     │ 50400000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           6.34 µs       │ 7.66 µs       │ 7.596 µs      │ 7.539 µs      │ 504     │ 50400000
   ├─ metrics            7.002 µs      │ 9.293 µs      │ 8.938 µs      │ 8.818 µs      │ 504     │ 50400000
   ├─ prometheus         3.795 µs      │ 5.483 µs      │ 5.259 µs      │ 5.187 µs      │ 504     │ 50400000
   ╰─ prometheus_client  2.016 µs      │ 2.413 µs      │ 2.35 µs       │ 2.335 µs      │ 504     │ 50400000
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
Timer precision: 2.35 µs
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           79.24 ns      │ 20.43 µs      │ 302.2 ns      │ 343.4 ns      │ 50000   │ 5000000
├─ metrics            215.7 ns      │ 180.9 µs      │ 433.3 ns      │ 658.5 ns      │ 50000   │ 5000000
├─ prometheus         465.6 ns      │ 422.2 µs      │ 719.2 ns      │ 693 ns        │ 50000   │ 5000000
╰─ prometheus_client  119.9 ns      │ 4.27 ms       │ 259.8 ns      │ 412.9 ns      │ 50000   │ 5000000
```

#### Macbook Pro M2 Max (12 Threads)

```
Timer precision: 41 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           40.58 ns      │ 2.05 ms       │ 541.5 ns      │ 652.7 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 3.276 MB      │ 0 B           │ 132.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 1.638 MB      │ 0 B           │ 62.91 B       │         │
├─ metrics            332.5 ns      │ 116.4 ms      │ 833.5 ns      │ 1.664 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 5             │ 6.635         │         │
│                       189 B       │ 42.46 MB      │ 293 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       205 B       │ 21.23 MB      │ 206 B         │ 341.8 B       │         │
├─ prometheus         82.58 ns      │ 87.2 ms       │ 1.082 µs      │ 1.281 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 829 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.453         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  40.58 ns      │ 676.4 ms      │ 457.5 ns      │ 740.3 ns      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        0           │ 3             │ 2             │ 1.817         │         │
                        0 B         │ 478.1 MB      │ 37 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        16 B        │ 239 MB        │ 16 B          │ 112 B         │         │
```

```
Timer precision: 41 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           58.88 ns      │ 14.49 µs      │ 228.8 ns      │ 252.1 ns      │ 50000   │ 5000000
├─ metrics            376.7 ns      │ 994.9 µs      │ 582.6 ns      │ 919.6 ns      │ 50000   │ 5000000
├─ prometheus         534.2 ns      │ 548.8 µs      │ 667.2 ns      │ 708.9 ns      │ 50000   │ 5000000
╰─ prometheus_client  114.2 ns      │ 3.079 ms      │ 248 ns        │ 360.6 ns      │ 50000   │ 5000000
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
│  ├─ 100             3.582 µs      │ 5.728 µs      │ 3.645 µs      │ 3.742 µs      │ 100     │ 1000
│  ├─ 1000            36.79 µs      │ 46.49 µs      │ 38.23 µs      │ 38.65 µs      │ 100     │ 1000
│  ├─ 10000           357 µs        │ 412.5 µs      │ 366.8 µs      │ 367.4 µs      │ 100     │ 1000
│  ╰─ 100000          3.758 ms      │ 4.097 ms      │ 3.868 ms      │ 3.89 ms       │ 100     │ 1000
├─ metrics                          │               │               │               │         │
│  ├─ 100             65.81 µs      │ 72.24 µs      │ 66.48 µs      │ 66.74 µs      │ 100     │ 1000
│  ├─ 1000            762.1 µs      │ 905.8 µs      │ 774.7 µs      │ 795.2 µs      │ 100     │ 1000
│  ├─ 10000           7.736 ms      │ 8.829 ms      │ 7.822 ms      │ 7.867 ms      │ 100     │ 1000
│  ╰─ 100000          185.1 ms      │ 241.1 ms      │ 222.6 ms      │ 220.4 ms      │ 100     │ 1000
├─ prometheus                       │               │               │               │         │
│  ├─ 100             27.82 µs      │ 29.99 µs      │ 28.89 µs      │ 28.84 µs      │ 100     │ 1000
│  ├─ 1000            455.2 µs      │ 698.2 µs      │ 496.2 µs      │ 498.9 µs      │ 100     │ 1000
│  ├─ 10000           4.645 ms      │ 5.095 ms      │ 4.74 ms       │ 4.755 ms      │ 100     │ 1000
│  ╰─ 100000          81.43 ms      │ 101 ms        │ 93.82 ms      │ 92.88 ms      │ 100     │ 1000
╰─ prometheus_client                │               │               │               │         │
   ├─ 100             7.195 µs      │ 10.6 µs       │ 7.245 µs      │ 7.453 µs      │ 100     │ 1000
   ├─ 1000            73.99 µs      │ 87.81 µs      │ 76.66 µs      │ 77.47 µs      │ 100     │ 1000
   ├─ 10000           744 µs        │ 789.1 µs      │ 765.1 µs      │ 764.3 µs      │ 100     │ 1000
   ╰─ 100000          7.531 ms      │ 8.1 ms        │ 7.714 ms      │ 7.74 ms       │ 100     │ 1000
```
