# S-12: Scale Benchmark Framework

## Status: Reference Implementation

The benchmark framework describes the metrics and methodology.
Actual benchmarking is performed via integration tests and CLI-driven scripts.

## Metrics

| Metric | Tool | Target |
|--------|------|--------|
| Block Map insert throughput | Integration test | >10,000 ops/sec |
| Block Map lookup latency | Integration test | p50 < 1ms |
| Catalog insert throughput | Integration test | >5,000 files/sec |
| Block Store write throughput | Integration test | >100 MB/sec |
| Directory distribution | Integration test | Uniform distribution |

## How to Run

```bash
# Run all benchmarks (as integration tests)
cargo test --features repository --test scale_bench

# Run individual benchmark
cargo test --features repository -- scale_bench::block_map_insert_10k
```
