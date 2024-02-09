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
Timer precision: 2.29 µs
counters                 fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           2.266 ms      │ 3.721 ms      │ 3.151 ms      │ 3.188 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       2           │ 2             │ 2             │ 2             │         │
│  │                       48 B        │ 48 B          │ 48 B          │ 48 B          │         │
│  │                     grow:         │               │               │               │         │
│  │                       7           │ 7             │ 7             │ 7             │         │
│  │                       1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ measured_sparse    6.316 ms      │ 15.04 ms      │ 13.63 ms      │ 13.09 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       66          │ 66            │ 66            │ 66            │         │
│  │                       1.584 KB    │ 1.584 KB      │ 1.584 KB      │ 1.584 KB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       64          │ 64            │ 64            │ 64            │         │
│  │                       1.536 KB    │ 1.536 KB      │ 1.536 KB      │ 1.536 KB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       7           │ 7             │ 7             │ 7             │         │
│  │                       1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ metrics            16.02 ms      │ 23.83 ms      │ 17.38 ms      │ 18.19 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       72299       │ 72299         │ 72299         │ 72299         │         │
│  │                       4.624 MB    │ 4.624 MB      │ 4.624 MB      │ 4.624 MB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       72298       │ 72298         │ 72298         │ 72298         │         │
│  │                       4.625 MB    │ 4.625 MB      │ 4.625 MB      │ 4.625 MB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       61          │ 61            │ 61            │ 61            │         │
│  │                       2.634 KB    │ 2.634 KB      │ 2.634 KB      │ 2.634 KB      │         │
│  ├─ prometheus         37.8 ms       │ 52.55 ms      │ 49.65 ms      │ 49.31 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       118         │ 167.2         │ 118           │ 118.1         │         │
│  │                       6.589 KB    │ 9.148 KB      │ 6.589 KB      │ 6.599 KB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       117         │ 132.6         │ 117           │ 117           │         │
│  │                       6.581 KB    │ 7.702 KB      │ 6.581 KB      │ 6.585 KB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       7           │ 13            │ 7             │ 7.014         │         │
│  │                       1.656 KB    │ 1.692 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ╰─ prometheus_client  10.12 ms      │ 21.28 ms      │ 19.1 ms       │ 18.81 ms      │ 504     │ 2520
│                        alloc:        │               │               │               │         │
│                          1           │ 1             │ 1             │ 1.008         │         │
│                          8 B         │ 8 B           │ 8 B           │ 8.982 B       │         │
│                        dealloc:      │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.001         │         │
│                          0 B         │ 0 B           │ 0 B           │ 0.385 B       │         │
│                        grow:         │               │               │               │         │
│                          7           │ 7             │ 7             │ 7             │         │
│                          1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
╰─ high_cardinality                    │               │               │               │         │
   ├─ measured           12.86 ms      │ 126.6 ms      │ 59.47 ms      │ 64.23 ms      │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       164         │ 66            │ 66            │ 76.16         │         │
   │                       1.635 MB    │ 1.584 KB      │ 1.584 KB      │ 453.9 KB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       159         │ 64            │ 64            │ 73.78         │         │
   │                       294.7 KB    │ 1.536 KB      │ 1.536 KB      │ 183.6 KB      │         │
   │                     grow:         │               │               │               │         │
   │                       18.5        │ 22            │ 21            │ 20.77         │         │
   │                       5.111 MB    │ 54.52 MB      │ 27.26 MB      │ 30.48 MB      │         │
   ├─ metrics            32.47 ms      │ 1.495 s       │ 617.2 ms      │ 651.8 ms      │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       495791      │ 8266972       │ 4035831       │ 4426565       │         │
   │                       22.53 MB    │ 360.9 MB      │ 177.7 MB      │ 205.2 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       488594      │ 8259889       │ 4028687       │ 4419420       │         │
   │                       23.79 MB    │ 391 MB        │ 192.2 MB      │ 220.9 MB      │         │
   │                     grow:         │               │               │               │         │
   │                       109417      │ 1875621       │ 913983        │ 1002786       │         │
   │                       5.187 MB    │ 85.03 MB      │ 42.13 MB      │ 44.41 MB      │         │
   ├─ prometheus         89.23 ms      │ 2.801 s       │ 914.1 ms      │ 1.022 s       │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       248071      │ 3042368       │ 1651153       │ 1649280       │         │
   │                       12.46 MB    │ 157 MB        │ 85.08 MB      │ 85.07 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       222877      │ 3017503       │ 1626172       │ 1624276       │         │
   │                       11.61 MB    │ 156.2 MB      │ 84.23 MB      │ 84.18 MB      │         │
   │                     grow:         │               │               │               │         │
   │                       5416        │ 5350          │ 5373          │ 5378          │         │
   │                       3.181 MB    │ 67.14 MB      │ 29.39 MB      │ 29.42 MB      │         │
   ╰─ prometheus_client  1.314 s       │ 1.942 s       │ 1.596 s       │ 1.603 s       │ 108     │ 216
                         alloc:        │               │               │               │         │
                           7052        │ 6563          │ 6975          │ 6858          │         │
                           1.196 MB    │ 557.8 KB      │ 705.1 KB      │ 938.4 KB      │         │
                         dealloc:      │               │               │               │         │
                           3457        │ 3016          │ 3398          │ 3285          │         │
                           894.8 KB    │ 491.2 KB      │ 638 KB        │ 733 KB        │         │
                         grow:         │               │               │               │         │
                           18.5        │ 22            │ 21            │ 20.78         │         │
                           5.111 MB    │ 54.52 MB      │ 27.26 MB      │ 30.51 MB      │         │
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

