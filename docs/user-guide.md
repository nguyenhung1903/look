# look User Guide

look is a fast, keyboard-first launcher for macOS.

It helps you do three things quickly in one window:

- launch installed apps
- search local files and folders by name
- search clipboard history snippets (`c"` prefix)
- run quick commands (calculator, shell, kill, and system info)

The interface is local-first, lightweight, and designed for low-friction daily use.

## Installation

Compatibility:

- currently targets macOS `15.0+`

### Homebrew tap (recommended once release is published)

```bash
brew tap kunkka19xx/tap
brew install --cask look
```

Update and reinstall via Homebrew:

```bash
brew update
brew upgrade --cask look
```

Unsigned release behavior:

- if the distributed app is not Developer ID signed/notarized, macOS may block first launch
- open once via Finder with right-click `Open` and confirm, or use `System Settings` -> `Privacy & Security` -> `Open Anyway`

### Curl installer

```bash
curl -fsSL https://raw.githubusercontent.com/kunkka19xx/look/main/scripts/install-look.sh | bash
```

Install a specific release version with curl installer:

```bash
curl -fsSL https://raw.githubusercontent.com/kunkka19xx/look/main/scripts/install-look.sh | bash -s -- --version 1.0.0 --repo kunkka19xx/look
```

### Installer options

- choose version: `--version <version>` or env `LOOK_VERSION=<version>`
- choose repository: `--repo kunkka19xx/look` or env `LOOK_REPO=kunkka19xx/look`
- use a direct zip URL: `--url <release-zip-url>` or env `LOOK_DOWNLOAD_URL=<release-zip-url>`

Signing and notarization (optional):

- requires paid Apple Developer Program membership
- when not configured, look can still be installed and used (with first-run Gatekeeper confirmation)

Install target:

- installs to `/Applications` when writable
- otherwise installs to `~/Applications`

### Verify installation

- app path is usually `/Applications/Look.app` or `~/Applications/Look.app`
- run `which lookapp` to verify CLI shim is installed and available on PATH
- macOS already includes `/usr/bin/look`; use `lookapp` for this project CLI command
- launch once from Finder or Spotlight, then test global hotkey (`Cmd+Space`, if available)
- if app does not open from hotkey, check the Spotlight conflict section below

### Enable `Cmd+Space` for look

- open `System Settings` -> `Keyboard` -> `Keyboard Shortcuts...` -> `Spotlight`
- disable `Show Spotlight search` or rebind it to another shortcut
- relaunch look and test `Cmd+Space`
- optional: enable `launch_at_login=true` (Advanced tab) so look is ready after sign-in

If look is fully quit and Spotlight shortcut is disabled, relaunch from Terminal:

```bash
open "/Applications/Look.app"
```

### Uninstall

Homebrew install:

```bash
brew uninstall --cask look
brew untap kunkka19xx/tap  # optional, only if you no longer use this tap
```

Curl/manual install:

```bash
rm -rf "/Applications/Look.app"
rm -rf "$HOME/Applications/Look.app"
```

Optional cleanup of local data/config:

```bash
rm -f "$HOME/.look.config"
rm -f "$HOME/Library/Application Support/look/look.db"
```

Note: optional cleanup removes your indexed data, usage history, and custom settings.

## What makes look different

- one focused launcher window instead of many utility apps
- keyboard-first workflow with instant mode switching
- transparent, blur-based UI that can be themed
- optional command mode for utility tasks without leaving context

## How look compares

Compared with broader launcher platforms (for example Raycast and similar tools), look intentionally emphasizes:

- simplicity over large extension/plugin ecosystems
- lightweight local-first behavior
- open-source development model
- free usage without paid feature tiers

look is best for users who want a fast, minimal, predictable launcher rather than a large all-in-one productivity platform.

## Core usage

### 1) Launch apps and find files

Type in the main search bar to filter results.

This includes:

- installed apps
- local files/folders (Desktop/Documents/Downloads)
- curated System Settings entries (for example display, network, bluetooth)

- press `Tab` to move down the list
- press `Shift+Tab` to move up the list
- press `Up` / `Down` to move selection
- press `Enter` to open the selected result
- press `Cmd+C` to copy selected file/folder to pasteboard
- press `Cmd+F` to reveal selected app/file/folder in Finder
- press `Cmd+H` to open/close the in-window shortcut help screen
- click a row to open it

If results look stale or missing after config/indexing changes:

- press `Cmd+Shift+;` to reload config and refresh backend index
- check `file_scan_roots`, `file_scan_depth`, and `file_scan_limit` in `~/.look.config`

Path-style query is supported directly in normal search:

- type path fragments like `git/books-pc`, `git/books-pc/readme`, or deeper segments to bias matches toward path hits

Quick prefix action in the same input:

- type `a"term`: search apps only
- type `f"term`: search files only
- type `d"term`: search folders only
- type `r"pattern`: search by regex (case-insensitive)
- type `c"term`: search recent clipboard text history (latest 10)

### 2) Clipboard history search

Clipboard history is available directly from the main query input:

