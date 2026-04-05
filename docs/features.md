# Features Plan

This page defines the near-term feature scope for `look` and how each feature maps to backend responsibilities.

## Product pillars

- keyboard-first, low-latency launcher
- local-first indexing and ranking
- command utilities integrated in the same query flow
- predictable behavior with clear safety cues

## Feature tracks

## 1) Core launcher search

### Current

- app search UI
- command mode shell/calc UI flow
- settings panel and theme customization
- SQLite-backed candidate persistence and usage tracking
- dynamic app/settings/file indexing from backend sources

### Next

- unified result model and action execution
- configurable index roots and excludes from settings

## 2) Command mode

### Current

- `Cmd+/` to enter command mode
- `calc` and `shell`
- live calculator preview

### Next

- command registry in backend
- safer shell execution policy (warning + confirmation path)
- richer built-in commands (`open`, `help`, `theme`, etc.)

## 3) Ranking and personalization

### Next

- usage event logging
- recency + frequency scoring
- per-query behavior tuning

## 4) Settings and persistence

### Current

- theme settings persisted in app storage

### Next

- indexing roots and exclude rules
- command/security preferences
- backend-driven settings persistence

## 5) Performance and reliability

### Next

- in-memory top-N query path
- incremental indexing updates
- benchmark suite for query/index latency
- structured error handling across backend and bridge
- focused test coverage for ranking/indexing/ffi behavior
- development-friendly logging and diagnostics
