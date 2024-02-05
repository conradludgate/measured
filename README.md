# measured

A better\* metrics crate

## Goals

1. Low memory usage and low fragmentation
2. Fast
3. ~~Less atomics, more thread locals~~
4. Fairly ergonomic
5. Eventual Consistency

## Non-goals

- facade macros
- strong consistency

## Benchmark results

The benchmark runs on multiple threads a simple counter increment, with various labels,
and then samples/encodes the values into the Prometheus text format. This crate outperforms both `metrics` and `prometheus`
when it comes to both speed and allocations. The `prometheus_client` crate shares some goals to this library and performs well, but still not as fast.

The `fixed_cardinality` group has only 2 label pairs and 18 distinct label groups.
The `high_cardinality` group has 3 label pairs and 18,000 distinct label groups.

### Macbook Pro M2 Max (12 threads):

```
Timer precision: 41 ns
counters                 fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                   │               │               │               │         │
│  ├─ measured           27.21 ms      │ 29.65 ms      │ 29.44 ms      │ 29.37 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.162         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 3.904 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.569         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 134.7 B       │         │
│  ├─ measured_sparse    13.4 ms       │ 16.42 ms      │ 14.99 ms      │ 15.09 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 5.5           │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 132 B         │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 5.333         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 128 B         │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.583         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 138 B         │         │
│  ├─ metrics            32.79 ms      │ 38.76 ms      │ 36.82 ms      │ 36.74 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 6024          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 385.3 KB      │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 6024          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 385.4 KB      │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 5.083         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 219.5 B       │         │
│  ├─ prometheus         143.3 ms      │ 148 ms        │ 147.1 ms      │ 146.9 ms      │ 504     │ 2520
│  │                     alloc:        │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 9.833         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 549 B         │         │
│  │                     dealloc:      │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 9.75          │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 548.4 B       │         │
│  │                     grow:         │               │               │               │         │
│  │                       0           │ 0             │ 0             │ 0.583         │         │
│  │                       0 B         │ 0 B           │ 0 B           │ 138 B         │         │
│  ╰─ prometheus_client  134.7 ms      │ 140 ms        │ 138.9 ms      │ 138.7 ms      │ 504     │ 2520
│                        alloc:        │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.092         │         │
│                          0 B         │ 0 B           │ 0 B           │ 1.636 B       │         │
│                        dealloc:      │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.001         │         │
│                          0 B         │ 0 B           │ 0 B           │ 0.376 B       │         │
│                        grow:         │               │               │               │         │
│                          0           │ 0             │ 0             │ 0.583         │         │
│                          0 B         │ 0 B           │ 0 B           │ 138 B         │         │
╰─ high_cardinality                    │               │               │               │         │
   ├─ measured           11.66 ms      │ 152.2 ms      │ 68.63 ms      │ 74 ms         │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 0             │ 6.361         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 32.96 KB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 6.185         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 15.93 KB      │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 0             │ 1.731         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 2.54 MB       │         │
   ├─ metrics            96.21 ms      │ 2.093 s       │ 940 ms        │ 963.8 ms      │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 2217252       │ 371087        │         │
   │                       0 B         │ 0 B           │ 93.89 MB      │ 17.08 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 2213683       │ 370492        │         │
   │                       0 B         │ 0 B           │ 101.8 MB      │ 18.4 MB       │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 502296        │ 84067         │         │
   │                       0 B         │ 0 B           │ 21.8 MB       │ 3.781 MB      │         │
   ├─ prometheus         70.52 ms      │ 2.698 s       │ 1.078 s       │ 1.108 s       │ 108     │ 216
   │                     alloc:        │               │               │               │         │
   │                       0           │ 0             │ 0             │ 137749        │         │
   │                       0 B         │ 0 B           │ 0 B           │ 7.098 MB      │         │
   │                     dealloc:      │               │               │               │         │
   │                       0           │ 0             │ 0             │ 135665        │         │
   │                       0 B         │ 0 B           │ 0 B           │ 7.028 MB      │         │
   │                     grow:         │               │               │               │         │
   │                       0           │ 0             │ 0             │ 448.2         │         │
   │                       0 B         │ 0 B           │ 0 B           │ 2.469 MB      │         │
   ╰─ prometheus_client  116.9 ms      │ 740.4 ms      │ 376.2 ms      │ 378.2 ms      │ 108     │ 216
                         alloc:        │               │               │               │         │
                           0           │ 0             │ 0             │ 458.1         │         │
                           0 B         │ 0 B           │ 0 B           │ 11.59 KB      │         │
                         dealloc:      │               │               │               │         │
                           0           │ 0             │ 0             │ 160.3         │         │
                           0 B         │ 0 B           │ 0 B           │ 6.011 KB      │         │
                         grow:         │               │               │               │         │
                           0           │ 0             │ 0             │ 1.731         │         │
                           0 B         │ 0 B           │ 0 B           │ 2.54 MB       │         │
```

### Ryzen 9 7950X (32 threads):

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
