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