- type `c"` to list recent clipboard items
- type `c"word` to filter history by text
- press `Enter` on a clipboard row to copy that item back to macOS clipboard
- use the `Delete` button in preview panel to remove sensitive clipboard entries from look history

Notes:

- clipboard history currently stores text clipboard items only
- history is in-memory for the running app session

Translation privacy control:

- translation network access is disabled by default
- `translate_allow_network` controls whether translation requests are allowed
- optional env override: `LOOK_TRANSLATE_ALLOW_NETWORK=true`
- recommended default: keep this disabled for a local-first workflow

### 3) Web search handoff

If you want to search the web from the same query:

- press `Cmd+Enter`

Current default provider: Google.

### 4) Command mode

To enter command mode:

- press `Cmd+/`

Command mode starts with `calc` selected by default.

In command mode:

- `Tab` switches to next command
- `Cmd+1` / `Cmd+2` / `Cmd+3` selects command directly
- `Enter` runs the current command input
- `Escape` exits command mode back to app list
- `Shift+Escape` hides launcher
- `Cmd+Escape` returns to command list on `calc`

Spotlight-style behavior:

- launcher hides on `Escape` from normal app/file list
- launcher also hides automatically when app loses focus
- launcher runs as accessory app (hidden from `Cmd+Tab` app switcher)

Available commands:

- `calc`: evaluate math expressions
- `shell`: run shell commands
- `kill`: force kill a running app (see Kill command shortcuts above)
- `sys`: show system info (model, macOS, memory, CPU usage, battery, uptime, disk)

Examples:

- `2/5` -> `0.4000`
- `v9` -> `3.0000` (`v` maps to sqrt)
- `2 x 5` -> `10.0000` (`x` maps to `*`)

Math output formatting:

- 4 digits after decimal
- grouped thousands (example: `1,000,000.0000`)

Safety cues:

- shell input containing `sudo` shows an orange warning border

## Settings and customization

To open settings:

- press `Cmd+Shift+,`

Settings are shown inside the same launcher window and include:

- appearance: tint color, blur style, blur opacity, font family/size, text color, border style
- advanced: background, indexing, privacy/logging, and startup controls (`file_scan_depth`, `file_scan_limit`, `translate_allow_network`, `backend_log_level`, `launch_at_login`)
- shortcuts: built-in documentation tab

Global hotkey:

- launcher hotkey is `Cmd+Space` (Spotlight-style toggle)
- configure Spotlight conflict in the Installation section above

Hotkey behavior notes:

- `Cmd+Space` only works when Spotlight is not bound to the same shortcut
- window managers (for example tiling tools) may alter focus behavior; if focus looks stuck, press the hotkey again or click inside the launcher to refocus

Troubleshooting first-run launch block (Gatekeeper):

- if macOS says the app cannot be opened, right-click `Look.app` in Finder -> `Open` -> confirm
- or use `System Settings` -> `Privacy & Security` -> `Open Anyway`
- this is expected for unsigned/not-notarized builds

Other settings UX:

- **Save Config** writes current UI values back into `~/.look.config`
- font name field supports installed-font suggestions in a dropdown

Background image modes:

- `Center`
- `Fill`
- `Stretch`
- `Duplicate`

You can also configure backend indexing behavior with a user config file:

- path: `~/.look.config`
- optional override path: `LOOK_CONFIG_PATH=/path/to/custom.config`
- first launch creates this file automatically with defaults if it does not exist
- live reload: press `Cmd+Shift+;` after editing the file

Local dev run note (repository `make app-run`):

- local dev launch uses `LOOK_CONFIG_PATH=$HOME/.look.dev.config` by default
- local dev launch exports `LOOK_DEV_HINT=1`, and the app shows a red `TEST APP` badge
- this helps distinguish local test app from installed release/Homebrew app

Supported config keys (`key=value`):

Backend indexing keys:

- `app_scan_roots`: comma-separated app roots to scan (absolute paths); default: `/Applications,/System/Applications,/System/Applications/Utilities`
- `app_scan_depth`: recursion depth for app scanning (positive integer); default: `3`
- `app_exclude_paths`: comma-separated paths to exclude from app indexing (supports `~/...`, absolute paths, and home-relative names); default: empty
- `app_exclude_names`: comma-separated app display names to exclude (case-insensitive, `.app` suffix optional); default: empty
- `file_scan_roots`: comma-separated file roots to scan; supports `~/...`, absolute paths, and home-relative names like `Documents`; default: `Desktop,Documents,Downloads`
- `file_scan_depth`: recursion depth for file scanning (positive integer); default: `4`
- `file_scan_limit`: max indexed files per refresh (positive integer); default: `8000`
- `file_exclude_paths`: comma-separated paths to exclude from file/folder indexing (supports `~/...`, absolute paths, and home-relative names); default: empty
- `translate_allow_network`: allow network translation requests (`true`/`false`); default: `false`
- `backend_log_level`: backend log verbosity (`error`/`info`/`debug`); default: `error`
- `launch_at_login`: auto-start look after user sign-in (`true`/`false`); default: `true`
- `skip_dir_names`: comma-separated directory names to ignore during file scan (case-insensitive); default: `node_modules,target,build,dist,library,applications,old firefox data`

