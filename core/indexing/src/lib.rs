use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum CandidateKind {
    App,
    File,
    Folder,
}

impl CandidateKind {
    pub const APP_KEY: &'static str = "app";
    pub const FILE_KEY: &'static str = "file";
    pub const FOLDER_KEY: &'static str = "folder";

    pub fn as_str(&self) -> &'static str {
        match self {
            CandidateKind::App => Self::APP_KEY,
            CandidateKind::File => Self::FILE_KEY,
            CandidateKind::Folder => Self::FOLDER_KEY,
        }
    }

    pub fn from_key(value: &str) -> Option<Self> {
        match value {
            Self::APP_KEY => Some(CandidateKind::App),
            Self::FILE_KEY => Some(CandidateKind::File),
            Self::FOLDER_KEY => Some(CandidateKind::Folder),
            _ => None,
        }
    }
}

impl fmt::Display for CandidateKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Candidate {
    pub id: String,
    pub kind: CandidateKind,
    pub title: String,
    pub subtitle: Option<String>,
    pub path: String,
    pub use_count: u64,
    pub last_used_at_unix_s: Option<i64>,
}

impl Candidate {
    pub fn new(id: &str, kind: CandidateKind, title: &str, path: &str) -> Self {
        Self {
            id: id.to_string(),
            kind,
            title: title.to_string(),
            subtitle: Some(path.to_string()),
            path: path.to_string(),
            use_count: 0,
            last_used_at_unix_s: None,
        }
    }
}

pub trait Source {
    fn collect(&self) -> Vec<Candidate>;
}
