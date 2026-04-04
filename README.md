# look

A minimal, rofi-inspired macOS launcher focused on fast local actions:

- launch installed apps
- search local files and folders by name
- quick command mode for calculator and shell execution

Default behavior:

- web search handoff with `Cmd+Enter` (Google)

The project is designed around low latency, keyboard-first interaction, and a small native footprint.

User-level behavior can be configured with `~/.look.config` (indexing + UI theme/font; see `docs/user-guide.md` for supported keys).

## Repository layout

```text
.
├── apps/
│   └── macos/
│       └── LauncherApp/
├── core/
│   ├── engine/
│   ├── indexing/
│   ├── matching/
│   ├── ranking/
│   └── storage/
├── bridge/
│   └── ffi/
├── docs/
├── benchmarks/
├── scripts/
├── assets/
└── examples/
```

## Current starter status

- Swift macOS app scaffold is located at `apps/macos/LauncherApp/look-app/` with project file `apps/macos/LauncherApp/look-app.xcodeproj`.
- Rust core workspace is initialized under `core/`.
- FFI bridge crate is initialized under `bridge/ffi/`.
- Architecture, roadmap, and initial design decisions are documented under `docs/`.
- UI currently includes: borderless launcher window, theme/settings panel, command mode, and keyboard-first navigation.
- Backend currently includes: SQLite-backed candidate storage, dynamic app/settings/file indexing, and usage event logging.
- User-facing guide: `docs/user-guide.md`.
- Backend contributor guide: `docs/backend-guide.md`.
- Feature planning: `docs/features.md`.
- Task breakdown: `docs/tasks.md`.

## Current keyboard UX

- `Tab` / `Shift+Tab`: move through app results or switch command type in command mode
- `Up` / `Down`: move through app results or command list
- `/`: enter command mode (defaults to `calc`)
- `Shift+Esc`: exit command mode
- `Enter`: launch selected app or execute active command
- `Cmd+Enter`: web search current query using Google
- `Cmd+Shift+,`: open/close settings panel
- `Cmd+Shift+;`: reload `.look.config`
- `Cmd+-`, `Cmd+=`, `Cmd+0`: temporary UI zoom out/in/reset

## Quick start

Prerequisites:

- macOS
- Xcode (for app shell)
- Rust stable toolchain (for core engine)

Rust workspace checks:

```bash
cd core
cargo check --workspace
```

FFI bridge checks:

```bash
cd bridge/ffi
cargo check
```

## Product scope

In scope for first milestone:

- global hotkey opens launcher
- query app index and launch with Enter
- query file/folder name index and open/reveal
- web search handoff with Google
- command mode with `calc` and `shell`
- predictable, local-first behavior

Out of scope for v1:

- plugins
- clipboard history
- online-first behavior
- semantic/vector search
- content indexing

## License

MIT
