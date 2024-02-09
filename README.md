# measured

A better metrics crate

[![docs](https://img.shields.io/docsrs/measured?style=flat-square)](https://docs.rs/measured/latest/measured/)

## Benchmark results

The benchmark runs on multiple threads a simple counter increment, with various labels,
and then samples/encodes the values into the Prometheus text format. This crate outperforms both `metrics` and `prometheus`
when it comes to both speed and allocations. The `prometheus_client` crate shares some goals to this library and performs well, but still not as fast.

### Counters

The `fixed_cardinality` group has only 2 label pairs and 18 distinct label groups and runs 20000 times per thread.
The `high_cardinality` group has 3 label pairs and 2,000 distinct label groups per thread and runs 2000 times per thread.

Time in both groups also includes text encoding

The test runs on the max (N) threads for the CPU, and all label groups are selected at random (repeatable)
with each thread having a unique sequence

#### Ryzen 9 7950X (32 threads):

```
Timer precision: 2.22 µs
counters                 fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           3.871 ms      │ 6.786 ms      │ 5.879 ms      │ 5.754 ms      │ 512     │ 2560
│  │                     alloc:        │               │               │               │         │
│  │                       2           │ 2             │ 2             │ 2             │         │
│  │                       48 B        │ 48 B          │ 48 B          │ 48 B          │         │
│  │                     grow:         │               │               │               │         │
│  │                       7           │ 7             │ 7             │ 7             │         │
│  │                       1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ measured_sparse    4.355 ms      │ 9.268 ms      │ 6.879 ms      │ 6.904 ms      │ 512     │ 2560
│  │                     alloc:        │               │               │               │         │
│  │                       2           │ 2             │ 2             │ 2.007         │         │
│  │                       48 B        │ 48 B          │ 48 B          │ 48.59 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       7           │ 7             │ 7             │ 7             │         │
│  │                       1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ metrics            10.99 ms      │ 13.54 ms      │ 11.86 ms      │ 11.86 ms      │ 512     │ 2560
│  │                     alloc:        │               │               │               │         │
│  │                       40299       │ 40299         │ 40299         │ 40299         │         │
│  │                       2.576 MB    │ 2.576 MB      │ 2.576 MB      │ 2.576 MB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       40298       │ 40298         │ 40298         │ 40298         │         │
│  │                       2.577 MB    │ 2.577 MB      │ 2.577 MB      │ 2.577 MB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       61          │ 61            │ 61            │ 61            │         │
│  │                       2.634 KB    │ 2.634 KB      │ 2.634 KB      │ 2.634 KB      │         │
│  ├─ prometheus         32.43 ms      │ 46.04 ms      │ 42.43 ms      │ 42.2 ms       │ 512     │ 2560
│  │                     alloc:        │               │               │               │         │
│  │                       118         │ 118           │ 118           │ 118.1         │         │
│  │                       6.589 KB    │ 6.589 KB      │ 6.589 KB      │ 6.602 KB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       117         │ 117           │ 117           │ 117           │         │
│  │                       6.581 KB    │ 6.581 KB      │ 6.581 KB      │ 6.585 KB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       7           │ 7             │ 7             │ 7.014         │         │
│  │                       1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ╰─ prometheus_client  53.64 ms      │ 69.07 ms      │ 66.76 ms      │ 66.15 ms      │ 512     │ 2560
│                        alloc:        │               │               │               │         │
│                          1           │ 1             │ 1             │ 1.009         │         │
│                          8 B         │ 8 B           │ 10.4 B        │ 9.267 B       │         │
│                        dealloc:      │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.001         │         │
│                          0 B         │ 0 B           │ 0 B           │ 0.829 B       │         │
│                        grow:         │               │               │               │         │
│                          7           │ 7             │ 7             │ 7             │         │
│                          1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
╰─ high_cardinality                    │               │               │               │         │
   ├─ measured           27.5 ms       │ 115.1 ms      │ 68.78 ms      │ 71.65 ms      │ 128     │ 256
   │                     alloc:        │               │               │               │         │
   │                       117         │ 22.5          │ 14            │ 21.64         │         │
   │                       406.6 KB    │ 1.538 MB      │ 373.1 KB      │ 1.982 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       106         │ 20.5          │ 12            │ 19.02         │         │
   │                       189.5 KB    │ 770.9 KB      │ 189.2 KB      │ 129.5 KB      │         │
   │                     grow:         │               │               │               │         │
   │                       22.5        │ 24            │ 24.5          │ 24.15         │         │
   │                       5.113 MB    │ 27.26 MB      │ 13.63 MB      │ 18.7 MB       │         │
   ├─ metrics            228 ms        │ 1.592 s       │ 711.8 ms      │ 783 ms        │ 128     │ 256
   │                     alloc:        │               │               │               │         │
   │                       1356104     │ 5483885       │ 2776753       │ 3420460       │         │
   │                       69.93 MB    │ 281 MB        │ 141.3 MB      │ 167.4 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       1352105     │ 5479912       │ 2772765       │ 3416572       │         │
   │                       74.57 MB    │ 300.3 MB      │ 151.1 MB      │ 179.6 MB      │         │
   │                     grow:         │               │               │               │         │
   │                       305487      │ 1243624       │ 628363        │ 774683        │         │
   │                       11.78 MB    │ 47.48 MB      │ 23.84 MB      │ 31.34 MB      │         │
   ├─ prometheus         654.7 ms      │ 2.787 s       │ 1.685 s       │ 1.65 s        │ 128     │ 256
   │                     alloc:        │               │               │               │         │
   │                       519361      │ 2010249       │ 1269061       │ 1261922       │         │
   │                       26.67 MB    │ 103.8 MB      │ 65.48 MB      │ 65.18 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       505381      │ 1996409       │ 1255123       │ 1248315       │         │
   │                       26.2 MB     │ 103.3 MB      │ 65.01 MB      │ 64.69 MB      │         │
   │                     grow:         │               │               │               │         │
   │                       3014        │ 2996          │ 3010          │ 2937          │         │
   │                       8.408 MB    │ 33.57 MB      │ 25.18 MB      │ 23.02 MB      │         │
   ╰─ prometheus_client  27.83 ms      │ 5.692 s       │ 5.275 s       │ 5.106 s       │ 128     │ 256
                         alloc:        │               │               │               │         │
                           2001        │ 4928          │ 4945          │ 4868          │         │
                           27.12 KB    │ 427.7 KB      │ 429.6 KB      │ 651.6 KB      │         │
                         dealloc:      │               │               │               │         │
                           2000        │ 2946          │ 2950          │ 2923          │         │
                           27.11 KB    │ 751.3 KB      │ 757.3 KB      │ 852.3 KB      │         │
                         grow:         │               │               │               │         │
                           19          │ 960.5         │ 970.5         │ 941.7         │         │
                           6.815 MB    │ 27.62 MB      │ 17.4 MB       │ 18.93 MB      │         │
```

#### Macbook Pro M2 Max (12 threads):

```
Timer precision: 41 ns
counters                 fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           13.86 ms      │ 16.27 ms      │ 16.02 ms      │ 15.94 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.162         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 3.904 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.569         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 134.7 B       │         │
│  ├─ measured_sparse    6.337 ms      │ 8.403 ms      │ 8.076 ms      │ 8.039 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 1             │ 0.166         │         │
│  │                       0 B         │ 0 B           │ 24 B          │ 4 B           │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 3.4           │ 0.583         │         │
│  │                       0 B         │ 0 B           │ 828 B         │ 138 B         │         │
│  ├─ metrics            17.52 ms      │ 22.96 ms      │ 20.87 ms      │ 20.8 ms       │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 3358          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 214.7 KB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 3358          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 214.7 KB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 5.083         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 219.5 B       │         │
│  ├─ prometheus         75.22 ms      │ 81.62 ms      │ 80.89 ms      │ 80.58 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 9.833         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 549 B         │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 9.75          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 548.4 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.583         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 138 B         │         │
│  ╰─ prometheus_client  73.34 ms      │ 77.37 ms      │ 76.32 ms      │ 76.19 ms      │ 504     │ 2520
│                        alloc:        │               │               │               │         │
│                          0           │ 0             │ 0.4           │ 0.083         │         │
│                          0 B         │ 0 B           │ 4 B           │ 0.666 B       │         │
│                        grow:         │               │               │               │         │
│                          0           │ 0             │ 3.4           │ 0.583         │         │
│                          0 B         │ 0 B           │ 828 B         │ 138 B         │         │
╰─ high_cardinality                    │               │               │               │         │
   ├─ measured           7.706 ms      │ 71.54 ms      │ 32.35 ms      │ 34.49 ms      │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 0             │ 0.666         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 1.909 KB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 0.472         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 801.4 B       │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 0             │ 1.657         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 1.293 MB      │         │
   ├─ metrics            90.01 ms      │ 1.045 s       │ 526.1 ms      │ 523.4 ms      │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 0             │ 197792        │         │
   │                       0 B         │ 0 B           │ 0 B           │ 8.963 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 197755        │         │
   │                       0 B         │ 0 B           │ 0 B           │ 9.689 MB      │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 0             │ 44793         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 1.943 MB      │         │
   ├─ prometheus         58.7 ms       │ 1 s           │ 398.3 ms      │ 442.5 ms      │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       0           │ 1594124       │ 0             │ 74606         │         │
   │                       0 B         │ 82.49 MB      │ 0 B           │ 3.859 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 1594123       │ 0             │ 74476         │         │
   │                       0 B         │ 82.49 MB      │ 0 B           │ 3.854 MB      │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 21            │ 0             │ 29.41         │         │
   │                       0 B         │ 33.55 MB      │ 0 B           │ 1.437 MB      │         │
   ╰─ prometheus_client  61.44 ms      │ 281.2 ms      │ 179.3 ms      │ 163.1 ms      │ 108     │ 216
                         alloc:        │               │               │               │         │
                           2001        │ 0             │ 0             │ 185.7         │         │
                           27.12 KB    │ 0 B           │ 0 B           │ 3.884 KB      │         │
                         dealloc:      │               │               │               │         │
                           2000        │ 0             │ 0             │ 167.1         │         │
                           27.11 KB    │ 0 B           │ 0 B           │ 2.995 KB      │         │
                         grow:         │               │               │               │         │
                           19          │ 0             │ 0             │ 1.657         │         │
                           6.815 MB    │ 0 B           │ 0 B           │ 1.293 MB      │         │
```

### Histograms

The `fixed_cardinality` group has only 2 label pairs and 18 distinct label groups.
The `no_cardinality` group has no label pairs and tests the timer mechanisms

The test runs on the max (N) threads for the CPU, and all label groups are selected at random (repeatable)
with each thread having a unique sequence

#### Ryzen 9 7950X (32 threads):

```
Timer precision: 2.25 µs
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           9.143 ms      │ 11.6 ms       │ 10.64 ms      │ 10.61 ms      │ 512     │ 2560
│  │                     alloc:        │               │               │               │         │
│  │                       2.2         │ 2             │ 2             │ 2.029         │         │
│  │                       124.8 B     │ 48 B          │ 48 B          │ 64.32 B       │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0.2         │ 0             │ 0             │ 0.026         │         │
│  │                       153.6 B     │ 0 B           │ 0 B           │ 18.82 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       11.2        │ 11            │ 11            │ 11.02         │         │
│  │                       26.69 KB    │ 26.61 KB      │ 26.61 KB      │ 26.62 KB      │         │
│  ├─ measured_sparse    10.03 ms      │ 11.95 ms      │ 10.82 ms      │ 10.79 ms      │ 512     │ 2560
│  │                     alloc:        │               │               │               │         │
│  │                       2           │ 2             │ 2             │ 2.021         │         │
│  │                       48 B        │ 48 B          │ 48 B          │ 56.39 B       │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.014         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 10.2 B        │         │
│  │                     grow:         │               │               │               │         │
│  │                       11          │ 11            │ 11            │ 11.01         │         │
│  │                       26.61 KB    │ 26.61 KB      │ 26.61 KB      │ 26.62 KB      │         │
│  ├─ metrics            82.64 ms      │ 91.73 ms      │ 85.76 ms      │ 85.8 ms       │ 512     │ 2560
│  │                     alloc:        │               │               │               │         │
│  │                       41553       │ 41561         │ 41481         │ 41709         │         │
│  │                       3.295 MB    │ 3.305 MB      │ 3.307 MB      │ 3.248 MB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       41563       │ 41408         │ 41462         │ 41708         │         │
│  │                       3.304 MB    │ 3.143 MB      │ 3.284 MB      │ 3.252 MB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       94.6        │ 95.6          │ 85.2          │ 113.3         │         │
│  │                       28.43 KB    │ 28.55 KB      │ 27.84 KB      │ 29.96 KB      │         │
│  ├─ prometheus         45.52 ms      │ 54.23 ms      │ 50.21 ms      │ 49.9 ms       │ 512     │ 2560
│  │                     alloc:        │               │               │               │         │
│  │                       460         │ 460           │ 460           │ 460.1         │         │
│  │                       11.48 KB    │ 11.48 KB      │ 11.48 KB      │ 11.49 KB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       459         │ 459           │ 459           │ 459           │         │
│  │                       11.65 KB    │ 11.65 KB      │ 11.64 KB      │ 11.65 KB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       29          │ 29            │ 29            │ 29.02         │         │
│  │                       26.79 KB    │ 26.78 KB      │ 26.77 KB      │ 26.78 KB      │         │
│  ╰─ prometheus_client  53.95 ms      │ 73.95 ms      │ 71.4 ms       │ 70.74 ms      │ 512     │ 2560
│                        alloc:        │               │               │               │         │
│                          1           │ 1             │ 1             │ 1.022         │         │
│                          8 B         │ 8 B           │ 8 B           │ 12.81 B       │         │
│                        dealloc:      │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.007         │         │
│                          0 B         │ 0 B           │ 0 B           │ 5.029 B       │         │
│                        grow:         │               │               │               │         │
│                          11          │ 11            │ 11            │ 11            │         │
│                          26.61 KB    │ 26.61 KB      │ 26.61 KB      │ 26.61 KB      │         │
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           144.7 ms      │ 156.4 ms      │ 147.8 ms      │ 148.5 ms      │ 512     │ 2560
   │                     alloc:        │               │               │               │         │
   │                       2           │ 2             │ 2             │ 2             │         │
   │                       48 B        │ 48 B          │ 48 B          │ 48.15 B       │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 0             │         │
   │                       0 B         │ 0 B           │ 0 B           │ 0.3 B         │         │
   │                     grow:         │               │               │               │         │
   │                       6           │ 6             │ 6             │ 6             │         │
   │                       824 B       │ 824 B         │ 824 B         │ 824.1 B       │         │
   ├─ metrics            233.1 ms      │ 248.4 ms      │ 236.2 ms      │ 236.6 ms      │ 512     │ 2560
   │                     alloc:        │               │               │               │         │
   │                       22013       │ 23869         │ 22868         │ 21940         │         │
   │                       2.498 MB    │ 2.436 MB      │ 2.609 MB      │ 2.324 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       21678       │ 25617         │ 22892         │ 21939         │         │
   │                       2.154 MB    │ 4.288 MB      │ 2.658 MB      │ 2.327 MB      │         │
   │                     grow:         │               │               │               │         │
   │                       27.8        │ 213           │ 99.8          │ 36.45         │         │
   │                       2.375 KB    │ 16.2 KB       │ 7.757 KB      │ 3.025 KB      │         │
   ├─ prometheus         145.7 ms      │ 156.7 ms      │ 147 ms        │ 147.5 ms      │ 512     │ 2560
   │                     alloc:        │               │               │               │         │
   │                       30          │ 30            │ 30            │ 30            │         │
   │                       2.049 KB    │ 2.049 KB      │ 2.049 KB      │ 2.049 KB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       29          │ 29            │ 29            │ 29            │         │
   │                       2.051 KB    │ 2.05 KB       │ 2.05 KB       │ 2.05 KB       │         │
   │                     grow:         │               │               │               │         │
   │                       7           │ 7             │ 7             │ 7             │         │
   │                       834 B       │ 833 B         │ 833.6 B       │ 833.4 B       │         │
   ╰─ prometheus_client  2.074 s       │ 2.149 s       │ 2.109 s       │ 2.11 s        │ 512     │ 2560
                         alloc:        │               │               │               │         │
                           1           │ 1             │ 1             │ 1             │         │
                           8 B         │ 8 B           │ 8 B           │ 8 B           │         │
                         grow:         │               │               │               │         │
                           6           │ 6             │ 6             │ 6             │         │
                           824 B       │ 824 B         │ 824 B         │ 824 B         │         │
```

#### Macbook Pro M2 Max (12 threads):

```
Timer precision: 41 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           6.044 ms      │ 14.43 ms      │ 10.91 ms      │ 10.92 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.172         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 7.561 B       │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.009         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 3.657 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.894         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 2.165 KB      │         │
│  ├─ measured_sparse    8.487 ms      │ 11.62 ms      │ 10.12 ms      │ 10.09 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.168         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 4.761 B       │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.001         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 0.761 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.916         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 2.218 KB      │         │
│  ├─ metrics            20.01 ms      │ 29.88 ms      │ 23.19 ms      │ 23.31 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 3460          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 253.7 KB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 3459          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 253 KB        │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 10.1          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 2.517 KB      │         │
│  ├─ prometheus         59.25 ms      │ 76.23 ms      │ 75.31 ms      │ 74.65 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 460           │ 0             │ 38.33         │         │
│  │                       0 B         │ 11.48 KB      │ 0 B           │ 957.3 B       │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 459           │ 0             │ 38.25         │         │
│  │                       0 B         │ 11.65 KB      │ 0 B           │ 970.7 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 29            │ 0             │ 2.416         │         │
│  │                       0 B         │ 26.79 KB      │ 0 B           │ 2.232 KB      │         │
│  ╰─ prometheus_client  71.87 ms      │ 74.84 ms      │ 74.18 ms      │ 74.07 ms      │ 504     │ 2520
│                        alloc:        │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.083         │         │
│                          0 B         │ 0 B           │ 0 B           │ 0.666 B       │         │
│                        grow:         │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.916         │         │
│                          0 B         │ 0 B           │ 0 B           │ 2.218 KB      │         │
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           86.05 ms      │ 92.41 ms      │ 91.69 ms      │ 91.5 ms       │ 504     │ 2520
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 0             │ 0.169         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 4.914 B       │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 0.002         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 0.914 B       │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 0             │ 0.5           │         │
   │                       0 B         │ 0 B           │ 0 B           │ 68.66 B       │         │
   ├─ metrics            109.1 ms      │ 165 ms        │ 160.9 ms      │ 156.4 ms      │ 504     │ 2520
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 0             │ 1738          │         │
   │                       0 B         │ 0 B           │ 0 B           │ 91.02 KB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 1743          │         │
   │                       0 B         │ 0 B           │ 0 B           │ 96.31 KB      │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 0             │ 3.892         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 315 B         │         │
   ├─ prometheus         58.45 ms      │ 84.68 ms      │ 81.73 ms      │ 79.27 ms      │ 504     │ 2520
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 0             │ 2.5           │         │
   │                       0 B         │ 0 B           │ 0 B           │ 170.7 B       │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 2.416         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 170.9 B       │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 0             │ 0.583         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 69.5 B        │         │
   ╰─ prometheus_client  28.14 ms      │ 32.68 ms      │ 30.78 ms      │ 30.93 ms      │ 504     │ 2520
                         alloc:        │               │               │               │         │
                           0           │ 0             │ 0             │ 0.083         │         │
                           0 B         │ 0 B           │ 0 B           │ 0.666 B       │         │
                         grow:         │               │               │               │         │
                           0           │ 0             │ 0             │ 0.5           │         │
                           0 B         │ 0 B           │ 0 B           │ 68.66 B       │         │
```
