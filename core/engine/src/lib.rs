pub mod action;
pub mod config;
pub mod index;
mod normalize;
mod query;
pub mod result;
mod scoring;
mod search;

pub use action::{ActionKind, LaunchAction};
use config::RuntimeConfig;
use index::discover_candidates;
use look_indexing::{Candidate, CandidateKind};
use look_storage::{SearchSettings, SqliteStore, StorageError};
pub use result::{LaunchResult, LaunchResultAction};
use std::collections::HashSet;
use std::path::Path;

#[derive(Default)]
pub struct QueryEngine {
    candidates: Vec<Candidate>,
}

impl QueryEngine {
    pub fn new(candidates: Vec<Candidate>) -> Self {
        Self { candidates }
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<LaunchResult> {
        let scored = self.search_scored(query, limit);
        scored
            .into_iter()
            .map(|(candidate, score)| LaunchResult::from((&candidate, score)))
            .collect()
    }

    pub fn record_usage_in_memory(&mut self, candidate_id: &str, used_at_unix_s: i64) -> bool {
        if let Some(candidate) = self.candidates.iter_mut().find(|c| c.id == candidate_id) {
            candidate.use_count = candidate.use_count.saturating_add(1);
            candidate.last_used_at_unix_s = Some(used_at_unix_s);
            return true;
        }
        false
    }

    pub fn demo_seed() -> Self {
        Self {
            candidates: Self::demo_candidates(),
        }
    }

    pub fn demo_candidates() -> Vec<Candidate> {
        vec![
            Candidate::new(
                "app.safari",
                CandidateKind::App,
                "Safari",
                "/Applications/Safari.app",
            ),
            Candidate::new(
                "app.vscode",
                CandidateKind::App,
                "Visual Studio Code",
                "/Applications/Visual Studio Code.app",
            ),
            Candidate::new(
                "file.notes",
                CandidateKind::File,
                "Notes.txt",
                "/Users/user/Documents/Notes.txt",
            ),
            Candidate::new(
                "folder.docs",
                CandidateKind::Folder,
                "Documents",
                "/Users/user/Documents",
            ),
        ]
    }

    pub fn from_sqlite(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let store = SqliteStore::open(path)?;
        let candidates = store.load_candidates(None)?;
        if candidates.is_empty() {
            return Ok(Self {
                candidates: Self::demo_candidates(),
            });
        }

        Ok(Self { candidates })
    }

    pub fn bootstrap_sqlite(path: impl AsRef<Path>) -> Result<(), StorageError> {
        let mut store = SqliteStore::open(path)?;
        let runtime_config = RuntimeConfig::load();
        let discovered_candidates = discover_candidates(&runtime_config);
        if discovered_candidates.is_empty() {
            return Ok(());
        }

        let existing = store.load_candidates(None)?;
        if looks_like_demo_seed(&existing) {
            store.replace_candidates(&discovered_candidates)?;
        } else {
            store.upsert_candidates(&discovered_candidates)?;
        }
        Ok(())
    }

    pub fn build_web_search_url(query: &str, settings: SearchSettings) -> Option<String> {
        let normalized_query = query.trim();
        if !settings.web_search_enabled || normalized_query.is_empty() {
            return None;
        }

        Some(
            settings
                .web_search_engine
                .build_search_url(normalized_query),
        )
    }
}

fn looks_like_demo_seed(candidates: &[Candidate]) -> bool {
    if candidates.len() > 6 {
        return false;
    }

    let ids: HashSet<&str> = candidates.iter().map(|c| c.id.as_str()).collect();
    ids.contains("app.safari") && ids.contains("app.vscode")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scoring::default_browse_score;

    fn sample_engine() -> QueryEngine {
        QueryEngine::new(vec![
            Candidate::new(
                "app.safari",
                CandidateKind::App,
                "Safari",
                "/Applications/Safari.app",
            ),
            Candidate::new(
                "app.vscode",
                CandidateKind::App,
                "Visual Studio Code",
                "/Applications/Visual Studio Code.app",
            ),
            Candidate::new(
                "file.notes",
                CandidateKind::File,
                "Notes.txt",
                "/Users/test/Documents/Notes.txt",
            ),
            Candidate::new(
                "folder.docs",
                CandidateKind::Folder,
                "Documents",
                "/Users/test/Documents",
            ),
        ])
    }

    #[test]
    fn app_prefix_filters_to_apps() {
        let engine = sample_engine();
        let results = engine.search_scored("a\"saf", 10);
        assert!(
            results
                .iter()
                .all(|(candidate, _)| candidate.kind == CandidateKind::App)
        );
        assert!(
            results
                .iter()
                .any(|(candidate, _)| candidate.id == "app.safari")
        );
    }

    #[test]
    fn file_prefix_filters_to_files() {
        let engine = sample_engine();
        let results = engine.search_scored("f\"notes", 10);
        assert!(
            results
                .iter()
                .all(|(candidate, _)| candidate.kind == CandidateKind::File)
        );
        assert_eq!(
            results.first().map(|(candidate, _)| candidate.id.as_str()),
            Some("file.notes")
        );
    }

    #[test]
    fn directory_prefix_filters_to_folders() {
        let engine = sample_engine();
        let results = engine.search_scored("d\"doc", 10);
        assert!(
            results
                .iter()
                .all(|(candidate, _)| candidate.kind == CandidateKind::Folder)
        );
        assert_eq!(
            results.first().map(|(candidate, _)| candidate.id.as_str()),
            Some("folder.docs")
        );
    }

    #[test]
    fn regex_prefix_matches_by_pattern() {
        let engine = sample_engine();
        let results = engine.search_scored("r\"^Visual.*Code$", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "app.vscode");
    }

    #[test]
    fn regex_prefix_returns_empty_on_invalid_pattern() {
        let engine = sample_engine();
        let results = engine.search_scored("r\"([", 10);
        assert!(results.is_empty());
    }

    #[test]
    fn vietnamese_diacritics_query_matches_ascii_titles() {
        let engine = QueryEngine::new(vec![Candidate::new(
            "app.terminal",
            CandidateKind::App,
            "Terminal",
            "/System/Applications/Utilities/Terminal.app",
        )]);

        let results = engine.search_scored("tẻrminal", 10);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0.id, "app.terminal");
    }

