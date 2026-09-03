// src/main.rs
use ::sifr_runtime::SifrInt;
fn main() {
    let s: String = "hello".to_string();
    println!("{s}");
    assert_eq!(s.to_string(), "hello");
    println!("{s}");
    assert_eq!(s.to_string(), "hello");
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    println!("length: {}", SifrInt::from(nums.len()));
    assert_eq!(
        format!("length: {}", SifrInt::from(nums.len())),
        "length: 5"
    );
    println!("sum: {}", nums.iter().cloned().sum::<SifrInt>());
    assert_eq!(
        format!("sum: {}", nums.iter().cloned().sum::<SifrInt>()),
        "sum: 15"
    );
}
