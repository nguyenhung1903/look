use crate::index::SETTINGS_CANDIDATE_ID_PREFIX;
use look_indexing::{Candidate, CandidateKind};
use std::collections::HashSet;

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

pub fn discover_system_settings_entries(seen: &mut HashSet<String>, out: &mut Vec<Candidate>) {
    for entry in SETTINGS_CATALOG {
        let key = format!(
            "{SETTINGS_CANDIDATE_ID_PREFIX}{}",
            entry.bundle_id.to_lowercase()
        );
        if !seen.insert(key.clone()) {
            continue;
        }

        let mut candidate = Candidate::new(
            &key,
            CandidateKind::App,
            entry.title,
            &format!("{SETTINGS_URL_SCHEME_PREFIX}{}", entry.bundle_id),
        );
        candidate.subtitle = Some(format!("{SETTINGS_SUBTITLE_PREFIX}{}", entry.aliases));
        out.push(candidate);
    }
}