...

#### Macbook Pro M2 Max (12 threads):

```
Timer precision: 41 ns
histograms               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           8.611 ms      │ 10.91 ms      │ 10.61 ms      │ 10.51 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.162         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 3.904 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.894         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 2.165 KB      │         │
│  ├─ measured_sparse    9.604 ms      │ 11.14 ms      │ 10.42 ms      │ 10.39 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.166         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 4 B           │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.916         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 2.218 KB      │         │
│  ├─ metrics            19.16 ms      │ 23.47 ms      │ 21.63 ms      │ 21.6 ms       │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 3461          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 253.9 KB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 3460          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 253.5 KB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 10.11         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 2.525 KB      │         │
│  ├─ prometheus         64.42 ms      │ 77.35 ms      │ 76.59 ms      │ 76.08 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 38.4          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 961.3 B       │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 38.26         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 971.9 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 2.43          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 2.232 KB      │         │
│  ╰─ prometheus_client  4.772 ms      │ 72.08 ms      │ 69.11 ms      │ 68.4 ms       │ 504     │ 2520
│                        alloc:        │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.084         │         │
│                          0 B         │ 0 B           │ 0 B           │ 0.749 B       │         │
│                        grow:         │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.916         │         │
│                          0 B         │ 0 B           │ 0 B           │ 2.218 KB      │         │
╰─ no_cardinality                      │               │               │               │         │
   ├─ measured           29.74 ms      │ 35.43 ms      │ 34.22 ms      │ 33.89 ms      │ 504     │ 2520
   │                     alloc:        │               │               │               │         │
   │                       0           │ 2             │ 0             │ 0.166         │         │
   │                       0 B         │ 48 B          │ 0 B           │ 4 B           │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 6             │ 0             │ 0.5           │         │
   │                       0 B         │ 824 B         │ 0 B           │ 68.66 B       │         │
   ├─ metrics            100.5 ms      │ 154.4 ms      │ 141.9 ms      │ 139.9 ms      │ 504     │ 2520
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 0             │ 1741          │         │
   │                       0 B         │ 0 B           │ 0 B           │ 93.61 KB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 1738          │         │
   │                       0 B         │ 0 B           │ 0 B           │ 90.62 KB      │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 0             │ 3.934         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 318 B         │         │
   ├─ prometheus         59.91 ms      │ 84.64 ms      │ 77.11 ms      │ 76.66 ms      │ 504     │ 2520
   │                     alloc:        │               │               │               │         │
   │                       0           │ 30            │ 0             │ 2.5           │         │
   │                       0 B         │ 2.049 KB      │ 0 B           │ 170.7 B       │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 29            │ 0             │ 2.416         │         │
   │                       0 B         │ 2.051 KB      │ 0 B           │ 170.9 B       │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 7             │ 0             │ 0.583         │         │
   │                       0 B         │ 834.6 B       │ 0 B           │ 69.51 B       │         │
   ╰─ prometheus_client  26.92 ms      │ 32.95 ms      │ 31.09 ms      │ 31.09 ms      │ 504     │ 2520
                         alloc:        │               │               │               │         │
                           0           │ 0             │ 0             │ 0.083         │         │
                           0 B         │ 0 B           │ 0 B           │ 0.666 B       │         │
                         grow:         │               │               │               │         │
                           0           │ 0             │ 0             │ 0.5           │         │
                           0 B         │ 0 B           │ 0 B           │ 68.66 B       │         │
```
