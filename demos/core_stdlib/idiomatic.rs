use std::collections::BTreeMap;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use serde_json::Value;

#[derive(Debug, Clone)]
struct IOError {
    message: String,
}

impl IOError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl From<std::io::Error> for IOError {
    fn from(error: std::io::Error) -> Self {
        Self::new(error.to_string())
    }
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
}

impl JSONDecodeError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone)]
struct JsonValue(Value);

impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(&self.0).unwrap_or_else(|_| "null".to_string())
        )
    }
}

fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    std::fs::write(path, content).map_err(IOError::from)
}

fn exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

fn read_text(path: &str) -> Result<String, IOError> {
    std::fs::read_to_string(path).map_err(IOError::from)
}

fn read_lines(path: &str) -> Result<Vec<String>, IOError> {
    Ok(read_text(path)?
        .lines()
        .map(|line| line.to_string())
        .collect::<Vec<_>>())
}

fn json_loads(text: &str) -> Result<JsonValue, JSONDecodeError> {
    serde_json::from_str(text)
        .map(JsonValue)
        .map_err(|error| JSONDecodeError::new(error.to_string()))
}

fn env_store() -> &'static Mutex<BTreeMap<String, String>> {
    static STORE: OnceLock<Mutex<BTreeMap<String, String>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn setenv(key: &str, value: &str) {
    env_store()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .insert(key.to_string(), value.to_string());
}

fn getenv_opt(key: &str) -> Option<String> {
    env_store()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .get(key)
        .cloned()
}

fn run_command(command: &str) -> Result<String, IOError> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(IOError::from)?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end_matches(['\r', '\n'])
        .to_string())
}

fn sqrt(value: f64) -> f64 {
    value.sqrt()
}

fn floor(value: f64) -> i64 {
    value.floor() as i64
}

fn ceil(value: f64) -> i64 {
    value.ceil() as i64
}

fn main() {
    println!("=== sifr.io ===");
    if let Err(error) = write_text("/tmp/sifr_demo.txt", "Hello from Sifr!\nLine 2") {
        println!("write error: {}", error.message);
    }

    let file_exists = exists("/tmp/sifr_demo.txt");
    println!("File exists: {file_exists}");

    match read_text("/tmp/sifr_demo.txt") {
        Ok(content) => println!("Content: {content}"),
        Err(error) => println!("read error: {}", error.message),
    }

    match read_lines("/tmp/sifr_demo.txt") {
        Ok(lines) => println!("Line count: {}", lines.len()),
        Err(error) => println!("lines error: {}", error.message),
    }

    println!("=== sifr.json ===");
    match json_loads("{\"language\":\"sifr\",\"version\":1}") {
        Ok(data) => println!("Parsed JSON: {data}"),
        Err(error) => println!("json error: {}", error.message),
    }

    println!("=== sifr.env ===");
    setenv("SIFR_DEMO", "active");
    if let Some(value) = getenv_opt("SIFR_DEMO") {
        println!("SIFR_DEMO = {value}");
    }
    if getenv_opt("SIFR_NONEXISTENT").is_none() {
        println!("SIFR_NONEXISTENT not set");
    }

    println!("=== sifr.os ===");
    match run_command("echo Sifr OS module works") {
        Ok(output) => println!("{output}"),
        Err(error) => println!("os error: {}", error.message),
    }

    println!("=== sifr.math ===");
    println!("sqrt(25.0) = {}", sqrt(25.0));
    println!("floor(3.7) = {}", floor(3.7));
    println!("ceil(3.2) = {}", ceil(3.2));
    println!("pi = {}", std::f64::consts::PI);
    println!("e = {}", std::f64::consts::E);

    println!("=== Demo complete ===");
}
