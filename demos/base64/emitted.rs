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
fn bytes_from_ints(values: &Vec<i64>) -> Result<Vec<u8>, ValueError> {
    return {
        let __vals = values;
        let mut __out = Vec::new();
        for __pair in __vals.iter().enumerate() {
            if (*__pair.1 < 0) || (*__pair.1 > 255) {
                return Err(ValueError {
                    message: format!(
                        "byte out of range at index {}: {}", __pair.0, * __pair.1
                    ),
                });
            }
            __out.push(*__pair.1 as u8);
        }
        Ok(__out)
    };
}
fn bytes_with_size(size: i64) -> Result<Vec<u8>, ValueError> {
    return {
        let __size = size;
        if __size < 0 {
            return Err(ValueError {
                message: "bytes(size) requires a non-negative size"
                    .to_string()
                    .to_string(),
            });
        }
        Ok((0..__size).map(|_| 0 as u8).collect::<Vec<u8>>())
    };
}
fn encode_utf8_result(s: &String) -> Result<Vec<u8>, ParseError> {
    return Ok({
        let __s = s;
        __s.as_bytes().to_vec()
    });
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

// --- stdlib: sifr.base64 ---
fn b64encode(s: &String) -> String {
    return base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &s.as_bytes(),
    );
}
fn b64decode(s: &String) -> Result<String, ParseError> {
    return {
        let __bytes = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &s.as_bytes(),
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        String::from_utf8(__bytes)
            .map_err(|e| ParseError {
                message: e.to_string(),
            })
    };
}
fn standard_b64encode(s: &String) -> String {
    return base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &s.as_bytes(),
    );
}
fn standard_b64decode(s: &String) -> Result<String, ParseError> {
    return {
        let __bytes = base64::Engine::decode(
                &base64::engine::general_purpose::STANDARD,
                &s.as_bytes(),
            )
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        String::from_utf8(__bytes)
            .map_err(|e| ParseError {
                message: e.to_string(),
            })
    };
}
fn b16encode(s: &String) -> Result<String, ParseError> {
    let __sifr_try_res: Result<Result<String, ParseError>, ParseError> = (|| {
        let raw: String = Ok(
            ({
                let __s = s;
                __s.as_bytes().to_vec()
            })
                .iter()
                .map(|__byte| format!("{:02x}", * __byte))
                .collect::<Vec<String>>()
                .join(""),
        )?;
        return Ok(Ok(raw.to_uppercase()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn b16decode(s: &String) -> Result<String, ParseError> {
    let __sifr_try_res: Result<Result<String, ParseError>, ParseError> = (|| {
        let data: Vec<u8> = ({
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
        })?;
        return Ok(
            String::from_utf8(data.iter().copied().collect::<Vec<u8>>())
                .map_err(|e| ParseError {
                    message: e.to_string(),
                }),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}

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

fn encode_b64_or_empty(payload: &String) -> String {
    return b64encode(payload);
}

fn encode_standard_b64_or_empty(payload: &String) -> String {
    return standard_b64encode(payload);
}

fn encode_urlsafe_b64_or_empty(payload: &String) -> String {
    return base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE, &payload.as_bytes());
}

fn decode_b64_or_empty(payload: &String) -> String {
    let __sifr_try_res: Result<String, ParseError> = (|| {
    let decoded: String = b64decode(payload)?;
    return Ok(decoded);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _: String = format!("{}", format!("{}{}", "unexpected: ".to_string(), e.message));
            return "".to_string();
        },
    }
}

fn decode_standard_b64_or_empty(payload: &String) -> String {
    let __sifr_try_res: Result<String, ParseError> = (|| {
    let decoded: String = standard_b64decode(payload)?;
    return Ok(decoded);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _: String = format!("{}", format!("{}{}", "unexpected: ".to_string(), e.message));
            return "".to_string();
        },
    }
}

fn decode_urlsafe_b64_or_empty(payload: &String) -> String {
    let __sifr_try_res: Result<String, ParseError> = (|| {
    let decoded: String = ({
    let __bytes = base64::Engine::decode(&base64::engine::general_purpose::URL_SAFE, &payload.as_bytes()).map_err(|e| ParseError { message: e.to_string() })?;
    String::from_utf8(__bytes).map_err(|e| ParseError { message: e.to_string() })
})?;
    return Ok(decoded);
    unreachable!("sifr try/except return capture fell through");
})();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        },
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _: String = format!("{}", format!("{}{}", "unexpected: ".to_string(), e.message));
            return "".to_string();
        },
    }
}

fn b16_encode_or_empty(payload: &String) -> String {
    let mut encoded: String = "".to_string();
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let out: String = b16encode(payload)?;
    encoded = out;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", format!("{}{}", "unexpected: ".to_string(), e.message));
    }
    return encoded;
}

fn b16_decode_or_empty(payload: &String) -> String {
    let mut decoded: String = "".to_string();
    let __sifr_try_res: Result<(), ParseError> = (|| {
    let out: String = b16decode(payload)?;
    decoded = out;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", format!("{}{}", "unexpected: ".to_string(), e.message));
    }
    return decoded;
}

fn collect_positive_actual() -> Vec<String> {
    let mut actual: Vec<String> = vec![];
    actual.push(encode_b64_or_empty(&"foo".to_string()));
    actual.push(decode_b64_or_empty(&"Zm9v".to_string()));
    actual.push(encode_standard_b64_or_empty(&"foo".to_string()));
    actual.push(decode_standard_b64_or_empty(&"Zm9v".to_string()));
    let urlsafe_encoded: String = encode_urlsafe_b64_or_empty(&"hello".to_string());
    let urlsafe_encoded_for_decode: String = format!("{}{}", urlsafe_encoded, "".to_string());
    actual.push(urlsafe_encoded);
    actual.push(decode_urlsafe_b64_or_empty(&urlsafe_encoded_for_decode));
    let b16_encoded: String = b16_encode_or_empty(&"Hi".to_string());
    let b16_encoded_for_decode: String = format!("{}{}", b16_encoded, "".to_string());
    actual.push(b16_encoded);
    actual.push(b16_decode_or_empty(&b16_encoded_for_decode));
    return actual;
}

fn collect_decode_actual_ok(inputs: &Vec<String>) -> Vec<bool> {
    let mut actual_ok: Vec<bool> = vec![];
    for payload in inputs.iter().cloned() {
        let __sifr_try_res: Result<(), ParseError> = (|| {
    let decoded: String = b64decode(&payload)?;
    let _: String = format!("{}", decoded);
    actual_ok.push(true);
    return Ok(());
})();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            actual_ok.push(false);
        }
    }
    return actual_ok;
}

fn main() {
    let expected: Vec<String> = vec!["Zm9v".to_string(), "foo".to_string(), "Zm9v".to_string(), "foo".to_string(), "aGVsbG8=".to_string(), "hello".to_string(), "4869".to_string(), "Hi".to_string()];
    let actual: Vec<String> = collect_positive_actual();
    assert_vector_eq(&actual, &expected);
    let decode_inputs: Vec<String> = vec!["not base64!!!".to_string(), "Zm9v".to_string()];
    let expected_ok: Vec<bool> = vec![false, true];
    let actual_ok: Vec<bool> = collect_decode_actual_ok(&decode_inputs);
    assert_bool_vector_eq(&actual_ok, &expected_ok);
    println!("base64 base64 parity demo: pass");
}
