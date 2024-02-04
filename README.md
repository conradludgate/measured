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
when it comes to both speed and allocations.

The `fixed_cardinality` group has only 2 label pairs and 18 distinct label groups.
The `high_cardinality` group has 3 label pairs and 18,000 distinct label groups.

### Macbook Pro M2 Max (12 threads):

```
Timer precision: 41 ns
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                │               │               │               │         │
│  ├─ measured        18.19 ms      │ 27.91 ms      │ 27.07 ms      │ 26.48 ms      │ 504     │ 2520
│  │                  alloc:        │               │               │               │         │
│  │                    0           │ 0             │ 0             │ 0.162         │         │
│  │                    0 B         │ 0 B           │ 0 B           │ 3.904 B       │         │
│  │                  grow:         │               │               │               │         │
│  │                    0           │ 0             │ 0             │ 0.569         │         │
│  │                    0 B         │ 0 B           │ 0 B           │ 134.7 B       │         │
│  ├─ metrics         32.02 ms      │ 39.51 ms      │ 36.89 ms      │ 36.85 ms      │ 504     │ 2520
│  │                  alloc:        │               │               │               │         │
│  │                    0           │ 0             │ 0             │ 6024          │         │
│  │                    0 B         │ 0 B           │ 0 B           │ 385.3 KB      │         │
│  │                  dealloc:      │               │               │               │         │
│  │                    0           │ 0             │ 0             │ 6024          │         │
│  │                    0 B         │ 0 B           │ 0 B           │ 385.4 KB      │         │
│  │                  grow:         │               │               │               │         │
│  │                    0           │ 0             │ 0             │ 5.083         │         │
│  │                    0 B         │ 0 B           │ 0 B           │ 219.5 B       │         │
│  ╰─ prometheus      112.4 ms      │ 143.1 ms      │ 136.3 ms      │ 135.3 ms      │ 504     │ 2520
│                     alloc:        │               │               │               │         │
│                       0           │ 0             │ 59            │ 9.833         │         │
│                       0 B         │ 0 B           │ 3.294 KB      │ 549 B         │         │
│                     dealloc:      │               │               │               │         │
│                       0           │ 0             │ 58.4          │ 9.75          │         │
│                       0 B         │ 0 B           │ 3.29 KB       │ 548.4 B       │         │
│                     grow:         │               │               │               │         │
│                       0           │ 0             │ 3.4           │ 0.583         │         │
│                       0 B         │ 0 B           │ 828 B         │ 138 B         │         │
╰─ high_cardinality                 │               │               │               │         │
   ├─ measured        1.305 s       │ 3.213 s       │ 1.917 s       │ 2.108 s       │ 24      │ 48
   │                  alloc:        │               │               │               │         │
   │                    0           │ 0             │ 0             │ 3.083         │         │
   │                    0 B         │ 0 B           │ 0 B           │ 285.7 KB      │         │
   │                  dealloc:      │               │               │               │         │
   │                    0           │ 0             │ 0             │ 2.916         │         │
   │                    0 B         │ 0 B           │ 0 B           │ 142.8 KB      │         │
   │                  grow:         │               │               │               │         │
   │                    0           │ 0             │ 0             │ 1.854         │         │
   │                    0 B         │ 0 B           │ 0 B           │ 6.247 MB      │         │
   ├─ metrics         1.3 s         │ 5.652 s       │ 2.988 s       │ 3.153 s       │ 24      │ 48
   │                  alloc:        │               │               │               │         │
   │                    0           │ 0             │ 0             │ 939753        │         │
   │                    0 B         │ 0 B           │ 0 B           │ 41.07 MB      │         │
   │                  dealloc:      │               │               │               │         │
   │                    0           │ 0             │ 0             │ 933848        │         │
   │                    0 B         │ 0 B           │ 0 B           │ 44.17 MB      │         │
   │                  grow:         │               │               │               │         │
   │                    0           │ 0             │ 0             │ 210875        │         │
   │                    0 B         │ 0 B           │ 0 B           │ 9.677 MB      │         │
   ╰─ prometheus      1.323 s       │ 6.181 s       │ 2.987 s       │ 3.344 s       │ 24      │ 48
                      alloc:        │               │               │               │         │
                        0           │ 0             │ 0             │ 375835        │         │
                        0 B         │ 0 B           │ 0 B           │ 19.14 MB      │         │
                      dealloc:      │               │               │               │         │
                        0           │ 0             │ 0             │ 355183        │         │
                        0 B         │ 0 B           │ 0 B           │ 18.44 MB      │         │
                      grow:         │               │               │               │         │
                        0           │ 0             │ 0             │ 4427          │         │
                        0 B         │ 0 B           │ 0 B           │ 6.32 MB       │         │
```

