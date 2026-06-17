fn main() {
    let pairs: Vec<(i64, i64)> = vec![(2 as i64, 5 as i64), (4 as i64, 7 as i64)];
    let mut totals: Vec<i64> = vec![];
    for pair in pairs.iter().copied() {
        totals.push(pair.0 + pair.1);
    }
    println!("{:?}", totals);
    let mixed: Vec<Box<dyn std::any::Any>> = vec![];
    let mut count: i64 = 0 as i64;
    for _value in mixed.iter() {
        count = count + (1 as i64);
    }
    println!("{}", count);
    println!("clone_generic_cloning_hardening_demo: pass");
}
