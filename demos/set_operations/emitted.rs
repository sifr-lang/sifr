// src/main.rs
use ::std::collections::HashSet;

fn main() {
    let mut fruits: HashSet<String> = HashSet::from(["apple".to_string(), "banana".to_string(), "cherry".to_string()]);
    println!("{}", fruits.len() as i64);
    fruits.insert("date".to_string());
    println!("{}", fruits.contains(&"date".to_string()));
    fruits.remove(&"banana".to_string());
    println!("{}", fruits.len() as i64);
    let nums: HashSet<i64> = HashSet::from([10_i64, 20_i64, 30_i64]);
    let mut total: i64 = 0_i64;
    for n in nums.iter().copied() {
        total += n;
    }
    println!("{}", total);
}
