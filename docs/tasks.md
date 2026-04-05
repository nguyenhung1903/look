# Implementation Tasks

This is the execution breakdown for the next backend-focused milestone.

## Priority queue (current)

Now:

- [x] benchmark query latency and indexing throughput
- [x] collect first dated baseline note in `docs/bench-notes/YYYY-MM-DD.md`
- [x] add diagnostics/debug toggles for development

Next:

- [x] add ffi-level smoke tests for `look_search_json` and `look_record_usage`
- [x] add structured error mapping for UI feedback
- [ ] route settings updates to backend persistence

Later:

- [ ] add in-memory cache for top-N results
- [ ] add result pagination/streaming for large result sets (optional)
- [ ] Homebrew release packaging

## Milestone A: Storage foundation (SQLite)

- [x] add `core/storage` SQLite layer and connection manager
- [x] define schema (`candidates`, `usage_events`, `settings`, `index_state`)
- [x] add migration system with schema versioning
- [x] add CRUD APIs for candidates and usage events
- [x] add unit tests for migration and basic read/write

## Milestone B: Unified engine model

- [x] define shared `ActionKind` in core
- [x] refactor engine query output to typed action results (`LaunchResult`, `LaunchResultAction`)
- [x] ensure result payloads can cross FFI boundary cleanly
- [ ] add result pagination/streaming for large result sets (optional)

## Milestone C: Indexing pipeline

- [x] implement app index source (`/Applications`, `/System/Applications`, `~/Applications`)
- [x] implement file/folder index source with root config defaults
- [x] add System Settings source from `.appex`/`.prefPane` discovery
- [x] support exclude paths and hidden file policy (`app_exclude_paths`, `app_exclude_names`, `file_exclude_paths`)
- [x] add startup full scan + snapshot upsert
- [x] persist index snapshots to SQLite

## Milestone D: Bridge and app integration

- [x] finalize FFI API (`init`, `search`, `record_action`, `translate`)
- [x] switch SwiftUI launcher from seed data to engine results
- [x] wire action execution path (open app/path/web, run command)
- [x] translate text via FFI (`t"word` + Enter)
- [x] command mode with `calc`, `shell`, `kill` commands
- [x] command keyboard shortcuts (Cmd+/, Cmd+1/2/3, Tab, Esc hide, Cmd+Esc)
- [ ] route settings updates to backend persistence
- [x] add structured error mapping for UI feedback

## Milestone E: Ranking and safety

- [x] log execution events to `usage_events`
- [x] implement recency/frequency score boost
- [x] apply shell safety policy (`sudo` warning, confirm mode option)
- [x] add numeric guardrails for calc
- [x] add tests for scoring and safety rules (scoring coverage added)

## Milestone F: Performance and polish

- [x] benchmark query latency and indexing throughput
- [ ] add in-memory cache for top-N results
- [x] optimize startup path and background index scheduling
- [x] add diagnostics/debug toggles for development
- [ ] update docs and user guide for finalized behavior

## Milestone G: Reliability (errors, tests, logs)

- [ ] add structured error model across engine/storage/ffi boundaries
- [ ] add safe user-facing fallback messages for action failures
- [x] add unit tests for search scoring and empty-query top-picks behavior
- [ ] add unit tests for curated settings catalog integrity (id/title/target validity)
- [x] add storage tests for usage-event writes and candidate upsert semantics
- [x] add ffi-level smoke tests for `look_search_json` and `look_record_usage`
- [x] add debug logging hooks (startup indexing summary, query timing, action execution outcome)
- [x] add a log-level toggle (`error`/`info`/`debug`) via env var for local troubleshooting

## Backlog: UI Enhancements

- [x] **App list preview**: 2-column layout with icon/name on left, info/preview on right (image preview, app info)
- [x] **System info command**: Add `/sys` command to view system info (memory, CPU with usage %, battery, uptime, disk), zoomable with app font size
- [x] **Command list 2-column**: Make command list 2-column layout for better visibility
- [ ] **Homebrew release**: Package app for homebrew installation
- [x] **Build script**: Create release build and curl installer script

## Evergreen: Search quality and performance (always-on)

- [ ] **Indexing improvement loop**: continually refine scan roots, excludes, and incremental refresh strategy
- [ ] **Matching improvement loop**: improve typo tolerance, tokenization, and relevance scoring for mixed app/file queries
- [ ] **Optimization loop**: keep reducing query latency, startup cost, and memory use as regular maintenance

## Weekly checklist: quality + performance

Run this checklist at least once per week (or before release cut):

- [ ] collect baseline metrics from the same sample dataset and keep results in a dated note (`docs/bench-notes/YYYY-MM-DD.md`)
- [ ] measure query latency (`p50`, `p95`) for empty query, short query (2-4 chars), and long query (8+ chars)
- [ ] measure startup time (app launch -> first usable search result)
- [ ] compare index size and memory usage versus last baseline
- [ ] verify top-5 relevance for a fixed smoke query set (apps, files, folders, settings)
- [ ] review at least 3 recent user-reported misses and convert into matching/indexing improvements
- [ ] add/update at least one test for any ranking/matching/indexing behavior change

Suggested guardrails (adjust as project evolves):

- `query latency p50`: <= 30ms
- `query latency p95`: <= 80ms
- `startup to first result`: <= 700ms
- `peak memory (idle window)`: <= 220MB
- `relevance smoke pass rate`: >= 90% in top-5

Escalation rule:

- if any guardrail regresses by >10% week-over-week, open a focused perf/quality issue before merging unrelated polish work
