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
counters               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                 │               │               │               │         │
│  ├─ measured         25.12 ms      │ 28.04 ms      │ 27.83 ms      │ 27.69 ms      │ 504     │ 2520
│  │                   alloc:        │               │               │               │         │
│  │                     0           │ 0             │ 1             │ 0.162         │         │
│  │                     0 B         │ 0 B           │ 24 B          │ 3.904 B       │         │
│  │                   grow:         │               │               │               │         │
│  │                     0           │ 0             │ 3.4           │ 0.569         │         │
│  │                     0 B         │ 0 B           │ 828 B         │ 134.7 B       │         │
│  ├─ measured_sparse  13.79 ms      │ 16.18 ms      │ 14.79 ms      │ 14.86 ms      │ 504     │ 2520
│  │                   alloc:        │               │               │               │         │
│  │                     0           │ 0             │ 0             │ 5.507         │         │
│  │                     0 B         │ 0 B           │ 0 B           │ 132.5 B       │         │
│  │                   dealloc:      │               │               │               │         │
│  │                     0           │ 0             │ 0             │ 5.333         │         │
│  │                     0 B         │ 0 B           │ 0 B           │ 128 B         │         │
│  │                   grow:         │               │               │               │         │
│  │                     0           │ 0             │ 0             │ 0.583         │         │
│  │                     0 B         │ 0 B           │ 0 B           │ 138 B         │         │
│  ├─ metrics          36.7 ms       │ 43.75 ms      │ 41.51 ms      │ 41.56 ms      │ 504     │ 2520
│  │                   alloc:        │               │               │               │         │
│  │                     0           │ 0             │ 36149         │ 6024          │         │
│  │                     0 B         │ 0 B           │ 2.312 MB      │ 385.3 KB      │         │
│  │                   dealloc:      │               │               │               │         │
│  │                     0           │ 0             │ 36149         │ 6024          │         │
│  │                     0 B         │ 0 B           │ 2.312 MB      │ 385.4 KB      │         │
│  │                   grow:         │               │               │               │         │
│  │                     0           │ 0             │ 30.4          │ 5.083         │         │
│  │                     0 B         │ 0 B           │ 1.317 KB      │ 219.5 B       │         │
│  ╰─ prometheus       136.6 ms      │ 148.1 ms      │ 146.8 ms      │ 146.2 ms      │ 504     │ 2520
│                      alloc:        │               │               │               │         │
│                        0           │ 0             │ 0             │ 9.833         │         │
│                        0 B         │ 0 B           │ 0 B           │ 549 B         │         │
│                      dealloc:      │               │               │               │         │
│                        0           │ 0             │ 0             │ 9.75          │         │
│                        0 B         │ 0 B           │ 0 B           │ 548.4 B       │         │
│                      grow:         │               │               │               │         │
│                        0           │ 0             │ 0             │ 0.583         │         │
│                        0 B         │ 0 B           │ 0 B           │ 138 B         │         │
╰─ high_cardinality                  │               │               │               │         │
   ├─ measured         156.7 ms      │ 477.5 ms      │ 271.3 ms      │ 281.3 ms      │ 24      │ 48
   │                   alloc:        │               │               │               │         │
   │                     0           │ 0             │ 0             │ 9.625         │         │
   │                     0 B         │ 0 B           │ 0 B           │ 276.1 KB      │         │
   │                   dealloc:      │               │               │               │         │
   │                     0           │ 0             │ 0             │ 9.187         │         │
   │                     0 B         │ 0 B           │ 0 B           │ 138 KB        │         │
   │                   grow:         │               │               │               │         │
   │                     0           │ 0             │ 0             │ 1.854         │         │
   │                     0 B         │ 0 B           │ 0 B           │ 6.247 MB      │         │
   ├─ metrics          1.244 s       │ 5.287 s       │ 2.83 s        │ 3.028 s       │ 24      │ 48
   │                   alloc:        │               │               │               │         │
   │                     0           │ 0             │ 0             │ 989854        │         │
   │                     0 B         │ 0 B           │ 0 B           │ 42.78 MB      │         │
   │                   dealloc:      │               │               │               │         │
   │                     0           │ 0             │ 0             │ 983959        │         │
   │                     0 B         │ 0 B           │ 0 B           │ 45.94 MB      │         │
   │                   grow:         │               │               │               │         │
   │                     0           │ 0             │ 0             │ 222264        │         │
   │                     0 B         │ 0 B           │ 0 B           │ 9.863 MB      │         │
   ╰─ prometheus       1.266 s       │ 5.912 s       │ 2.831 s       │ 3.203 s       │ 24      │ 48
                       alloc:        │               │               │               │         │
                         0           │ 6325403       │ 0             │ 378459        │         │
                         0 B         │ 323.7 MB      │ 0 B           │ 19.28 MB      │         │
                       dealloc:      │               │               │               │         │
                         0           │ 6079744       │ 0             │ 357809        │         │
                         0 B         │ 315.4 MB      │ 0 B           │ 18.58 MB      │         │
                       grow:         │               │               │               │         │
                         0           │ 52663         │ 0             │ 4426          │         │
                         0 B         │ 101 MB        │ 0 B           │ 6.32 MB       │         │
