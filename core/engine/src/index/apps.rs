use crate::config::RuntimeConfig;
use look_indexing::{Candidate, CandidateKind};
use std::collections::HashSet;
use std::env;
use std::fs;

pub fn discover_installed_apps(
    config: &RuntimeConfig,
    seen: &mut HashSet<String>,
    out: &mut Vec<Candidate>,
) {
    let mut roots = config.app_scan_roots.clone();
    if let Ok(home) = env::var("HOME") {
        let home_apps = format!("{home}/Applications");
        if !roots.iter().any(|root| root == &home_apps) {
            roots.push(home_apps);
        }
    }

    for root in roots {
        walk_apps(&root, config.app_scan_depth, seen, out);
    }
}

fn walk_apps(path: &str, depth: usize, seen: &mut HashSet<String>, out: &mut Vec<Candidate>) {
    if depth == 0 {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_dir() {
            continue;
        }

        let app_path = entry.path();
        let Some(app_path_str) = app_path.to_str() else {
            continue;
        };

        if app_path_str.ends_with(".app") {
            let key = format!("app:{}", app_path_str.to_lowercase());
            if seen.insert(key.clone()) {
                let title = app_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("App")
                    .to_string();
                out.push(Candidate::new(
                    &key,
                    CandidateKind::App,
                    &title,
                    app_path_str,
                ));
            }
        } else {
            walk_apps(app_path_str, depth - 1, seen, out);
        }
    }
}
