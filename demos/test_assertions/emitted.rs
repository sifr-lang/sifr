// src/main.rs
mod sifr_generated_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ParseError {
        pub message: String,
    }
    impl ::std::fmt::Display for ParseError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ParseError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JSONDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl ::std::fmt::Display for JSONDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JSONDecodeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JsonIntegerRangeError {
        pub message: String,
        pub path: String,
        pub profile: String,
    }
    impl ::std::fmt::Display for JsonIntegerRangeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JsonIntegerRangeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JsonLimitError {
        pub message: String,
        pub limit: SifrInt,
    }
    impl ::std::fmt::Display for JsonLimitError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JsonLimitError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TOMLDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl ::std::fmt::Display for TOMLDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for TOMLDecodeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl ::std::fmt::Display for RegexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for RegexError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TimeoutError {
        pub message: String,
    }
    impl ::std::fmt::Display for TimeoutError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for TimeoutError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ScopeFailure {
        pub message: String,
    }
    impl ::std::fmt::Display for ScopeFailure {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ScopeFailure {}
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::JSONDecodeError;
pub use sifr_generated_project_nominals::JsonIntegerRangeError;
pub use sifr_generated_project_nominals::JsonLimitError;
pub use sifr_generated_project_nominals::ParseError;
pub use sifr_generated_project_nominals::RegexError;
pub use sifr_generated_project_nominals::ScopeFailure;
pub use sifr_generated_project_nominals::TOMLDecodeError;
pub use sifr_generated_project_nominals::TimeoutError;
pub use sifr_generated_project_nominals::ValueError;
const INF: f64 = f64::INFINITY;
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn assert_not_almost_eq(actual: f64, expected: f64, tolerance: f64) {
    assert!(tolerance >= 0.0_f64);
    if actual == expected {
        assert!(false);
    }
    let mut diff: f64 = actual - expected;
    if diff < 0.0_f64 {
        diff = 0.0_f64 - diff;
    }
    if diff != diff {
        return;
    }
    assert!(diff > tolerance);
}
fn assert_ge<T: Clone + 'static + PartialOrd>(a: &T, b: &T) {
    assert!(*a >= *b);
}
fn assert_le<T: Clone + 'static + PartialOrd>(a: &T, b: &T) {
    assert!(*a <= *b);
}
fn assert_some<T: Clone + 'static>(value: Option<T>) {
    assert!(value.is_some());
}
fn assert_none<T: Clone + 'static>(value: Option<T>) {
    assert!(value.is_none());
}
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn assert_ok<T: Clone + 'static>(value: Result<T, Error>) {
    let sifr_generated_try_res: Result<(), Error> = (|| {
        let _out: T = value?;
        Ok(())
    })();
    if let Err(_e) = sifr_generated_try_res {
        assert!(false);
    }
}
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn assert_err<T: Clone + 'static>(value: Result<T, Error>) {
    let sifr_generated_try_res: Result<(), Error> = (|| {
        let _out: T = value?;
        assert!(false);
        Ok(())
    })();
    let _ = sifr_generated_try_res;
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}
impl Error {
    const fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for Error {}
impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::new(err.message)
    }
}
impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::new(err.message)
    }
}
impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}
impl From<JSONDecodeError> for Error {
    fn from(err: JSONDecodeError) -> Self {
        Self::new(err.message)
    }
}
impl From<JsonIntegerRangeError> for Error {
    fn from(err: JsonIntegerRangeError) -> Self {
        Self::new(err.message)
    }
}
impl From<JsonLimitError> for Error {
    fn from(err: JsonLimitError) -> Self {
        Self::new(err.message)
    }
}
impl From<TOMLDecodeError> for Error {
    fn from(err: TOMLDecodeError) -> Self {
        Self::new(err.message)
    }
}
impl From<RegexError> for Error {
    fn from(err: RegexError) -> Self {
        Self::new(err.message)
    }
}
impl From<TimeoutError> for Error {
    fn from(err: TimeoutError) -> Self {
        Self::new(err.message)
    }
}
impl From<ScopeFailure> for Error {
    fn from(err: ScopeFailure) -> Self {
        Self::new(err.message)
    }
}
fn parse_num(s: &str) -> Result<SifrInt, ValueError> {
    if s == "bad" {
        return Err(ValueError::new("parse failure".to_string()));
    }
    Ok(SifrInt::from_i64(10))
}
fn main() {
    println!("=== Core equality/truth assertions ===");
    assert_eq!("sifr", "sifr");
    assert_ne!("sifr", "rust");
    assert!(&SifrInt::from_i64(2) > &SifrInt::from_i64(1));
    {
        let sifr_generated_cond = &SifrInt::from_i64(1) > &SifrInt::from_i64(2);
        assert!(!sifr_generated_cond);
    };
    println!("core assertions ok");
    println!("=== Almost-equality semantics ===");
    {
        let sifr_generated_lhs = 0.1_f64 + 0.2_f64;
        let sifr_generated_rhs = 0.3_f64;
        let sifr_generated_tol = 0.0001_f64;
        assert!(
            sifr_generated_lhs == sifr_generated_rhs
                || (sifr_generated_lhs - sifr_generated_rhs).abs() <= sifr_generated_tol,
            "assert_almost_eq failed: {sifr_generated_lhs} != {sifr_generated_rhs} (tolerance {sifr_generated_tol})"
        );
    };
    {
        let sifr_generated_lhs = INF;
        let sifr_generated_rhs = INF;
        let sifr_generated_tol = 0.0_f64;
        assert!(
            sifr_generated_lhs == sifr_generated_rhs
                || (sifr_generated_lhs - sifr_generated_rhs).abs() <= sifr_generated_tol,
            "assert_almost_eq failed: {sifr_generated_lhs} != {sifr_generated_rhs} (tolerance {sifr_generated_tol})"
        );
    };
    assert_not_almost_eq(1.1_f64, 1.0_f64, 0.05_f64);
    println!("almost assertions ok");
    println!("=== Comparable assertions ===");
    assert!(
        &SifrInt::from_i64(5) > &SifrInt::from_i64(4),
        "assert_gt failed: {} is not > {}",
        SifrInt::from_i64(5),
        SifrInt::from_i64(4)
    );
    assert_ge(&SifrInt::from_i64(5), &SifrInt::from_i64(5));
    assert!(
        "a".to_string() < "b".to_string(),
        "assert_lt failed: a is not < b"
    );
    assert_le(&"b".to_string(), &"b".to_string());
    println!("comparison assertions ok");
    println!("=== Result/Option adapted assertions ===");
    assert_ok(parse_num(&"ok".to_string()).map_err(::std::convert::Into::<Error>::into));
    assert_err(parse_num(&"bad".to_string()).map_err(::std::convert::Into::<Error>::into));
    let maybe_name: Option<String> = Some("sifr".to_string());
    let maybe_missing: Option<String> = None;
    assert_some(maybe_name);
    assert_none(maybe_missing);
    println!("result/option assertions ok");
}
