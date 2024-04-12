# measured

A better metrics crate

[![docs](https://img.shields.io/docsrs/measured?style=flat-square)](https://docs.rs/measured/latest/measured/)

## Benchmark results

### Counters

Increment a counter. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads. Medium contention.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 2.36 µs
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           17.4 ns       │ 27.21 ns      │ 23.3 ns       │ 23.34 ns      │ 512     │ 51200000
├─ measured_sparse    22.63 ns      │ 178.6 ns      │ 63.04 ns      │ 71.53 ns      │ 512     │ 51200000
├─ metrics            673 ns        │ 848.5 ns      │ 722.4 ns      │ 727.5 ns      │ 512     │ 51200000
├─ prometheus         1.95 µs       │ 2.827 µs      │ 2.259 µs      │ 2.257 µs      │ 512     │ 51200000
╰─ prometheus_client  1.657 µs      │ 3.481 µs      │ 3.228 µs      │ 3.12 µs       │ 512     │ 51200000
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
Timer precision: 2.4 µs
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           32.03 ns      │ 61.72 ns      │ 50.21 ns      │ 50.11 ns      │ 512     │ 51200000
│  ├─ measured_sparse    37.92 ns      │ 237.9 ns      │ 113.7 ns      │ 115.6 ns      │ 512     │ 51200000
│  ├─ metrics            3.988 µs      │ 5.149 µs      │ 4.248 µs      │ 4.269 µs      │ 512     │ 51200000
│  ├─ prometheus         2.309 µs      │ 2.733 µs      │ 2.515 µs      │ 2.505 µs      │ 512     │ 51200000
│  ╰─ prometheus_client  2.368 µs      │ 3.146 µs      │ 2.759 µs      │ 2.765 µs      │ 512     │ 51200000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           6.948 µs      │ 8.209 µs      │ 7.116 µs      │ 7.151 µs      │ 512     │ 51200000
   ├─ metrics            11.6 µs       │ 13.92 µs      │ 11.79 µs      │ 11.87 µs      │ 512     │ 51200000
   ├─ prometheus         7.164 µs      │ 7.658 µs      │ 7.238 µs      │ 7.264 µs      │ 512     │ 51200000
   ╰─ prometheus_client  98.58 µs      │ 103.9 µs      │ 100.9 µs      │ 101.2 µs      │ 512     │ 51200000
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
Timer precision: 2.25 µs
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           2.109 µs      │ 32.2 ms       │ 2.679 µs      │ 2.804 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 276.8 MB      │ 0 B           │ 159.3 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 138.4 MB      │ 0 B           │ 76.34 B       │         │
├─ metrics            2.149 µs      │ 18.02 ms      │ 2.839 µs      │ 3.204 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       7           │ 8             │ 7             │ 6.635         │         │
│                       394 B       │ 21.23 MB      │ 392 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       205 B       │ 10.61 MB      │ 204 B         │ 341.8 B       │         │
├─ prometheus         2.109 µs      │ 41.65 ms      │ 3.199 µs      │ 3.336 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       20          │ 21            │ 20            │ 18.17         │         │
│                       838 B       │ 142.6 MB      │ 823 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       6           │ 7             │ 6             │ 5.453         │         │
│                       355 B       │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       3           │ 3             │ 3             │ 2.726         │         │
│                       20 B        │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  0 ns          │ 443.2 ms      │ 2.709 µs      │ 2.933 µs      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        2           │ 3             │ 2             │ 1.817         │         │
                        34 B        │ 478.1 MB      │ 36 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        16 B        │ 239 MB        │ 16 B          │ 112 B         │         │
```

```
Timer precision: 2.35 µs
high_cardinality      fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           76.51 ns      │ 326.4 µs      │ 241.8 ns      │ 264.5 ns      │ 50000   │ 5000000
├─ metrics            229.3 ns      │ 343.2 µs      │ 418.3 ns      │ 636.5 ns      │ 50000   │ 5000000
├─ prometheus         454.6 ns      │ 443.2 µs      │ 747.5 ns      │ 707.2 ns      │ 50000   │ 5000000
╰─ prometheus_client  124.5 ns      │ 4.349 ms      │ 273.4 ns      │ 429.5 ns      │ 50000   │ 5000000
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
Timer precision: 2.29 µs
encoding              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured                         │               │               │               │         │
│  ├─ 100             3.062 µs      │ 5.622 µs      │ 3.307 µs      │ 3.453 µs      │ 100     │ 1000
│  ├─ 1000            29.45 µs      │ 42.99 µs      │ 29.91 µs      │ 30.5 µs       │ 100     │ 1000
│  ├─ 10000           328.6 µs      │ 557.7 µs      │ 345.2 µs      │ 351.2 µs      │ 100     │ 1000
│  ╰─ 100000          3.432 ms      │ 4.172 ms      │ 3.543 ms      │ 3.559 ms      │ 100     │ 1000
├─ metrics                          │               │               │               │         │
│  ├─ 100             37.55 µs      │ 49.92 µs      │ 38.15 µs      │ 38.8 µs       │ 100     │ 1000
│  ├─ 1000            374.4 µs      │ 408.1 µs      │ 377.6 µs      │ 378.6 µs      │ 100     │ 1000
│  ├─ 10000           4.104 ms      │ 8.8 ms        │ 4.151 ms      │ 4.201 ms      │ 100     │ 1000
│  ╰─ 100000          62.65 ms      │ 72.31 ms      │ 65.12 ms      │ 65.41 ms      │ 100     │ 1000
├─ prometheus                       │               │               │               │         │
│  ├─ 100             21.06 µs      │ 23.77 µs      │ 21.38 µs      │ 21.59 µs      │ 100     │ 1000
│  ├─ 1000            242.8 µs      │ 250 µs        │ 245.4 µs      │ 245.4 µs      │ 100     │ 1000
│  ├─ 10000           3.368 ms      │ 3.758 ms      │ 3.417 ms      │ 3.476 ms      │ 100     │ 1000
│  ╰─ 100000          63.83 ms      │ 68.72 ms      │ 65.35 ms      │ 65.81 ms      │ 100     │ 1000
╰─ prometheus_client                │               │               │               │         │
   ├─ 100             5.111 µs      │ 7.671 µs      │ 5.526 µs      │ 5.592 µs      │ 100     │ 1000
   ├─ 1000            50.67 µs      │ 55.61 µs      │ 52.57 µs      │ 52.83 µs      │ 100     │ 1000
   ├─ 10000           518.1 µs      │ 544.8 µs      │ 538.4 µs      │ 537 µs        │ 100     │ 1000
   ╰─ 100000          5.313 ms      │ 5.458 ms      │ 5.442 ms      │ 5.425 ms      │ 100     │ 1000
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
