# Troubleshooting

## Hotkey does not open launcher

- check Spotlight shortcut conflict (`Cmd+Space`)
- relaunch look and try again

## Results seem stale or incomplete

- reload config with `Cmd+Shift+;`
- verify `file_scan_roots`, `file_scan_depth`, `file_scan_limit`
- verify exclude rules are not too broad

## Translation returns warnings/no result

- check `translate_allow_network=true` if network translation is expected
- confirm connectivity

## macOS blocks first launch

- right-click app -> `Open` -> confirm
- or `Privacy & Security` -> `Open Anyway`

## Need to inspect local DB

```bash
sqlite3 "$HOME/Library/Application Support/look/look.db" "SELECT id,title,use_count FROM candidates ORDER BY use_count DESC LIMIT 20;"
```
