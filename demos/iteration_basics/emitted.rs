// src/main.rs
fn main() {
    let mut output: Vec<String> = Vec::new();
    let s: String = "hello".to_string();
    for c in s.chars().map(|c| c.to_string()) {
        output.push(c.to_owned());
    }
    let keys: Vec<String> = vec!["a".to_string(), "b".to_string()];
    for k in keys.iter().cloned() {
        output.push(k.to_owned());
    }
    println!("Iteration demo output:");
    for item in output.iter().cloned() {
        println!("{item}");
    }
    assert_eq!(
        output,
        vec![
            "h".to_string(),
            "e".to_string(),
            "l".to_string(),
            "l".to_string(),
            "o".to_string(),
            "a".to_string(),
            "b".to_string()
        ]
    );
}
