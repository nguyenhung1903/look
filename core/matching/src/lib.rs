pub fn fuzzy_score(query: &str, title: &str) -> Option<i64> {
    let q = query;
    let t = title;

    // Tier 1: exact match — highest possible score
    if t == q {
        return Some(2_000);
    }

    // Tier 2: prefix match — high score with length penalty
    if t.starts_with(q) {
        return Some(1_500 - (t.len() as i64 - q.len() as i64).max(0));
    }

    // Tier 3: enhanced subsequence matching with heuristic bonuses
    let qchars: Vec<char> = q.chars().collect();
    if qchars.is_empty() {
        return Some(1_500 - t.len() as i64);
    }

    let tchars: Vec<char> = t.chars().collect();
    let mut qi = 0usize;
    let mut score = 0i64;
    let mut prev_match_idx: Option<usize> = None;
    let mut consecutive: i64 = 0;

    for (ti, &ch) in tchars.iter().enumerate() {
        if qi >= qchars.len() {
            break;
        }
        if ch != qchars[qi] {
            continue;
        }
        qi += 1;

        // Base: each matched character
        score += 10;

        // Consecutive bonus: escalating reward for adjacent matches
        // "sc" in "screen" (s→c consecutive) scores higher than "s...c" spread apart
        if let Some(prev) = prev_match_idx {
            if prev + 1 == ti {
                consecutive += 1;
                score += consecutive * 8;
            } else {
                consecutive = 0;
                // Gap penalty: distance between matches
                let gap = (ti - prev - 1) as i64;
                score -= gap.min(20);
            }
        }

        // Word boundary bonus: match right after a separator or at start
        // "vsc" matching V|isual S|tudio C|ode — each capital/boundary hit is strong signal
        if ti == 0 || matches!(tchars[ti - 1], ' ' | '-' | '_' | '/' | '.') {
            score += 20;
        }

        // Position weight: earlier matches are more likely intentional
        if ti < 5 {
            score += (5 - ti as i64) * 3;
        }

        prev_match_idx = Some(ti);
    }

    if qi == qchars.len() {
        Some(score.max(1))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── existing tests (behavior preserved) ──

    #[test]
    fn exact_match_returns_max_score() {
        assert_eq!(fuzzy_score("safari", "safari"), Some(2_000));
    }

    #[test]
    fn prefix_match_returns_high_score() {
        let score = fuzzy_score("saf", "safari").unwrap();
        assert!(score > 1_000);
        assert!(score < 2_000);
    }

    #[test]
    fn prefix_score_penalizes_longer_titles() {
        let short = fuzzy_score("saf", "safari").unwrap();
        let long = fuzzy_score("saf", "safari browser extended").unwrap();
        assert!(short > long);
    }

    #[test]
    fn subsequence_match_returns_positive_score() {
        let score = fuzzy_score("vsc", "visual studio code").unwrap();
        assert!(score > 0);
    }

    #[test]
    fn no_match_returns_none() {
        assert_eq!(fuzzy_score("xyz", "safari"), None);
    }

    #[test]
    fn empty_query_matches_everything() {
        // Empty string is a prefix of any string, so it goes through the
        // starts_with branch: 1500 - title.len()
        let score = fuzzy_score("", "safari").unwrap();
        assert!(score > 0);
    }

    #[test]
    fn empty_title_only_matches_empty_query() {
        assert_eq!(fuzzy_score("", ""), Some(2_000));
        assert_eq!(fuzzy_score("a", ""), None);
    }

    #[test]
    fn single_char_query() {
        assert!(fuzzy_score("s", "safari").is_some());
        assert!(fuzzy_score("z", "safari").is_none());
    }

    // ── new tests: consecutive bonus ──

    #[test]
    fn consecutive_matches_score_higher_than_spread() {
        // "ab" in "ab xyz" (consecutive a→b) vs "ab" in "a.x.b" (spread)
        let consecutive = fuzzy_score("ab", "ab xyz").unwrap();
        let spread = fuzzy_score("ab", "a x b").unwrap();
        assert!(
            consecutive > spread,
            "consecutive ({consecutive}) should beat spread ({spread})"
        );
    }

    #[test]
    fn longer_consecutive_run_scores_higher() {
        // "abc" consecutive in "abcdef" vs only "ab" consecutive in "ab x c"
        let full_run = fuzzy_score("abc", "abcdef").unwrap();
        let partial_run = fuzzy_score("abc", "ab x c").unwrap();
        assert!(
            full_run > partial_run,
            "full run ({full_run}) should beat partial ({partial_run})"
        );
    }

    // ── new tests: word boundary bonus ──

    #[test]
    fn boundary_match_preferred() {
        // "vsc" → "vs code" matches at word boundaries (v, s consecutive + c at boundary)
        // "vsc" → "visual studio code" matches v at start, then s/c at boundaries
        let tight = fuzzy_score("vsc", "vs code").unwrap();
        let loose = fuzzy_score("vsc", "visual studio code").unwrap();
        assert!(
            tight > loose,
            "tight boundary match ({tight}) should beat loose ({loose})"
        );
    }

    #[test]
    fn start_of_word_matches_score_higher() {
        // "c" at start of "code" (word boundary) vs "c" buried in "bicycle"
        let at_boundary = fuzzy_score("c", "my code").unwrap();
        let mid_word = fuzzy_score("c", "bicycle").unwrap();
        assert!(
            at_boundary > mid_word,
            "boundary ({at_boundary}) should beat mid-word ({mid_word})"
        );
    }

    // ── new tests: gap penalty ──

    #[test]
    fn smaller_gap_scores_higher() {
        // "ac" with small gap in "a.c.x.y" vs large gap in "a.x.x.x.x.c"
        let small_gap = fuzzy_score("ac", "a c x").unwrap();
        let large_gap = fuzzy_score("ac", "a x x x x c").unwrap();
        assert!(
            small_gap > large_gap,
            "small gap ({small_gap}) should beat large gap ({large_gap})"
        );
    }

    // ── new tests: position weight ──

    #[test]
    fn earlier_match_scores_higher() {
        // "x" near start vs "x" at the end
        let early = fuzzy_score("x", "x at start").unwrap();
        let late = fuzzy_score("x", "at the end x").unwrap();
        assert!(
            early > late,
            "early match ({early}) should beat late match ({late})"
        );
    }

    // ── new tests: combined heuristics ──

    #[test]
    fn initials_match_is_strong() {
        // "gc" matching initials of "google chrome"
        let score = fuzzy_score("gc", "google chrome").unwrap();
        // Should get boundary bonus on both g (start) and c (word boundary)
        assert!(score > 40, "initials match should score well ({score})");
    }

    #[test]
    fn score_hierarchy_is_consistent() {
        let exact = fuzzy_score("safari", "safari").unwrap();
        let prefix = fuzzy_score("saf", "safari").unwrap();
        let subseq = fuzzy_score("sri", "safari").unwrap();

        assert!(exact > prefix, "exact ({exact}) > prefix ({prefix})");
        assert!(prefix > subseq, "prefix ({prefix}) > subsequence ({subseq})");
    }
}
