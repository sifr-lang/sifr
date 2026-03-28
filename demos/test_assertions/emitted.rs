// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_ne<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual != * expected);
}
fn assert_true(value: bool) {
    assert!(value);
}
fn assert_false(value: bool) {
    assert!(! value);
}
fn assert_almost_eq(actual: f64, expected: f64, tolerance: f64) {
    assert!(tolerance >= (0.0 as f64));
    if actual == expected {
        return;
    }
    let mut diff: f64 = actual - expected;
    if diff < (0.0 as f64) {
        diff = (0.0 as f64) - diff;
    }
    if diff != diff {
        assert!(false);
    }
    assert!(diff <= tolerance);
}
fn assert_not_almost_eq(actual: f64, expected: f64, tolerance: f64) {
    assert!(tolerance >= (0.0 as f64));
    if actual == expected {
        assert!(false);
    }
    let mut diff: f64 = actual - expected;
    if diff < (0.0 as f64) {
        diff = (0.0 as f64) - diff;
    }
    if diff != diff {
        return;
    }
    assert!(diff > tolerance);
}
fn assert_gt<T: Clone + std::fmt::Display + PartialOrd + 'static>(a: &T, b: &T) {
    assert!(* a > * b);
}
fn assert_ge<T: Clone + std::fmt::Display + PartialOrd + 'static>(a: &T, b: &T) {
    assert!(* a >= * b);
}
fn assert_lt<T: Clone + std::fmt::Display + PartialOrd + 'static>(a: &T, b: &T) {
    assert!(* a < * b);
}
fn assert_le<T: Clone + std::fmt::Display + PartialOrd + 'static>(a: &T, b: &T) {
    assert!(* a <= * b);
}
fn assert_some<T: Clone + std::fmt::Display + PartialOrd + 'static>(value: Option<T>) {
    assert!(value.is_some());
}
fn assert_none<T: Clone + std::fmt::Display + PartialOrd + 'static>(value: Option<T>) {
    assert!(value.is_none());
}
fn assert_ok<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    value: Result<T, Error>,
) {
    let __sifr_try_res: Result<(), Error> = (|| {
        let out: T = value?;
        return Ok(());
    })();
    if let Err(e) = __sifr_try_res {
        assert!(false);
    }
}
fn assert_err<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    value: Result<T, Error>,
) {
    let __sifr_try_res: Result<(), Error> = (|| {
        let out: T = value?;
        assert!(false);
        return Ok(());
    })();
    if let Err(e) = __sifr_try_res {}
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
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for Error {
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

fn parse_num(s: &String) -> Result<i64, ValueError> {
    if s.clone() == "bad".to_string() {
        return Err(ValueError::new("parse failure".to_string()));
    }
    return Ok(10 as i64);
}

fn main() {
    println!("=== Core equality/truth assertions ===");
    assert_eq!("sifr", "sifr");
    assert_ne!("sifr", "rust");
    assert!((2 as i64) > (1 as i64));
    {
    let __cond = (1 as i64) > (2 as i64);
    assert!(!__cond)
};
    println!("core assertions ok");
    println!("=== Almost-equality semantics ===");
    {
    let __lhs = (0.1 as f64) + (0.2 as f64);
    let __rhs = 0.3 as f64;
    let __tol = 0.0001 as f64;
    assert!((__lhs == __rhs) || ((__lhs - __rhs).abs() <= __tol), "assert_almost_eq failed: {} != {} (tolerance {})", __lhs, __rhs, __tol)
};
    {
    let __lhs = f64::INFINITY;
    let __rhs = f64::INFINITY;
    let __tol = 0.0 as f64;
    assert!((__lhs == __rhs) || ((__lhs - __rhs).abs() <= __tol), "assert_almost_eq failed: {} != {} (tolerance {})", __lhs, __rhs, __tol)
};
    assert_not_almost_eq(1.1 as f64, 1.0 as f64, 0.05 as f64);
    println!("almost assertions ok");
    println!("=== Comparable assertions ===");
    assert!((5 as i64) > (4 as i64), "assert_gt failed: {} is not > {}", 5 as i64, 4 as i64);
    assert_ge(&(5 as i64), &(5 as i64));
    assert!("a".to_string() < "b".to_string(), "assert_lt failed: {} is not < {}", "a", "b");
    assert_le(&"b".to_string(), &"b".to_string());
    println!("comparison assertions ok");
    println!("=== Result/Option adapted assertions ===");
    assert_ok((parse_num(&"ok".to_string())).map_err(|__e| Error::new(__e.to_string())));
    assert_err((parse_num(&"bad".to_string())).map_err(|__e| Error::new(__e.to_string())));
    let maybe_name: Option<String> = Some("sifr".to_string());
    let maybe_missing: Option<String> = None;
    assert_some(maybe_name);
    assert_none(maybe_missing);
    println!("result/option assertions ok");
}
