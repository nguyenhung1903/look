use crate::QueryEngine;
use crate::config::*;
use crate::normalize::normalize_for_search;
use crate::query::ParsedQuery;
use crate::scoring::{
    ScoredMatch, contains_match_score, default_browse_score, finalize_top_k,
    finalize_top_k_with_search, kind_bias, path_depth_penalty, path_match_score, push_top_k,
    query_kind_penalty,
};
use look_indexing::{Candidate, CandidateKind};
use look_matching::{fuzzy_quality_bonus_prepared, fuzzy_score_prepared, prepare_query};
use look_ranking::rank_score;
use regex::RegexBuilder;
use std::collections::BinaryHeap;
use std::time::{SystemTime, UNIX_EPOCH};

const RERANK_POOL_MULTIPLIER: usize = 4;
const RERANK_TOP_N: usize = 80;
const RERANK_MIN_QUERY_CHARS: usize = 3;
const REGEX_SIZE_LIMIT_BYTES: usize = 1024 * 1024;

fn strip_search_titles(
    mut ranked: Vec<(Candidate, i64, String)>,
    limit: usize,
) -> Vec<(Candidate, i64)> {
    ranked.truncate(limit);
    ranked
        .into_iter()
        .map(|(candidate, score, _)| (candidate, score))
        .collect()
}

impl QueryEngine {
    fn kind_matches(candidate: &Candidate, kind_filter: Option<&CandidateKind>) -> bool {
        kind_filter.is_none_or(|kind| &candidate.kind == kind)
    }

    fn search_empty_query(
        &self,
        kind_filter: Option<&CandidateKind>,
        limit: usize,
    ) -> Vec<(Candidate, i64)> {
        let now_unix_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let mut top = BinaryHeap::new();
        for candidate in self
            .candidates
            .iter()
            .filter(|candidate| Self::kind_matches(candidate, kind_filter))
        {
            let score = default_browse_score(candidate, now_unix_s);
            push_top_k(&mut top, ScoredMatch::new(candidate.clone(), score), limit);
        }

        finalize_top_k(top)
    }

    fn search_regex_query(
        &self,
        raw_query: Option<&String>,
        kind_filter: Option<&CandidateKind>,
        limit: usize,
    ) -> Vec<(Candidate, i64)> {
        let Some(regex) = raw_query.and_then(|pattern| {
            RegexBuilder::new(pattern)
                .case_insensitive(true)
                .size_limit(REGEX_SIZE_LIMIT_BYTES)
                .build()
                .ok()
        }) else {
            return vec![];
        };

        let mut top = BinaryHeap::new();
        for candidate in self
            .candidates
            .iter()
            .filter(|candidate| Self::kind_matches(candidate, kind_filter))
        {
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

            let final_score = regex_score + kind_bias(candidate) + path_depth_penalty(candidate);
            push_top_k(
                &mut top,
                ScoredMatch::new(candidate.clone(), final_score),
                limit,
            );
        }

        finalize_top_k(top)
    }

    fn search_text_query(
        &self,
        normalized_query: &str,
        kind_filter: Option<&CandidateKind>,
        limit: usize,
    ) -> Vec<(Candidate, i64)> {
        let prepared_query = prepare_query(normalized_query);
        let mut top = BinaryHeap::new();
        let has_path_hint = normalized_query.contains('/');
        let pool_limit = (limit.saturating_mul(RERANK_POOL_MULTIPLIER)).max(RERANK_TOP_N);

        for candidate in self
            .candidates
            .iter()
            .filter(|candidate| Self::kind_matches(candidate, kind_filter))
        {
            let title_search = normalize_for_search(&candidate.title);
            let subtitle_search = candidate
                .subtitle
                .as_ref()
                .map(|subtitle| normalize_for_search(subtitle));
            let path_search = normalize_for_search(&candidate.path);

            let title_score = fuzzy_score_prepared(&prepared_query, &title_search);
            let subtitle_score = subtitle_search
                .as_ref()
                .and_then(|subtitle| fuzzy_score_prepared(&prepared_query, subtitle))
                .map(|value| value / 2);
            let contains_score =
                contains_match_score(normalized_query, &title_search, subtitle_search.as_deref());
            let path_score = if has_path_hint {
                path_match_score(normalized_query, &path_search)
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

            let final_score = rank_score(base, normalized_query, candidate, &title_search)
                + kind_bias(candidate)
                + query_kind_penalty(normalized_query, candidate)
                + path_depth_penalty(candidate);
            push_top_k(
                &mut top,
                ScoredMatch::new_with_search_title(candidate.clone(), final_score, title_search),
                pool_limit,
            );
        }

        let mut ranked = finalize_top_k_with_search(top);

        if normalized_query.chars().count() < RERANK_MIN_QUERY_CHARS {
            return strip_search_titles(ranked, limit);
        }

        // Two-stage retrieval:
        // 1) fast scorer over full candidate set
        // 2) quality rerank only on top-N candidates to keep latency predictable
        let rerank_count = ranked.len().min(RERANK_TOP_N);
        for entry in ranked.iter_mut().take(rerank_count) {
            entry.1 += fuzzy_quality_bonus_prepared(&prepared_query, &entry.2);
        }

        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.title.cmp(&b.0.title)));
        strip_search_titles(ranked, limit)
    }

    pub fn search_scored(&self, query: &str, limit: usize) -> Vec<(Candidate, i64)> {
        if limit == 0 {
            return vec![];
        }

        let parsed_query = ParsedQuery::from_input(query);
        let kind_filter = parsed_query.kind_filter.as_ref();
        if parsed_query.normalized_query.is_empty() && !parsed_query.is_regex {
            return self.search_empty_query(kind_filter, limit);
        }

        if parsed_query.is_regex {
            return self.search_regex_query(parsed_query.raw_query.as_ref(), kind_filter, limit);
        }

        self.search_text_query(&parsed_query.normalized_query, kind_filter, limit)
    }
}