UI keys:

- `ui_tint_red`: launcher tint red channel (`0..1`); default: `0.08`
- `ui_tint_green`: launcher tint green channel (`0..1`); default: `0.10`
- `ui_tint_blue`: launcher tint blue channel (`0..1`); default: `0.12`
- `ui_tint_opacity`: launcher tint opacity (`0..1`); default: `0.55`
- `ui_blur_material`: blur material (`hudWindow`, `sidebar`, `menu`, `underWindowBackground`); default: `hudWindow`
- `ui_blur_opacity`: blur layer opacity (`0..1`); default: `0.95`
- `ui_font_name`: macOS installed font family/name (example: `SF Pro Text`, `Menlo`); default: `SF Pro Text`
- `ui_font_size`: base UI font size (positive number); default: `14`
- `ui_font_red`: text red channel (`0..1`); default: `0.96`
- `ui_font_green`: text green channel (`0..1`); default: `0.96`
- `ui_font_blue`: text blue channel (`0..1`); default: `0.98`
- `ui_font_opacity`: text opacity (`0..1`); default: `0.96`
- `ui_border_thickness`: launcher border thickness (positive number); default: `1.0`
- `ui_border_red`: border red channel (`0..1`); default: `1.0`
- `ui_border_green`: border green channel (`0..1`); default: `1.0`
- `ui_border_blue`: border blue channel (`0..1`); default: `1.0`
- `ui_border_opacity`: border opacity (`0..1`); default: `0.12`

Config behavior:

- unknown keys are ignored
- invalid values are ignored and existing/default values are kept
- `#` starts a comment on a line

Logging privacy:

- default log level is `error`
- set `LOOK_LOG_LEVEL=info` or `LOOK_LOG_LEVEL=debug` only for local troubleshooting
- debug logs avoid raw query text and candidate IDs/actions

Example:

```text
# ~/.look.config
# Backend indexing
app_scan_roots=/Applications,/System/Applications,/System/Applications/Utilities
app_scan_depth=3
app_exclude_paths=
app_exclude_names=
file_scan_roots=Desktop,Documents,Downloads
file_scan_depth=4
file_scan_limit=8000
file_exclude_paths=
translate_allow_network=false
backend_log_level=error
launch_at_login=true
skip_dir_names=node_modules,target,build,dist,library,applications,old firefox data

# UI theme
ui_tint_red=0.08
ui_tint_green=0.10
ui_tint_blue=0.12
ui_tint_opacity=0.55
ui_blur_material=hudWindow
ui_blur_opacity=0.95
ui_font_name=SF Pro Text
ui_font_size=14
ui_font_red=0.96
ui_font_green=0.96
ui_font_blue=0.98
ui_font_opacity=0.96
ui_border_thickness=1.0
ui_border_red=1.0
ui_border_green=1.0
ui_border_blue=1.0
ui_border_opacity=0.12
```

## Keyboard shortcuts reference

- `Tab`: next result / next command
- `Shift+Tab`: previous result / previous command
- `Enter`: open selected result, run command, or confirm kill
- `a"`: apps-only search prefix
- `f"`: files-only search prefix
- `d"`: folders-only search prefix
- `r"`: regex search prefix
- `c"`: clipboard history search prefix
- `Cmd+/`: enter command mode
- `Cmd+H`: toggle in-window keyboard help screen
- `Escape`: back to app list (in command mode), otherwise hide launcher
- `Shift+Escape`: hide launcher
- `Cmd+Enter`: search query on Google
- `Cmd+C`: copy selected file/folder to pasteboard
- `Cmd+F`: reveal selected app/file/folder in Finder
- `Cmd+Escape`: back to command list (`calc`) while staying in command mode
- `Cmd+Q`: hide launcher
- `Cmd+Option+Q`: quit app
- `Cmd+Shift+,`: open/close settings panel
- `Cmd+Shift+;`: reload `.look.config`
- `Cmd+-`: zoom out (temporary UI scale)
- `Cmd+=` (`Cmd++`): zoom in (temporary UI scale)
- `Cmd+0`: reset temporary UI scale

### Kill command shortcuts

- `Up` / `Down`: navigate app list
- `Cmd+1` / `Cmd+2` / `Cmd+3`: switch command
- `Enter`: select app (shows confirmation)
- `Y` / click "Yes": confirm kill
- `N` / click "No": cancel
- `Cmd+Escape`: back to command list (calc)

## What look is for

look is built for:

- users who prefer keyboard navigation
- fast app/file lookup without distractions
- quick command-style utility actions in one place

It is not trying to be a full plugin ecosystem or cloud assistant. The core goal is speed, clarity, and predictable local behavior.

## Roadmap style:

- look will continue to add features that keep the launcher simple, fast, and local-first
- user ideas are encouraged; strong proposals can be added to upcoming milestones
- near-future direction includes plugin/extension injection support for developer workflows

## Platform roadmap

- current focus: macOS
- planned next platform: Windows
- Linux is not a near-term target; existing Linux launcher tooling (for example `rofi`) already serves similar use cases well

## Author

- Kunkka
