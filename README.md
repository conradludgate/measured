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
Timer precision: 2.17 µs
counters               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                 │               │               │               │         │
│  ├─ measured         4.749 ms      │ 9.478 ms      │ 8.783 ms      │ 8.575 ms      │ 512     │ 2560
│  │                   alloc:        │               │               │               │         │
│  │                     2           │ 2             │ 2             │ 2             │         │
│  │                     48 B        │ 48 B          │ 48 B          │ 48 B          │         │
│  │                   grow:         │               │               │               │         │
│  │                     7           │ 7             │ 7             │ 7             │         │
│  │                     1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ measured_sparse  11.3 ms       │ 15.97 ms      │ 13.27 ms      │ 13.34 ms      │ 512     │ 2560
│  │                   alloc:        │               │               │               │         │
│  │                     130         │ 130           │ 130           │ 130           │         │
│  │                     3.12 KB     │ 3.12 KB       │ 3.12 KB       │ 3.122 KB      │         │
│  │                   dealloc:      │               │               │               │         │
│  │                     128         │ 128           │ 128           │ 128           │         │
│  │                     3.072 KB    │ 3.072 KB      │ 3.072 KB      │ 3.072 KB      │         │
│  │                   grow:         │               │               │               │         │
│  │                     7           │ 7             │ 7             │ 7             │         │
│  │                     1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ metrics          17.26 ms      │ 28.91 ms      │ 22.93 ms      │ 23.04 ms      │ 512     │ 2560
│  │                   alloc:        │               │               │               │         │
│  │                     72299       │ 72299         │ 72299         │ 72299         │         │
│  │                     4.624 MB    │ 4.624 MB      │ 4.624 MB      │ 4.624 MB      │         │
│  │                   dealloc:      │               │               │               │         │
│  │                     72298       │ 72298         │ 72298         │ 72298         │         │
│  │                     4.625 MB    │ 4.625 MB      │ 4.625 MB      │ 4.625 MB      │         │
│  │                   grow:         │               │               │               │         │
│  │                     61          │ 61            │ 61            │ 61            │         │
│  │                     2.634 KB    │ 2.634 KB      │ 2.634 KB      │ 2.634 KB      │         │
│  ╰─ prometheus       70 ms         │ 78.98 ms      │ 76.17 ms      │ 75.79 ms      │ 512     │ 2560
│                      alloc:        │               │               │               │         │
│                        118         │ 118           │ 118           │ 118.1         │         │
│                        6.589 KB    │ 6.589 KB      │ 6.589 KB      │ 6.598 KB      │         │
│                      dealloc:      │               │               │               │         │
│                        117         │ 117           │ 117           │ 117           │         │
│                        6.581 KB    │ 6.581 KB      │ 6.581 KB      │ 6.585 KB      │         │
│                      grow:         │               │               │               │         │
│                        7           │ 7             │ 7             │ 7.014         │         │
│                        1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
╰─ high_cardinality                  │               │               │               │         │
   ├─ measured         39.08 ms      │ 236.1 ms      │ 125.2 ms      │ 122.2 ms      │ 128     │ 256
   │                   alloc:        │               │               │               │         │
   │                     201.5       │ 133.5         │ 138           │ 145.8         │         │
   │                     1.299 MB    │ 719.9 KB      │ 630.4 KB      │ 469.3 KB      │         │
   │                   dealloc:      │               │               │               │         │
   │                     198         │ 131.5         │ 136           │ 143.2         │         │
   │                     125.2 KB    │ 361.5 KB      │ 316.8 KB      │ 177.5 KB      │         │
   │                   grow:         │               │               │               │         │
   │                     19.5        │ 22            │ 21.5          │ 21.14         │         │
   │                     10.22 MB    │ 54.52 MB      │ 40.89 MB      │ 36.79 MB      │         │
   ├─ metrics          187.5 ms      │ 2.619 s       │ 1.19 s        │ 1.271 s       │ 128     │ 256
   │                   alloc:        │               │               │               │         │
   │                     1381795     │ 9496832       │ 5472939       │ 5498082       │         │
   │                     63.58 MB    │ 399.4 MB      │ 251 MB        │ 241.8 MB      │         │
   │                   dealloc:      │               │               │               │         │
   │                     1374611     │ 9489767       │ 5465801       │ 5490947       │         │
   │                     67.99 MB    │ 432.8 MB      │ 270.1 MB      │ 261.5 MB      │         │
   │                   grow:         │               │               │               │         │
   │                     310784      │ 2155139       │ 1240600       │ 1246316       │         │
   │                     15.27 MB    │ 89.58 MB      │ 54.25 MB      │ 53.49 MB      │         │
   ╰─ prometheus       650.3 ms      │ 8.064 s       │ 2.807 s       │ 3.283 s       │ 128     │ 256
                       alloc:        │               │               │               │         │
                         675331      │ 3476277       │ 2075552       │ 2074924       │         │
                         34.59 MB    │ 179.5 MB      │ 107 MB        │ 107 MB        │         │
                       dealloc:      │               │               │               │         │
                         650228      │ 3451440       │ 2050600       │ 2049954       │         │
                         33.74 MB    │ 178.7 MB      │ 106.2 MB      │ 106.2 MB      │         │
                       grow:         │               │               │               │         │
                         5399        │ 5344          │ 5368          │ 5371          │         │
                         12.61 MB    │ 67.14 MB      │ 33.59 MB      │ 37.65 MB      │         │
```
