use crate::runtime_config::log_debug;
use crate::state::{cstr_to_string, store_json_allocation, with_engine};
use std::ffi::CString;
use std::os::raw::c_char;
use std::time::Instant;

const MAX_SEARCH_COUNT_QUERY_LEN: u32 = 1000;
const DEFAULT_SEARCH_LIMIT: u32 = 20;
const MAX_SEARCH_LIMIT: u32 = 100;

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

pub(crate) fn look_search_count_impl(query_len: u32) -> FfiSearchResult {
    let len = query_len.min(MAX_SEARCH_COUNT_QUERY_LEN);
    let query = "x".repeat(len as usize);
    let results = with_engine(|engine| engine.search(&query, DEFAULT_SEARCH_LIMIT as usize));
    FfiSearchResult {
        count: results.len() as u32,
    }
}

pub(crate) fn look_search_json_impl(query: *const c_char, limit: u32) -> *mut c_char {
    let query = cstr_to_string(query);
    let max = if limit == 0 {
        DEFAULT_SEARCH_LIMIT
    } else {
        limit.min(MAX_SEARCH_LIMIT)
    };
    let started_at = Instant::now();

    let results = with_engine(|engine| engine.search(&query, max as usize));
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
