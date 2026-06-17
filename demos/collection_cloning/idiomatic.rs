fn double(n: i64) -> i64 {
    n * 2
}

fn is_even(n: i64) -> bool {
    n % 2 == 0
}

fn main() {
    let nums = [1_i64, 2, 3, 4];

    let mapped: Vec<i64> = nums.iter().copied().map(double).collect();
    let filtered: Vec<i64> = nums.iter().copied().filter(|n| is_even(*n)).collect();

    let first = nums[0];
    let rest = nums[1..].to_vec();

    println!("{mapped:?}");
    println!("{filtered:?}");
    println!("{first}");
    println!("{rest:?}");
    println!("clone_collection_cloning_lock_demo: pass");
}
