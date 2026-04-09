use crate::state::{cstr_to_string, store_json_allocation};
use look_storage::percent_encode;
use std::ffi::CString;
use std::os::raw::c_char;

const TRANSLATE_URL_PREFIX: &str =
    "https://translate.googleapis.com/translate_a/single?client=gtx&sl=auto&tl=";
const TRANSLATE_URL_MIDDLE: &str = "&dt=t&q=";
const CURL_BIN: &str = "curl";
const CURL_USER_AGENT: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
const CURL_ARGS_PREFIX: [&str; 9] = [
    "-s",
    "-m",
    "3",
    "--user-agent",
    CURL_USER_AGENT,
    "--tlsv1.2",
    "-H",
    "Accept-Language: en-US,en;q=0.9",
    "--compressed",
];

const ERROR_EMPTY_TEXT: &str = "empty_text";
const ERROR_REQUEST_FAILED: &str = "translate_request_failed";
const ERROR_DECODE_FAILED: &str = "translate_decode_failed";
const ERROR_EXEC_FAILED: &str = "translate_exec_failed";
const ERROR_PARSE_FAILED: &str = "translate_parse_failed";
const ERROR_EMPTY_RESULT: &str = "translate_empty_result";

const MESSAGE_EMPTY_TEXT: &str = "Type text after t\" to translate";
const MESSAGE_REQUEST_FAILED: &str = "Translation request failed";
const MESSAGE_DECODE_FAILED: &str = "Translation response decode failed";
const MESSAGE_EXEC_FAILED: &str = "Translation command execution failed";
const MESSAGE_PARSE_FAILED: &str = "Translation response parse failed";
const MESSAGE_EMPTY_RESULT: &str = "Translation returned empty result";

const JSON_ERROR_FALLBACK: &str = "{\"error\":\"json error\"}";
const JSON_TRANSLATE_ERROR_FALLBACK: &str = "{\"original\":\"\",\"translated\":\"\",\"error\":{\"code\":\"unknown\",\"message\":\"Unknown translation error\"}}";

#[derive(serde::Deserialize)]
struct TranslateResponse(serde_json::Value);

pub(crate) fn look_translate_json_impl(
    text: *const c_char,
    target_lang: *const c_char,
) -> *mut c_char {
    let text = cstr_to_string(text);
    let target_lang = cstr_to_string(target_lang);

    if text.trim().is_empty() {
        return translate_error_json(&text, ERROR_EMPTY_TEXT, MESSAGE_EMPTY_TEXT);
    }

    if !is_valid_lang_code(&target_lang) {
        return translate_error_json(&text, "invalid_target_lang", "Invalid target language code");
    }

    let encoded_text = percent_encode(&text);
    let mut url = String::with_capacity(
        TRANSLATE_URL_PREFIX.len()
            + TRANSLATE_URL_MIDDLE.len()
            + target_lang.len()
            + encoded_text.len(),
    );
    url.push_str(TRANSLATE_URL_PREFIX);
    url.push_str(&target_lang);
    url.push_str(TRANSLATE_URL_MIDDLE);
    url.push_str(&encoded_text);

    let output = std::process::Command::new(CURL_BIN)
        .args(CURL_ARGS_PREFIX)
        .arg(&url)
        .output();

    let body = match output {
        Ok(out) => {
            if !out.status.success() {
                return translate_error_json(&text, ERROR_REQUEST_FAILED, MESSAGE_REQUEST_FAILED);
            }
            match String::from_utf8(out.stdout) {
                Ok(s) => s,
                Err(_) => {
                    return translate_error_json(&text, ERROR_DECODE_FAILED, MESSAGE_DECODE_FAILED);
                }
            }
        }
        Err(_) => {
            return translate_error_json(&text, ERROR_EXEC_FAILED, MESSAGE_EXEC_FAILED);
        }
    };

    let parsed: TranslateResponse = match serde_json::from_str(&body) {
        Ok(p) => p,
        Err(_) => {
            return translate_error_json(&text, ERROR_PARSE_FAILED, MESSAGE_PARSE_FAILED);
        }
    };

    let translated = parse_translate_response(&parsed.0);
    if translated.trim().is_empty() {
        return translate_error_json(&text, ERROR_EMPTY_RESULT, MESSAGE_EMPTY_RESULT);
    }

    let result = serde_json::json!({
        "original": text,
        "translated": translated,
        "error": null
    });

    let json = serde_json::to_string(&result).unwrap_or_else(|_| JSON_ERROR_FALLBACK.to_string());
    let cstring = CString::new(json).expect("valid json");
    store_json_allocation(cstring)
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
    let json = serde_json::to_string(&payload)
        .unwrap_or_else(|_| JSON_TRANSLATE_ERROR_FALLBACK.to_string());
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

/// Validates that `code` looks like a BCP-47 language tag accepted by Google
/// Translate (e.g. "en", "vi", "zh-CN", "pt-BR").
fn is_valid_lang_code(code: &str) -> bool {
    let code = code.trim();
    if code.is_empty() || code.len() > 10 {
        return false;
    }
    code.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}
