use std::env;
use std::path::{Path, PathBuf};

pub const APP_SCAN_ROOTS: [&str; 4] = [
    "/Applications",
    "/System/Applications",
    "/System/Applications/Utilities",
    "/System/Library/CoreServices/Finder.app/Contents/Applications",
];

pub const APP_SCAN_DEPTH: usize = 3;
pub const APP_EXCLUDE_PATHS: [&str; 0] = [];
pub const APP_EXCLUDE_NAMES: [&str; 0] = [];

pub const FILE_SCAN_ROOT_SUFFIXES: [&str; 3] = ["Desktop", "Documents", "Downloads"];
pub const FILE_SCAN_DEPTH: usize = 4;
pub const FILE_SCAN_DEPTH_MIN: usize = 1;
pub const FILE_SCAN_DEPTH_MAX: usize = 12;
pub const FILE_SCAN_LIMIT: usize = 8000;
pub const FILE_SCAN_LIMIT_MIN: usize = 500;
pub const FILE_SCAN_LIMIT_MAX: usize = 50_000;
pub const FILE_EXCLUDE_PATHS: [&str; 0] = [];

pub const SCORE_TITLE_CONTAINS: i64 = 1200;
pub const SCORE_SUBTITLE_CONTAINS: i64 = 900;
pub const SCORE_TOKEN_ALL_MATCH: i64 = 850;
pub const SCORE_REGEX_TITLE_AND_PATH: i64 = 1500;
pub const SCORE_REGEX_TITLE_ONLY: i64 = 1300;
pub const SCORE_REGEX_PATH_ONLY: i64 = 1100;
pub const SCORE_REGEX_SUBTITLE_ONLY: i64 = 1000;

pub const BIAS_APP: i64 = 220;
pub const BIAS_FOLDER: i64 = 0;
pub const BIAS_FILE: i64 = -20;

pub const BIAS_SETTINGS_MATCH: i64 = 420;
pub const BIAS_APP_ON_SETTINGS_QUERY: i64 = 120;
pub const BIAS_NON_APP_ON_SETTINGS_QUERY: i64 = -260;

pub const QUERY_SETTINGS_HINTS: [&str; 6] = [
    "setting",
    "display",
    "network",
    "bluetooth",
    "privacy",
    "sound",
];

pub const SKIP_DIR_NAMES: [&str; 15] = [
    "node_modules",
    "target",
    "build",
    "dist",
    "library",
    "applications",
    "old firefox data",
    "deriveddata",
    "pods",
    "vendor",
    "out",
    "coverage",
    "tmp",
    "cache",
    "venv",
];

#[derive(Clone, Debug)]
pub struct RuntimeConfig {
    pub app_scan_roots: Vec<String>,
    pub app_scan_depth: usize,
    pub app_exclude_paths: Vec<String>,
    pub app_exclude_names: Vec<String>,
    pub file_scan_roots: Vec<String>,
    pub file_scan_depth: usize,
    pub file_scan_limit: usize,
    pub file_exclude_paths: Vec<String>,
    pub skip_dir_names: Vec<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            app_scan_roots: APP_SCAN_ROOTS
                .iter()
                .map(|value| value.to_string())
                .collect(),
            app_scan_depth: APP_SCAN_DEPTH,
            app_exclude_paths: APP_EXCLUDE_PATHS
                .iter()
                .map(|value| value.to_string())
                .collect(),
            app_exclude_names: APP_EXCLUDE_NAMES
                .iter()
                .map(|value| value.to_string())
                .collect(),
            file_scan_roots: default_file_scan_roots(),
            file_scan_depth: FILE_SCAN_DEPTH,
            file_scan_limit: FILE_SCAN_LIMIT,
            file_exclude_paths: FILE_EXCLUDE_PATHS
                .iter()
                .map(|value| value.to_string())
                .collect(),
            skip_dir_names: SKIP_DIR_NAMES
                .iter()
                .map(|value| value.to_string())
                .collect(),
        }
    }
}

impl RuntimeConfig {
    pub fn load() -> Self {
        let mut config = Self::default();
        if let Some(path) = config_path() {
            ensure_default_config_file(&path);
            config.apply_from_file(&path);
        }
        config
    }

