use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Digest, Sha256};

const PI: f64 = std::f64::consts::PI;

#[derive(Debug)]
struct ParseError(String);

fn assert_eq_value<T: PartialEq + std::fmt::Debug>(actual: T, expected: T) {
    assert_eq!(actual, expected);
}

fn assert_true(value: bool) {
    assert!(value);
}

fn sqrt(value: f64) -> f64 {
    value.sqrt()
}

fn sha256(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn b64encode(text: &str) -> String {
    STANDARD.encode(text.as_bytes())
}

fn b64decode(text: &str) -> Result<String, ParseError> {
    let bytes = STANDARD
        .decode(text.as_bytes())
        .map_err(|err| ParseError(err.to_string()))?;
    String::from_utf8(bytes).map_err(|err| ParseError(err.to_string()))
}

fn main() {
    assert_eq_value(1 + 1, 2);
    assert_true(true);

    let result = sqrt(9.0);
    assert_true(result == 3.0);
    assert_true(PI > 3.14);

    let digest = sha256("hello");
    assert_true(digest.len() == 64);

    let encoded = b64encode("Hello!");
    match b64decode(&encoded) {
        Ok(decoded) => assert_eq_value(decoded, "Hello!".to_string()),
        Err(err) => {
            panic!("unexpected base64 error: {}", err.0);
        }
    }

    println!("stdlib_migration demo: all checks passed!");
}
