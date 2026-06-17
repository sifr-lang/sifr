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

// --- stdlib: sifr.re ---
const IGNORECASE: i64 = 2 as i64;
fn search_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Option<String>, RegexError> {
    return {
        let __flags_val = flags;
        let mut __flag_str = String::new();
        if (__flags_val & 2) != 0 {
            __flag_str.push_str("(?i)");
        }
        if (__flags_val & 8) != 0 {
            __flag_str.push_str("(?m)");
        }
        if (__flags_val & 16) != 0 {
            __flag_str.push_str("(?s)");
        }
        if (__flags_val & 64) != 0 {
            __flag_str.push_str("(?x)");
        }
        let __pat = __flag_str + &pattern;
        let __re = regex::Regex::new(&__pat)
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            })?;
        Ok(__re.find(&text).map(|m| m.as_str().to_string()))
    };
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

fn collect_primary_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut match_ok: bool = false;
    let mut find_ok: bool = false;
    let mut replace_ok: bool = false;
    let mut findall_ok: bool = false;
    let mut split_ok: bool = false;
    let mut case_fold_ok: bool = false;
    let __sifr_try_res: Result<(), RegexError> = (|| {
    let m: bool = regex::Regex::new(&"[0-9]+".to_string()).map(|re| re.is_match(&"42 bottles".to_string())).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    match_ok = m;
    let found_num: Option<String> = regex::Regex::new(&"[0-9]+".to_string()).map(|re| re.find(&"id=9000".to_string()).map(|m| m.as_str().to_string())).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    find_ok = (found_num).map_or("None".to_string().to_string(), |__v| format!("{}", __v)) == "9000".to_string();
    let replaced: String = regex::Regex::new(&"\\s+".to_string()).map(|re| re.replace_all(&"hello   world".to_string(), &*"-".to_string()).to_string()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    replace_ok = replaced == "hello-world".to_string();
    let all_alpha: Vec<String> = regex::Regex::new(&"[a-z]+".to_string()).map(|re| re.find_iter(&"ab 12 cd".to_string()).map(|m| m.as_str().to_string()).collect::<Vec<String>>()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    findall_ok = format!("{:?}", all_alpha) == "[\"ab\", \"cd\"]".to_string();
    let split_parts: Vec<String> = regex::Regex::new(&":+".to_string()).map(|re| re.split(&"a:b::c".to_string()).map(|s| s.to_string()).collect::<Vec<String>>()).map_err(|e| RegexError { message: e.to_string(), detail: e.to_string() })?;
    split_ok = format!("{:?}", split_parts) == "[\"a\", \"b\", \"c\"]".to_string();
    let case_fold: Option<String> = search_flags(&"hello".to_string(), &"HELLO".to_string(), IGNORECASE)?;
    case_fold_ok = case_fold.is_some();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    actual.push(match_ok);
    actual.push(find_ok);
    actual.push(replace_ok);
    actual.push(findall_ok);
    actual.push(split_ok);
    actual.push(case_fold_ok);
    return actual;
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let actual: Vec<bool> = collect_primary_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("regex re parity demo: pass");
}
