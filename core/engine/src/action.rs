use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionKind {
    OpenApp,
    OpenFile,
    OpenFolder,
    OpenUrl,
    RevealInFinder,
    ExecuteCommand,
    WebSearch,
    Translate,
}

impl ActionKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionKind::OpenApp => "open_app",
            ActionKind::OpenFile => "open_file",
            ActionKind::OpenFolder => "open_folder",
            ActionKind::OpenUrl => "open_url",
            ActionKind::RevealInFinder => "reveal",
            ActionKind::ExecuteCommand => "execute",
            ActionKind::WebSearch => "web_search",
            ActionKind::Translate => "translate",
        }
    }
}

impl FromStr for ActionKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "open_app" => Ok(ActionKind::OpenApp),
            "open_file" => Ok(ActionKind::OpenFile),
            "open_folder" => Ok(ActionKind::OpenFolder),
            "open_url" => Ok(ActionKind::OpenUrl),
            "reveal" => Ok(ActionKind::RevealInFinder),
            "execute" => Ok(ActionKind::ExecuteCommand),
            "web_search" => Ok(ActionKind::WebSearch),
            "translate" => Ok(ActionKind::Translate),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LaunchAction {
    pub kind: ActionKind,
    pub target: String,
    pub metadata: Option<serde_json::Value>,
}

impl LaunchAction {
    pub fn open_app(path: &str) -> Self {
        Self {
            kind: ActionKind::OpenApp,
            target: path.to_string(),
            metadata: None,
        }
    }

    pub fn open_file(path: &str) -> Self {
        Self {
            kind: ActionKind::OpenFile,
            target: path.to_string(),
            metadata: None,
        }
    }

    pub fn open_folder(path: &str) -> Self {
        Self {
            kind: ActionKind::OpenFolder,
            target: path.to_string(),
            metadata: None,
        }
    }

    pub fn open_url(url: &str) -> Self {
        Self {
            kind: ActionKind::OpenUrl,
            target: url.to_string(),
            metadata: None,
        }
    }

    pub fn web_search(query: &str) -> Self {
        Self {
            kind: ActionKind::WebSearch,
            target: query.to_string(),
            metadata: None,
        }
    }
}
