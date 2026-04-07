## Summary

-

## Why

-

## Changes

-

## Testing

- [ ] `cargo check --workspace --manifest-path core/Cargo.toml`
- [ ] `cargo check --manifest-path bridge/ffi/Cargo.toml`
- [ ] `cd apps/macos/LauncherApp && swift test`
- [ ] `xcodebuild -project "apps/macos/LauncherApp/look-app.xcodeproj" -scheme "Look" -configuration Debug -sdk macosx build`
- [ ] Manual verification completed (if UI/behavior changed)

## Screenshots / Recordings (if UI changed)

### Before

### After

## Risks / Notes

-

## Checklist

- [ ] PR title is clear and scoped
- [ ] Docs updated for user-visible changes
- [ ] No secrets or private files included
- [ ] Backward compatibility considered
