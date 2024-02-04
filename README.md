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

### Macbook Pro M2 Max:

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
