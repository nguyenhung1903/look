# Backend Guide

This guide explains how the Rust backend is organized and where to edit behavior safely.

## Purpose

The backend is responsible for:

- candidate discovery (apps, settings entries, files/folders)
- search matching and ranking
- persistence (SQLite)
- usage logging for personalization
- FFI-facing data for the macOS app shell

## High-level data flow

`index sources -> SQLite snapshot -> query engine -> ranked results -> FFI -> Swift UI`

At startup:

1. engine discovers candidates from index sources
2. candidates are upserted into SQLite
3. query path reads and ranks in-memory candidate set

On action execution:

1. Swift executes selected action (open app/file/folder/settings URL)
2. Swift reports usage through FFI
3. backend records event and updates usage counters

## Module map

## `core/engine`

- `core/engine/src/lib.rs`
  - query orchestration
  - ranking composition
  - SQLite load/refresh integration
- `core/engine/src/config.rs`
  - centralized constants/tunables
  - scan roots, depth, limits, score weights, query hints
- `core/engine/src/index/mod.rs`
  - index orchestration entry (`discover_candidates`)
- `core/engine/src/index/apps.rs`
  - installed app discovery
- `core/engine/src/index/settings.rs`
  - curated System Settings catalog entries and search aliases
- `core/engine/src/index/files.rs`
  - file/folder discovery from configured default roots

## `core/storage`

- `core/storage/src/lib.rs`
  - SQLite connection and migrations
  - tables: `candidates`, `usage_events`, `settings`, `index_state`
  - candidate upsert/replace/load APIs
  - usage event recording

## `bridge/ffi`

- `bridge/ffi/src/lib.rs`
  - C ABI surface used by Swift
  - current APIs include search JSON, usage recording, and runtime config reload

## Where to change behavior

## Tune search and ranking

Edit `core/engine/src/config.rs`:

- contains score weights (`SCORE_*`)
- kind/query bias values (`BIAS_*`)
- query hint tokens (`QUERY_SETTINGS_HINTS`)

## Tune indexing scope

Edit `core/engine/src/config.rs`:

- app roots/depth (`APP_SCAN_*`)
- file roots/depth/limits (`FILE_SCAN_*`)
- skipped directory list (`SKIP_DIR_NAMES`)

Then update source-specific logic in:

- `core/engine/src/index/apps.rs`
- `core/engine/src/index/settings.rs` (curated settings list)
- `core/engine/src/index/files.rs`

Runtime overrides are also supported through `~/.look.config` (or `LOOK_CONFIG_PATH`).

- format: one `key=value` per line (`#` starts a comment)
- supported keys: `app_scan_roots`, `app_scan_depth`, `file_scan_roots`, `file_scan_depth`, `file_scan_limit`, `skip_dir_names`
- unknown keys are ignored; invalid values fall back to defaults
- file is auto-created with defaults on first launch if missing
- app can reload config at runtime via `Cmd+Shift+;`
- the same config file is also read by Swift UI for theme/font overrides

## Tune persistence behavior

Edit `core/storage/src/lib.rs`:

- migration SQL
- upsert semantics
- usage-event updates

## Local verification

## Build checks

```bash
cd core
cargo check --workspace

cd ../bridge/ffi
cargo check
```

## App build

```bash
cd /path/to/look
make app-build
```

## Real DB path

- default: `~/Library/Application Support/look/look.db`

Inspect quickly:

```bash
sqlite3 "$HOME/Library/Application Support/look/look.db" "SELECT id,title,use_count FROM candidates ORDER BY use_count DESC LIMIT 20;"
```

## Contribution notes

- keep tunables in `config.rs`, avoid scattering magic numbers
- keep index-source logic inside `index/*`
- keep FFI boundary narrow and stable
- avoid introducing platform assumptions in non-index modules

## Reliability expectations

- errors should cross boundaries as typed/structured values where possible
- UI-facing errors should be actionable and non-technical
- core scoring and indexing behavior should be covered by unit tests
- bridge entry points should have smoke tests for serialization and null/invalid input
- logging should be lightweight by default and verbose only when explicitly enabled

## Known tech debt

- **System Settings depth:** current backend uses a curated top-level settings catalog for reliability and UX quality.
- **Missing sub-settings:** deep per-pane/per-subpage coverage is not complete yet (for example very granular settings pages).
- **Contribution opportunity:** if you have a robust, update-resilient way to discover and map sub-settings targets, contributions are welcome.
  - desired outcome: clean names, stable open targets, no noisy/internal entries.
  - preferred approach: source-driven + filtered, with fallback curation where needed.
