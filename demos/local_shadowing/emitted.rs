fn shadow_parameter(mut value: i64) -> i64 {
    value = value + (2 as i64);
    return value;
}

fn choose_label(flag: bool) -> String {
    let mut label: String = "cold".to_string();
    if flag {
        label = "warm".to_string();
    }
    return label;
}

fn main() {
    assert!(shadow_parameter(5 as i64) == (7 as i64));
    assert!(choose_label(true) == "warm".to_string());
    assert!(choose_label(false) == "cold".to_string());
    println!("local_shadowing: ok");
}
