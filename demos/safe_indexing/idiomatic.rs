use std::collections::HashMap;

fn safe_index<T: Copy>(items: &[T], index: i64) -> Option<T> {
    let len = items.len() as i64;
    let normalized = if index < 0 { len + index } else { index };
    if (0..len).contains(&normalized) {
        items.get(normalized as usize).copied()
    } else {
        None
    }
}

fn safe_char_at(text: &str, index: i64) -> Option<String> {
    let chars: Vec<char> = text.chars().collect();
    safe_index(&chars, index).map(|ch| ch.to_string())
}

fn main() {
    println!("=== Safe List Indexing ===");
    let nums = vec![10_i64, 20, 30];
    if let Some(value) = safe_index(&nums, 1) {
        println!("nums[1] = {value}");
    }
    if let Some(value) = safe_index(&nums, 99) {
        println!("nums[99] = {value}");
    } else {
        println!("nums[99] = None (safe!)");
    }

    println!("=== Safe Dict Indexing ===");
    let ages = HashMap::from([("alice".to_string(), 25_i64), ("bob".to_string(), 30)]);
    if let Some(value) = ages.get("alice").copied() {
        println!("ages[alice] = {value}");
    }
    if let Some(value) = ages.get("charlie").copied() {
        println!("ages[charlie] = {value}");
    } else {
        println!("ages[charlie] = None (safe!)");
    }

    println!("=== Safe String Indexing ===");
    let text = "hello";
    if let Some(ch) = safe_char_at(text, 0) {
        println!("s[0] = {ch}");
    }
    if let Some(ch) = safe_char_at(text, 99) {
        println!("s[99] = {ch}");
    } else {
        println!("s[99] = None (safe!)");
    }

    println!("=== Negative Indexing ===");
    let items = vec![1_i64, 2, 3, 4, 5];
    if let Some(last) = safe_index(&items, -1) {
        println!("last item: {last}");
    }
    if let Some(last) = safe_char_at(text, -1) {
        println!("last char: {last}");
    }

    println!("=== List pop() ===");
    let mut stack = vec![10_i64, 20, 30];
    if let Some(top) = stack.pop() {
        println!("popped: {top}");
    }
    let _ = stack.pop();
    let _ = stack.pop();
    if let Some(value) = stack.pop() {
        println!("got: {value}");
    } else {
        println!("empty pop: None");
    }

    println!("=== Dict get() and pop() ===");
    let mut data = HashMap::from([("alice".to_string(), 25_i64), ("bob".to_string(), 30)]);
    if let Some(value) = data.get("alice").copied() {
        println!("get alice: {value}");
    }
    if let Some(value) = data.get("missing").copied() {
        println!("get missing: {value}");
    } else {
        println!("get missing: None");
    }
    if let Some(value) = data.remove("bob") {
        println!("popped bob: {value}");
    }

    println!("=== String find() ===");
    let phrase = "hello, world!";
    if let Some(pos) = phrase.find("world") {
        println!("found world at: {pos}");
    }
    if let Some(pos) = phrase.find("xyz") {
        println!("found xyz at: {pos}");
    } else {
        println!("xyz not found");
    }

    println!("=== Del Statement ===");
    let mut config = HashMap::from([
        ("a".to_string(), 1_i64),
        ("b".to_string(), 2),
        ("c".to_string(), 3),
    ]);
    println!("before del: {}", config.len());
    let _ = config.remove("b");
    println!("after del: {}", config.len());

    println!("demo complete!");
}