```

### Ryzen 9 7950X (32 threads):

```
Timer precision: 2.3 µs
counters               fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ fixed_cardinality                 │               │               │               │         │
│  ├─ measured         7.366 ms      │ 12.65 ms      │ 10.58 ms      │ 10.61 ms      │ 512     │ 2560
│  │                   alloc:        │               │               │               │         │
│  │                     2           │ 2             │ 2             │ 2             │         │
│  │                     48 B        │ 48 B          │ 48 B          │ 48 B          │         │
│  │                   grow:         │               │               │               │         │
│  │                     7           │ 7             │ 7             │ 7             │         │
│  │                     1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ measured_sparse  13.15 ms      │ 20.41 ms      │ 13.73 ms      │ 14 ms         │ 512     │ 2560
│  │                   alloc:        │               │               │               │         │
│  │                     130         │ 130           │ 130           │ 130           │         │
│  │                     3.12 KB     │ 3.12 KB       │ 3.12 KB       │ 3.122 KB      │         │
│  │                   dealloc:      │               │               │               │         │
│  │                     128         │ 128           │ 128           │ 128           │         │
│  │                     3.072 KB    │ 3.072 KB      │ 3.072 KB      │ 3.072 KB      │         │
│  │                   grow:         │               │               │               │         │
│  │                     7           │ 7             │ 7             │ 7             │         │
│  │                     1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
│  ├─ metrics          19.63 ms      │ 30.09 ms      │ 22.82 ms      │ 22.95 ms      │ 512     │ 2560
│  │                   alloc:        │               │               │               │         │
│  │                     72299       │ 72299         │ 72299         │ 72299         │         │
│  │                     4.624 MB    │ 4.624 MB      │ 4.624 MB      │ 4.624 MB      │         │
│  │                   dealloc:      │               │               │               │         │
│  │                     72298       │ 72298         │ 72298         │ 72298         │         │
│  │                     4.625 MB    │ 4.625 MB      │ 4.625 MB      │ 4.625 MB      │         │
│  │                   grow:         │               │               │               │         │
│  │                     61          │ 61            │ 61            │ 61            │         │
│  │                     2.634 KB    │ 2.634 KB      │ 2.634 KB      │ 2.634 KB      │         │
│  ╰─ prometheus       72.92 ms      │ 86.83 ms      │ 81.08 ms      │ 80.7 ms       │ 512     │ 2560
│                      alloc:        │               │               │               │         │
│                        118         │ 118           │ 118           │ 118.1         │         │
│                        6.589 KB    │ 6.589 KB      │ 6.589 KB      │ 6.597 KB      │         │
│                      dealloc:      │               │               │               │         │
│                        117         │ 117           │ 117           │ 117           │         │
│                        6.581 KB    │ 6.581 KB      │ 6.581 KB      │ 6.585 KB      │         │
│                      grow:         │               │               │               │         │
│                        7           │ 7             │ 7             │ 7.014         │         │
│                        1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
╰─ high_cardinality                  │               │               │               │         │
   ├─ measured         461.2 ms      │ 916.9 ms      │ 573.1 ms      │ 601.3 ms      │ 32      │ 64
   │                   alloc:        │               │               │               │         │
   │                     188         │ 223           │ 217           │ 202.2         │         │
   │                     3.776 MB    │ 4.453 MB      │ 4.259 MB      │ 5.27 MB       │         │
   │                   dealloc:      │               │               │               │         │
   │                     185.5       │ 220.5         │ 213.5         │ 197.7         │         │
   │                     1.628 MB    │ 1.966 MB      │ 1.606 MB      │ 1.642 MB      │         │
   │                   grow:         │               │               │               │         │
   │                     23          │ 24.5          │ 24            │ 23.75         │         │
   │                     109 MB      │ 163.5 MB      │ 163.5 MB      │ 161.8 MB      │         │
   ├─ metrics          4.822 s       │ 10.08 s       │ 6.641 s       │ 6.991 s       │ 32      │ 64
   │                   alloc:        │               │               │               │         │
   │                     17135415    │ 18879084      │ 17439347      │ 17731166      │         │
   │                     863.2 MB    │ 914 MB        │ 873.6 MB      │ 884.1 MB      │         │
   │                   dealloc:      │               │               │               │         │
   │                     17064902    │ 18808895      │ 17368823      │ 17660744      │         │
   │                     921.5 MB    │ 978.8 MB      │ 932.2 MB      │ 942.8 MB      │         │
   │                   grow:         │               │               │               │         │
   │                     3862027     │ 4258388       │ 3931099       │ 3997445       │         │
   │                     144.6 MB    │ 151 MB        │ 145.7 MB      │ 147.6 MB      │         │
   ╰─ prometheus       9.096 s       │ 18.84 s       │ 13.94 s       │ 13.99 s       │ 32      │ 64
                       alloc:        │               │               │               │         │
                         6677150     │ 7240913       │ 7031717       │ 6996261       │         │
                         342 MB      │ 371.2 MB      │ 360.5 MB      │ 359.6 MB      │         │
                       dealloc:      │               │               │               │         │
                         6430385     │ 6995289       │ 6785281       │ 6749788       │         │
                         333.6 MB    │ 362.8 MB      │ 352.1 MB      │ 350.7 MB      │         │
                       grow:         │               │               │               │         │
                         52906       │ 52662         │ 52835         │ 52843         │         │
                         101 MB      │ 101 MB        │ 101 MB        │ 101 MB        │         │
```
