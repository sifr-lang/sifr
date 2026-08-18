// src/main.rs
fn greater_than_two(x: i64) -> bool {
    x > (2_i64)
}

fn main() {
    let nums: Vec<i64> = vec![5_i64, 1_i64, 3_i64, 4_i64];
    let flags: Vec<bool> = vec![false, true, false];
    println!("{}", Box::new((flags).iter().copied()).any(|x| x));
    println!("{:?}", Box::new(nums.iter().copied().filter(|__filter_item| {
    let __filter_value = *__filter_item;
    greater_than_two(__filter_value)
})).collect::<Vec<_>>());
    println!("{:?}", {
    let mut __sifr_sorted_v = Box::new((nums).iter().copied()).collect::<Vec<_>>();
    __sifr_sorted_v.sort();
    __sifr_sorted_v
});
}
