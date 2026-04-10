# Configuration Reference

Runtime config file:

- default: `~/.look.config`
- override: `LOOK_CONFIG_PATH=/path/to/config`
- reload in app: `Cmd+Shift+;`

Format:

- one `key=value` per line
- `#` starts a comment
- unknown keys ignored
- invalid values fall back to defaults

## Backend keys

- `app_scan_roots`
- `app_scan_depth`
- `app_exclude_paths`
- `app_exclude_names`
- `file_scan_roots`
- `file_scan_depth`
- `file_scan_limit`
- `file_exclude_paths`
- `skip_dir_names`
- `translate_allow_network`
- `backend_log_level`
- `launch_at_login`

## UI keys

- `ui_tint_red`, `ui_tint_green`, `ui_tint_blue`, `ui_tint_opacity`
- `ui_blur_material`, `ui_blur_opacity`
- `ui_font_name`, `ui_font_size`
- `ui_font_red`, `ui_font_green`, `ui_font_blue`, `ui_font_opacity`
- `ui_border_thickness`
- `ui_border_red`, `ui_border_green`, `ui_border_blue`, `ui_border_opacity`

## Translation privacy

- `translate_allow_network=false` blocks network translation requests
- env override supported: `LOOK_TRANSLATE_ALLOW_NETWORK=true`
