use crate::index::SETTINGS_CANDIDATE_ID_PREFIX;
use look_indexing::{Candidate, CandidateKind};
use std::sync::mpsc;

const SETTINGS_URL_SCHEME_PREFIX: &str = "x-apple.systempreferences:";
const SETTINGS_SUBTITLE_PREFIX: &str = "System Settings ";

struct SettingsEntry {
    title: &'static str,
    bundle_id: &'static str,
    aliases: &'static str,
}

const SETTINGS_CATALOG: [SettingsEntry; 16] = [
    SettingsEntry {
        title: "General",
        bundle_id: "com.apple.systempreferences.GeneralSettings",
        aliases: "settings general about software update storage",
    },
    SettingsEntry {
        title: "Apple ID",
        bundle_id: "com.apple.systempreferences.AppleIDSettings",
        aliases: "settings apple id icloud media purchases",
    },
    SettingsEntry {
        title: "Wi-Fi",
        bundle_id: "com.apple.wifi-settings-extension",
        aliases: "settings wifi wireless network internet",
    },
    SettingsEntry {
        title: "Bluetooth",
        bundle_id: "com.apple.BluetoothSettings",
        aliases: "settings bluetooth devices pairing",
    },
    SettingsEntry {
        title: "Network",
        bundle_id: "com.apple.Network-Settings.extension",
        aliases: "settings network ethernet dns proxy vpn",
    },
    SettingsEntry {
        title: "Sound",
        bundle_id: "com.apple.Sound-Settings.extension",
        aliases: "settings sound audio input output volume",
    },
    SettingsEntry {
        title: "Display",
        bundle_id: "com.apple.Displays-Settings.extension",
        aliases: "settings display monitor resolution refresh night shift",
    },
    SettingsEntry {
        title: "Wallpaper",
        bundle_id: "com.apple.Wallpaper-Settings.extension",
        aliases: "settings wallpaper background screen saver",
    },
    SettingsEntry {
        title: "Screen Time",
        bundle_id: "com.apple.Screen-Time-Settings.extension",
        aliases: "settings screen time limits downtime",
    },
    SettingsEntry {
        title: "Focus",
        bundle_id: "com.apple.Focus-Settings.extension",
        aliases: "settings focus do not disturb notifications",
    },
    SettingsEntry {
        title: "Notifications",
        bundle_id: "com.apple.Notifications-Settings.extension",
        aliases: "settings notifications alerts",
    },
    SettingsEntry {
        title: "Battery",
        bundle_id: "com.apple.Battery-Settings.extension",
        aliases: "settings battery power energy",
    },
    SettingsEntry {
        title: "Lock Screen",
        bundle_id: "com.apple.Lock-Screen-Settings.extension",
        aliases: "settings lock screen timeout",
    },
    SettingsEntry {
        title: "Privacy & Security",
        bundle_id: "com.apple.settings.PrivacySecurity.extension",
        aliases: "settings privacy security permissions firewall",
    },
    SettingsEntry {
        title: "Keyboard",
        bundle_id: "com.apple.Keyboard-Settings.extension",
        aliases: "settings keyboard shortcuts input",
    },
    SettingsEntry {
        title: "Trackpad",
        bundle_id: "com.apple.Trackpad-Settings.extension",
        aliases: "settings trackpad gestures pointer",
    },
];

pub fn discover_system_settings_entries(tx: mpsc::SyncSender<Candidate>) {
    for entry in SETTINGS_CATALOG {
        let key = format!(
            "{SETTINGS_CANDIDATE_ID_PREFIX}{}",
            entry.bundle_id.to_lowercase()
        );
        let mut candidate = Candidate::new(
            &key,
            CandidateKind::App,
            entry.title,
            &format!("{SETTINGS_URL_SCHEME_PREFIX}{}", entry.bundle_id),
        );
        candidate.subtitle = Some(format!("{SETTINGS_SUBTITLE_PREFIX}{}", entry.aliases).into());
        let _ = tx.send(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use look_indexing::CandidateIdKind;
    use std::collections::HashSet;

    #[test]
    fn curated_settings_catalog_has_valid_fields() {
        let mut seen_bundle_ids = HashSet::new();
        let mut seen_titles = HashSet::new();

        for entry in SETTINGS_CATALOG {
            assert!(!entry.title.trim().is_empty(), "title must be non-empty");
            assert!(
                seen_titles.insert(entry.title.to_ascii_lowercase()),
                "duplicate title: {}",
                entry.title
            );

            assert!(
                !entry.bundle_id.trim().is_empty(),
                "bundle_id must be non-empty"
            );
            assert!(
                entry.bundle_id.starts_with("com.apple."),
                "bundle_id should use com.apple.* namespace: {}",
                entry.bundle_id
            );
            assert!(
                entry
                    .bundle_id
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '.' || ch == '-' || ch == '_'),
                "bundle_id has invalid chars: {}",
                entry.bundle_id
            );
            assert!(
                seen_bundle_ids.insert(entry.bundle_id.to_ascii_lowercase()),
                "duplicate bundle_id: {}",
                entry.bundle_id
            );

            assert!(
                !entry.aliases.trim().is_empty(),
                "aliases must be non-empty"
            );
            assert!(
                entry.aliases.contains("settings"),
                "aliases should include settings hint: {}",
                entry.aliases
            );
        }
    }

    #[test]
    fn discovery_outputs_valid_settings_candidates() {
        let (tx, rx) = mpsc::sync_channel(64);
        discover_system_settings_entries(tx);
        let discovered: Vec<Candidate> = rx.into_iter().collect();

        assert_eq!(discovered.len(), SETTINGS_CATALOG.len());

        for candidate in discovered {
            assert_eq!(candidate.kind, CandidateKind::App);
            assert!(candidate.id.starts_with(CandidateIdKind::PREFIX_SETTING));
            assert!(candidate.path.starts_with(SETTINGS_URL_SCHEME_PREFIX));
            assert!(
                candidate
                    .subtitle
                    .as_deref()
                    .is_some_and(|s| s.starts_with(SETTINGS_SUBTITLE_PREFIX))
            );
        }
    }
}
