# Architecture

The project uses a layered architecture:

```text
macOS App Shell (Swift / AppKit)
    |
    | bridge / FFI
    v
Core Engine (Rust)
    |- indexing
    |- matching
    |- ranking
    |- storage
```

## Module responsibilities

- `apps/macos`: launcher window, keyboard input, global hotkey, action dispatch
- `bridge/ffi`: narrow stable interface between Swift shell and Rust core
- `core/indexing`: scan sources and build/update candidate index
- `core/matching`: fuzzy/exact/prefix candidate matching
- `core/ranking`: history and recency-aware score adjustments
- `core/storage`: persistence for index metadata, config, and usage history
- `core/engine`: query pipeline orchestration and top-N result retrieval

## Engine internals

- `core/engine/src/config.rs`: centralized engine tunables (scan roots, limits, ranking weights)
- `core/engine/src/index/apps.rs`: installed app discovery
- `core/engine/src/index/settings.rs`: System Settings entry discovery
- `core/engine/src/index/files.rs`: local file/folder discovery
- `core/engine/src/index/mod.rs`: discovery orchestration
- `core/engine/src/lib.rs`: search/ranking orchestration and storage integration

## Optional web search action

- local search remains primary and default
- web search is an explicit handoff action via `Cmd+Enter`
- current default provider is Google

## Command mode

- `Cmd+/` enters command mode
- command mode currently supports `calc` and `shell`
- `calc` supports live result preview and 4-decimal formatted output
- `shell` executes command text and returns stdout/stderr summary
- shell commands containing `sudo` trigger an orange warning border
- `Esc` exits command mode back to app/file list
- `Shift+Esc` hides launcher window

## Window behavior

- global hotkey `Cmd+Space` toggles launcher visibility (when not intercepted by Spotlight)
- `Escape` hides launcher when not in command mode
- launcher auto-hides on focus loss

## Settings panel

- `Cmd+Shift+,` toggles in-app settings/docs panel
- appearance controls: tint color, blur style, blur opacity
- advanced controls: background image, indexing depth/limit, translation network privacy, backend log level
- settings are persisted locally

## Query pipeline

`query -> candidate collection -> matching -> ranking -> top results`

Candidate collection currently includes:

- installed apps
- curated System Settings quick links (stored as `setting:*` candidates)
- local files/folders from Desktop/Documents/Downloads

## Data model

```text
Candidate {
  id
  kind        // app | file | folder
  title
  subtitle
  path
  use_count
  last_used_at
}
```

## Performance goals

- launcher appears in under 50 ms perceived latency
- query updates in under 10 ms for top-N from memory
- near-zero idle CPU
- small and stable memory footprint
