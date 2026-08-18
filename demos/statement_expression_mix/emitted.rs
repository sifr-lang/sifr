// src/main.rs
fn main() {
    let mut acc: i64 = 0_i64;
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    let _broke = false;
    for n in nums.iter().copied() {
        acc += n;
    }
    if !_broke {
        acc += 1_i64;
    }
    let mut i: i64 = 0_i64;
    let _broke = false;
    while i < (3_i64) {
        acc += i;
        i += 1_i64;
    }
    if !_broke {
        acc += 2_i64;
    }
    let ready: bool = true;
    if ready {
        acc += 10_i64;
    } else {
        acc += 100_i64;
    }
    assert!(acc > (0_i64));
    println!("acc = {}", acc);
    assert!((format!("{}", format!("acc = {}", acc)) == "acc = 22"));
}
