use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine;
use regex::Regex;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
}

impl RegexError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

fn time() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or(0.0)
}

fn sleep(seconds: f64) {
    thread::sleep(Duration::from_secs_f64(seconds.max(0.0)));
}

fn random_state() -> &'static AtomicU64 {
    static STATE: AtomicU64 = AtomicU64::new(0);
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0x9e37_79b9_7f4a_7c15);
    let _ = STATE.compare_exchange(0, seed | 1, Ordering::SeqCst, Ordering::SeqCst);
    &STATE
}

fn next_random_u64() -> u64 {
    let state = random_state();
    let mut current = state.load(Ordering::Relaxed);
    loop {
        let mut next = current;
        next ^= next << 13;
        next ^= next >> 7;
        next ^= next << 17;
        match state.compare_exchange(current, next, Ordering::SeqCst, Ordering::SeqCst) {
            Ok(_) => return next,
            Err(observed) => current = observed,
        }
    }
}

fn randint(low: i64, high: i64) -> Result<i64, ValueError> {
    if low > high {
        return Err(ValueError::new("randint: min must be <= max"));
    }
    let span = (high - low + 1) as u64;
    Ok(low + (next_random_u64() % span) as i64)
}

fn random() -> f64 {
    (next_random_u64() as f64) / (u64::MAX as f64 + 1.0)
}

fn compile_regex(pattern: &str) -> Result<Regex, RegexError> {
    Regex::new(pattern).map_err(|error| RegexError::new(error.to_string()))
}

fn has_match(pattern: &str, text: &str) -> Result<bool, RegexError> {
    Ok(compile_regex(pattern)?.is_match(text))
}

fn search(pattern: &str, text: &str) -> Result<Option<String>, RegexError> {
    Ok(compile_regex(pattern)?
        .find(text)
        .map(|matched| matched.as_str().to_string()))
}

fn sub(pattern: &str, replacement: &str, text: &str) -> Result<String, RegexError> {
    Ok(compile_regex(pattern)?
        .replace_all(text, replacement)
        .into_owned())
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

fn md5_hash(text: &str) -> String {
    md5::compute(text.as_bytes())
        .0
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join("")
}

fn b64encode(text: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(text.as_bytes())
}

fn b64decode(text: &str) -> Result<String, ParseError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(text)
        .map_err(|error| ParseError::new(error.to_string()))?;
    String::from_utf8(bytes).map_err(|error| ParseError::new(error.to_string()))
}

fn main() {
    println!("=== sifr.time ===");
    let t1 = time();
    println!("Current epoch: {t1}");
    sleep(0.01);
    let t2 = time();
    println!("Time advanced: {}", t2 > t1);

    println!("=== sifr.random ===");
    match randint(1, 100) {
        Ok(r) => println!("Random int [1,100]: {r}"),
        Err(error) => println!("Random integer error: {}", error.message),
    }
    let f = random();
    println!("Random float [0,1): {f}");

    println!("=== sifr.re ===");
    match has_match("[0-9]+", "hello 42") {
        Ok(matched) => {
            println!("Match digits in 'hello 42': {matched}");
            if let Ok(Some(found)) = search("[0-9]+", "price is $42.99") {
                println!("Found: {found}");
            }
            match sub("[0-9]+", "N", "a1b2c3") {
                Ok(replaced) => println!("Replace: {replaced}"),
                Err(error) => println!("regex error: {}", error.message),
            }
        }
        Err(error) => println!("regex error: {}", error.message),
    }

    println!("=== sifr.hashlib ===");
    println!("SHA-256('sifr'): {}", sha256("sifr"));
    println!("MD5('sifr'): {}", md5_hash("sifr"));

    println!("=== sifr.base64 ===");
    let encoded = b64encode("Hello, Sifr!");
    println!("Base64 encode: {encoded}");
    match b64decode(&encoded) {
        Ok(decoded) => println!("Base64 decode: {decoded}"),
        Err(error) => println!("base64 error: {}", error.message),
    }

    println!("=== Demo complete ===");
}
