use crate::runtime_config::{log_error, log_info};
use look_engine::QueryEngine;
use std::collections::HashMap;
use std::env;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Instant;

static ENGINE_CACHE: OnceLock<Mutex<QueryEngine>> = OnceLock::new();
static JSON_ALLOCS: OnceLock<Mutex<HashMap<usize, CString>>> = OnceLock::new();
static BOOTSTRAP_REFRESH_STARTED: OnceLock<()> = OnceLock::new();

pub(crate) fn default_db_path() -> PathBuf {
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

pub(crate) fn with_engine<T>(f: impl FnOnce(&QueryEngine) -> T) -> T {
    let lock = engine_cache();
    let guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&guard)
}

pub(crate) fn with_engine_mut<T>(f: impl FnOnce(&mut QueryEngine) -> T) -> T {
    let lock = engine_cache();
    let mut guard = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

pub(crate) fn refresh_engine_cache() {
    if let Some(lock) = ENGINE_CACHE.get() {
        let path = default_db_path();
        if let Ok(engine) = QueryEngine::from_sqlite(path)
            && let Ok(mut guard) = lock.lock()
        {
            *guard = engine;
        }
    }
}

pub(crate) fn store_json_allocation(cstring: CString) -> *mut c_char {
    let ptr = cstring.as_ptr() as usize;

    let lock = JSON_ALLOCS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut allocations = lock.lock().unwrap_or_else(|poisoned| poisoned.into_inner());

    if allocations.contains_key(&ptr) {
        allocations.remove(&ptr);
    }
    allocations.insert(ptr, cstring);

    ptr as *mut c_char
}

pub(crate) fn free_json_allocation(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }

    if let Some(lock) = JSON_ALLOCS.get()
        && let Ok(mut allocations) = lock.lock()
    {
        allocations.remove(&(ptr as usize));
    }
}

pub(crate) fn cstr_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_string_lossy()
        .into_owned()
}

fn engine_cache() -> &'static Mutex<QueryEngine> {
    let cache = ENGINE_CACHE.get_or_init(|| {
        let path = default_db_path();
        let engine = QueryEngine::from_sqlite(&path).unwrap_or_else(|_| QueryEngine::demo_seed());
        Mutex::new(engine)
    });
    start_background_bootstrap_refresh();
    cache
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
