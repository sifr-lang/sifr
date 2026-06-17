fn main() {
    let pairs = [(2_i64, 5_i64), (4_i64, 7_i64)];
    let totals: Vec<i64> = pairs
        .into_iter()
        .map(|(left, right)| left + right)
        .collect();
    println!("{totals:?}");

    let mixed: Vec<Box<dyn std::any::Any>> = Vec::new();
    println!("{}", mixed.len() as i64);

    println!("clone_generic_cloning_hardening_demo: pass");
}
