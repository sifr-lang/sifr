// src/main.rs
fn double(n: i64) -> i64 {
    n * (2_i64)
}

fn is_even(n: i64) -> bool {
    (n % (2_i64)) == (0_i64)
}

fn main() {
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64];
    let mapped: Vec<i64> = Box::new(nums.iter().copied().map(|__sifr_map_item| double(__sifr_map_item))).collect::<Vec<_>>();
    let filtered: Vec<i64> = Box::new(nums.iter().copied().filter(|__filter_item| {
    let __filter_value = *__filter_item;
    is_even(__filter_value)
})).collect::<Vec<_>>();
    let first: i64 = 0_i64;
    let rest: Vec<i64> = vec![];
    let _star_tmp = &nums;
    let first = _star_tmp[0];
    let rest = _star_tmp[1.._star_tmp.len()].to_vec();
    println!("{}", format!("{:?}", mapped));
    println!("{}", format!("{:?}", filtered));
    println!("{}", format!("{}", first));
    println!("{}", format!("{:?}", rest));
    println!("clone_collection_cloning_lock_demo: pass");
}
