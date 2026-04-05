pub mod action;
pub mod config;
pub mod index;
pub mod result;

pub use action::{ActionKind, LaunchAction};
use config::*;
use index::discover_candidates;
use look_indexing::{Candidate, CandidateKind};
use look_matching::fuzzy_score;
use look_ranking::rank_score;
use look_storage::{SearchSettings, SqliteStore, StorageError};
use regex::RegexBuilder;
pub use result::{LaunchResult, LaunchResultAction};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

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

    pub fn search_scored(&self, query: &str, limit: usize) -> Vec<(Candidate, i64)> {
        if limit == 0 {
            return vec![];
        }

        let parsed_query = ParsedQuery::from_input(query);
        if parsed_query.normalized_query.is_empty() && !parsed_query.is_regex {
            let now_unix_s = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let mut top = BinaryHeap::new();
            for candidate in self.candidates.iter().filter(|candidate| {
                parsed_query
                    .kind_filter
                    .as_ref()
                    .is_none_or(|kind| &candidate.kind == kind)
            }) {
                let score = default_browse_score(candidate, now_unix_s);
                push_top_k(&mut top, ScoredMatch::new(candidate.clone(), score), limit);
            }

            return finalize_top_k(top);
        }

        if parsed_query.is_regex {
            let Some(regex) = parsed_query.raw_query.as_ref().and_then(|pattern| {
                RegexBuilder::new(pattern)
                    .case_insensitive(true)
                    .build()
                    .ok()
            }) else {
                return vec![];
            };

            let mut top = BinaryHeap::new();
            for candidate in self.candidates.iter().filter(|candidate| {
                parsed_query
                    .kind_filter
                    .as_ref()
                    .is_none_or(|kind| &candidate.kind == kind)
            }) {
                let title_match = regex.is_match(&candidate.title);
                let path_match = regex.is_match(&candidate.path);
                let subtitle_match = candidate
                    .subtitle
                    .as_ref()
                    .is_some_and(|subtitle| regex.is_match(subtitle));

                if !(title_match || path_match || subtitle_match) {
                    continue;
                }

                let regex_score = match (title_match, path_match, subtitle_match) {
                    (true, true, _) => SCORE_REGEX_TITLE_AND_PATH,
                    (true, false, _) => SCORE_REGEX_TITLE_ONLY,
                    (false, true, _) => SCORE_REGEX_PATH_ONLY,
                    (false, false, true) => SCORE_REGEX_SUBTITLE_ONLY,
                    _ => SCORE_REGEX_PATH_ONLY,
                };

                let final_score =
                    regex_score + kind_bias(candidate) + path_depth_penalty(candidate);
                push_top_k(
                    &mut top,
                    ScoredMatch::new(candidate.clone(), final_score),
                    limit,
                );
            }

            return finalize_top_k(top);
        }

        let normalized_query = parsed_query.normalized_query;
        let mut top = BinaryHeap::new();
        let has_path_hint = normalized_query.contains('/');

        for candidate in self.candidates.iter().filter(|candidate| {
            parsed_query
                .kind_filter
                .as_ref()
                .is_none_or(|kind| &candidate.kind == kind)
        }) {
            let title_lower = candidate.title.to_lowercase();
            let subtitle_lower = candidate
                .subtitle
                .as_ref()
                .map(|subtitle| subtitle.to_lowercase());

            let title_score = fuzzy_score(&normalized_query, &title_lower);
            let subtitle_score = subtitle_lower
                .as_ref()
                .and_then(|subtitle| fuzzy_score(&normalized_query, subtitle))
                .map(|value| value / 2);
            let contains_score =
                contains_match_score(&normalized_query, &title_lower, subtitle_lower.as_deref());
            let path_score = if has_path_hint {
                path_match_score(&normalized_query, &candidate.path.to_lowercase())
            } else {
                None
            };

            let base = [title_score, subtitle_score, contains_score, path_score]
                .into_iter()
                .flatten()
                .max();

            let Some(base) = base else {
                continue;
            };

            let final_score = rank_score(base, &normalized_query, candidate, &title_lower)
                + kind_bias(candidate)
                + query_kind_penalty(&normalized_query, candidate)
                + path_depth_penalty(candidate);
            push_top_k(
                &mut top,
                ScoredMatch::new(candidate.clone(), final_score),
                limit,
            );
        }

        finalize_top_k(top)
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

#[derive(Clone, Debug)]
struct ParsedQuery {
    normalized_query: String,
    raw_query: Option<String>,
    kind_filter: Option<CandidateKind>,
    is_regex: bool,
}

impl ParsedQuery {
    fn from_input(input: &str) -> Self {
        let trimmed = input.trim();

        if let Some(rest) = strip_prefixed_query(trimmed, b'd') {
            return Self {
                normalized_query: rest.to_lowercase(),
                raw_query: Some(rest.to_string()),
                kind_filter: Some(CandidateKind::Folder),
                is_regex: false,
            };
        }

        if let Some(rest) = strip_prefixed_query(trimmed, b'f') {
            return Self {
                normalized_query: rest.to_lowercase(),
                raw_query: Some(rest.to_string()),
                kind_filter: Some(CandidateKind::File),
                is_regex: false,
            };
        }

        if let Some(rest) = strip_prefixed_query(trimmed, b'a') {
            return Self {
                normalized_query: rest.to_lowercase(),
                raw_query: Some(rest.to_string()),
                kind_filter: Some(CandidateKind::App),
                is_regex: false,
            };
        }

        if let Some(rest) = strip_prefixed_query(trimmed, b'r') {
            return Self {
                normalized_query: String::new(),
                raw_query: Some(rest.to_string()),
                kind_filter: None,
                is_regex: true,
            };
        }

        Self {
            normalized_query: trimmed.to_lowercase(),
            raw_query: if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            },
            kind_filter: None,
            is_regex: false,
        }
    }
}

