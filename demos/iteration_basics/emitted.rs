// src/main.rs
use ::std::collections::HashMap;

static __SIFR_HOISTED_DICT_0: ::std::sync::LazyLock<HashMap<String, i64>> = ::std::sync::LazyLock::new(|| HashMap::from([("a".to_string(), 1_i64), ("b".to_string(), 2_i64)]));

fn main() {
    let mut output: Vec<String> = vec![];
    let s: String = "hello".to_string();
    for c in s.chars().map(|c| c.to_string()) {
        output.push(c.clone());
    }
    let d = &*__SIFR_HOISTED_DICT_0;
    let keys: Vec<String> = vec!["a".to_string(), "b".to_string()];
    for k in keys.iter().cloned() {
        output.push(k.clone());
    }
    println!("Iteration demo output:");
    for item in output.iter().cloned() {
        println!("{}", item);
    }
    assert!(output == vec!["h".to_string(), "e".to_string(), "l".to_string(), "l".to_string(), "o".to_string(), "a".to_string(), "b".to_string()]);
}
