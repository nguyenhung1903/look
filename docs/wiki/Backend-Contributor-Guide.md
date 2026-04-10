# Backend Contributor Guide

Detailed contributor guide:

- `docs/backend-guide.md`

## Edit map

- query parsing: `core/engine/src/query.rs`
- scoring/ranking: `core/engine/src/scoring.rs`, `core/engine/src/config.rs`
- search orchestration: `core/engine/src/search.rs`
- indexing logic: `core/engine/src/index/*`
- persistence/migrations: `core/storage/src/lib.rs`
- FFI endpoints: `bridge/ffi/src/*_api.rs`

## Verification

```bash
cd core && cargo check --workspace
cd ../bridge/ffi && cargo check
cargo test --workspace --manifest-path core/Cargo.toml
cargo test --manifest-path bridge/ffi/Cargo.toml
make app-run
```

## Rules of thumb

- keep tunables centralized in config
- keep query-time work bounded
- keep FFI payloads stable and typed
- keep indexing-source logic isolated under `index/*`
