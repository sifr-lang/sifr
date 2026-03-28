use std::collections::HashMap;

fn main() {
    let mut output: Vec<String> = vec![];
    let s: String = "hello".to_string();
    for c in s.chars().map(|c| c.to_string()) {
        output.push(c);
    }
    let d: HashMap<String, i64> = HashMap::from([("a".to_string(), 1 as i64), ("b".to_string(), 2 as i64)]);
    let keys: Vec<String> = vec!["a".to_string(), "b".to_string()];
    for k in keys.iter().cloned() {
        output.push(k);
    }
    println!("Iteration demo output:");
    for item in output.iter().cloned() {
        println!("{}", item);
    }
    assert!(output == vec!["h".to_string(), "e".to_string(), "l".to_string(), "l".to_string(), "o".to_string(), "a".to_string(), "b".to_string()]);
}
