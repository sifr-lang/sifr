// src/main.rs
fn shadow_parameter(mut value: i64) -> i64 {
    value += 2_i64;
    value
}

fn choose_label(flag: bool) -> String {
    let mut label: String = "cold".to_string();
    if flag {
        label = "warm".to_string();
    }
    label
}

fn main() {
    assert!((shadow_parameter(5_i64) == (7_i64)));
    assert!((choose_label(true) == "warm"));
    assert!((choose_label(false) == "cold"));
    println!("local_shadowing: ok");
}
