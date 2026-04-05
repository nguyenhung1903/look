use crate::runtime_config::network_translation_allowed;
use crate::state::{cstr_to_string, store_json_allocation};
use std::ffi::CString;
use std::os::raw::c_char;

#[derive(serde::Deserialize)]
struct TranslateResponse(serde_json::Value);

pub(crate) fn look_translate_json_impl(
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
