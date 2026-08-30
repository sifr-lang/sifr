// src/main.rs
use ::sifr_runtime::SifrInt;

fn inc(x: SifrInt) -> SifrInt {
    &x + &SifrInt::from_i64(1)
}

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)];
    println!("{:?}", Box::new(nums.iter().cloned().map(|__sifr_map_item| inc(__sifr_map_item))).collect::<Vec<_>>());
    println!("{:?}", Box::new((nums).iter().cloned().rev()).collect::<Vec<_>>());
    let list_comp: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for x in nums.iter().cloned() {
        __sifr_list_comp.push(x);
    }
    __sifr_list_comp
};
    println!("{:?}", list_comp);
    println!("{:?}", nums.iter().cloned().map(|x| x).collect::<Vec<_>>());
}
