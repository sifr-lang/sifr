use std::collections::HashSet;

fn main() {
    let mut fruits: HashSet<String> = HashSet::from([
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ]);
    println!("{}", fruits.len());
    fruits.insert("date".to_string());
    println!("{}", fruits.contains("date"));
    fruits.remove("banana");
    println!("{}", fruits.len());
    let nums: HashSet<i64> = HashSet::from([10, 20, 30]);
    let total: i64 = nums.iter().copied().sum();
    println!("{}", total);
}
