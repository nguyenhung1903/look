mod config;
mod index;

use config::*;
use index::discover_candidates;
use look_indexing::{Candidate, CandidateKind};
use look_matching::fuzzy_score;
use look_ranking::rank_score;
use look_storage::{SearchSettings, SqliteStore, StorageError};
use std::collections::HashSet;
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

    pub fn search(&self, query: &str, limit: usize) -> Vec<ScoredCandidate> {
        let normalized_query = query.trim().to_lowercase();
        if normalized_query.is_empty() {
            let now_unix_s = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let mut browse = self
                .candidates
                .iter()
                .map(|candidate| ScoredCandidate {
                    candidate: candidate.clone(),
                    score: default_browse_score(candidate, now_unix_s),
                })
                .collect::<Vec<_>>();

            browse.sort_by(|a, b| {
                b.score
                    .cmp(&a.score)
                    .then_with(|| a.candidate.title.cmp(&b.candidate.title))
            });
            browse.truncate(limit);
            return browse;
        }

        let mut scored: Vec<ScoredCandidate> = self
            .candidates
            .iter()
            .filter_map(|candidate| {
                let title_score = fuzzy_score(&normalized_query, &candidate.title);
                let subtitle_score = candidate
                    .subtitle
                    .as_ref()
                    .and_then(|subtitle| fuzzy_score(&normalized_query, subtitle))
                    .map(|value| value / 2);
                let contains_score = contains_match_score(&normalized_query, candidate);

                let base = [title_score, subtitle_score, contains_score]
                    .into_iter()
                    .flatten()
                    .max()?;

                let final_score = rank_score(base, &normalized_query, candidate)
                    + kind_bias(candidate)
                    + query_kind_penalty(&normalized_query, candidate)
                    + path_depth_penalty(candidate);
                Some(ScoredCandidate {
                    candidate: candidate.clone(),
                    score: final_score,
                })
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .cmp(&a.score)
                .then_with(|| a.candidate.title.cmp(&b.candidate.title))
        });
        scored.truncate(limit);
        scored
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

fn contains_match_score(query: &str, candidate: &Candidate) -> Option<i64> {
    let title = candidate.title.to_lowercase();
    let subtitle = candidate.subtitle.as_ref().map(|s| s.to_lowercase());

    if title.contains(query) {
        return Some(SCORE_TITLE_CONTAINS);
    }

    if let Some(sub) = subtitle.as_deref()
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
        .all(|t| title.contains(t) || subtitle.as_deref().is_some_and(|sub| sub.contains(t)))
    {
        return Some(SCORE_TOKEN_ALL_MATCH);
    }

    None
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
pub struct ScoredCandidate {
    pub candidate: Candidate,
    pub score: i64,
}
