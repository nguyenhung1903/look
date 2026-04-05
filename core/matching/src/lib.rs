pub fn fuzzy_score(query: &str, title: &str) -> Option<i64> {
    let q = query;
    let t = title;

    if t == q {
        return Some(2_000);
    }

    if t.starts_with(q) {
        return Some(1_500 - (t.len() as i64 - q.len() as i64).max(0));
    }

    let mut qi = 0usize;
    let qchars: Vec<char> = q.chars().collect();
    let mut score = 0i64;

    for ch in t.chars() {
        if qi < qchars.len() && ch == qchars[qi] {
            qi += 1;
            score += 10;
        }
    }

    if qi == qchars.len() {
        Some(score)
    } else {
        None
    }
}