fn strip_prefixed_query(input: &str, prefix: u8) -> Option<&str> {
    let bytes = input.as_bytes();
    if bytes.len() < 2 {
        return None;
    }

    if !bytes[0].eq_ignore_ascii_case(&prefix) || bytes[1] != b'"' {
        return None;
    }

    Some(input[2..].trim())
}

fn looks_like_demo_seed(candidates: &[Candidate]) -> bool {
    if candidates.len() > 6 {
        return false;
    }

    let ids: HashSet<&str> = candidates.iter().map(|c| c.id.as_str()).collect();
    ids.contains("app.safari") && ids.contains("app.vscode")
}

fn contains_match_score(query: &str, title: &str, subtitle: Option<&str>) -> Option<i64> {
    if title.contains(query) {
        return Some(SCORE_TITLE_CONTAINS);
    }

    if let Some(sub) = subtitle
        && sub.contains(query)
    {
        return Some(SCORE_SUBTITLE_CONTAINS);
    }

    let terms: Vec<&str> = query.split_whitespace().collect();
    if terms.is_empty() {
        return None;
    }

    if terms
        .iter()
        .all(|t| title.contains(t) || subtitle.is_some_and(|sub| sub.contains(t)))
    {
        return Some(SCORE_TOKEN_ALL_MATCH);
    }

    None
}

fn path_match_score(query: &str, path: &str) -> Option<i64> {
    if !query.contains('/') {
        return None;
    }

    let normalized = query.trim().trim_matches('/');
    if normalized.is_empty() {
        return None;
    }

    if path.contains(normalized) {
        return Some(1_350);
    }

    let tokens: Vec<&str> = normalized
        .split('/')
        .filter(|token| !token.is_empty())
        .collect();
    if tokens.len() < 2 {
        return None;
    }

    let mut cursor = 0usize;
    let mut total_gap = 0usize;
    for token in tokens {
        let remaining = &path[cursor..];
        let found_at = remaining.find(token)?;
        total_gap += found_at;
        cursor += found_at + token.len();
    }

    let penalty = (total_gap as i64).min(250);
    Some(1_050 - penalty)
}

fn kind_bias(candidate: &Candidate) -> i64 {
    match candidate.kind {
        CandidateKind::App => BIAS_APP,
        CandidateKind::Folder => BIAS_FOLDER,
        CandidateKind::File => BIAS_FILE,
    }
}

fn query_kind_penalty(query: &str, candidate: &Candidate) -> i64 {
    let looks_like_settings_query = QUERY_SETTINGS_HINTS
        .iter()
        .any(|token| query.contains(token));

    if looks_like_settings_query {
        match candidate.kind {
            CandidateKind::App => {
                if candidate
                    .subtitle
                    .as_deref()
                    .unwrap_or("")
                    .contains("System Settings")
                {
                    BIAS_SETTINGS_MATCH
                } else {
                    BIAS_APP_ON_SETTINGS_QUERY
                }
            }
            CandidateKind::Folder | CandidateKind::File => BIAS_NON_APP_ON_SETTINGS_QUERY,
        }
    } else {
        0
    }
}

fn path_depth_penalty(candidate: &Candidate) -> i64 {
    match candidate.kind {
        CandidateKind::App => 0,
        CandidateKind::File | CandidateKind::Folder => {
            let depth = candidate
                .path
                .split('/')
                .filter(|part| !part.is_empty())
                .count() as i64;
            -(depth / 2)
        }
    }
}

fn default_browse_score(candidate: &Candidate, now_unix_s: i64) -> i64 {
    let kind_boost = match candidate.kind {
        CandidateKind::App => 600,
        CandidateKind::Folder => 120,
        CandidateKind::File => 0,
    };

    let frequency = (candidate.use_count as i64) * 35;
    let recency = candidate
        .last_used_at_unix_s
        .map(|last| {
            let age_s = (now_unix_s - last).max(0);
            let age_hours = age_s / 3600;
            (2000 - (age_hours * 8)).max(0)
        })
        .unwrap_or(0);

    kind_boost + frequency + recency
}

#[derive(Clone, Debug)]
struct ScoredMatch {
    candidate: Candidate,
    score: i64,
    sort_title: String,
}

impl PartialEq for ScoredMatch {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.sort_title == other.sort_title
    }
}

impl Eq for ScoredMatch {}

impl ScoredMatch {
    fn new(candidate: Candidate, score: i64) -> Self {
        let sort_title = candidate.title.to_lowercase();
        Self {
            candidate,
            score,
            sort_title,
        }
    }
}

impl Ord for ScoredMatch {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.score.cmp(&other.score) {
            Ordering::Less => Ordering::Greater,
            Ordering::Greater => Ordering::Less,
            Ordering::Equal => self.sort_title.cmp(&other.sort_title),
        }
    }
}

impl PartialOrd for ScoredMatch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn push_top_k(heap: &mut BinaryHeap<ScoredMatch>, item: ScoredMatch, limit: usize) {
    if heap.len() < limit {
        heap.push(item);
        return;
    }

    if let Some(worst) = heap.peek()
        && item < *worst
    {
        let _ = heap.pop();
        heap.push(item);
    }
}

fn finalize_top_k(heap: BinaryHeap<ScoredMatch>) -> Vec<(Candidate, i64)> {
    let mut out: Vec<(Candidate, i64)> = heap
        .into_iter()
        .map(|entry| (entry.candidate, entry.score))
        .collect();
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.title.cmp(&b.0.title)));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
