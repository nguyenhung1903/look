use crate::config::RuntimeConfig;
use look_indexing::{Candidate, CandidateKind};
use std::collections::HashSet;
use std::fs;

pub fn discover_local_files_and_folders(
    config: &RuntimeConfig,
    seen: &mut HashSet<String>,
    out: &mut Vec<Candidate>,
) {
    let roots = &config.file_scan_roots;

    let mut file_count = 0usize;
    for root in roots {
        walk_files(
            root,
            config.file_scan_depth,
            seen,
            out,
            &mut file_count,
            config.file_scan_limit,
            &config.skip_dir_names,
        );
    }
}

fn walk_files(
    path: &str,
    depth: usize,
    seen: &mut HashSet<String>,
    out: &mut Vec<Candidate>,
    file_count: &mut usize,
    file_limit: usize,
    skip_dir_names: &[String],
) {
    if depth == 0 || *file_count >= file_limit {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    for entry in entries.flatten() {
        if *file_count >= file_limit {
            return;
        }

        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path_buf = entry.path();
        let Some(path_str) = path_buf.to_str() else {
            continue;
        };
        let Some(name) = path_buf.file_name().and_then(|s| s.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }

        if file_type.is_dir() {
            if !name.ends_with(".app") {
                if should_skip_dir(name, skip_dir_names) {
                    continue;
                }

                let key = format!("folder:{}", path_str.to_lowercase());
                if seen.insert(key.clone()) {
                    let mut c = Candidate::new(&key, CandidateKind::Folder, name, path_str);
                    c.subtitle = Some("folder".to_string());
                    out.push(c);
                }
                walk_files(
                    path_str,
                    depth - 1,
                    seen,
                    out,
                    file_count,
                    file_limit,
                    skip_dir_names,
                );
            }
        } else if file_type.is_file() {
            *file_count += 1;
            let key = format!("file:{}", path_str.to_lowercase());
            if seen.insert(key.clone()) {
                let mut c = Candidate::new(&key, CandidateKind::File, name, path_str);
                c.subtitle = Some("file".to_string());
                out.push(c);
            }
        }
    }
}

fn should_skip_dir(name: &str, skip_dir_names: &[String]) -> bool {
    let lower = name.to_lowercase();
    skip_dir_names.iter().any(|entry| entry == &lower)
}
