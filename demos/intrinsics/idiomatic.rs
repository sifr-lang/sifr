fn assert_eq_value<T: PartialEq + std::fmt::Debug>(actual: T, expected: T) {
    assert_eq!(actual, expected);
}

fn assert_true(value: bool) {
    assert!(value);
}

fn main() {
    assert_eq_value(1 + 1, 2);
    assert_eq_value("hello".to_string() + " world", "hello world".to_string());
    assert_true(true);

    let result = 16.0_f64.sqrt();
    assert_true(result == 4.0);
    assert_true(std::f64::consts::PI > 3.14);

    let line = "intrinsics demo: all checks passed!".to_string();
    println!("{line}");
    assert_eq_value(
        line,
        "intrinsics demo: all checks passed!".to_string(),
    );
}
