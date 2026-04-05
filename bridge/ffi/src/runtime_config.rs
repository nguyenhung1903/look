use std::collections::HashMap;
use std::env;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Copy, Debug)]
pub(crate) struct RuntimeConfig {
    pub(crate) translate_allow_network: bool,
    pub(crate) log_level: LogLevel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum LogLevel {
    Error,
    Info,
    Debug,
}

static RUNTIME_CONFIG: OnceLock<Mutex<RuntimeConfig>> = OnceLock::new();

pub(crate) fn reload_runtime_config() {
    let loaded = load_runtime_config();
    if let Some(lock) = RUNTIME_CONFIG.get()
        && let Ok(mut guard) = lock.lock()
    {
        *guard = loaded;
    } else {
        let _ = RUNTIME_CONFIG.set(Mutex::new(loaded));
    }
}

pub(crate) fn network_translation_allowed() -> bool {
    with_runtime_config(|cfg| cfg.translate_allow_network)
}

pub(crate) fn log_debug(message: &str) {
    if current_log_level() >= LogLevel::Debug {
        eprintln!("[look][debug] {message}");
    }
}

pub(crate) fn log_info(message: &str) {
    if current_log_level() >= LogLevel::Info {
        eprintln!("[look][info] {message}");
    }
}

pub(crate) fn log_error(message: &str) {
    if current_log_level() >= LogLevel::Error {
        eprintln!("[look][error] {message}");
    }
}

fn current_log_level() -> LogLevel {
    with_runtime_config(|cfg| cfg.log_level)
}

fn with_runtime_config<T>(f: impl FnOnce(&RuntimeConfig) -> T) -> T {
    let lock = runtime_config();
    let guard = lock.lock().expect("runtime config lock poisoned");
    f(&guard)
}

fn runtime_config() -> &'static Mutex<RuntimeConfig> {
    RUNTIME_CONFIG.get_or_init(|| Mutex::new(load_runtime_config()))
}

fn load_runtime_config() -> RuntimeConfig {
    let mut from_file: HashMap<String, String> = HashMap::new();
    if let Ok(contents) = std::fs::read_to_string(default_config_path()) {
        for raw_line in contents.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            if let Some(split) = line.find('=') {
                let key = line[..split].trim().to_string();
                let value = line[split + 1..].trim().to_string();
                from_file.insert(key, value);
            }
        }
    }

    let translate_allow_network = env_bool("LOOK_TRANSLATE_ALLOW_NETWORK")
        .or_else(|| {
            from_file
                .get("translate_allow_network")
                .and_then(|v| parse_bool(v))
        })
        .unwrap_or(false);

    let log_level = env::var("LOOK_LOG_LEVEL")
        .ok()
        .and_then(|v| parse_log_level(&v))
        .or_else(|| {
            from_file
                .get("backend_log_level")
                .and_then(|v| parse_log_level(v))
        })
        .unwrap_or(LogLevel::Error);

    RuntimeConfig {
        translate_allow_network,
        log_level,
    }
}

fn default_config_path() -> PathBuf {
    if let Ok(custom) = env::var("LOOK_CONFIG_PATH")
        && !custom.trim().is_empty()
    {
        return PathBuf::from(custom);
    }

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".look.config")
}

fn env_bool(name: &str) -> Option<bool> {
    env::var(name).ok().and_then(|value| parse_bool(&value))
}

fn parse_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_log_level(value: &str) -> Option<LogLevel> {
    match value.trim().to_ascii_lowercase().as_str() {
        "debug" => Some(LogLevel::Debug),
        "info" => Some(LogLevel::Info),
        "error" => Some(LogLevel::Error),
        _ => None,
    }
}
