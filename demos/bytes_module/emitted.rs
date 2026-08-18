// src/main.rs
// --- stdlib: sifr.test ---
fn assert_vector_eq(actual: &Vec<String>, expected: &Vec<String>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize].clone()) == expected.get(i as usize).cloned());
        i += 1_i64;
    }
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        Self { message, kind: "Other".to_string() }
    }
}

impl ::std::fmt::Display for IOError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for IOError {
}

fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
    let __sifr_io_kind = (&e as &dyn ::std::any::Any).downcast_ref::<std::io::Error>().map(::std::io::Error::kind);
    match __sifr_io_kind {
    Some(::std::io::ErrorKind::NotFound) => {
        "FileNotFound".to_string()
    },
    Some(::std::io::ErrorKind::PermissionDenied) => {
        "PermissionDenied".to_string()
    },
    Some(::std::io::ErrorKind::AlreadyExists) => {
        "FileExists".to_string()
    },
    Some(::std::io::ErrorKind::IsADirectory) => {
        "IsADirectory".to_string()
    },
    Some(::std::io::ErrorKind::NotADirectory) => {
        "NotADirectory".to_string()
    },
    Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
        "DirectoryNotEmpty".to_string()
    },
    _ => {
        "Other".to_string()
    },
}
};
    IOError { message: msg, kind }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ParseError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ValueError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        Self { message, line: 0, column: 0 }
    }
}

impl ::std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JsonIntegerRangeError {
    message: String,
    path: String,
    profile: String,
}

impl JsonIntegerRangeError {
    fn new(message: String) -> Self {
        Self { message, path: String::new(), profile: String::new() }
    }
}

impl ::std::fmt::Display for JsonIntegerRangeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for JsonIntegerRangeError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JsonLimitError {
    message: String,
    limit: i64,
}

impl JsonLimitError {
    fn new(message: String) -> Self {
        Self { message, limit: 0 }
    }
}

impl ::std::fmt::Display for JsonLimitError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for JsonLimitError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        Self { message, line: 0, column: 0 }
    }
}

impl ::std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        Self { message, detail: String::new() }
    }
}

impl ::std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for RegexError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TimeoutError {
    message: String,
}

