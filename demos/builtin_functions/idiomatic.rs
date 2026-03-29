fn main() {
    let max_value = 3_i64.max(7);
    let min_value = 3_i64.min(7);
    let power = 2_i64.pow(10);
    let result = (0_i64..10)
        .step_by(2)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(" ");

    println!("max(3, 7) = {max_value}");
    assert_eq!(format!("max(3, 7) = {max_value}"), "max(3, 7) = 7");

    println!("min(3, 7) = {min_value}");
    assert_eq!(format!("min(3, 7) = {min_value}"), "min(3, 7) = 3");

    println!("pow(2, 10) = {power}");
    assert_eq!(format!("pow(2, 10) = {power}"), "pow(2, 10) = 1024");

    println!("{result}");
    assert_eq!(result, "0 2 4 6 8");
}
