fn main() {
    let s: String = "hello".to_string();
    println!("{}", s);
    assert!(format!("{}", s) == "hello".to_string());
    println!("{}", s);
    assert!(format!("{}", s) == "hello".to_string());
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64, 5 as i64];
    println!("length: {}", nums.len() as i64);
    assert!(format!("{}", format!("length: {}", nums.len() as i64)) == "length: 5".to_string());
    println!("sum: {}", (nums).iter().copied().sum::<i64>());
    assert!(
        format!(
            "{}",
            format!("sum: {}", (nums).iter().copied().sum::<i64>())
        ) == "sum: 15".to_string()
    );
}