impl TimeoutError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for TimeoutError {
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ScopeFailure {
    message: String,
}

impl ScopeFailure {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl ::std::fmt::Display for ScopeFailure {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for ScopeFailure {
}

fn render_opt_int(value: Option<i64>) -> String {
    let Some(value) = value else {
        return "None".to_string();
    };
    format!("{}", value)
}

fn collect_primary_actual(payload: &Vec<u8>) -> Vec<String> {
    let mut actual: Vec<String> = vec![];
    actual.push(format!("{}", {
    let __bytes_receiver = &payload;
    {
    let __needle = 115_i64;
    if (__needle < 0) || (__needle > 255) { 0 } else { __bytes_receiver.iter().filter(|__x| **__x == (__needle as u8)).count() as i64 }
}
}));
    actual.push(render_opt_int({
    let __bytes_receiver = &payload;
    {
    let __needle = 45_i64;
    if (__needle < 0) || (__needle > 255) { None } else { {
    let __len = __bytes_receiver.len() as i64;
    let __start = 0;
    let __stop = __len;
    let mut __i = __start;
    let mut __result = None;
    while (__i < __stop) && (__result == None) {
        if let Some(__x) = __bytes_receiver.get(__i as usize) {
            if *__x == (__needle as u8) {
                __result = Some(__i);
            }
        }
        __i += 1;
    }
    __result
} }
}
}));
    actual.push(format!("{}", payload.starts_with(&vec![(98_i64) as u8, (121_i64) as u8, (116_i64) as u8, (101_i64) as u8, (115_i64) as u8])));
    actual.push(format!("{}", payload.ends_with(&vec![(101_i64) as u8, (51_i64) as u8, (48_i64) as u8])));
    actual
}

fn bytes_to_hex_or_empty(payload: &Vec<u8>) -> String {
    let __sifr_try_res: Result<String, ParseError> = (|| {
    let hx: String = {
    let __bytes_receiver = &payload;
    let mut __hex = String::with_capacity(__bytes_receiver.len().saturating_mul(2));
    for __byte in __bytes_receiver.iter() {
        __hex.push_str(&format!("{:02x}", *__byte));
    }
    __hex
};
    return Ok(hx);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _ = format!("{}", e.message);
            return "".to_string();
        },
    }
}

fn bytes_from_hex_to_text_or_empty(payload: &String) -> String {
    let __sifr_try_res: Result<String, ParseError> = (|| {
    let parsed: Vec<u8> = ({
    let s: String = payload.to_string();
    let mut cleaned = String::new();
    for ch in s.chars() {
        if ch.is_ascii_whitespace() {
            continue;
        }
        if !ch.is_ascii_hexdigit() {
            return Err(ParseError { message: format!("invalid hex character: {}", ch) });
        }
        cleaned.push(ch);
    }
    if (cleaned.len() % 2) != 0 {
        return Err(ParseError { message: "fromhex() arg must contain an even number of hexadecimal digits".to_string().to_string() });
    }
    let mut result = Vec::new();
    for pair in cleaned.as_bytes().chunks(2) {
        let pair_str = ::std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?;
        result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?);
    }
    Ok::<Vec<u8>, ParseError>(result)
})?;
    let txt: String = ::sifr_runtime::encoding::decode_text(&parsed, &"utf-8".to_string(), &"strict".to_string()).map_err(|__message| ParseError { message: __message })?;
    return Ok(txt);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _ = format!("{}", e.message);
            return "".to_string();
        },
    }
}

fn collect_invalid_actual_ok() -> Vec<bool> {
    let mut invalid_actual_ok: Vec<bool> = vec![];
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let odd: Vec<u8> = ({
    let s: String = "abc".to_string().to_string();
    let mut cleaned = String::new();
    for ch in s.chars() {
        if ch.is_ascii_whitespace() {
            continue;
        }
        if !ch.is_ascii_hexdigit() {
            return Err(ParseError { message: format!("invalid hex character: {}", ch) });
        }
        cleaned.push(ch);
    }
    if (cleaned.len() % 2) != 0 {
        return Err(ParseError { message: "fromhex() arg must contain an even number of hexadecimal digits".to_string().to_string() });
    }
    let mut result = Vec::new();
    for pair in cleaned.as_bytes().chunks(2) {
        let pair_str = ::std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?;
        result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?);
    }
    Ok::<Vec<u8>, ParseError>(result)
})?;
    let _ = format!("{}", odd.len() as i64);
    invalid_actual_ok.push(true);
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        invalid_actual_ok.push(false);
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let bad_utf8: String = ::sifr_runtime::encoding::decode_text(&vec![(255_i64) as u8], &"utf-8".to_string(), &"strict".to_string()).map_err(|__message| ParseError { message: __message })?;
    let _ = format!("{}", bad_utf8);
    invalid_actual_ok.push(true);
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        invalid_actual_ok.push(false);
    }
    invalid_actual_ok
}

fn main() {
    let payload: Vec<u8> = vec![(98_i64) as u8, (121_i64) as u8, (116_i64) as u8, (101_i64) as u8, (115_i64) as u8, (45_i64) as u8, (98_i64) as u8, (121_i64) as u8, (116_i64) as u8, (101_i64) as u8, (115_i64) as u8, (95_i64) as u8, (109_i64) as u8, (111_i64) as u8, (100_i64) as u8, (117_i64) as u8, (108_i64) as u8, (101_i64) as u8];
    let expected: Vec<String> = vec!["2".to_string(), "5".to_string(), "true".to_string(), "false".to_string()];
    let actual: Vec<String> = collect_primary_actual(&payload);
    assert_vector_eq(&actual, &expected);
    let hex_text: String = bytes_to_hex_or_empty(&vec![(72_i64) as u8, (105_i64) as u8]);
    let __sifr_chars_hex_text: Vec<char> = hex_text.chars().collect::<Vec<char>>();
    assert!((format!("{}", (hex_text.chars().count() as i64) > (0_i64)) == "true"));
    assert!((format!("{}", hex_text) == "4869"));
    let roundtrip_text: String = bytes_from_hex_to_text_or_empty(&"48 69".to_string());
    let __sifr_chars_roundtrip_text: Vec<char> = roundtrip_text.chars().collect::<Vec<char>>();
    assert!((format!("{}", (roundtrip_text.chars().count() as i64) > (0_i64)) == "true"));
    assert!((format!("{}", roundtrip_text) == "Hi"));
    let invalid_expected_ok: Vec<bool> = vec![false, false];
    let invalid_actual_ok: Vec<bool> = collect_invalid_actual_ok();
    assert_bool_vector_eq(&invalid_actual_ok, &invalid_expected_ok);
    println!("bytes_module bytes parity demo: pass");
}
