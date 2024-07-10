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
├─ measured           19.17 ns      │ 140.5 ns      │ 132 ns        │ 131.5 ns      │ 512     │ 51200000
├─ measured_papaya    306 ns        │ 382.9 ns      │ 326.1 ns      │ 331.8 ns      │ 512     │ 51200000
├─ measured_sparse    329.7 ns      │ 340 ns        │ 335.4 ns      │ 335.3 ns      │ 512     │ 51200000
├─ metrics            520 ns        │ 548.2 ns      │ 527.5 ns      │ 527.7 ns      │ 512     │ 51200000
├─ prometheus         2.009 µs      │ 2.155 µs      │ 2.125 µs      │ 2.11 µs       │ 512     │ 51200000
╰─ prometheus_client  3.222 µs      │ 3.587 µs      │ 3.528 µs      │ 3.498 µs      │ 512     │ 51200000
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
│  ├─ measured           236.6 ns      │ 416.2 ns      │ 373.3 ns      │ 370.3 ns      │ 512     │ 51200000
│  ├─ measured_papaya    441.4 ns      │ 625.3 ns      │ 548 ns        │ 550 ns        │ 512     │ 51200000
│  ├─ measured_sparse    298.5 ns      │ 561.6 ns      │ 506.6 ns      │ 503.1 ns      │ 512     │ 51200000
│  ├─ metrics            636.5 ns      │ 899.2 ns      │ 744 ns        │ 749.1 ns      │ 512     │ 51200000
│  ├─ prometheus         1.477 µs      │ 1.733 µs      │ 1.618 µs      │ 1.612 µs      │ 512     │ 51200000
│  ╰─ prometheus_client  1.849 µs      │ 2.793 µs      │ 2.618 µs      │ 2.577 µs      │ 512     │ 51200000
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           3.276 µs      │ 3.936 µs      │ 3.771 µs      │ 3.728 µs      │ 512     │ 51200000
   ├─ metrics            1.068 µs      │ 1.345 µs      │ 1.211 µs      │ 1.208 µs      │ 512     │ 51200000
   ├─ prometheus         2.998 µs      │ 3.771 µs      │ 3.605 µs      │ 3.523 µs      │ 512     │ 51200000
   ╰─ prometheus_client  23.6 µs       │ 24.06 µs      │ 23.96 µs      │ 23.94 µs      │ 512     │ 51200000
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
Timer precision: 10ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           59.81 ns      │ 558.2 µs      │ 239.8 ns      │ 293.7 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 1             │ 0             │ 0.001         │         │
│                       0 B         │ 1.638 MB      │ 0 B           │ 132.5 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 1             │ 0             │ 0             │         │
│                       0 B         │ 819.2 KB      │ 0 B           │ 62.92 B       │         │
├─ measured_papaya    79.81 ns      │ 2.153 ms      │ 329.8 ns      │ 526.5 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       1           │ 2             │ 1             │ 1             │         │
│                       32 B        │ 75.49 MB      │ 32 B          │ 110.8 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 0             │ 0             │ 0.091         │         │
│                       0 B         │ 0 B           │ 0 B           │ 23.89 B       │         │
├─ metrics            129.8 ns      │ 20.31 ms      │ 319.8 ns      │ 646.9 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       3           │ 8             │ 7             │ 6.635         │         │
│                       189 B       │ 21.23 MB      │ 398 B         │ 648 B         │         │
│                     dealloc:      │               │               │               │         │
│                       4           │ 5             │ 4             │ 4             │         │
│                       205 B       │ 10.61 MB      │ 208 B         │ 341.8 B       │         │
├─ prometheus         79.81 ns      │ 72.24 ms      │ 579.8 ns      │ 690.3 ns      │ 5000000 │ 5000000
│                     alloc:        │               │               │               │         │
│                       0           │ 21            │ 20            │ 18.17         │         │
│                       0 B         │ 142.6 MB      │ 832 B         │ 810.4 B       │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 7             │ 6             │ 5.452         │         │
│                       0 B         │ 71.3 MB       │ 355 B         │ 351.1 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 3             │ 3             │ 2.726         │         │
│                       0 B         │ 20 B          │ 20 B          │ 18.17 B       │         │
╰─ prometheus_client  69.81 ns      │ 415.2 ms      │ 219.8 ns      │ 399.1 ns      │ 5000000 │ 5000000
                      alloc:        │               │               │               │         │
                        0           │ 3             │ 2             │ 1.817         │         │
                        0 B         │ 478.1 MB      │ 41 B          │ 225.3 B       │         │
                      dealloc:      │               │               │               │         │
                        1           │ 2             │ 1             │ 1             │         │
                        16 B        │ 239 MB        │ 19 B          │ 112 B         │         │

