#![allow(unsafe_code)]

use serde_json::Value;
use std::ffi::{CStr, CString, c_char, c_int};
use std::fmt;

#[repr(C)]
struct PgQueryError {
    message: *mut c_char,
    funcname: *mut c_char,
    filename: *mut c_char,
    lineno: c_int,
    cursorpos: c_int,
    context: *mut c_char,
}

#[repr(C)]
struct PgQueryParseResult {
    parse_tree: *mut c_char,
    stderr_buffer: *mut c_char,
    error: *mut PgQueryError,
}

#[repr(C)]
struct PgQueryNormalizeResult {
    normalized_query: *mut c_char,
    error: *mut PgQueryError,
}

unsafe extern "C" {
    fn pg_query_parse(input: *const c_char) -> PgQueryParseResult;
    fn pg_query_free_parse_result(result: PgQueryParseResult);
    fn pg_query_normalize(input: *const c_char) -> PgQueryNormalizeResult;
    fn pg_query_free_normalize_result(result: PgQueryNormalizeResult);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RawParserError {
    pub message: String,
    pub cursor: u32,
}

impl fmt::Display for RawParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at byte {}", self.message, self.cursor)
    }
}

impl std::error::Error for RawParserError {}

pub(crate) fn parse_json(source: &str) -> Result<Value, RawParserError> {
    let input = CString::new(source).map_err(|_| RawParserError {
        message: "PostgreSQL source contains a NUL byte".to_string(),
        cursor: 0,
    })?;
    // SAFETY: `input` is NUL-terminated for the duration of the call. The
    // returned allocation is copied before the matching libpg_query free call.
    let result = unsafe { pg_query_parse(input.as_ptr()) };
    let parsed = if result.error.is_null() {
        if result.parse_tree.is_null() {
            Err(RawParserError {
                message: "libpg_query returned no parse tree".to_string(),
                cursor: 0,
            })
        } else {
            // SAFETY: libpg_query guarantees a NUL-terminated parse tree when
            // `error` is null and owns it until the result is freed.
            let bytes = unsafe { CStr::from_ptr(result.parse_tree) }.to_bytes();
            serde_json::from_slice(bytes).map_err(|_| RawParserError {
                message: "libpg_query returned malformed parser JSON".to_string(),
                cursor: 0,
            })
        }
    } else {
        // SAFETY: a non-null error pointer refers to a live `PgQueryError`
        // until the parse result is freed.
        let error = unsafe { &*result.error };
        let message = copy_message(error.message, "PostgreSQL parse failed");
        Err(RawParserError {
            message,
            cursor: u32::try_from(error.cursorpos.saturating_sub(1)).unwrap_or(0),
        })
    };
    // SAFETY: this is the unique matching release for `result`.
    unsafe { pg_query_free_parse_result(result) };
    parsed
}

pub(crate) fn normalize(source: &str) -> Result<String, RawParserError> {
    let input = CString::new(source).map_err(|_| RawParserError {
        message: "PostgreSQL source contains a NUL byte".to_string(),
        cursor: 0,
    })?;
    // SAFETY: `input` remains live for the call; the result is released below.
    let result = unsafe { pg_query_normalize(input.as_ptr()) };
    let normalized = if result.error.is_null() {
        if result.normalized_query.is_null() {
            Err(RawParserError {
                message: "libpg_query returned no normalized query".to_string(),
                cursor: 0,
            })
        } else {
            // SAFETY: libpg_query guarantees a NUL-terminated string here.
            Ok(unsafe { CStr::from_ptr(result.normalized_query) }
                .to_string_lossy()
                .into_owned())
        }
    } else {
        // SAFETY: a non-null error is live until the result is released.
        let error = unsafe { &*result.error };
        Err(RawParserError {
            message: copy_message(error.message, "PostgreSQL normalization failed"),
            cursor: u32::try_from(error.cursorpos.saturating_sub(1)).unwrap_or(0),
        })
    };
    // SAFETY: this is the unique matching release for `result`.
    unsafe { pg_query_free_normalize_result(result) };
    normalized
}

fn copy_message(pointer: *const c_char, fallback: &str) -> String {
    if pointer.is_null() {
        return fallback.to_string();
    }
    // SAFETY: libpg_query error messages are NUL-terminated and live until the
    // enclosing result is freed. The returned Rust string owns its bytes.
    unsafe { CStr::from_ptr(pointer) }
        .to_string_lossy()
        .into_owned()
}
