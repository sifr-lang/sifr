// src/main.rs
use ::sifr_runtime::SifrInt;

fn is_even(x: SifrInt) -> bool {
    (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0))
}

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)];
    let evens: Box<dyn Iterator<Item = SifrInt>> = Box::new(nums.iter().cloned().filter(|__filter_item| {
    let __filter_value = __filter_item.clone();
    is_even(__filter_value)
}));
    println!("{:?}", evens.collect::<Vec<_>>());
    let rev: Box<dyn Iterator<Item = SifrInt>> = Box::new((nums).iter().cloned().rev());
    println!("{:?}", rev.collect::<Vec<_>>());
    let indexed: Box<dyn Iterator<Item = (SifrInt, SifrInt)>> = Box::new((nums).iter().cloned().enumerate().map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(10), __pair.1)));
    println!("{:?}", indexed.collect::<Vec<_>>());
    println!("{}", Box::new((nums).iter().cloned()).sum::<SifrInt>());
    println!("{:?}", {
    let mut __sifr_sorted_v = Box::new((nums).iter().cloned()).collect::<Vec<_>>();
    __sifr_sorted_v.sort();
    if true { {
    __sifr_sorted_v.reverse();
} };
    __sifr_sorted_v
});
    let collected: Vec<SifrInt> = Box::new(nums.iter().cloned().filter(|__filter_item| {
    let __filter_value = __filter_item.clone();
    is_even(__filter_value)
})).collect::<Vec<_>>();
    println!("{:?}", collected);
}
