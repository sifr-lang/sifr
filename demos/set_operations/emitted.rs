// src/main.rs
use ::std::collections::HashSet;

use ::sifr_runtime::SifrInt;

fn main() {
    let mut fruits: HashSet<String> = HashSet::from(["apple".to_string(), "banana".to_string(), "cherry".to_string()]);
    println!("{}", SifrInt::from(fruits.len()));
    fruits.insert("date".to_string());
    println!("{}", fruits.contains(&"date".to_string()));
    fruits.remove(&"banana".to_string());
    println!("{}", SifrInt::from(fruits.len()));
    let nums: HashSet<SifrInt> = HashSet::from([SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30)]);
    let mut total: SifrInt = SifrInt::from_i64(0);
    for n in nums.iter().cloned() {
        total = &total + &n;
    }
    println!("{}", total);
}
