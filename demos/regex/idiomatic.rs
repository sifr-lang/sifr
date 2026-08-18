use regex::Regex;
use std::fmt::{self, Display};

const IGNORECASE: i64 = 2;
const MULTILINE: i64 = 8;
const DOTALL: i64 = 16;
const VERBOSE: i64 = 64;

fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
}

impl Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for RegexError {}

impl From<regex::Error> for RegexError {
    fn from(error: regex::Error) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

fn compile_pattern(pattern: &str) -> Result<Regex, RegexError> {
    Ok(Regex::new(pattern)?)
}

fn with_flags(pattern: &str, flags: i64) -> String {
    let mut prefix = String::new();
    if flags & IGNORECASE != 0 {
        prefix.push_str("(?i)");
    }
    if flags & MULTILINE != 0 {
        prefix.push_str("(?m)");
    }
    if flags & DOTALL != 0 {
        prefix.push_str("(?s)");
    }
    if flags & VERBOSE != 0 {
        prefix.push_str("(?x)");
    }
    prefix + pattern
}

fn has_match(pattern: &str, text: &str) -> Result<bool, RegexError> {
    Ok(compile_pattern(pattern)?
        .find(text)
        .is_some_and(|matched| matched.start() == 0))
}

fn search(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
    Ok(compile_pattern(pattern)?
        .find(text)
        .map(|matched| matched.as_str().to_string()))
}

fn sub(pattern: &str, replacement: &str, text: &str) -> Result<String, RegexError> {
    Ok(compile_pattern(pattern)?
        .replace_all(text, replacement)
        .into_owned())
}

fn findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
    Ok(compile_pattern(pattern)?
        .find_iter(text)
        .map(|matched| matched.as_str().to_string())
        .collect())
}

fn split(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
    Ok(compile_pattern(pattern)?
        .split(text)
        .map(str::to_string)
        .collect())
}

fn search_flags(pattern: &str, text: &str, flags: i64) -> Result<Option<String>, RegexError> {
    Ok(compile_pattern(&with_flags(pattern, flags))?
        .find(text)
        .map(|matched| matched.as_str().to_string()))
}

fn collect_primary_actual() -> Vec<bool> {
    match (|| -> Result<Vec<bool>, RegexError> {
        Ok(vec![
            has_match("[0-9]+", "42 bottles")?,
            search("[0-9]+", "id=9000")?.as_deref() == Some("9000"),
            sub("\\s+", "-", "hello   world")? == "hello-world",
            format!("{:?}", findall("[a-z]+", "ab 12 cd")?) == "[\"ab\", \"cd\"]",
            format!("{:?}", split(":+", "a:b::c")?) == "[\"a\", \"b\", \"c\"]",
            search_flags("hello", "HELLO", IGNORECASE)?.is_some(),
        ])
    })() {
        Ok(actual) => actual,
        Err(error) => {
            let _ = error.to_string();
            vec![false; 6]
        }
    }
}

fn main() {
    let expected = [true, true, true, true, true, true];
    let actual = collect_primary_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("regex re parity demo: pass");
}
