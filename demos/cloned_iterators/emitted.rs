// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(2), SifrInt::from_i64(4), SifrInt::from_i64(6), SifrInt::from_i64(8)];
    let doubled: Vec<SifrInt> = Box::new(nums.iter().cloned().map(|__sifr_map_item| (|x| &x * &SifrInt::from_i64(2))(__sifr_map_item))).collect::<Vec<_>>();
    let evens: Vec<SifrInt> = Box::new((nums).iter().cloned().filter(move |__filter_item| {
    let x = __filter_item.clone();
    (&x.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == &SifrInt::from_i64(0))
})).collect::<Vec<_>>();
    let comp: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for x in nums.iter().cloned() {
        __sifr_list_comp.push(&x + &SifrInt::from_i64(1));
    }
    __sifr_list_comp
};
    println!("{:?}", doubled);
    println!("{:?}", evens);
    println!("{:?}", comp);
    println!("{}", SifrInt::from(nums.len()));
    for n in nums.iter().cloned() {
        println!("{}", n);
    }
    println!("{:?}", Box::new(vec![SifrInt::from_i64(9), SifrInt::from_i64(10), SifrInt::from_i64(11)].into_iter().map(|__sifr_map_item| (|x| &x - &SifrInt::from_i64(1))(__sifr_map_item))).collect::<Vec<_>>());
    println!("clone_cloned_iterators_comprehension_demo: pass");
}
