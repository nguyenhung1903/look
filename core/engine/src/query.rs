use look_indexing::CandidateKind;

#[derive(Clone, Debug)]
pub(crate) struct ParsedQuery {
    pub(crate) normalized_query: String,
    pub(crate) raw_query: Option<String>,
    pub(crate) kind_filter: Option<CandidateKind>,
    pub(crate) is_regex: bool,
}

impl ParsedQuery {
    pub(crate) fn from_input(input: &str) -> Self {
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
