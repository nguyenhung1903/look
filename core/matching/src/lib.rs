pub struct PreparedQuery<'a> {
    raw: &'a str,
    chars: Vec<char>,
}

pub fn prepare_query(query: &str) -> PreparedQuery<'_> {
    PreparedQuery {
        raw: query,
        chars: query.chars().collect(),
    }
}

pub fn fuzzy_score(query: &str, title: &str) -> Option<i64> {
    let prepared = prepare_query(query);
    fuzzy_score_prepared(&prepared, title)
}

pub fn fuzzy_score_prepared(query: &PreparedQuery<'_>, title: &str) -> Option<i64> {
    let q = query.raw;
    let t = title;

    if t == q {
        return Some(2_000);
    }

    if t.starts_with(q) {
        return Some(1_500 - (t.len() as i64 - q.len() as i64).max(0));
    }

    let mut qi = 0usize;
    let mut score = 0i64;

    for ch in t.chars() {
        if qi < query.chars.len() && ch == query.chars[qi] {
            qi += 1;
            score += 10;
        }
    }

    if qi == query.chars.len() {
        Some(score)
    } else {
        None
    }
}

pub fn fuzzy_quality_bonus_prepared(query: &PreparedQuery<'_>, title: &str) -> i64 {
    if query.chars.is_empty() || title == query.raw || title.starts_with(query.raw) {
        return 0;
    }

    let mut qi = 0usize;
    let mut bonus = 0i64;
    let mut prev_match_idx: Option<usize> = None;
    let mut consecutive: i64 = 0;
    let mut prev_char: Option<char> = None;

    for (ti, ch) in title.chars().enumerate() {
        if qi >= query.chars.len() {
            break;
        }

        if ch != query.chars[qi] {
            prev_char = Some(ch);
            continue;
        }

        qi += 1;

        if let Some(prev) = prev_match_idx {
            if prev + 1 == ti {
                consecutive += 1;
                bonus += consecutive * 8;
            } else {
                consecutive = 0;
                let gap = (ti - prev - 1) as i64;
                bonus -= gap.min(20);
            }
        }

        if ti == 0 || matches!(prev_char, Some(' ' | '-' | '_' | '/' | '.')) {
            bonus += 20;
        }

        if ti < 5 {
            bonus += (5 - ti as i64) * 3;
        }

        prev_match_idx = Some(ti);
        prev_char = Some(ch);
    }

    if qi == query.chars.len() {
        bonus.max(0)
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn score_hierarchy_is_consistent() {
        let exact = fuzzy_score("safari", "safari").unwrap();
        let prefix = fuzzy_score("saf", "safari").unwrap();
        let subseq = fuzzy_score("sri", "safari").unwrap();

        assert!(exact > prefix, "exact ({exact}) > prefix ({prefix})");
        assert!(
            prefix > subseq,
            "prefix ({prefix}) > subsequence ({subseq})"
        );
    }

    #[test]
    fn prepared_query_matches_raw_behavior() {
        let prepared = prepare_query("vsc");
        assert_eq!(
            fuzzy_score_prepared(&prepared, "visual studio code"),
            fuzzy_score("vsc", "visual studio code")
        );
    }

    #[test]
    fn quality_bonus_prefers_compact_word_boundaries() {
        let prepared = prepare_query("vsc");
        let tight = fuzzy_quality_bonus_prepared(&prepared, "vs code");
        let loose = fuzzy_quality_bonus_prepared(&prepared, "visual studio code");
        assert!(tight > loose);
    }
}