```

```
Timer precision: 10 ns
memory                fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured           79.61 ns      │ 11.15 µs      │ 239.2 ns      │ 273.1 ns      │ 50000   │ 5000000
├─ measured_papaya    101.9 ns      │ 21.86 µs      │ 318.3 ns      │ 477.9 ns      │ 50000   │ 5000000
├─ metrics            194.8 ns      │ 421.7 µs      │ 387.2 ns      │ 619.3 ns      │ 50000   │ 5000000
├─ prometheus         437.9 ns      │ 724.8 µs      │ 695.3 ns      │ 662.2 ns      │ 50000   │ 5000000
╰─ prometheus_client  106.2 ns      │ 4.113 ms      │ 228.3 ns      │ 368.5 ns      │ 50000   │ 5000000
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
Timer precision: 10ns
encoding              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured                         │               │               │               │         │
│  ├─ 100             3.015 µs      │ 4.275 µs      │ 3.159 µs      │ 3.176 µs      │ 100     │ 1000
│  ├─ 1000            28.8 µs       │ 41.09 µs      │ 29.35 µs      │ 29.55 µs      │ 100     │ 1000
│  ├─ 10000           332.6 µs      │ 444.8 µs      │ 343 µs        │ 347.7 µs      │ 100     │ 1000
│  ╰─ 100000          4.25 ms       │ 5.28 ms       │ 4.323 ms      │ 4.378 ms      │ 100     │ 1000
├─ measured_papaya                  │               │               │               │         │
│  ├─ 100             3.061 µs      │ 3.821 µs      │ 3.131 µs      │ 3.172 µs      │ 100     │ 1000
│  ├─ 1000            32.79 µs      │ 35.15 µs      │ 33.32 µs      │ 33.21 µs      │ 100     │ 1000
│  ├─ 10000           411.9 µs      │ 428.3 µs      │ 417.2 µs      │ 418.3 µs      │ 100     │ 1000
│  ╰─ 100000          5.408 ms      │ 6.432 ms      │ 5.511 ms      │ 5.555 ms      │ 100     │ 1000
├─ metrics                          │               │               │               │         │
│  ├─ 100             37.17 µs      │ 38.83 µs      │ 37.82 µs      │ 37.85 µs      │ 100     │ 1000
│  ├─ 1000            373.1 µs      │ 382.4 µs      │ 375.7 µs      │ 375.9 µs      │ 100     │ 1000
│  ├─ 10000           4.119 ms      │ 4.203 ms      │ 4.153 ms      │ 4.152 ms      │ 100     │ 1000
│  ╰─ 100000          63.02 ms      │ 66.24 ms      │ 64.54 ms      │ 64.54 ms      │ 100     │ 1000
├─ prometheus                       │               │               │               │         │
│  ├─ 100             21.58 µs      │ 22.68 µs      │ 21.73 µs      │ 21.76 µs      │ 100     │ 1000
│  ├─ 1000            251.2 µs      │ 255.7 µs      │ 252.4 µs      │ 252.5 µs      │ 100     │ 1000
│  ├─ 10000           3.466 ms      │ 3.512 ms      │ 3.482 ms      │ 3.482 ms      │ 100     │ 1000
│  ╰─ 100000          63.13 ms      │ 64.02 ms      │ 63.43 ms      │ 63.47 ms      │ 100     │ 1000
╰─ prometheus_client                │               │               │               │         │
   ├─ 100             5.446 µs      │ 6.272 µs      │ 5.729 µs      │ 5.711 µs      │ 100     │ 1000
   ├─ 1000            56.84 µs      │ 57.87 µs      │ 57.45 µs      │ 57.47 µs      │ 100     │ 1000
   ├─ 10000           557 µs        │ 587.3 µs      │ 573.7 µs      │ 573.2 µs      │ 100     │ 1000
   ╰─ 100000          5.899 ms      │ 6.063 ms      │ 6.018 ms      │ 5.989 ms      │ 100     │ 1000
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
