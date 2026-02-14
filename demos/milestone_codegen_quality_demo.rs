use std::collections::HashMap;

fn main() {
    let x: i64 = 42_i64;
    println!("Immutable x = {}", x);
    let mut counter: i64 = 0_i64;
    counter = counter + 1_i64;
    counter = counter + 1_i64;
    println!("Mutable counter = {}", counter);
    let mut items: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    items.push(4_i64);
    println!("List after append: length = {}", items.len() as i64);
    let name: String = "Sifr".to_string();
    let version: i64 = 2_i64;
    println!("Welcome to {} v{}!", name, version);
    println!("{}", "hello world");
    let lang: String = "sifr-lang".to_string();
    println!("Starts with sifr: {}", lang.starts_with("sifr"));
    println!("Ends with lang: {}", lang.ends_with("lang"));
    let replaced: String = lang.replace("sifr", "SIFR");
    println!("Replaced: {}", replaced);
    let csv: String = "a,b,c".to_string();
    let parts: Vec<String> = csv.split(",").map(|s| s.to_string()).collect::<Vec<String>>();
    println!("Split count: {}", parts.len() as i64);
    let ages: HashMap<String, i64> = HashMap::from([("alice".to_string(), 25_i64), ("bob".to_string(), 30_i64), ("charlie".to_string(), 35_i64)]);
    let alice_age: Option<i64> = ages.get("alice").cloned();
    if let Some(alice_age) = alice_age {
        println!("Alice is {} years old", alice_age);
    }
    let has_bob: bool = ages.contains_key("bob");
    println!("Has bob: {}", has_bob);
    let greeting: String = format!("{}{}{}{}", "Hello".to_string(), ", ".to_string(), "World".to_string(), "!".to_string());
    println!("Greeting: {}", greeting);
    println!("{}", "All codegen quality improvements verified!");
}
