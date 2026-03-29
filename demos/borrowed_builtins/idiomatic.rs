fn main() {
    let text = "hello".to_string();
    println!("{text}");
    assert_eq!(text.to_string(), "hello");
    println!("{text}");
    assert_eq!(text.to_string(), "hello");

    let nums = [1_i64, 2, 3, 4, 5];
    let length_line = format!("length: {}", nums.len());
    println!("{length_line}");
    assert_eq!(length_line, "length: 5");

    let sum_line = format!("sum: {}", nums.iter().copied().sum::<i64>());
    println!("{sum_line}");
    assert_eq!(sum_line, "sum: 15");
}
