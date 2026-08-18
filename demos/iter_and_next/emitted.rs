// src/main.rs
fn main() {
    let values: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64];
    let mut it: Box<dyn Iterator<Item = i64>> = Box::new((values).iter().copied());
    let first: Option<i64> = it.next();
    println!("{}", (first).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    let mut remaining_total: i64 = 0_i64;
    for item in it {
        remaining_total += item;
    }
    println!("{}", remaining_total);
    let mut pair_total: i64 = 0_i64;
    for (i, value) in Box::new((values).iter().copied().enumerate().map(|__pair| ((__pair.0 as i64) + 0, __pair.1))) {
        pair_total = (pair_total + i) + value;
    }
    println!("{}", pair_total);
}
