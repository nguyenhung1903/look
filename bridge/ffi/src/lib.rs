#![allow(unsafe_code)]

use look_engine::QueryEngine;
use look_storage::SqliteStore;
use std::collections::HashMap;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[repr(C)]
pub struct FfiSearchResult {
    pub count: u32,
}

#[derive(serde::Serialize)]
struct FfiSearchPayload {
    query: String,
    count: usize,
    results: Vec<look_engine::LaunchResult>,
    error: Option<FfiErrorPayload>,
}

#[derive(serde::Serialize)]
struct FfiErrorPayload {
    code: &'static str,
    message: String,
}

#[unsafe(no_mangle)]
pub extern "C" fn look_search_count(query_len: u32) -> FfiSearchResult {
    let query = "x".repeat(query_len as usize);
    let results = with_engine(|engine| engine.search(&query, 20));
    FfiSearchResult {
        count: results.len() as u32,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn look_search_json(query: *const c_char, limit: u32) -> *mut c_char {
    let query = cstr_to_string(query);
    let max = if limit == 0 { 20 } else { limit as usize };
    let started_at = Instant::now();

    let results = with_engine(|engine| engine.search(&query, max));
    let result_count = results.len();

    let payload = FfiSearchPayload {
        query: query.clone(),
        count: result_count,
        results,
        error: None,
    };

    let json = serde_json::to_string(&payload).unwrap_or_else(|_| {
        serde_json::json!({
            "query": query,
            "count": 0,
            "results": [],
            "error": {
                "code": "serialize_failed",
                "message": "Failed to serialize search results"
            }
        })
        .to_string()
    });
    let cstring = CString::new(json).unwrap_or_else(|_| {
        CString::new("{\"query\":\"\",\"count\":0,\"results\":[]}").expect("valid static json")
    });
    log_debug(&format!(
        "search query_len={} limit={} count={} elapsed_ms={}",
        query.len(),
        max,
        result_count,
        started_at.elapsed().as_millis()
    ));
    store_json_allocation(cstring)
}

#[unsafe(no_mangle)]
pub extern "C" fn look_record_usage(candidate_id: *const c_char, action: *const c_char) -> bool {
    let candidate_id = cstr_to_string(candidate_id);
    let action = cstr_to_string(action);

    if candidate_id.trim().is_empty() || action.trim().is_empty() {
        return false;
    }

    let Ok(store) = SqliteStore::open(default_db_path()) else {
        return false;
    };

    let ok = store
        .record_usage_event(candidate_id.trim(), action.trim())
        .is_ok();

    if ok {
        let used_at_unix_s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        with_engine_mut(|engine| {
            let _ = engine.record_usage_in_memory(candidate_id.trim(), used_at_unix_s);
        });
    }

    if ok {
        log_debug("record_usage success");
    } else {
        log_error("record_usage failed");
    }

    ok
}

#[unsafe(no_mangle)]
pub extern "C" fn look_reload_config() -> bool {
    reload_runtime_config();
    let path = default_db_path();
    if QueryEngine::bootstrap_sqlite(&path).is_err() {
        return false;
    }
    refresh_engine_cache();
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn look_free_cstring(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    if let Some(lock) = JSON_ALLOCS.get()
        && let Ok(mut allocations) = lock.lock()
    {
        allocations.remove(&(ptr as usize));
    }
}

fn store_json_allocation(cstring: CString) -> *mut c_char {
    let ptr = cstring.as_ptr() as usize;
    let lock = JSON_ALLOCS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut allocations = lock.lock().expect("json allocations lock poisoned");
    allocations.insert(ptr, cstring);
    ptr as *mut c_char
}

fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

fn default_db_path() -> PathBuf {
    if let Ok(custom) = env::var("LOOK_DB_PATH")
        && !custom.trim().is_empty()
    {
        return PathBuf::from(custom);
    }

    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join("Library")
        .join("Application Support")
        .join("look")
        .join("look.db")
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

#[derive(Clone, Copy, Debug)]
struct RuntimeConfig {
    translate_allow_network: bool,
    log_level: LogLevel,
}

static ENGINE_CACHE: OnceLock<Mutex<QueryEngine>> = OnceLock::new();
static JSON_ALLOCS: OnceLock<Mutex<HashMap<usize, CString>>> = OnceLock::new();
static BOOTSTRAP_REFRESH_STARTED: OnceLock<()> = OnceLock::new();
static RUNTIME_CONFIG: OnceLock<Mutex<RuntimeConfig>> = OnceLock::new();

fn engine_cache() -> &'static Mutex<QueryEngine> {
    let cache = ENGINE_CACHE.get_or_init(|| {
        let path = default_db_path();
        let engine = QueryEngine::from_sqlite(&path).unwrap_or_else(|_| QueryEngine::demo_seed());
        Mutex::new(engine)
    });
    start_background_bootstrap_refresh();
    cache
}

fn with_engine<T>(f: impl FnOnce(&QueryEngine) -> T) -> T {
    let lock = engine_cache();
    let guard = lock.lock().expect("engine cache lock poisoned");
    f(&guard)
}

fn with_engine_mut<T>(f: impl FnOnce(&mut QueryEngine) -> T) -> T {
    let lock = engine_cache();
    let mut guard = lock.lock().expect("engine cache lock poisoned");
    f(&mut guard)
}

fn refresh_engine_cache() {
    if let Some(lock) = ENGINE_CACHE.get() {
        let path = default_db_path();
        if let Ok(engine) = QueryEngine::from_sqlite(path)
            && let Ok(mut guard) = lock.lock()
        {
            *guard = engine;
        }
    }
}

fn start_background_bootstrap_refresh() {
    let _ = BOOTSTRAP_REFRESH_STARTED.get_or_init(|| {
        thread::spawn(|| {
            let started_at = Instant::now();
            let path = default_db_path();
            match QueryEngine::bootstrap_sqlite(&path) {
                Ok(()) => {
                    refresh_engine_cache();
                    let candidate_count = with_engine(|engine| engine.search("", 2000).len());
                    log_info(&format!(
                        "bootstrap refresh ok candidates={} elapsed_ms={}",
                        candidate_count,
                        started_at.elapsed().as_millis()
                    ));
                }
                Err(err) => {
                    log_error(&format!(
                        "bootstrap refresh failed error={} elapsed_ms={}",
                        err,
                        started_at.elapsed().as_millis()
                    ));
                }
            }
        });
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum LogLevel {
    Error,
    Info,
    Debug,
}

fn log_level() -> LogLevel {
    with_runtime_config(|cfg| cfg.log_level)
}

fn log_debug(message: &str) {
    if log_level() >= LogLevel::Debug {
        eprintln!("[look][debug] {message}");
    }
}

fn log_info(message: &str) {
    if log_level() >= LogLevel::Info {
        eprintln!("[look][info] {message}");
    }
}

fn log_error(message: &str) {
    if log_level() >= LogLevel::Error {
        eprintln!("[look][error] {message}");
    }
}

fn with_runtime_config<T>(f: impl FnOnce(&RuntimeConfig) -> T) -> T {
    let lock = runtime_config();
    let guard = lock.lock().expect("runtime config lock poisoned");
    f(&guard)
}

fn runtime_config() -> &'static Mutex<RuntimeConfig> {
    RUNTIME_CONFIG.get_or_init(|| Mutex::new(load_runtime_config()))
}

fn reload_runtime_config() {
    let loaded = load_runtime_config();
    if let Some(lock) = RUNTIME_CONFIG.get()
        && let Ok(mut guard) = lock.lock()
    {
        *guard = loaded;
    } else {
        let _ = RUNTIME_CONFIG.set(Mutex::new(loaded));
    }
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

fn translate_error_json(text: &str, code: &'static str, message: &str) -> *mut c_char {
    let payload = serde_json::json!({
        "original": text,
        "translated": "",
        "error": {
            "code": code,
            "message": message,
        }
    });
    let json = serde_json::to_string(&payload).unwrap_or_else(|_| {
        "{\"original\":\"\",\"translated\":\"\",\"error\":{\"code\":\"unknown\",\"message\":\"Unknown translation error\"}}".to_string()
    });
    let cstring = CString::new(json).expect("valid json");
    store_json_allocation(cstring)
}

#[derive(serde::Deserialize)]
struct TranslateResponse(serde_json::Value);

#[unsafe(no_mangle)]
pub extern "C" fn look_translate_json(
    text: *const c_char,
    target_lang: *const c_char,
) -> *mut c_char {
    let text = cstr_to_string(text);
    let target_lang = cstr_to_string(target_lang);

    if !network_translation_allowed() {
        return translate_error_json(
            &text,
            "translate_network_disabled",
            "Network translation is disabled. Enable in Advanced settings or set LOOK_TRANSLATE_ALLOW_NETWORK=true.",
        );
    }

    if text.trim().is_empty() {
        return translate_error_json(&text, "empty_text", "Type text after t\" to translate");
    }

    let encoded_text = urlencodingencode(&text);
    let url = format!(
        "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl={}&dt=t&q={}",
        target_lang, encoded_text
    );

    let output = std::process::Command::new("curl")
        .args(["-s", "-m", "3", "-A", "Mozilla/5.0", &url])
        .output();

    let body = match output {
        Ok(out) => {
            if !out.status.success() {
                return translate_error_json(
                    &text,
                    "translate_request_failed",
                    "Translation request failed",
                );
            }
            match String::from_utf8(out.stdout) {
                Ok(s) => s,
                Err(_) => {
                    return translate_error_json(
                        &text,
                        "translate_decode_failed",
                        "Translation response decode failed",
                    );
                }
            }
        }
        Err(_) => {
            return translate_error_json(
                &text,
                "translate_exec_failed",
                "Translation command execution failed",
            );
        }
    };

    let parsed: TranslateResponse = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(_) => {
            return translate_error_json(
                &text,
                "translate_parse_failed",
                "Translation response parse failed",
            );
        }
    };

    let translated = parse_translate_response(&parsed.0);
    if translated.trim().is_empty() {
        return translate_error_json(
            &text,
            "translate_empty_result",
            "Translation returned empty result",
        );
    }

    let result = serde_json::json!({
        "original": text,
        "translated": translated,
        "error": null
    });

    let json =
        serde_json::to_string(&result).unwrap_or_else(|_| "{\"error\":\"json error\"}".to_string());
    let cstring = CString::new(json).expect("valid json");
    store_json_allocation(cstring)
}

fn parse_translate_response(value: &serde_json::Value) -> String {
    let arr = match value.as_array() {
        Some(a) => a,
        None => return String::new(),
    };

    let translations = match arr.first() {
        Some(v) => match v.as_array() {
            Some(a) => a,
            None => return String::new(),
        },
        None => return String::new(),
    };

    let mut result = String::new();
    for group in translations {
        if let Some(parts) = group.as_array()
            && let Some(translated) = parts.first()
            && let Some(s) = translated.as_str()
        {
            result.push_str(s);
        }
    }
    result
}

fn network_translation_allowed() -> bool {
    with_runtime_config(|cfg| cfg.translate_allow_network)
}

fn urlencodingencode(s: &str) -> String {
    let mut result = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use look_indexing::{Candidate, CandidateKind};
    use std::fs;
    use std::sync::Mutex;

    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    #[test]
    fn ffi_search_and_record_usage_smoke() {
        let lock = TEST_MUTEX.get_or_init(|| Mutex::new(()));
        let _guard = lock.lock().expect("test lock poisoned");

        let db_path = unique_test_db_path();
        let _ = fs::remove_file(&db_path);

        let mut store = SqliteStore::open(&db_path).expect("open sqlite store");
        let candidate = Candidate {
            id: "app:smoke.test".to_string(),
            kind: CandidateKind::App,
            title: "Smoke Test App".to_string(),
            subtitle: Some("smoke app".to_string()),
            path: "/Applications/Smoke Test App.app".to_string(),
            use_count: 0,
            last_used_at_unix_s: None,
        };
        store
            .upsert_candidates(&[candidate])
            .expect("insert smoke candidate");

        unsafe {
            env::set_var("LOOK_DB_PATH", db_path.as_os_str());
        }

        let query = CString::new("smoke").expect("query cstring");
        let ptr = look_search_json(query.as_ptr(), 10);
        assert!(!ptr.is_null());

        let raw = unsafe { CStr::from_ptr(ptr) }
            .to_string_lossy()
            .into_owned();
        look_free_cstring(ptr);

        let payload: serde_json::Value = serde_json::from_str(&raw).expect("valid search payload");
        let count = payload
            .get("count")
            .and_then(|value| value.as_u64())
            .unwrap_or(0);
        assert!(count >= 1);

        let has_smoke = payload
            .get("results")
            .and_then(|value| value.as_array())
            .is_some_and(|results| {
                results.iter().any(|item| {
                    item.get("id")
                        .and_then(|value| value.as_str())
                        .is_some_and(|id| id == "app:smoke.test")
                })
            });
        assert!(has_smoke);

        let id = CString::new("app:smoke.test").expect("id cstring");
        let action = CString::new("open").expect("action cstring");
        assert!(look_record_usage(id.as_ptr(), action.as_ptr()));

        let empty = CString::new("").expect("empty cstring");
        assert!(!look_record_usage(empty.as_ptr(), action.as_ptr()));

        let loaded = SqliteStore::open(&db_path)
            .expect("reopen sqlite")
            .load_candidates(None)
            .expect("load candidates after usage");
        let updated = loaded
            .iter()
            .find(|candidate| candidate.id == "app:smoke.test")
            .expect("smoke candidate exists");
        assert_eq!(updated.use_count, 1);
        assert!(updated.last_used_at_unix_s.is_some());

        let _ = fs::remove_file(&db_path);
    }

    fn unique_test_db_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        env::temp_dir().join(format!("look-ffi-smoke-{nanos}.db"))
    }
}
