fn main() {
    let mut acc: i64 = 0 as i64;
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64];
    let mut _broke = false;
    for n in nums.iter().copied() {
        acc += n;
    }
    if !_broke {
        acc += 1 as i64;
    }
    let mut i: i64 = 0 as i64;
    let mut _broke = false;
    while i < (3 as i64) {
        acc += i;
        i += 1 as i64;
    }
    if !_broke {
        acc += 2 as i64;
    }
    let ready: bool = true;
    if ready {
        acc += 10 as i64;
    } else {
        acc += 100 as i64;
    }
    assert!(acc > (0 as i64));
    println!("acc = {}", acc);
    assert!(format!("{}", format!("acc = {}", acc)) == "acc = 22".to_string());
}
