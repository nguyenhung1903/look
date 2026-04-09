use crate::config::RuntimeConfig;
mod apps;
mod files;
mod settings;

use look_indexing::{Candidate, CandidateIdKind};
use std::collections::HashSet;

pub(super) const APP_CANDIDATE_ID_PREFIX: &str = CandidateIdKind::PREFIX_APP;
pub(super) const FILE_CANDIDATE_ID_PREFIX: &str = CandidateIdKind::PREFIX_FILE;
pub(super) const FOLDER_CANDIDATE_ID_PREFIX: &str = CandidateIdKind::PREFIX_FOLDER;
pub(super) const SETTINGS_CANDIDATE_ID_PREFIX: &str = CandidateIdKind::PREFIX_SETTING;

pub fn discover_candidates(config: &RuntimeConfig) -> Vec<Candidate> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();

    apps::discover_installed_apps(config, &mut seen, &mut out);
    settings::discover_system_settings_entries(&mut seen, &mut out);
    files::discover_local_files_and_folders(config, &mut seen, &mut out);

    out
}
