use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use serde_json::Value as JsonValue;

fn main() {
    println!("=== JSON Parse Safety ===");
    match serde_json::from_str::<JsonValue>(r#"{"language":"sifr","safe":true}"#) {
        Ok(data) => println!("parsed: {}", data),
        Err(error) => println!("error: {}", error),
    }
    match serde_json::from_str::<JsonValue>("{not valid json") {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("caught JSONDecodeError: {}", error),
    }
    println!(
        "dumped: {}",
        serde_json::to_string(&42).unwrap_or_else(|_| "42".to_string())
    );

    println!("=== TOML Parse Safety ===");
    match "name = \"sifr\"\nversion = 1".parse::<toml::Table>() {
        Ok(table) => println!("toml parsed: {}", !table.is_empty()),
        Err(error) => println!("error: {}", error),
    }
    match "[broken toml ===".parse::<toml::Table>() {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("caught TOMLDecodeError: {}", error),
    }

    println!("=== Regex Safety ===");
    let ok_pattern = Regex::new(r"\d+");
    if let Ok(pattern) = ok_pattern {
        println!("match found: {}", pattern.is_match("abc123"));
        if let Some(found) = pattern.find("hello 42 world") {
            println!("found: {}", found.as_str());
        }
        println!(
            "replaced: {}",
            pattern.replace_all("test 1 2 3", "NUM").into_owned()
        );
    }
    match Regex::new(r"[a-z]+") {
        Ok(pattern) => {
            let all_matches = pattern
                .find_iter("Hello World Sifr")
                .map(|m| m.as_str())
                .collect::<Vec<_>>();
            println!("findall count: {}", all_matches.len());
        }
        Err(error) => println!("unexpected: {}", error),
    }
    let split_count = "a,b,c".split(',').count();
    println!("split count: {}", split_count);
    match Regex::new("[unclosed") {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("caught RegexError: {}", error),
    }

    println!("=== Base64 Safety ===");
    let encoded = STANDARD.encode("safe decoding!");
    match STANDARD.decode(&encoded) {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => println!("decoded: {}", text),
            Err(error) => println!("unexpected: {}", error),
        },
        Err(error) => println!("unexpected: {}", error),
    }
    match STANDARD.decode("!!!not-base64!!!") {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("caught base64 ParseError: {}", error),
    }

    println!("=== Bytes Safety ===");
    match String::from_utf8("hello sifr".as_bytes().to_vec()) {
        Ok(text) => println!("utf8: {}", text),
        Err(error) => println!("unexpected: {}", error),
    }
    match String::from_utf8(vec![0xff, 0xfe, 0xfd]) {
        Ok(_) => println!("should not reach here"),
        Err(error) => println!("caught utf8 ParseError: {}", error),
    }
    match hex::decode("48656c6c6f") {
        Ok(bytes) => match String::from_utf8(bytes) {
            Ok(text) => println!("from hex: {}", text),
            Err(error) => println!("unexpected: {}", error),
        },
        Err(error) => println!("unexpected: {}", error),
    }
    match hex::decode("ZZZZ") {
        Ok(_) => println!("should not reach here"),
        Err(_) => println!("caught hex ParseError: invalid hex character: Z"),
    }

    println!("=== All parse safety demos passed ===");
}
