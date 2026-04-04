use look_engine::QueryEngine;
use look_indexing::CandidateKind;
use look_storage::SqliteStore;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[repr(C)]
pub struct FfiSearchResult {
    pub count: u32,
}

#[derive(Serialize)]
struct FfiSearchItem {
    id: String,
    kind: String,
    title: String,
    subtitle: Option<String>,
    path: String,
    score: i64,
}

#[derive(Serialize)]
struct FfiSearchPayload {
    query: String,
    count: usize,
    results: Vec<FfiSearchItem>,
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

    let results = with_engine(|engine| engine.search(&query, max));

    let payload = FfiSearchPayload {
        query,
        count: results.len(),
        results: results
            .into_iter()
            .map(|entry| FfiSearchItem {
                id: entry.candidate.id,
                kind: match entry.candidate.kind {
                    CandidateKind::App => "app".to_string(),
                    CandidateKind::File => "file".to_string(),
                    CandidateKind::Folder => "folder".to_string(),
                },
                title: entry.candidate.title,
                subtitle: entry.candidate.subtitle,
                path: entry.candidate.path,
                score: entry.score,
            })
            .collect(),
    };

    let json = serde_json::to_string(&payload)
        .unwrap_or_else(|_| "{\"query\":\"\",\"count\":0,\"results\":[]}".to_string());
    let cstring = CString::new(json).unwrap_or_else(|_| {
        CString::new("{\"query\":\"\",\"count\":0,\"results\":[]}").expect("valid static json")
    });
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
        refresh_engine_cache();
    }

    ok
}

#[unsafe(no_mangle)]
pub extern "C" fn look_reload_config() -> bool {
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

static ENGINE_CACHE: OnceLock<Mutex<QueryEngine>> = OnceLock::new();
static JSON_ALLOCS: OnceLock<Mutex<HashMap<usize, CString>>> = OnceLock::new();

fn engine_cache() -> &'static Mutex<QueryEngine> {
    ENGINE_CACHE.get_or_init(|| {
        let path = default_db_path();
        let _ = QueryEngine::bootstrap_sqlite(&path);
        let engine = QueryEngine::from_sqlite(&path).unwrap_or_else(|_| QueryEngine::demo_seed());
        Mutex::new(engine)
    })
}

fn with_engine<T>(f: impl FnOnce(&QueryEngine) -> T) -> T {
    let lock = engine_cache();
    let guard = lock.lock().expect("engine cache lock poisoned");
    f(&guard)
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
