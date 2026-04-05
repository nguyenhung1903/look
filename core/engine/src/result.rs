use serde::{Deserialize, Serialize};

use look_indexing::{Candidate, CandidateKind};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaunchResult {
    pub id: String,
    pub kind: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub path: String,
    pub score: i64,
    pub action: LaunchResultAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum LaunchResultAction {
    Open { path: String },
    OpenFolder { path: String },
    Reveal { path: String },
    WebSearch { query: String },
    Translate { text: String, target_lang: String },
}

impl From<(&Candidate, i64)> for LaunchResult {
    fn from((candidate, score): (&Candidate, i64)) -> Self {
        let action = match candidate.kind {
            CandidateKind::App => LaunchResultAction::Open {
                path: candidate.path.clone(),
            },
            CandidateKind::File => LaunchResultAction::Open {
                path: candidate.path.clone(),
            },
            CandidateKind::Folder => LaunchResultAction::OpenFolder {
                path: candidate.path.clone(),
            },
        };

        Self {
            id: candidate.id.clone(),
            kind: candidate.kind.to_string(),
            title: candidate.title.clone(),
            subtitle: candidate.subtitle.clone(),
            path: candidate.path.clone(),
            score,
            action,
        }
    }
}
