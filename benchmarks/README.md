# Benchmarks

This directory will hold latency and throughput checks for:

- query-to-results latency
- ranking overhead
- indexing throughput and update cost

Benchmark-first discipline is part of the project identity.

## Run benchmark

From `core/`:

```bash
cargo run -p look-engine --example perf_bench
```

Optional env overrides:

- `LOOK_DB_PATH=/path/to/look.db` to benchmark a specific database snapshot

Save each run into `docs/bench-notes/YYYY-MM-DD.md`.
