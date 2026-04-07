use crate::config::RuntimeConfig;
use crate::index::{FILE_CANDIDATE_ID_PREFIX, FOLDER_CANDIDATE_ID_PREFIX};
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
            &config.file_exclude_paths,
            &config.skip_dir_names,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn walk_files(
    path: &str,
    depth: usize,
    seen: &mut HashSet<String>,
    out: &mut Vec<Candidate>,
    file_count: &mut usize,
    file_limit: usize,
    file_exclude_paths: &[String],
    skip_dir_names: &[String],
) {
    if should_exclude_path(path, file_exclude_paths) {
        return;
    }

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
        if should_exclude_path(path_str, file_exclude_paths) {
            continue;
        }
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

                let key = format!("{FOLDER_CANDIDATE_ID_PREFIX}{}", path_str.to_lowercase());
                if seen.insert(key.clone()) {
                    let mut c = Candidate::new(&key, CandidateKind::Folder, name, path_str);
                    c.subtitle = Some(CandidateKind::Folder.as_str().to_string());
                    out.push(c);
                }
                walk_files(
                    path_str,
                    depth - 1,
                    seen,
                    out,
                    file_count,
                    file_limit,
                    file_exclude_paths,
                    skip_dir_names,
                );
            }
        } else if file_type.is_file() {
            *file_count += 1;
            let key = format!("{FILE_CANDIDATE_ID_PREFIX}{}", path_str.to_lowercase());
            if seen.insert(key.clone()) {
                let mut c = Candidate::new(&key, CandidateKind::File, name, path_str);
                c.subtitle = Some(CandidateKind::File.as_str().to_string());
                out.push(c);
            }
        }
    }
}

fn should_skip_dir(name: &str, skip_dir_names: &[String]) -> bool {
    let lower = name.to_lowercase();
    skip_dir_names.iter().any(|entry| entry == &lower)
}

fn should_exclude_path(path: &str, file_exclude_paths: &[String]) -> bool {
    let normalized_path = path.trim_end_matches('/');
    file_exclude_paths.iter().any(|entry| {
        let normalized_exclude = entry.trim().trim_end_matches('/');
        if normalized_exclude.is_empty() {
            return false;
        }
        normalized_path == normalized_exclude
            || normalized_path.starts_with(&format!("{normalized_exclude}/"))
    })
}

#[cfg(test)]
mod tests {
    use super::should_exclude_path;

    #[test]
    fn excludes_nested_paths_and_exact_matches() {
        let excludes = vec!["/Users/demo/Downloads/tmp".to_string()];
        assert!(should_exclude_path("/Users/demo/Downloads/tmp", &excludes));
        assert!(should_exclude_path(
            "/Users/demo/Downloads/tmp/cache/file.txt",
            &excludes
        ));
    }

    #[test]
    fn does_not_exclude_unrelated_paths() {
        let excludes = vec!["/Users/demo/Downloads/tmp".to_string()];
        assert!(!should_exclude_path(
            "/Users/demo/Downloads/template",
            &excludes
        ));
        assert!(!should_exclude_path(
            "/Users/demo/Documents/report.md",
            &excludes
        ));
    }

    #[test]
    fn handles_trailing_slashes_and_blank_entries() {
        let excludes = vec!["/Users/demo/Downloads/tmp/".to_string(), " ".to_string()];
        assert!(should_exclude_path("/Users/demo/Downloads/tmp", &excludes));
        assert!(should_exclude_path(
            "/Users/demo/Downloads/tmp/cache/a.txt",
            &excludes
        ));
    }

    #[test]
    fn path_prefix_is_boundary_aware() {
        let excludes = vec!["/Users/demo/Down".to_string()];
        assert!(!should_exclude_path("/Users/demo/Downloads", &excludes));
    }
}
