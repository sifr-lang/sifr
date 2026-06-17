// --- stdlib: sifr.uuid ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UUID {
    _hex: String,
}
impl UUID {
    fn new(hex_str: String) -> Self {
        return Self {
            _hex: format!("{}{}", hex_str, "".to_string()),
        };
    }
    fn hex(&self) -> String {
        let mut result: String = "".to_string();
        let mut i: i64 = 0 as i64;
        while i < (self._hex.clone().chars().count() as i64) {
            let ch: Option<String> = {
                let __sifr_index_str = &self._hex;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
            };
            if let Some(ch) = ch {
                if ch != "-".to_string() {
                    result = format!("{}{}", result, ch);
                }
            }
            i = i + (1 as i64);
        }
        return result;
    }
    fn urn(&self) -> String {
        return format!("{}{}", "urn:uuid:".to_string(), self._hex.clone());
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._hex.clone(), "".to_string());
    }
    fn version(&self) -> i64 {
        let marker: Option<String> = {
            let __sifr_index_str = &self._hex;
            let __sifr_index_i = 14 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let Some(marker) = marker else {
            return -(1 as i64);
        };
        return _hex_digit_value(&marker);
    }
}
impl std::fmt::Display for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "UUID(_hex={})", self._hex);
    }
}
fn _to_lower_hex_char(ch: &String) -> String {
    if ch.clone() == "A".to_string() {
        return "a".to_string();
    }
    if ch.clone() == "B".to_string() {
        return "b".to_string();
    }
    if ch.clone() == "C".to_string() {
        return "c".to_string();
    }
    if ch.clone() == "D".to_string() {
        return "d".to_string();
    }
    if ch.clone() == "E".to_string() {
        return "e".to_string();
    }
    if ch.clone() == "F".to_string() {
        return "f".to_string();
    }
    return format!("{}{}", ch, "".to_string());
}
fn _is_hex_char(ch: &String) -> bool {
    if ch.clone() == "0".to_string() {
        return true;
    }
    if ch.clone() == "1".to_string() {
        return true;
    }
    if ch.clone() == "2".to_string() {
        return true;
    }
    if ch.clone() == "3".to_string() {
        return true;
    }
    if ch.clone() == "4".to_string() {
        return true;
    }
    if ch.clone() == "5".to_string() {
        return true;
    }
    if ch.clone() == "6".to_string() {
        return true;
    }
    if ch.clone() == "7".to_string() {
        return true;
    }
    if ch.clone() == "8".to_string() {
        return true;
    }
    if ch.clone() == "9".to_string() {
        return true;
    }
    if ch.clone() == "a".to_string() {
        return true;
    }
    if ch.clone() == "b".to_string() {
        return true;
    }
    if ch.clone() == "c".to_string() {
        return true;
    }
    if ch.clone() == "d".to_string() {
        return true;
    }
    if ch.clone() == "e".to_string() {
        return true;
    }
    if ch.clone() == "f".to_string() {
        return true;
    }
    if ch.clone() == "A".to_string() {
        return true;
    }
    if ch.clone() == "B".to_string() {
        return true;
    }
    if ch.clone() == "C".to_string() {
        return true;
    }
    if ch.clone() == "D".to_string() {
        return true;
    }
    if ch.clone() == "E".to_string() {
        return true;
    }
    if ch.clone() == "F".to_string() {
        return true;
    }
    return false;
}
fn _hex_digit_value(ch: &String) -> i64 {
    if ch.clone() == "0".to_string() {
        return 0 as i64;
    }
    if ch.clone() == "1".to_string() {
        return 1 as i64;
    }
    if ch.clone() == "2".to_string() {
        return 2 as i64;
    }
    if ch.clone() == "3".to_string() {
        return 3 as i64;
    }
    if ch.clone() == "4".to_string() {
        return 4 as i64;
    }
    if ch.clone() == "5".to_string() {
        return 5 as i64;
    }
    if ch.clone() == "6".to_string() {
        return 6 as i64;
    }
    if ch.clone() == "7".to_string() {
        return 7 as i64;
    }
    if ch.clone() == "8".to_string() {
        return 8 as i64;
    }
    if ch.clone() == "9".to_string() {
        return 9 as i64;
    }
    if ((ch.clone() == "a".to_string()) || (ch.clone() == "A".to_string())) {
        return 10 as i64;
    }
    if ((ch.clone() == "b".to_string()) || (ch.clone() == "B".to_string())) {
        return 11 as i64;
    }
    if ((ch.clone() == "c".to_string()) || (ch.clone() == "C".to_string())) {
        return 12 as i64;
    }
    if ((ch.clone() == "d".to_string()) || (ch.clone() == "D".to_string())) {
        return 13 as i64;
    }
    if ((ch.clone() == "e".to_string()) || (ch.clone() == "E".to_string())) {
        return 14 as i64;
    }
    if ((ch.clone() == "f".to_string()) || (ch.clone() == "F".to_string())) {
        return 15 as i64;
    }
    return -(1 as i64);
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = {
            let __sifr_index_str = &value;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(ch) = ch {
            result = format!("{}{}", result, ch);
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _starts_with(value: &String, prefix: &String) -> bool {
    if (value.len() as i64) < (prefix.len() as i64) {
        return false;
    }
    let mut i: i64 = 0 as i64;
    while i < (prefix.chars().count() as i64) {
        let left: Option<String> = {
            let __sifr_index_str = &value;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let right: Option<String> = Some({
            let Some(__indexed_char) = prefix.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if left != right {
            return false;
        }
        i = i + (1 as i64);
    }
    return true;
}
fn _canonical_uuid_text(input_text: &String) -> Result<String, ValueError> {
    let mut normalized_input: String = format!("{}{}", input_text, "".to_string());
    if _starts_with(&normalized_input, &"urn:uuid:".to_string()) {
        normalized_input = _substring(
            &normalized_input,
            9 as i64,
            normalized_input.chars().count() as i64,
        );
    }
    if (normalized_input.chars().count() as i64) >= (2 as i64) {
        let first: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 0 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let last: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = (normalized_input.chars().count() as i64) - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if ((first == Some("{".to_string())) && (last == Some("}".to_string()))) {
            normalized_input = _substring(
                &normalized_input,
                1 as i64,
                (normalized_input.chars().count() as i64) - (1 as i64),
            );
        }
    }
    let input_len: i64 = normalized_input.chars().count() as i64;
    let mut hex_only: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < input_len {
        let ch_opt: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "-".to_string() {} else {
                if !(_is_hex_char(&ch)) {
                    return Err(ValueError::new("invalid UUID hex string".to_string()));
                }
                hex_only = format!("{}{}", hex_only, _to_lower_hex_char(& ch));
            }
        }
        i = i + (1 as i64);
    }
    if (hex_only.chars().count() as i64) != (32 as i64) {
        return Err(
            ValueError::new("UUID hex string must be 32 hex characters".to_string()),
        );
    }
    if input_len == (36 as i64) {
        let h1: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 8 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h2: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 13 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h3: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 18 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let h4: Option<String> = {
            let __sifr_index_str = &normalized_input;
            let __sifr_index_i = 23 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if ((((h1 != Some("-".to_string())) || (h2 != Some("-".to_string())))
            || (h3 != Some("-".to_string()))) || (h4 != Some("-".to_string())))
        {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    } else {
        if input_len != (32 as i64) {
            return Err(ValueError::new("invalid UUID hex string".to_string()));
        }
    }
    let mut canonical: String = "".to_string();
    let mut j: i64 = 0 as i64;
    while j < (hex_only.chars().count() as i64) {
        if (((j == (8 as i64)) || (j == (12 as i64))) || (j == (16 as i64)))
            || (j == (20 as i64))
        {
            canonical = format!("{}{}", canonical, "-".to_string());
        }
        let part: Option<String> = Some({
            let Some(__indexed_char) = hex_only.chars().nth(j as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(part) = part {
            canonical = format!("{}{}", canonical, part);
        }
        j = j + (1 as i64);
    }
    return Ok(canonical);
}
fn uuid4_obj() -> UUID {
    return UUID::new({
        let seg1 = rand::random::<u32>();
        let seg2 = rand::random::<u16>();
        let seg3 = (rand::random::<u16>() & 4095) | 16384;
        let seg4 = (rand::random::<u16>() & 16383) | 32768;
        let seg5_hi = rand::random::<u32>();
        let seg5_lo = rand::random::<u16>();
        let seg5 = ((seg5_hi as u64) << 16) | (seg5_lo as u64);
        format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", seg1, seg2, seg3, seg4, seg5)
    });
}
fn uuid_from_hex(hex_str: &String) -> Result<UUID, ValueError> {
    let __sifr_try_res: Result<Result<UUID, ValueError>, ValueError> = (|| {
        let canonical: String = _canonical_uuid_text(hex_str)?;
        return Ok(Ok(UUID::new(canonical)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message));
        }
    }
}
fn uuid3(namespace: &UUID, name: &String) -> UUID {
    return UUID::new({
        let __ns = uuid::Uuid::parse_str(&namespace.to_str())
            .unwrap_or(uuid::Uuid::nil());
        let __id = uuid::Uuid::new_v3(&__ns, name.as_bytes());
        __id.hyphenated().to_string()
    });
}
fn uuid5(namespace: &UUID, name: &String) -> UUID {
    return UUID::new({
        let __ns = uuid::Uuid::parse_str(&namespace.to_str())
            .unwrap_or(uuid::Uuid::nil());
        let __id = uuid::Uuid::new_v5(&__ns, name.as_bytes());
        __id.hyphenated().to_string()
    });
}
fn NAMESPACE_DNS() -> UUID {
    return UUID::new("6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string());
}

// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
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

fn is_canonical_shape(value: &String) -> bool {
    if (value.len() as i64) != (36 as i64) {
        return false;
    }
    let h1: Option<String> = {
    let __sifr_index_str = &value;
    let __sifr_index_i = 8 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    let h2: Option<String> = {
    let __sifr_index_str = &value;
    let __sifr_index_i = 13 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    let h3: Option<String> = {
    let __sifr_index_str = &value;
    let __sifr_index_i = 18 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    let h4: Option<String> = {
    let __sifr_index_str = &value;
    let __sifr_index_i = 23 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
};
    return ((((h1 == Some("-".to_string())) && (h2 == Some("-".to_string()))) && (h3 == Some("-".to_string()))) && (h4 == Some("-".to_string())));
}

fn collect_generated_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let id_text: String = {
    let seg1 = rand::random::<u32>();
    let seg2 = rand::random::<u16>();
    let seg3 = (rand::random::<u16>() & 4095) | 16384;
    let seg4 = (rand::random::<u16>() & 16383) | 32768;
    let seg5_hi = rand::random::<u32>();
    let seg5_lo = rand::random::<u16>();
    let seg5 = ((seg5_hi as u64) << 16) | (seg5_lo as u64);
    format!("{:08x}-{:04x}-{:04x}-{:04x}-{:012x}", seg1, seg2, seg3, seg4, seg5)
};
    actual.push(is_canonical_shape(&id_text));
    actual.push(({
    let __sifr_index_str = &id_text;
    let __sifr_index_i = 14 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
}) == Some("4".to_string()));
    let mut obj: UUID = uuid4_obj();
    actual.push(is_canonical_shape(&obj.to_str()) && (obj.version() == (4 as i64)));
    return actual;
}

fn collect_parse_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut parsed_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let mut parsed: UUID = uuid_from_hex(&"550E8400E29B41D4A716446655440000".to_string())?;
    parsed_ok = parsed.to_str() == "550e8400-e29b-41d4-a716-446655440000".to_string();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        parsed_ok = false;
    }
    actual.push(parsed_ok);
    let mut parsed_v1_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let mut parsed_v1: UUID = uuid_from_hex(&"550e8400-e29b-11d4-a716-446655440000".to_string())?;
    parsed_v1_ok = parsed_v1.version() == (1 as i64);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        parsed_v1_ok = false;
    }
    actual.push(parsed_v1_ok);
    return actual;
}

fn collect_negative_and_class_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut invalid_rejected: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let _bad: UUID = uuid_from_hex(&"invalid".to_string())?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        invalid_rejected = true;
    }
    actual.push(invalid_rejected);
    let mut ctor_passthrough: UUID = UUID::new("550e8400-e29b-41d4-a716-44665544000z".to_string());
    actual.push((ctor_passthrough.to_str()).as_str() == ("550e8400-e29b-41d4-a716-44665544000z".to_string()).as_str());
    let mut ctor_curly_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let mut ctor_curly: UUID = uuid_from_hex(&"{550E8400-E29B-41D4-A716-446655440000}".to_string())?;
    ctor_curly_ok = ctor_curly.to_str() == "550e8400-e29b-41d4-a716-446655440000".to_string();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        ctor_curly_ok = false;
    }
    actual.push(ctor_curly_ok);
    let mut obj: UUID = uuid4_obj();
    actual.push((obj.hex().chars().count() as i64) == (32 as i64));
    actual.push(uuid3(&NAMESPACE_DNS(), &"python.org".to_string()).version() == (3 as i64));
    actual.push(uuid5(&NAMESPACE_DNS(), &"python.org".to_string()).version() == (5 as i64));
    return actual;
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_generated_actual());
    append_all(&mut actual, &collect_parse_actual());
    append_all(&mut actual, &collect_negative_and_class_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("uuid uuid parity demo: pass");
}
