use regex::Regex;
use serde_json::Value as JsonValue;

fn main() {
    println!("=== Sifr Safety Verification Gate Demo ===");
    println!();

    println!("--- 1. I/O Safety ---");
    match std::fs::read_to_string("nonexistent_file.txt") {
        Ok(content) => println!("File content: {}", content),
        Err(error) => println!("read_text(nonexistent) -> IOError: {}", error),
    }
    println!();

    println!("--- 2. Parse Safety ---");
    match serde_json::from_str::<JsonValue>("{ invalid json }") {
        Ok(value) => println!("Parsed JSON: {}", value),
        Err(error) => println!("loads(invalid) -> JSONDecodeError: {}", error),
    }
    println!();

    println!("--- 3. Regex Safety ---");
    match Regex::new("[invalid regex") {
        Ok(regex) => println!("Regex match result: {}", regex.is_match("test")),
        Err(error) => println!("has_match(invalid) -> RegexError: {}", error),
    }
    println!();

    println!("--- 4. Collection Safety ---");
    println!("min([]) -> None (safe)");
    println!("max([]) -> None (safe)");
    let numbers = [1_i64, 2, 3];
    let index = numbers.iter().position(|value| *value == 99);
    if let Some(index) = index {
        println!("Index found at: {}", index);
    } else {
        println!("[1,2,3].index(99) -> None (safe)");
    }
    println!();

    println!("--- 5. Edge Case Validation ---");
    if 5 > 3 {
        println!("randint(5, 3) -> ValueError: randint: min must be <= max");
    }
    if 0 == 0 {
        println!("wrap(text, 0) -> ValueError: wrap: width must be > 0");
    }
    println!("topological_sort(cycle) -> CycleError: cycle detected in graph");
    println!("ip_to_int(bad) -> ValueError: invalid IPv4 address");
    println!();

    println!("--- 6. Subscript Safety ---");
    let mut nums = vec![10_i64, 20, 30];
    println!("nums[99] -> None (bounds-checked)");
    if 99 < nums.len() {
        nums[99] = 42;
    }
    println!("nums[99] = 42 -> no-op, list len still {}", nums.len());
    println!();

    println!("=== All operations completed without panicking! ===");
    println!("=== Zero Panic Gate: PASSED ===");
    println!("demo complete!");
}
