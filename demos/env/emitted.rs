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

// --- stdlib: sifr.env ---
fn getenv_opt(key: &String) -> Option<String> {
    return {
        let __k = key;
        if __k.is_empty() || (__k.contains('=') || __k.as_bytes().contains(&0)) {
            None
        } else {
            std::env::var(__k).ok()
        }
    };
}
fn getenv(key: &String, default_value: &String) -> String {
    let val: Option<String> = {
        let __k = key;
        if __k.is_empty() || (__k.contains('=') || __k.as_bytes().contains(&0)) {
            None
        } else {
            std::env::var(__k).ok()
        }
    };
    let Some(val) = val else {
        return format!("{}{}", default_value, "".to_string());
    };
    return val;
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

fn main() {
    {
    let __k = "SIFR_ENV_SAMPLE".to_string();
    if !__k.is_empty() && (!__k.contains('=') && !__k.as_bytes().contains(&0)) {
        std::env::remove_var(__k);
    }
};
    {
    let __k = "SIFR_ENV_SAMPLE".to_string();
    let __v = "env".to_string();
    if !__k.is_empty() && (!__k.contains('=') && (!__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0))) {
        std::env::set_var(__k, __v);
    }
};
    let with_default: String = getenv(&"SIFR_ENV_SAMPLE".to_string(), &"fallback".to_string());
    println!("{}", with_default);
    assert!(format!("{}", with_default) == "env".to_string());
    {
    let __k = "SIFR_ENV_SAMPLE".to_string();
    if !__k.is_empty() && (!__k.contains('=') && !__k.as_bytes().contains(&0)) {
        std::env::remove_var(__k);
    }
};
    let without_default: Option<String> = getenv_opt(&"SIFR_ENV_SAMPLE".to_string());
    assert!(format!("{}", without_default.is_none()) == "true".to_string());
    assert!(format!("{}", getenv(&"SIFR_ENV_SAMPLE".to_string(), &"fallback".to_string())) == "fallback".to_string());
    let invalid_expected_lookup_found: Vec<bool> = vec![false, false];
    let mut invalid_actual_lookup_found: Vec<bool> = vec![];
    {
    let __k = "".to_string();
    let __v = "x".to_string();
    if !__k.is_empty() && (!__k.contains('=') && (!__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0))) {
        std::env::set_var(__k, __v);
    }
};
    invalid_actual_lookup_found.push(({
    let __k = "".to_string();
    if __k.is_empty() || (__k.contains('=') || __k.as_bytes().contains(&0)) { None } else { std::env::var(__k).ok() }
}) != None);
    {
    let __k = "A=B".to_string();
    let __v = "x".to_string();
    if !__k.is_empty() && (!__k.contains('=') && (!__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0))) {
        std::env::set_var(__k, __v);
    }
};
    invalid_actual_lookup_found.push(({
    let __k = "A=B".to_string();
    if __k.is_empty() || (__k.contains('=') || __k.as_bytes().contains(&0)) { None } else { std::env::var(__k).ok() }
}) != None);
    assert_bool_vector_eq(&invalid_actual_lookup_found, &invalid_expected_lookup_found);
    println!("env env parity demo: pass");
}
