# Features Plan

This page defines the near-term feature scope for `look` and how each feature maps to backend responsibilities.

## Product pillars

- keyboard-first, low-latency launcher
- local-first indexing and ranking
- command utilities integrated in the same query flow
- predictable behavior with clear safety cues
- extensible roadmap guided by practical user feedback

## Feature tracks

## 1) Core launcher search

### Current

- app search UI
- command mode (`calc`, `shell`, `kill`, `sys`)
- settings panel and theme customization
- SQLite-backed candidate persistence and usage tracking
- dynamic app/settings/file indexing from backend sources
- query prefixes: `a"` apps, `f"` files, `d"` folders, `r"` regex
- query prefix: `c"` for clipboard history (latest 10 text clips)
- slash-path query bias (example: `git/books-pc`)
- `Cmd+F` reveal selected app/file/folder in Finder

### Next

- unified result model and action execution
- deeper settings sub-page coverage and quality filtering

## 2) Command mode

### Current

- `Cmd+/` to enter command mode
- `calc`, `shell`, `kill`, and `sys`
- live calculator preview and quick copy result behavior
- `Esc` exits command mode to app list
- `Shift+Esc` hides launcher

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

- settings persisted to `~/.look.config`
- advanced controls for indexing, translation privacy, backend log level, and launch-at-login

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

## 6) Extensibility roadmap

### Planned

- accept and evaluate community feature proposals continuously
- add plugin/extension injection path for developer customization
- keep extension model optional so base launcher remains simple and fast
