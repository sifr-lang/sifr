// src/main.rs
fn append_zero(mut values: Vec<i64>) -> Vec<i64> {
    values.push(0_i64);
    values
}

fn append_marker(mut words: Vec<String>) -> Vec<String> {
    words.push("done".to_string());
    words
}

fn main() {
    assert!((format!("{:?}", append_zero(vec![2_i64, 3_i64, 4_i64])) == "[2, 3, 4, 0]"));
    assert!((format!("{:?}", append_marker(vec!["compile".to_string(), "check".to_string()])) == "[\"compile\", \"check\", \"done\"]"));
    println!("own_mut_appends: ok");
}