    fn apply_from_file(&mut self, path: &Path) {
        let Ok(contents) = std::fs::read_to_string(path) else {
            return;
        };

        let home = env::var("HOME").ok();
        for raw_line in contents.lines() {
            let line = strip_comments(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();

            match key {
                "app_scan_roots" => {
                    let parsed = parse_csv(value);
                    if !parsed.is_empty() {
                        self.app_scan_roots = parsed;
                    }
                }
                "app_scan_depth" => {
                    if let Some(parsed) = parse_positive_usize(value) {
                        self.app_scan_depth = parsed;
                    }
                }
                "app_exclude_paths" => {
                    self.app_exclude_paths = parse_csv(value)
                        .into_iter()
                        .map(|entry| expand_path(&entry, home.as_deref()))
                        .collect::<Vec<_>>();
                }
                "app_exclude_names" => {
                    self.app_exclude_names = parse_csv(value)
                        .into_iter()
                        .map(|entry| normalize_app_name(&entry))
                        .collect::<Vec<_>>();
                }
                "file_scan_roots" => {
                    let parsed = parse_csv(value)
                        .into_iter()
                        .map(|entry| expand_path(&entry, home.as_deref()))
                        .collect::<Vec<_>>();
                    if !parsed.is_empty() {
                        self.file_scan_roots = parsed;
                    }
                }
                "file_scan_depth" => {
                    if let Some(parsed) = parse_positive_usize(value) {
                        self.file_scan_depth =
                            parsed.clamp(FILE_SCAN_DEPTH_MIN, FILE_SCAN_DEPTH_MAX);
                    }
                }
                "file_scan_limit" => {
                    if let Some(parsed) = parse_positive_usize(value) {
                        self.file_scan_limit =
                            parsed.clamp(FILE_SCAN_LIMIT_MIN, FILE_SCAN_LIMIT_MAX);
                    }
                }
                "file_exclude_paths" => {
                    self.file_exclude_paths = parse_csv(value)
                        .into_iter()
                        .map(|entry| expand_path(&entry, home.as_deref()))
                        .collect::<Vec<_>>();
                }
                "skip_dir_names" => {
                    let parsed = parse_csv(value)
                        .into_iter()
                        .map(|entry| entry.to_lowercase())
                        .collect::<Vec<_>>();
                    if !parsed.is_empty() {
                        for entry in parsed {
                            if !self
                                .skip_dir_names
                                .iter()
                                .any(|existing| existing == &entry)
                            {
                                self.skip_dir_names.push(entry);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

fn config_path() -> Option<PathBuf> {
    if let Ok(custom) = env::var("LOOK_CONFIG_PATH") {
        let trimmed = custom.trim();
        if !trimmed.is_empty() {
            return Some(PathBuf::from(trimmed));
        }
    }

    env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(".look.config"))
}

fn ensure_default_config_file(path: &Path) {
    if path.exists() {
        return;
    }

    let _ = std::fs::write(path, default_config_contents());
}

fn default_config_contents() -> &'static str {
    "# look configuration\n\
# Generated on first launch. Edit values and press Cmd+Shift+; to reload.\n\
\n\
# Backend indexing (file_scan_depth: 1-12, file_scan_limit: 500-50000)\n\
app_scan_roots=/Applications,/System/Applications,/System/Applications/Utilities,/System/Library/CoreServices/Finder.app/Contents/Applications\n\
app_scan_depth=3\n\
app_exclude_paths=\n\
app_exclude_names=\n\
file_scan_roots=Desktop,Documents,Downloads\n\
file_scan_depth=4\n\
file_scan_limit=8000\n\
file_exclude_paths=\n\
skip_dir_names=node_modules,target,build,dist,library,applications,old firefox data,deriveddata,pods,vendor,out,coverage,tmp,cache,venv\n\
\n\
# UI theme\n\
ui_tint_red=0.08\n\
ui_tint_green=0.10\n\
ui_tint_blue=0.12\n\
ui_tint_opacity=0.55\n\
ui_blur_material=hudWindow\n\
ui_blur_opacity=0.95\n\
ui_font_name=SF Pro Text\n\
ui_font_size=14\n\
ui_font_red=0.96\n\
ui_font_green=0.96\n\
ui_font_blue=0.98\n\
ui_font_opacity=0.96\n\
ui_border_thickness=1.0\n\
ui_border_red=1.0\n\
ui_border_green=1.0\n\
ui_border_blue=1.0\n\
ui_border_opacity=0.12\n"
}

fn default_file_scan_roots() -> Vec<String> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    FILE_SCAN_ROOT_SUFFIXES
        .iter()
        .map(|suffix| format!("{home}/{suffix}"))
        .collect()
}

fn strip_comments(value: &str) -> &str {
    value
        .split_once('#')
        .map(|(prefix, _)| prefix)
        .unwrap_or(value)
}

fn parse_csv(value: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut escaping = false;

    for ch in value.chars() {
        if escaping {
            current.push(ch);
            escaping = false;
            continue;
        }

        if ch == '\\' {
            escaping = true;
            continue;
        }

        if ch == ',' {
            let trimmed = current.trim();
            if !trimmed.is_empty() {
                values.push(trimmed.to_string());
            }
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if escaping {
        current.push('\\');
    }

    let trimmed = current.trim();
    if !trimmed.is_empty() {
        values.push(trimmed.to_string());
    }

    values
}

fn parse_positive_usize(value: &str) -> Option<usize> {
    value
        .trim()
        .parse::<usize>()
        .ok()
        .filter(|parsed| *parsed > 0)
}

fn expand_path(value: &str, home: Option<&str>) -> String {
    if value.starts_with("~/") {
        return home
            .map(|prefix| format!("{prefix}/{}", value.trim_start_matches("~/")))
            .unwrap_or_else(|| value.to_string());
    }

    if value.starts_with('/') {
        return value.to_string();
    }

    home.map(|prefix| format!("{prefix}/{value}"))
        .unwrap_or_else(|| value.to_string())
}

fn normalize_app_name(value: &str) -> String {
    value.trim().trim_end_matches(".app").trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_skips_empty_tokens() {
        let parsed = parse_csv("Desktop, Documents, ,Downloads");
        assert_eq!(parsed, vec!["Desktop", "Documents", "Downloads"]);
    }

    #[test]
    fn parse_csv_supports_escaped_commas() {
        let parsed = parse_csv("/Users/demo/Foo\\,Bar,/Users/demo/Baz");
        assert_eq!(parsed, vec!["/Users/demo/Foo,Bar", "/Users/demo/Baz"]);
    }

    #[test]
    fn expand_path_supports_home_tokens() {
        let home = Some("/Users/demo");
        assert_eq!(expand_path("~/Projects", home), "/Users/demo/Projects");
        assert_eq!(expand_path("Documents", home), "/Users/demo/Documents");
        assert_eq!(expand_path("/tmp", home), "/tmp");
    }

    #[test]
    fn parse_positive_usize_rejects_invalid_values() {
        assert_eq!(parse_positive_usize("5"), Some(5));
        assert_eq!(parse_positive_usize("0"), None);
        assert_eq!(parse_positive_usize("not-a-number"), None);
    }

    #[test]
    fn normalize_app_name_handles_suffix_and_case() {
        assert_eq!(normalize_app_name("Safari.app"), "safari");
        assert_eq!(
            normalize_app_name("  Visual Studio Code  "),
            "visual studio code"
        );
    }

    #[test]
    fn app_scan_roots_include_finder_embedded_apps() {
        assert!(
            APP_SCAN_ROOTS.iter().any(
                |root| root == &"/System/Library/CoreServices/Finder.app/Contents/Applications"
            )
        );
    }

    #[test]
    fn skip_dir_names_from_config_are_appended_not_replaced() {
        let tmp = std::env::temp_dir().join(format!(
            "look-config-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));

        std::fs::write(&tmp, "skip_dir_names=vendor\n").expect("should write temporary config");

        let mut config = RuntimeConfig::default();
        config.apply_from_file(&tmp);

        assert!(
            config
                .skip_dir_names
                .iter()
                .any(|name| name == "node_modules")
        );
        assert!(config.skip_dir_names.iter().any(|name| name == "vendor"));

        let _ = std::fs::remove_file(&tmp);
    }
}