    #[test]
    fn empty_query_prioritizes_recent_and_frequent_apps() {
        let mut frequent_app = Candidate::new(
            "app.frequent",
            CandidateKind::App,
            "Frequent",
            "/Applications/Frequent.app",
        );
        frequent_app.use_count = 25;
        frequent_app.last_used_at_unix_s = Some(4_102_444_800);

        let mut less_used_app = Candidate::new(
            "app.less",
            CandidateKind::App,
            "Less",
            "/Applications/Less.app",
        );
        less_used_app.use_count = 1;

        let folder = Candidate::new(
            "folder.docs",
            CandidateKind::Folder,
            "Documents",
            "/Users/test/Documents",
        );

        let file = Candidate::new(
            "file.notes",
            CandidateKind::File,
            "Notes.txt",
            "/Users/test/Documents/Notes.txt",
        );

        let engine = QueryEngine::new(vec![file, folder, less_used_app, frequent_app]);
        let results = engine.search_scored("", 4);
        let ordered_ids: Vec<&str> = results
            .iter()
            .map(|(candidate, _)| candidate.id.as_str())
            .collect();

        assert_eq!(ordered_ids[0], "app.frequent");
        assert_eq!(ordered_ids[1], "app.less");
        assert!(
            ordered_ids.iter().position(|id| *id == "folder.docs")
                < ordered_ids.iter().position(|id| *id == "file.notes")
        );
    }

    #[test]
    fn empty_query_can_prioritize_frequent_settings_entries() {
        let now = 1_775_462_400; // 2026-04-06 16:00:00 UTC

        let mut display_setting = Candidate::new(
            "setting:com.apple.displays-settings.extension",
            CandidateKind::App,
            "Display",
            "x-apple.systempreferences:com.apple.displays-settings.extension",
        );
        display_setting.subtitle = Some("System Settings display monitor".to_string());
        display_setting.use_count = 16;
        display_setting.last_used_at_unix_s = Some(now - 60 * 60 * 20);

        let mut newly_opened_app = Candidate::new(
            "app.new",
            CandidateKind::App,
            "Newly Opened",
            "/Applications/Newly Opened.app",
        );
        newly_opened_app.use_count = 1;
        newly_opened_app.last_used_at_unix_s = Some(now);

        assert!(
            default_browse_score(&display_setting, now)
                > default_browse_score(&newly_opened_app, now)
        );

        let engine = QueryEngine::new(vec![newly_opened_app, display_setting]);
        let results = engine.search_scored("", 10);
        assert_eq!(
            results.first().map(|(candidate, _)| candidate.id.as_str()),
            Some("setting:com.apple.displays-settings.extension")
        );
    }

    #[test]
    fn empty_query_prefers_more_recent_app_when_usage_is_equal() {
        let now = 1_775_462_400; // 2026-04-06 16:00:00 UTC

        let mut display_setting = Candidate::new(
            "setting:com.apple.displays-settings.extension",
            CandidateKind::App,
            "Display",
            "x-apple.systempreferences:com.apple.displays-settings.extension",
        );
        display_setting.subtitle = Some("System Settings display monitor".to_string());
        display_setting.use_count = 1;
        display_setting.last_used_at_unix_s = Some(now - 60 * 60 * 12);

        let mut newly_opened_app = Candidate::new(
            "app.new",
            CandidateKind::App,
            "Newly Opened",
            "/Applications/Newly Opened.app",
        );
        newly_opened_app.use_count = 1;
        newly_opened_app.last_used_at_unix_s = Some(now);

        assert!(
            default_browse_score(&newly_opened_app, now)
                > default_browse_score(&display_setting, now)
        );

        let engine = QueryEngine::new(vec![display_setting, newly_opened_app]);
        let results = engine.search_scored("", 10);
        assert_eq!(
            results.first().map(|(candidate, _)| candidate.id.as_str()),
            Some("app.new")
        );
    }

    #[test]
    fn slash_path_query_matches_nested_path_segments() {
        let engine = QueryEngine::new(vec![
            Candidate::new(
                "file.repo.readme",
                CandidateKind::File,
                "README.md",
                "/Users/test/Documents/git/books-pc/README.md",
            ),
            Candidate::new(
                "file.other",
                CandidateKind::File,
                "todo.txt",
                "/Users/test/Downloads/todo.txt",
            ),
        ]);

        let results = engine.search_scored("git/books-pc", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].0.id, "file.repo.readme");
    }
}
