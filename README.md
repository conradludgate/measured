# measured

A better metrics crate

[![docs](https://img.shields.io/docsrs/measured?style=flat-square)](https://docs.rs/measured/latest/measured/)

## Benchmark results

### Counters

Increment a counter. Keyed with 2 labels and 18 distinct label groupings (6 * 3). Runs concurrently among multiple threads. Medium contention.

#### Linux Ryzen 9 7950x (32 Threads)

```
Timer precision: 10 ns
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           22.68 ns      │ 27.49 ns      │ 23.93 ns      │ 23.9 ns       │ 512     │ 51200000
├─ measured_sparse    22.56 ns      │ 148.5 ns      │ 50.88 ns      │ 61.48 ns      │ 512     │ 51200000
├─ metrics            508.3 ns      │ 647.7 ns      │ 605.7 ns      │ 606.4 ns      │ 512     │ 51200000
├─ prometheus         1.547 µs      │ 1.7 µs        │ 1.657 µs      │ 1.645 µs      │ 512     │ 51200000
╰─ prometheus_client  2.99 µs       │ 3.38 µs       │ 3.317 µs      │ 3.262 µs      │ 512     │ 51200000
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
Timer precision: 10 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           22.66 ns      │ 40.59 ns      │ 33.01 ns      │ 32.76 ns      │ 512     │ 51200000
│  ├─ measured_sparse    30.74 ns      │ 172.9 ns      │ 68.02 ns      │ 75.12 ns      │ 512     │ 51200000
│  ├─ metrics            781.9 ns      │ 876.3 ns      │ 797.6 ns      │ 799.2 ns      │ 512     │ 51200000
│  ├─ prometheus         1.686 µs      │ 1.91 µs       │ 1.815 µs      │ 1.807 µs      │ 512     │ 51200000
│  ╰─ prometheus_client  991.5 ns      │ 3.278 µs      │ 3.121 µs      │ 3.042 µs      │ 512     │ 51200000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           51.55 ns      │ 234.2 ns      │ 78.52 ns      │ 113.4 ns      │ 512     │ 51200000
   ├─ metrics            2.911 µs      │ 3.162 µs      │ 3.065 µs      │ 3.058 µs      │ 512     │ 51200000
   ├─ prometheus         2.886 µs      │ 3.667 µs      │ 3.514 µs      │ 3.415 µs      │ 512     │ 51200000
   ╰─ prometheus_client  22.94 µs      │ 23.6 µs       │ 23.35 µs      │ 23.36 µs      │ 512     │ 51200000
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
Timer precision: 10 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           49.81 ns      │ 88.58 ms      │ 239.8 ns      │ 309.7 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 276.8 MB      │ 0 B           │ 159.3 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 138.4 MB      │ 0 B           │ 76.34 B       │         │
├─ metrics            139.8 ns      │ 19.81 ms      │ 319.8 ns      │ 651.9 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 7             │ 6.635         │         │
│                       190 B       │ 21.23 MB      │ 396 B         │ 648.1 B       │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       206 B       │ 10.61 MB      │ 206 B         │ 341.8 B       │         │
├─ prometheus         59.81 ns      │ 71.39 ms      │ 579.8 ns      │ 697.9 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 827 B         │ 810.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.453         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  59.81 ns      │ 421.3 ms      │ 219.8 ns      │ 406.3 ns      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        0           │ 3             │ 2             │ 1.817         │         │
                        0 B         │ 478.1 MB      │ 34 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        16 B        │ 239 MB        │ 16 B          │ 112 B         │         │
```

```
Timer precision: 10 ns
high_cardinality      fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           67.21 ns      │ 897.7 µs      │ 216.4 ns      │ 260.1 ns      │ 50000   │ 5000000
├─ metrics            213.4 ns      │ 411.1 µs      │ 406.6 ns      │ 641.2 ns      │ 50000   │ 5000000
├─ prometheus         430.5 ns      │ 719.7 µs      │ 704.5 ns      │ 664.2 ns      │ 50000   │ 5000000
╰─ prometheus_client  109.6 ns      │ 4.083 ms      │ 233.3 ns      │ 376.4 ns      │ 50000   │ 5000000
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
Timer precision: 10 ns
encoding              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured                         │               │               │               │         │
│  ├─ 100             2.603 µs      │ 4.233 µs      │ 2.719 µs      │ 2.757 µs      │ 100     │ 1000
│  ├─ 1000            27.92 µs      │ 42.98 µs      │ 28.54 µs      │ 28.95 µs      │ 100     │ 1000
│  ├─ 10000           319.2 µs      │ 464.5 µs      │ 330.5 µs      │ 335.9 µs      │ 100     │ 1000
│  ╰─ 100000          3.477 ms      │ 4.736 ms      │ 3.555 ms      │ 3.593 ms      │ 100     │ 1000
├─ metrics                          │               │               │               │         │
│  ├─ 100             39.32 µs      │ 41.54 µs      │ 40.01 µs      │ 40.04 µs      │ 100     │ 1000
│  ├─ 1000            390.2 µs      │ 402.2 µs      │ 396.8 µs      │ 396.5 µs      │ 100     │ 1000
│  ├─ 10000           4.261 ms      │ 4.37 ms       │ 4.31 ms       │ 4.308 ms      │ 100     │ 1000
│  ╰─ 100000          65.3 ms       │ 67.54 ms      │ 66.03 ms      │ 66.07 ms      │ 100     │ 1000
├─ prometheus                       │               │               │               │         │
│  ├─ 100             21.39 µs      │ 22.65 µs      │ 21.56 µs      │ 21.63 µs      │ 100     │ 1000
│  ├─ 1000            250 µs        │ 254.4 µs      │ 251.3 µs      │ 251.4 µs      │ 100     │ 1000
│  ├─ 10000           3.472 ms      │ 3.494 ms      │ 3.482 ms      │ 3.482 ms      │ 100     │ 1000
│  ╰─ 100000          61.76 ms      │ 63.16 ms      │ 62.54 ms      │ 62.53 ms      │ 100     │ 1000
╰─ prometheus_client                │               │               │               │         │
   ├─ 100             5.045 µs      │ 5.863 µs      │ 5.146 µs      │ 5.217 µs      │ 100     │ 1000
   ├─ 1000            53.36 µs      │ 56.61 µs      │ 54.11 µs      │ 54.18 µs      │ 100     │ 1000
   ├─ 10000           521.1 µs      │ 539.5 µs      │ 525.7 µs      │ 526.1 µs      │ 100     │ 1000
   ╰─ 100000          5.292 ms      │ 5.603 ms      │ 5.434 ms      │ 5.417 ms      │ 100     │ 1000
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
