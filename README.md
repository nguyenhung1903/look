# look

<img src="assets/icon.png" alt="look icon" width="96" />

A minimal, rofi-inspired macOS launcher focused on fast local actions:

- launch installed apps
- search local files and folders by name
- quick command mode for calculator, shell, kill, and system info

**Introduction video:**

[Video](https://www.youtube.com/watch?v=XtGcLfmXePU)

Default behavior:

- launch top result with `Enter`
- web search handoff with `Cmd+Enter` (Google)
- command mode with `Cmd+/` (`calc`, `shell`, `kill`, `sys`)
- force-quit flow in command mode (`kill`)

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

## Look's UI

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

## Current status

- Swift macOS app scaffold is located at `apps/macos/LauncherApp/look-app/` with project file `apps/macos/LauncherApp/look-app.xcodeproj`.
- Rust core workspace is initialized under `core/`.
- FFI bridge crate is initialized under `bridge/ffi/`.
- Architecture, roadmap, and initial design decisions are documented under `docs/`.
- UI includes: Spotlight-style launcher window (hidden from `Cmd+Tab`), theme/settings panel, command mode, and keyboard-first navigation.

Look's UI:

![Look UI 1](assets/look-ui/1.png)

![Look UI 2](assets/look-ui/2.png)

![Look UI 3](assets/look-ui/3.png)

![Look UI 4](assets/look-ui/4.png)

![Look UI 5](assets/look-ui/5.png)

![Look UI 6](assets/look-ui/6.png)

- Backend currently includes: SQLite-backed candidate storage, dynamic app/settings/file indexing, and usage event logging.
- User-facing guide: [docs/user-guide.md](docs/user-guide.md).
- Backend contributor guide: [docs/backend-guide.md](docs/backend-guide.md).
- Feature planning: [docs/features.md](docs/features.md).
- Task breakdown: [docs/tasks.md](docs/tasks.md).
- Architecture notes: [docs/architecture.md](docs/architecture.md).

## Current keyboard UX

- `Tab`: next result or next command
- `Up` / `Down`: navigate app list (in kill command)
- `Cmd+/`: enter command mode (defaults to `calc`)
- `Escape`: back to app list (when in command mode), otherwise hide launcher
- `Shift+Escape`: hide launcher
- `Cmd+1` / `Cmd+2` / `Cmd+3`: switch command directly
- `Cmd+Esc`: back to command list (`calc`) while staying in command mode
- `Cmd+Q`: hide launcher (Spotlight-style safety)
- `Cmd+Option+Q`: quit app
- `Enter`: launch selected app, execute active command, or confirm kill
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

Update:

```bash
brew upgrade --cask kunkka19xx/tap/look
```

Enable `Cmd+Space` for look (recommended):

- open `System Settings` -> `Keyboard` -> `Keyboard Shortcuts...` -> `Spotlight`
- disable `Show Spotlight search` or rebind it to another shortcut
- open look once, then use `Cmd+Space` as launcher toggle

If look is fully quit and Spotlight shortcut is disabled, relaunch from Terminal:

```bash
open "/Applications/Look.app"
```

**Unsigned release note:**

- if the app is not Developer ID signed/notarized, macOS Gatekeeper may block first launch
- first-run bypass: right-click `Look.app` -> `Open` -> confirm, or use `System Settings` -> `Privacy & Security` -> `Open Anyway`

Curl installer (after a GitHub release exists):

```bash
curl -fsSL https://raw.githubusercontent.com/kunkka19xx/look/main/scripts/install-look.sh | bash
which lookapp
```

CLI naming note:

- macOS already ships `/usr/bin/look`, so this project uses `lookapp` for terminal command examples

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

Future direction:

- we will keep adding useful built-in features when they stay aligned with the simple/fast philosophy
- community ideas are welcome; strong ideas with clear user value can be prioritized into the roadmap
- near-future exploration includes a plugin/extension injection model for developer customization

In scope for first milestone:

- global hotkey opens launcher
- query app index and launch with Enter
- query file/folder name index and open/reveal
- web search handoff with Google
- command mode with `calc`, `shell`, `kill`, and `sys`
- optional translation exists behind network opt-in (`translate_allow_network=true`)
- predictable, local-first behavior

Out of scope for v1:

- plugins
- clipboard history
- online-first behavior
- semantic/vector search
- content indexing

## Documentation

- User guide: [docs/user-guide.md](docs/user-guide.md)
- Architecture: [docs/architecture.md](docs/architecture.md)
- Features plan: [docs/features.md](docs/features.md)
- Backend guide: [docs/backend-guide.md](docs/backend-guide.md)
- Homebrew release notes: [docs/homebrew-release.md](docs/homebrew-release.md)
- Task tracking: [docs/tasks.md](docs/tasks.md)

## License

MIT

## Community

- Contributing guide: [CONTRIBUTING.md](CONTRIBUTING.md)
- Issue templates: [.github/ISSUE_TEMPLATE/](.github/ISSUE_TEMPLATE/)

## Author

- Kunkka
