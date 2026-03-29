fn append_zero(mut values: Vec<i64>) -> Vec<i64> {
    values.push(0);
    values
}

fn append_marker(mut words: Vec<String>) -> Vec<String> {
    words.push("done".to_string());
    words
}

fn main() {
    assert_eq!(format!("{:?}", append_zero(vec![2, 3, 4])), "[2, 3, 4, 0]");
    assert_eq!(
        format!(
            "{:?}",
            append_marker(vec!["compile".to_string(), "check".to_string()])
        ),
        "[\"compile\", \"check\", \"done\"]"
    );
    println!("own_mut_appends: ok");
}
