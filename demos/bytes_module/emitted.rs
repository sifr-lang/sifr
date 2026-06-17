// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_vector_eq(actual: &Vec<String>, expected: &Vec<String>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize].clone()) == expected.get(i as usize).cloned());
        i = i + (1 as i64);
    }
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i = i + (1 as i64);
    }
}

// --- stdlib: sifr.bytes ---
fn decode_utf8(data: &Vec<u8>) -> Result<String, ParseError> {
    return String::from_utf8(data.iter().copied().collect::<Vec<u8>>())
        .map_err(|e| ParseError {
            message: e.to_string(),
        });
}
fn bytes_from_hex(s: &String) -> Result<Vec<u8>, ParseError> {
    return {
        let s: String = s.to_string();
        let mut cleaned = String::new();
        for ch in s.chars() {
            if ch.is_ascii_whitespace() {
                continue;
            }
            if !ch.is_ascii_hexdigit() {
                return Err(ParseError {
                    message: format!("invalid hex character: {}", ch),
                });
            }
            cleaned.push(ch);
        }
        if (cleaned.len() % 2) != 0 {
            return Err(ParseError {
                message: "fromhex() arg must contain an even number of hexadecimal digits"
                    .to_string()
                    .to_string(),
            });
        }
        let mut result = Vec::new();
        for pair in cleaned.as_bytes().chunks(2) {
            let pair_str = std::str::from_utf8(pair)
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            result
                .push(
                    u8::from_str_radix(pair_str, 16)
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?,
                );
        }
        Ok(result)
    };
}
fn count_byte(data: &Vec<u8>, value: i64) -> i64 {
    let mut count: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            count = count + (1 as i64);
        }
    }
    return count;
}
fn find_byte(data: &Vec<u8>, value: i64) -> Option<i64> {
    let mut idx: i64 = 0 as i64;
    for b in data.iter().map(|__byte| *__byte as i64) {
        if b == value {
            return Some(idx);
        }
        idx = idx + (1 as i64);
    }
    return None;
}
fn starts_with(data: &Vec<u8>, prefix: &Vec<u8>) -> bool {
    if (prefix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let mut i: i64 = 0 as i64;
    while i < (prefix.len() as i64) {
        let a: Option<i64> = data.get(i as usize).map(|__byte| *__byte as i64);
        let b: Option<i64> = prefix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}
fn ends_with(data: &Vec<u8>, suffix: &Vec<u8>) -> bool {
    if (suffix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let offset: i64 = (data.len() as i64) - (suffix.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (suffix.len() as i64) {
        let a: Option<i64> = data
            .get((offset + i) as usize)
            .map(|__byte| *__byte as i64);
        let b: Option<i64> = suffix.get(i as usize).map(|__byte| *__byte as i64);
        let Some(a) = a else {
            return false;
        };
        let Some(b) = b else {
            return false;
        };
        if a != b {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self { message: message, kind: "Other".to_string() };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    return IOError { message: msg, kind: kind };
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ParseError {
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for ValueError {
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self { message: message, detail: String::new() };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {
}

fn render_opt_int(value: Option<i64>) -> String {
    let Some(value) = value else {
        return "None".to_string();
    };
    return format!("{}", value);
}

fn collect_primary_actual(payload: &Vec<u8>) -> Vec<String> {
    let mut actual: Vec<String> = vec![];
    actual.push(format!("{}", count_byte(payload, 115 as i64)));
    actual.push(render_opt_int(find_byte(payload, 45 as i64)));
    actual.push(format!("{}", starts_with(payload, &({
    let __s = "bytes".to_string();
    __s.as_bytes().to_vec()
}))));
    actual.push(format!("{}", ends_with(payload, &({
    let __s = "e30".to_string();
    __s.as_bytes().to_vec()
}))));
    return actual;
}

fn bytes_to_hex_or_empty(payload: &Vec<u8>) -> String {
    let __sifr_try_res: Result<String, ParseError> = (|| {
    let hx: String = Ok(payload.iter().map(|__byte| format!("{:02x}", *__byte)).collect::<Vec<String>>().join(""))?;
    return Ok(hx);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _: String = format!("{}", e.message);
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
        let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?;
        result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?);
    }
    Ok(result)
})?;
    let txt: String = String::from_utf8(parsed.iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError { message: e.to_string() })?;
    return Ok(txt);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _: String = format!("{}", e.message);
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
        let pair_str = std::str::from_utf8(pair).map_err(|e| ParseError { message: e.to_string() })?;
        result.push(u8::from_str_radix(pair_str, 16).map_err(|e| ParseError { message: e.to_string() })?);
    }
    Ok(result)
})?;
    let _: String = format!("{}", odd.len() as i64);
    invalid_actual_ok.push(true);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        invalid_actual_ok.push(false);
    }
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let bad_utf8: String = String::from_utf8(vec![(255 as i64) as u8].iter().copied().collect::<Vec<u8>>()).map_err(|e| ParseError { message: e.to_string() })?;
    let _: String = format!("{}", bad_utf8);
    invalid_actual_ok.push(true);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        invalid_actual_ok.push(false);
    }
    return invalid_actual_ok;
}

fn main() {
    let payload: Vec<u8> = {
    let __s = "bytes-bytes_module".to_string();
    __s.as_bytes().to_vec()
};
    let expected: Vec<String> = vec!["2".to_string(), "5".to_string(), "true".to_string(), "true".to_string()];
    let actual: Vec<String> = collect_primary_actual(&payload);
    assert_vector_eq(&actual, &expected);
    let hex_text: String = bytes_to_hex_or_empty(&({
    let __s = "Hi".to_string();
    __s.as_bytes().to_vec()
}));
    assert!(format!("{}", (hex_text.chars().count() as i64) > (0 as i64)) == "true".to_string());
    assert!(format!("{}", hex_text) == "4869".to_string());
    let roundtrip_text: String = bytes_from_hex_to_text_or_empty(&"48 69".to_string());
    assert!(format!("{}", (roundtrip_text.chars().count() as i64) > (0 as i64)) == "true".to_string());
    assert!(format!("{}", roundtrip_text) == "Hi".to_string());
    let invalid_expected_ok: Vec<bool> = vec![false, false];
    let invalid_actual_ok: Vec<bool> = collect_invalid_actual_ok();
    assert_bool_vector_eq(&invalid_actual_ok, &invalid_expected_ok);
    println!("bytes_module bytes parity demo: pass");
}