### Ryzen 9 7950X (32 threads):

```
Timer precision: 2.3 µs
counters              fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                │               │               │               │         │
│  ├─ measured        7.488 ms      │ 11.69 ms      │ 10.44 ms      │ 10.34 ms      │ 512     │ 2560
│  │                  alloc:        │               │               │               │         │
│  │                    2           │ 2             │ 2             │ 2             │         │
│  │                    48 B        │ 48 B          │ 48 B          │ 48 B          │         │
│  │                  grow:         │               │               │               │         │
│  │                    7           │ 7             │ 7             │ 7             │         │
│  │                    1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ metrics         21.6 ms       │ 30.44 ms      │ 23.38 ms      │ 23.52 ms      │ 512     │ 2560
│  │                  alloc:        │               │               │               │         │
│  │                    72299       │ 72299         │ 72299         │ 72299         │         │
│  │                    4.624 MB    │ 4.624 MB      │ 4.624 MB      │ 4.624 MB      │         │
│  │                  dealloc:      │               │               │               │         │
│  │                    72298       │ 72298         │ 72298         │ 72298         │         │
│  │                    4.625 MB    │ 4.625 MB      │ 4.625 MB      │ 4.625 MB      │         │
│  │                  grow:         │               │               │               │         │
│  │                    61          │ 61            │ 61            │ 61            │         │
│  │                    2.634 KB    │ 2.634 KB      │ 2.634 KB      │ 2.634 KB      │         │
│  ╰─ prometheus      51.81 ms      │ 62.79 ms      │ 58.47 ms      │ 58.23 ms      │ 512     │ 2560
│                     alloc:        │               │               │               │         │
│                       118         │ 118           │ 118           │ 118.1         │         │
│                       6.589 KB    │ 6.589 KB      │ 6.589 KB      │ 6.596 KB      │         │
│                     dealloc:      │               │               │               │         │
│                       117         │ 117           │ 117           │ 117           │         │
│                       6.581 KB    │ 6.581 KB      │ 6.581 KB      │ 6.584 KB      │         │
│                     grow:         │               │               │               │         │
│                       7           │ 7             │ 7             │ 7.014         │         │
│                       1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
╰─ high_cardinality                 │               │               │               │         │
   ├─ measured        2.454 s       │ 6.44 s        │ 4.454 s       │ 4.454 s       │ 32      │ 64
   │                  alloc:        │               │               │               │         │
   │                    55          │ 40.5          │ 51            │ 46.68         │         │
   │                    2.208 MB    │ 270.9 MB      │ 5.855 MB      │ 20.05 MB      │         │
   │                  dealloc:      │               │               │               │         │
   │                    52.5        │ 38            │ 49            │ 44.32         │         │
   │                    1.088 MB    │ 1.269 MB      │ 831 KB        │ 1.638 MB      │         │
   │                  grow:         │               │               │               │         │
   │                    22.5        │ 23.5          │ 23.5          │ 23.31         │         │
   │                    81.78 MB    │ 163.5 MB      │ 163.5 MB      │ 146.5 MB      │         │
   ├─ metrics         4.521 s       │ 9.559 s       │ 6.474 s       │ 6.829 s       │ 32      │ 64
   │                  alloc:        │               │               │               │         │
   │                    16659948    │ 18877063      │ 17269948      │ 17642773      │         │
   │                    852 MB      │ 917.3 MB      │ 872.5 MB      │ 881.5 MB      │         │
   │                  dealloc:      │               │               │               │         │
   │                    16589366    │ 18807060      │ 17199425      │ 17572351      │         │
   │                    907.2 MB    │ 980.4 MB      │ 928.6 MB      │ 939.9 MB      │         │
   │                  grow:         │               │               │               │         │
   │                    3753951     │ 4257970       │ 3892601       │ 3977356       │         │
   │                    142.8 MB    │ 151 MB        │ 145.1 MB      │ 146.4 MB      │         │
   ╰─ prometheus      9.044 s       │ 18.33 s       │ 13.63 s       │ 13.71 s       │ 32      │ 64
                      alloc:        │               │               │               │         │
                        6680198     │ 7253365       │ 6983598       │ 6986195       │         │
                        342.1 MB    │ 371.8 MB      │ 357.8 MB      │ 359.1 MB      │         │
                      dealloc:      │               │               │               │         │
                        6433328     │ 7008028       │ 6736970       │ 6739722       │         │
                        333.8 MB    │ 363.5 MB      │ 349.5 MB      │ 350.2 MB      │         │
                      grow:         │               │               │               │         │
                        52930       │ 52601         │ 52880         │ 52845         │         │
                        101 MB      │ 101 MB        │ 101 MB        │ 101 MB        │         │
```
