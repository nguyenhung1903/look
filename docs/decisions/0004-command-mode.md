# ADR 0004: Keyboard command mode in launcher UI

## Status

Accepted

## Context

Users want quick utility actions (math and shell) without leaving the launcher flow.

## Decision

- Use `Cmd+/` to enter command mode.
- Default selected command is `calc`.
- Support command switching with `Tab` / `Shift+Tab`.
- Keep command input when switching command type.
- `Esc` exits command mode back to app list.
- `Shift+Esc` hides launcher.

## Consequences

- command utilities stay keyboard-first and low-friction
- launcher preserves a single-window interaction model
- additional command features can be added incrementally
