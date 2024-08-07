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
├─ measured           79.55 ns      │ 243.9 ns      │ 154.7 ns      │ 154.7 ns      │ 504     │ 50400000
├─ measured_sparse    396.3 ns      │ 551.6 ns      │ 488.4 ns      │ 486.6 ns      │ 504     │ 50400000
├─ metrics            873.9 ns      │ 1.411 µs      │ 1.121 µs      │ 1.126 µs      │ 504     │ 50400000
├─ prometheus         3.222 µs      │ 4.58 µs       │ 4.361 µs      │ 4.281 µs      │ 504     │ 50400000
╰─ prometheus_client  2.614 µs      │ 4.061 µs      │ 3.885 µs      │ 3.811 µs      │ 504     │ 50400000
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
│  ├─ measured           132.8 ns      │ 444 ns        │ 409.6 ns      │ 401.3 ns      │ 504     │ 50400000
│  ├─ measured_sparse    320.9 ns      │ 565.4 ns      │ 492.9 ns      │ 491.1 ns      │ 504     │ 50400000
│  ├─ metrics            1.064 µs      │ 1.43 µs       │ 1.258 µs      │ 1.256 µs      │ 504     │ 50400000
│  ├─ prometheus         3.2 µs        │ 4.248 µs      │ 4.181 µs      │ 4.142 µs      │ 504     │ 50400000
│  ╰─ prometheus_client  2.799 µs      │ 4.5 µs        │ 4.387 µs      │ 4.302 µs      │ 504     │ 50400000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           3.636 µs      │ 7.291 µs      │ 7.143 µs      │ 6.945 µs      │ 504     │ 50400000
   ├─ metrics            1.733 µs      │ 2.053 µs      │ 2 µs          │ 1.988 µs      │ 504     │ 50400000
   ├─ prometheus         1.81 µs       │ 5.23 µs       │ 5.121 µs      │ 4.882 µs      │ 504     │ 50400000
   ╰─ prometheus_client  1.813 µs      │ 2.271 µs      │ 2.194 µs      │ 2.169 µs      │ 504     │ 50400000
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
├─ measured           41.58 ns      │ 1.521 ms      │ 457.5 ns      │ 578.2 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 3.276 MB      │ 0 B           │ 132.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 1.638 MB      │ 0 B           │ 62.91 B       │         │
├─ metrics            249.5 ns      │ 100.6 ms      │ 624.5 ns      │ 1.148 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 7             │ 6.635         │         │
│                       183 B       │ 42.46 MB      │ 388 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       191 B       │ 21.23 MB      │ 202 B         │ 341.8 B       │         │
├─ prometheus         124.5 ns      │ 86.45 ms      │ 1.04 µs       │ 1.228 µs      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 823 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.453         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  40.58 ns      │ 437.2 ms      │ 374.5 ns      │ 587.3 ns      │ 5000000 │ 5000000
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
├─ measured           97.5 ns       │ 26.6 µs       │ 327.9 ns      │ 370.8 ns      │ 50000   │ 5000000
├─ metrics            427.5 ns      │ 928.9 µs      │ 726.2 ns      │ 1.095 µs      │ 50000   │ 5000000
├─ prometheus         752.9 ns      │ 768.3 µs      │ 979.5 ns      │ 1.06 µs       │ 50000   │ 5000000
╰─ prometheus_client  186.2 ns      │ 3.909 ms      │ 387 ns        │ 552 ns        │ 50000   │ 5000000
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
│  ├─ 100             4.066 µs      │ 5.062 µs      │ 4.132 µs      │ 4.172 µs      │ 100     │ 1000
│  ├─ 1000            41.52 µs      │ 51.17 µs      │ 42.57 µs      │ 43 µs         │ 100     │ 1000
│  ├─ 10000           403.8 µs      │ 542.9 µs      │ 423.6 µs      │ 424.2 µs      │ 100     │ 1000
│  ╰─ 100000          4.221 ms      │ 4.76 ms       │ 4.393 ms      │ 4.399 ms      │ 100     │ 1000
├─ metrics                          │               │               │               │         │
│  ├─ 100             61.47 µs      │ 66.87 µs      │ 61.98 µs      │ 62.13 µs      │ 100     │ 1000
│  ├─ 1000            698.2 µs      │ 917.1 µs      │ 772.3 µs      │ 779.3 µs      │ 100     │ 1000
│  ├─ 10000           7.47 ms       │ 8.927 ms      │ 8.132 ms      │ 8.176 ms      │ 100     │ 1000
│  ╰─ 100000          161.5 ms      │ 214.8 ms      │ 197 ms        │ 196.8 ms      │ 100     │ 1000
├─ prometheus                       │               │               │               │         │
│  ├─ 100             27.76 µs      │ 33.16 µs      │ 28.62 µs      │ 28.84 µs      │ 100     │ 1000
│  ├─ 1000            460.8 µs      │ 828.3 µs      │ 469.2 µs      │ 480.3 µs      │ 100     │ 1000
│  ├─ 10000           4.802 ms      │ 6.212 ms      │ 5.112 ms      │ 5.159 ms      │ 100     │ 1000
│  ╰─ 100000          72.99 ms      │ 91.93 ms      │ 78.84 ms      │ 79.97 ms      │ 100     │ 1000
╰─ prometheus_client                │               │               │               │         │
   ├─ 100             7.145 µs      │ 9.07 µs       │ 7.191 µs      │ 7.381 µs      │ 100     │ 1000
   ├─ 1000            73.57 µs      │ 80.57 µs      │ 75.45 µs      │ 75.64 µs      │ 100     │ 1000
   ├─ 10000           732.4 µs      │ 771 µs        │ 740 µs        │ 742.7 µs      │ 100     │ 1000
   ╰─ 100000          7.441 ms      │ 8.54 ms       │ 7.613 ms      │ 7.683 ms      │ 100     │ 1000
```
