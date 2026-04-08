use crate::QueryEngine;
use crate::config::*;
use crate::normalize::normalize_for_search;
use crate::query::ParsedQuery;
use crate::scoring::{
    ScoredMatch, contains_match_score, default_browse_score, finalize_top_k, kind_bias,
    path_depth_penalty, path_match_score, push_top_k, query_kind_penalty,
};
use look_indexing::Candidate;
use look_matching::{fuzzy_quality_bonus_prepared, fuzzy_score_prepared, prepare_query};
use look_ranking::rank_score;
use regex::RegexBuilder;
use std::collections::BinaryHeap;
use std::time::{SystemTime, UNIX_EPOCH};

const RERANK_POOL_MULTIPLIER: usize = 4;
const RERANK_TOP_N: usize = 80;

impl QueryEngine {
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
        let prepared_query = prepare_query(&normalized_query);
        let mut top = BinaryHeap::new();
        let has_path_hint = normalized_query.contains('/');
        let pool_limit = (limit.saturating_mul(RERANK_POOL_MULTIPLIER)).max(RERANK_TOP_N);

        for candidate in self.candidates.iter().filter(|candidate| {
            parsed_query
                .kind_filter
                .as_ref()
                .is_none_or(|kind| &candidate.kind == kind)
        }) {
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
                contains_match_score(&normalized_query, &title_search, subtitle_search.as_deref());
            let path_score = if has_path_hint {
                path_match_score(&normalized_query, &path_search)
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

            let final_score = rank_score(base, &normalized_query, candidate, &title_search)
                + kind_bias(candidate)
                + query_kind_penalty(&normalized_query, candidate)
                + path_depth_penalty(candidate);
            push_top_k(
                &mut top,
                ScoredMatch::new(candidate.clone(), final_score),
                pool_limit,
            );
        }

        let mut ranked = finalize_top_k(top);

        // Two-stage retrieval:
        // 1) fast scorer over full candidate set
        // 2) quality rerank only on top-N candidates to keep latency predictable
        let rerank_count = ranked.len().min(RERANK_TOP_N);
        for entry in ranked.iter_mut().take(rerank_count) {
            let title_search = normalize_for_search(&entry.0.title);
            entry.1 += fuzzy_quality_bonus_prepared(&prepared_query, &title_search);
        }

        ranked.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.title.cmp(&b.0.title)));
        ranked.truncate(limit);
        ranked
    }
}
