// src/main.rs
fn main() {
    let pairs: Vec<(i64, i64)> = vec![(2_i64, 5_i64), (4_i64, 7_i64)];
    let mut totals: Vec<i64> = vec![];
    for pair in pairs.iter().copied() {
        totals.push(pair.0 + pair.1);
    }
    println!("{:?}", totals);
    let mixed: Vec<Box<dyn ::std::any::Any>> = vec![];
    let mut count: i64 = 0_i64;
    for _value in mixed.iter() {
        count += 1_i64;
    }
    println!("{}", count);
    println!("clone_generic_cloning_hardening_demo: pass");
}
