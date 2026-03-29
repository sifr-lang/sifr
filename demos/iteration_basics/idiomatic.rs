use std::collections::HashMap;

fn main() {
    let mut output = Vec::new();

    output.extend("hello".chars().map(|ch| ch.to_string()));

    let _dict = HashMap::from([("a", 1_i64), ("b", 2_i64)]);
    output.extend(["a", "b"].into_iter().map(str::to_string));

    println!("Iteration demo output:");
    for item in &output {
        println!("{item}");
    }

    assert_eq!(output, ["h", "e", "l", "l", "o", "a", "b"]);
}
