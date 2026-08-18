// src/main.rs
fn main() {
    let s: String = "hello".to_string();
    println!("{}", s);
    assert!((format!("{}", s) == "hello"));
    println!("{}", s);
    assert!((format!("{}", s) == "hello"));
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    println!("length: {}", nums.len() as i64);
    assert!((format!("{}", format!("length: {}", nums.len() as i64)) == "length: 5"));
    println!("sum: {}", (nums).iter().copied().sum::<i64>());
    assert!((format!("{}", format!("sum: {}", (nums).iter().copied().sum::<i64>())) == "sum: 15"));
}
