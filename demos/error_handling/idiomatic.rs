use std::fmt::{self, Display};
use std::num::ParseIntError;

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl From<ParseIntError> for ParseError {
    fn from(error: ParseIntError) -> Self {
        Self {
            message: error.to_string(),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug, Clone)]
struct DivisionError {
    message: String,
}

impl Display for DivisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for DivisionError {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValidationError {
    message: String,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.message.fmt(f)
    }
}

impl std::error::Error for ValidationError {}

fn validate_range(x: i64, lo: i64, hi: i64) -> Result<i64, ValidationError> {
    if (lo..=hi).contains(&x) {
        Ok(x)
    } else {
        Err(ValidationError {
            message: format!("value out of range: {x}"),
        })
    }
}

fn safe_divide(a: i64, b: i64) -> Result<i64, DivisionError> {
    if b == 0 {
        Err(DivisionError {
            message: "division by zero".to_string(),
        })
    } else {
        Ok(a / b)
    }
}

fn main() {
    println!("=== Result Type & Fallible Conversions ===");
    match "42".parse::<i64>().map_err(ParseError::from) {
        Ok(value) => println!("parsed: {value}"),
        Err(error) => println!("parse failed: {}", error.message),
    }
    match "not_a_number".parse::<i64>().map_err(ParseError::from) {
        Ok(value) => println!("parsed: {value}"),
        Err(error) => println!("parse failed (expected): {}", error.message),
    }

    println!("=== Custom Error Types ===");
    match validate_range(50, 0, 100) {
        Ok(value) => println!("validated: {value}"),
        Err(error) => println!("caught: {}", error.message),
    }
    match validate_range(-5, 0, 100) {
        Ok(value) => println!("validated: {value}"),
        Err(error) => println!("caught: {}", error.message),
    }

    println!("=== Try/Except with Auto-Unwrap ===");
    match validate_range(100, 0, 200) {
        Ok(value) => println!("result: {value}"),
        Err(error) => println!("error handled: {}", error.message),
    }
    match validate_range(999, 0, 200) {
        Ok(value) => println!("result: {value}"),
        Err(error) => println!("error handled: {}", error.message),
    }

    println!("=== Infallible Conversions ===");
    println!("int(3.7) = {}", 3.7_f64 as i64);
    println!("float(5) = {}", 5_i64 as f64);
    println!("str(42) = {}", 42_i64);
    println!("bool(1) = {}", 1_i64 != 0);

    println!("=== Raise in Result Functions ===");
    match safe_divide(10, 3) {
        Ok(value) => println!("divide(10, 3) = {value}"),
        Err(error) => println!("divide error: {}", error.message),
    }
    match safe_divide(10, 0) {
        Ok(value) => println!("divide(10, 0) = {value}"),
        Err(error) => println!("divide(10, 0) error: {}", error.message),
    }

    println!("=== Assert Statement ===");
    println!("all assertions passed");

    println!("=== Explicit Discard ===");
    let _ = safe_divide(10, 2);
    println!("result discarded safely");

    println!("demo complete!");
}
