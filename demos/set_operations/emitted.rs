// src/main.rs
use ::sifr_runtime::SifrInt;
use ::std::collections::HashSet;
fn main() {
    let mut fruits: HashSet<String> = HashSet::from([
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ]);
    println!("{}", SifrInt::from(fruits.len()));
    fruits.insert("date".to_string());
    println!("{}", fruits.contains("date"));
    fruits.remove("banana");
    println!("{}", SifrInt::from(fruits.len()));
    let nums: HashSet<SifrInt> = HashSet::from([
        SifrInt::from_i64(10),
        SifrInt::from_i64(20),
        SifrInt::from_i64(30),
    ]);
    let mut total: SifrInt = SifrInt::from_i64(0);
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for n in nums.iter() {
        total = ::std::ops::Add::add(&total, n);
    }
    println!("{total}");
}
