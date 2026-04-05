use look_indexing::Candidate;

pub fn rank_score(base_score: i64, query: &str, candidate: &Candidate, title_lower: &str) -> i64 {
    let mut score = base_score;

    if candidate.title.eq_ignore_ascii_case(query) {
        score += 500;
    }

    if title_lower.starts_with(query) {
        score += 200;
    }

    score += candidate.use_count as i64 * 5;

    if candidate.last_used_at_unix_s.is_some() {
        score += 25;
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use look_indexing::CandidateKind;

    fn test_candidate(title: &str, use_count: u64, last_used: Option<i64>) -> Candidate {
        Candidate {
            id: "test".to_string(),
            kind: CandidateKind::App,
            title: title.to_string(),
            subtitle: None,
            path: "/test".to_string(),
            use_count,
            last_used_at_unix_s: last_used,
        }
    }

    #[test]
    fn exact_match_gets_bonus() {
        let c = test_candidate("Safari", 0, None);
        let score = rank_score(100, "Safari", &c, "safari");
        assert!(score >= 100 + 500);
    }

    #[test]
    fn prefix_match_gets_bonus() {
        let c = test_candidate("Safari Browser", 0, None);
        let score = rank_score(100, "safari", &c, "safari browser");
        assert!(score >= 100 + 200);
    }

    #[test]
    fn usage_count_boosts_score() {
        let no_usage = test_candidate("App", 0, None);
        let high_usage = test_candidate("App", 10, None);
        let s1 = rank_score(100, "app", &no_usage, "app");
        let s2 = rank_score(100, "app", &high_usage, "app");
        assert!(s2 > s1);
        assert_eq!(s2 - s1, 50); // 10 * 5
    }

    #[test]
    fn recent_usage_gives_bonus() {
        let never_used = test_candidate("App", 0, None);
        let recently_used = test_candidate("App", 0, Some(1_000_000));
        let s1 = rank_score(100, "x", &never_used, "app");
        let s2 = rank_score(100, "x", &recently_used, "app");
        assert_eq!(s2 - s1, 25);
    }
}
