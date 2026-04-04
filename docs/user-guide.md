# look User Guide

look is a fast, keyboard-first launcher for macOS.

It helps you do three things quickly in one window:

- launch installed apps
- search local files and folders by name
- run quick commands (calculator and shell)

The interface is local-first, lightweight, and designed for low-friction daily use.

## What makes look different

- one focused launcher window instead of many utility apps
- keyboard-first workflow with instant mode switching
- transparent, blur-based UI that can be themed
- optional command mode for utility tasks without leaving context

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
- click a row to open it

### 2) Web search handoff

If you want to search the web from the same query:

- press `Cmd+Enter`

Current default provider: Google.

### 3) Command mode

To enter command mode:

- type `/`

Command mode starts with `calc` selected by default.

In command mode:

- `Tab` / `Shift+Tab` switches command type
- `Enter` runs the current command input
- `Shift+Esc` exits command mode

Available commands:

- `calc`: evaluate math expressions
- `shell`: run shell commands

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
- background: image picker, layout mode, image blur, image opacity
- shortcuts: built-in documentation tab

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

Supported config keys (`key=value`):

Backend indexing keys:

- `app_scan_roots`: comma-separated app roots to scan (absolute paths); default: `/Applications,/System/Applications,/System/Applications/Utilities`
- `app_scan_depth`: recursion depth for app scanning (positive integer); default: `3`
- `file_scan_roots`: comma-separated file roots to scan; supports `~/...`, absolute paths, and home-relative names like `Documents`; default: `Desktop,Documents,Downloads`
- `file_scan_depth`: recursion depth for file scanning (positive integer); default: `2`
- `file_scan_limit`: max indexed files per refresh (positive integer); default: `2000`
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

Example:

```text
# ~/.look.config
# Backend indexing
app_scan_roots=/Applications,/System/Applications,/System/Applications/Utilities
app_scan_depth=3
file_scan_roots=Desktop,Documents,Downloads
file_scan_depth=2
file_scan_limit=2000
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
- `Enter`: open selected result or run active command
- `/`: enter command mode
- `Shift+Esc`: exit command mode
- `Cmd+Enter`: search query on Google
- `Cmd+Shift+,`: open/close settings panel
- `Cmd+Shift+;`: reload `.look.config`
- `Cmd+-`: zoom out (temporary UI scale)
- `Cmd+=` (`Cmd++`): zoom in (temporary UI scale)
- `Cmd+0`: reset temporary UI scale

In Settings panel, use **Save Config** to write current UI values back to `~/.look.config`.
Font name supports installed-font suggestions in the UI.

## What look is for

look is built for:

- users who prefer keyboard navigation
- fast app/file lookup without distractions
- quick command-style utility actions in one place

It is not trying to be a full plugin ecosystem or cloud assistant. The core goal is speed, clarity, and predictable local behavior.
