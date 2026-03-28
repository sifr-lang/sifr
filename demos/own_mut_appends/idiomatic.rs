fn append_zero(mut values: Vec<i64>) -> Vec<i64> {
    values.push(0 as i64);
    return values;
}

fn append_marker(mut words: Vec<String>) -> Vec<String> {
    words.push("done".to_string());
    return words;
}

fn main() {
    assert!(
        format!("{:?}", append_zero(vec![2 as i64, 3 as i64, 4 as i64]))
            == "[2, 3, 4, 0]".to_string()
    );
    assert!(
        format!(
            "{:?}",
            append_marker(vec!["compile".to_string(), "check".to_string()])
        ) == "[\"compile\", \"check\", \"done\"]".to_string()
    );
    println!("own_mut_appends: ok");
}
