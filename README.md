# measured

A better\* metrics crate

## Goals

1. Low memory usage and low fragmentation
2. Fast
3. Less atomics, more thread locals
4. Fairly ergonomic
5. Eventual Consistency

## Non-goals

- facade macros
- strong consistency

## Benchmark results

The benchmark runs on multiple threads a simple counter increment, with various labels,
and then samples+encodes the values into the Prometheus text format. This crate outperforms both metrics and prometheus
when it comes to both speed and allocations.

### Macbook Pro M2 Max (12 threads):

```
Timer precision: 41 ns
counters       fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured    11.52 ms      │ 17.77 ms      │ 16.82 ms      │ 16.53 ms      │ 504     │ 2520
│              alloc:        │               │               │               │         │
│                0           │ 2             │ 0             │ 0.162         │         │
│                0 B         │ 48 B          │ 0 B           │ 3.904 B       │         │
│              grow:         │               │               │               │         │
│                0           │ 7             │ 0             │ 0.569         │         │
│                0 B         │ 1.656 KB      │ 0 B           │ 134.7 B       │         │
├─ metrics     35.08 ms      │ 46.11 ms      │ 38.78 ms      │ 38.93 ms      │ 504     │ 2520
│              alloc:        │               │               │               │         │
│                0           │ 0             │ 0             │ 6024          │         │
│                0 B         │ 0 B           │ 0 B           │ 385.3 KB      │         │
│              dealloc:      │               │               │               │         │
│                0           │ 0             │ 0             │ 6024          │         │
│                0 B         │ 0 B           │ 0 B           │ 385.4 KB      │         │
│              grow:         │               │               │               │         │
│                0           │ 0             │ 0             │ 5.083         │         │
│                0 B         │ 0 B           │ 0 B           │ 219.5 B       │         │
╰─ prometheus  118 ms        │ 143.2 ms      │ 137.5 ms      │ 136.9 ms      │ 504     │ 2520
               alloc:        │               │               │               │         │
                 0           │ 0             │ 0             │ 11.33         │         │
                 0 B         │ 0 B           │ 0 B           │ 758.4 B       │         │
               dealloc:      │               │               │               │         │
                 0           │ 0             │ 0             │ 11.25         │         │
                 0 B         │ 0 B           │ 0 B           │ 757.7 B       │         │
               grow:         │               │               │               │         │
                 0           │ 0             │ 0             │ 0.583         │         │
                 0 B         │ 0 B           │ 0 B           │ 138 B         │         │
```

### Ryzen 9 7950X (32 threads):

```
Timer precision: 2.34 µs
counters       fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ measured    7.54 ms       │ 11.72 ms      │ 10.62 ms      │ 10.51 ms      │ 512     │ 2560
│              alloc:        │               │               │               │         │
│                2           │ 2             │ 2             │ 2             │         │
│                48 B        │ 48 B          │ 48 B          │ 48 B          │         │
│              grow:         │               │               │               │         │
│                7           │ 7             │ 7             │ 7             │         │
│                1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
├─ metrics     20.64 ms      │ 27.96 ms      │ 21.89 ms      │ 22.07 ms      │ 512     │ 2560
│              alloc:        │               │               │               │         │
│                72299       │ 72299         │ 72299         │ 72299         │         │
│                4.624 MB    │ 4.624 MB      │ 4.624 MB      │ 4.624 MB      │         │
│              dealloc:      │               │               │               │         │
│                72298       │ 72298         │ 72298         │ 72298         │         │
│                4.625 MB    │ 4.625 MB      │ 4.625 MB      │ 4.625 MB      │         │
│              grow:         │               │               │               │         │
│                61          │ 61            │ 61            │ 61            │         │
│                2.634 KB    │ 2.634 KB      │ 2.634 KB      │ 2.634 KB      │         │
╰─ prometheus  75.5 ms       │ 86.02 ms      │ 81.85 ms      │ 81.45 ms      │ 512     │ 2560
               alloc:        │               │               │               │         │
                 118         │ 118           │ 118           │ 118.1         │         │
                 6.589 KB    │ 6.589 KB      │ 6.589 KB      │ 6.598 KB      │         │
               dealloc:      │               │               │               │         │
                 117         │ 117           │ 117           │ 117           │         │
                 6.581 KB    │ 6.581 KB      │ 6.581 KB      │ 6.584 KB      │         │
               grow:         │               │               │               │         │
                 7           │ 7             │ 7             │ 7.014         │         │
                 1.656 KB    │ 1.656 KB      │ 1.656 KB      │ 1.656 KB      │         │
```
