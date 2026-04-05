# look

<img src="assets/icon.png" alt="look icon" width="96" />

A minimal, rofi-inspired macOS launcher focused on fast local actions:

- launch installed apps
- search local files and folders by name
- quick command mode for calculator, shell, kill, and system info

Default behavior:

- translate text with `t"word` + `Enter`
- web search handoff with `Cmd+Enter` (Google)
- force quit apps with `/kill`

The project is designed around low latency, keyboard-first interaction, and a small native footprint.

## Positioning

Compared with larger launcher ecosystems (for example Raycast, Alfred, and similar tools), `look` is intentionally focused:

- simple core workflow: app/file/folder search + a few built-in commands
- lightweight and local-first behavior
- fully open source
- free to use
- no plugin marketplace complexity in the default experience

If you want a minimal launcher that stays fast and predictable, `look` is built for that.

User-level behavior can be configured with `~/.look.config` (indexing + UI theme/font; see [User Guide](docs/user-guide.md) for supported keys).

Indexing config supports include roots plus exclude rules for both apps and files.

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
- UI currently includes: spotlight-style launcher window, theme/settings panel, command mode, and keyboard-first navigation.
- Backend currently includes: SQLite-backed candidate storage, dynamic app/settings/file indexing, and usage event logging.
- User-facing guide: [docs/user-guide.md](docs/user-guide.md).
- Backend contributor guide: [docs/backend-guide.md](docs/backend-guide.md).
- Feature planning: [docs/features.md](docs/features.md).
- Task breakdown: [docs/tasks.md](docs/tasks.md).
- Architecture notes: [docs/architecture.md](docs/architecture.md).
- Release secrets checklist: [docs/release-secrets.md](docs/release-secrets.md).

## Current keyboard UX

- `Tab`: next result or next command
- `Up` / `Down`: navigate app list (in kill command)
- `Cmd+/`: enter command mode (defaults to `calc`)
- `Escape`: back to app list (when in command mode), otherwise hide launcher
- `Shift+Escape`: hide launcher
- `Cmd+1` / `Cmd+2` / `Cmd+3`: switch command directly
- `Cmd+Esc`: back to command list (`calc`) while staying in command mode
- `Enter`: launch selected app, execute active command, translate (if `t"...`), or confirm kill
- `Y` / `N`: confirm/cancel in kill command confirmation
- `Cmd+Enter`: web search current query using Google
- `a"` / `f"` / `d"` / `r"`: apps/files/folders/regex scoped query prefix
- `Cmd+Shift+,`: open/close settings panel
- `Cmd+Shift+;`: reload `.look.config`
- `Cmd+-`, `Cmd+=`, `Cmd+0`: temporary UI zoom out/in/reset

## Installation

Homebrew tap (recommended once release is published):

```bash
brew tap kunkka19xx/tap
brew install --cask look
```

Unsigned release note:

- if the app is not Developer ID signed/notarized, macOS Gatekeeper may block first launch
- first-run bypass: right-click `Look.app` -> `Open` -> confirm, or use `System Settings` -> `Privacy & Security` -> `Open Anyway`

Curl installer (after a GitHub release exists):

```bash
curl -fsSL https://raw.githubusercontent.com/kunkka19xx/look/main/scripts/install-look.sh | bash
```

Manual installer options:

```bash
curl -fsSL https://raw.githubusercontent.com/kunkka19xx/look/main/scripts/install-look.sh | bash -s -- --version <version> --repo kunkka19xx/look
```

or direct URL:

```bash
curl -fsSL https://raw.githubusercontent.com/kunkka19xx/look/main/scripts/install-look.sh | bash -s -- --url "https://github.com/kunkka19xx/look/releases/download/v<version>/Look-<version>-macOS.zip"
```

## Quick start

Prerequisites:

- macOS 15.0+
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

Prepare release artifacts/scripts (maintainers):

```bash
./scripts/build-release.sh 1.0.0
./scripts/generate-homebrew-cask.sh 1.0.0 <sha256> kunkka19xx/look
```

Signing/notarization is optional in CI:

- paid Apple Developer membership is required for Developer ID signing/notarization
- without those secrets, release workflow still builds artifacts and Homebrew cask can still be published

## Product scope

Platform direction:

- current primary target is macOS
- planned: Windows version (after macOS release quality is stable)
- Linux version is not a near-term priority because tools like `rofi` already cover much of this workflow well

In scope for first milestone:

- global hotkey opens launcher
- query app index and launch with Enter
- query file/folder name index and open/reveal
- web search handoff with Google
- translate text with `t"...`
- command mode with `calc`, `shell`, `kill`, and `sys`
- predictable, local-first behavior

Out of scope for v1:

- plugins
- clipboard history
- online-first behavior
- semantic/vector search
- content indexing

## License

MIT

## Community

- Contributing guide: [CONTRIBUTING.md](CONTRIBUTING.md)
- Issue templates: [.github/ISSUE_TEMPLATE/](.github/ISSUE_TEMPLATE/)
