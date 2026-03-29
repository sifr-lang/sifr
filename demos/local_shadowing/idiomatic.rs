fn shadow_parameter(mut value: i64) -> i64 {
    value += 2;
    value
}

fn choose_label(flag: bool) -> String {
    if flag { "warm" } else { "cold" }.to_string()
}

fn main() {
    assert_eq!(shadow_parameter(5), 7);
    assert_eq!(choose_label(true), "warm");
    assert_eq!(choose_label(false), "cold");
    println!("local_shadowing: ok");
}
