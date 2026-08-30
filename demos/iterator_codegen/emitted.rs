// src/main.rs
use ::sifr_runtime::SifrInt;

fn greater_than_two(x: SifrInt) -> bool {
    &x > &SifrInt::from_i64(2)
}

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(5), SifrInt::from_i64(1), SifrInt::from_i64(3), SifrInt::from_i64(4)];
    let flags: Vec<bool> = vec![false, true, false];
    println!("{}", Box::new((flags).iter().copied()).any(|x| x));
    println!("{:?}", Box::new(nums.iter().cloned().filter(|__filter_item| {
    let __filter_value = __filter_item.clone();
    greater_than_two(__filter_value)
})).collect::<Vec<_>>());
    println!("{:?}", {
    let mut __sifr_sorted_v = Box::new((nums).iter().cloned()).collect::<Vec<_>>();
    __sifr_sorted_v.sort();
    __sifr_sorted_v
});
}
