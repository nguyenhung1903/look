# Architecture Overview

Canonical architecture document:

- `docs/architecture.md`

## High-level stack

- **UI shell**: SwiftUI/AppKit (`apps/macos`)
- **Bridge**: FFI C ABI (`bridge/ffi`)
- **Core engine**: Rust (`core/engine`, plus indexing/matching/ranking/storage crates)
- **Persistence**: SQLite (`look.db`)

## Query path

`User input -> Swift debounce -> FFI search call -> Rust parse/match/rank -> top-k results -> Swift render`

## Data lifecycle

`discover candidates -> dedupe -> chunked upsert -> prune stale data -> refresh in-memory cache`

## Ranking model

Combines:

- fuzzy and contains/path matching,
- kind bias,
- usage + recency adjustments,
- top-k heap selection with bounded rerank.
