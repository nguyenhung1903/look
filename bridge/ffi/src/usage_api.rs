use crate::runtime_config::{log_debug, log_error};
use crate::state::{cstr_to_string, default_db_path, with_engine_mut};
use look_storage::SqliteStore;
use std::os::raw::c_char;
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn look_record_usage_impl(candidate_id: *const c_char, action: *const c_char) -> bool {
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
