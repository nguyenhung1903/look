# Getting Started

## Requirements

- macOS 15.0+

## Install

```bash
brew tap kunkka19xx/tap
brew install --cask look
```

If `Cmd+Space` is still opening Spotlight, rebind or disable Spotlight shortcut:

- `System Settings` -> `Keyboard` -> `Keyboard Shortcuts...` -> `Spotlight`

## First launch notes

If app is unsigned/not notarized, macOS may block first run:

- right-click `Look.app` -> `Open` -> confirm, or
- `System Settings` -> `Privacy & Security` -> `Open Anyway`

## Basic usage

- open launcher with `Cmd+Space`
- type query and press `Enter` to open selection
- `Cmd+F` reveal selected item in Finder
- `Cmd+Enter` web search current query

## Query prefixes

- `a"term` apps only
- `f"term` files only
- `d"term` folders only
- `r"pattern` regex mode
- `c"term` clipboard history mode
- `t"text` translate panel
- `tw"text` dictionary lookup panel
