// src/main.rs
use ::sifr_runtime::SifrInt;

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)];
    let doubled: Vec<SifrInt> = Box::new(nums.iter().cloned().map(|__sifr_map_item| (|x| &x * &SifrInt::from_i64(2))(__sifr_map_item))).collect::<Vec<_>>();
    println!("{:?}", doubled);
    let evens: Vec<SifrInt> = Box::new((nums).iter().cloned().filter(move |__filter_item| {
    let x = __filter_item.clone();
    (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0))
})).collect::<Vec<_>>();
    println!("{:?}", evens);
    let squares: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for x in nums.iter().cloned() {
        __sifr_list_comp.push(&x * &x);
    }
    __sifr_list_comp
};
    println!("{:?}", squares);
    let big_squares: Vec<SifrInt> = {
    let mut __sifr_list_comp = vec![];
    for x in nums.iter().cloned() {
        if &x > &SifrInt::from_i64(2) {
            __sifr_list_comp.push(&x * &x);
        }
    }
    __sifr_list_comp
};
    println!("{:?}", big_squares);
    let lo: Option<SifrInt> = (nums).iter().cloned().min();
    let hi: Option<SifrInt> = (nums).iter().cloned().max();
    if let Some(lo) = lo.clone() {
        println!("{}", lo);
    }
    if let Some(hi) = hi.clone() {
        println!("{}", hi);
    }
    println!("{}", (nums).iter().cloned().sum::<SifrInt>());
    let unsorted: Vec<SifrInt> = vec![SifrInt::from_i64(5), SifrInt::from_i64(3), SifrInt::from_i64(1), SifrInt::from_i64(4), SifrInt::from_i64(2)];
    println!("{:?}", {
    let mut __sifr_sorted_v = (unsorted).iter().cloned().collect::<Vec<_>>();
    __sifr_sorted_v.sort();
    __sifr_sorted_v
});
    println!("{:?}", Box::new((unsorted).iter().cloned().rev()).collect::<Vec<_>>());
    let letters: Vec<String> = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    println!("{:?}", Box::new((letters).iter().cloned().enumerate().map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(0), __pair.1))).collect::<Vec<_>>());
    let names: Vec<String> = vec!["Alice".to_string(), "Bob".to_string()];
    let ages: Vec<SifrInt> = vec![SifrInt::from_i64(30), SifrInt::from_i64(25)];
    println!("{:?}", Box::new((names).iter().cloned().zip((ages).iter().cloned()).map(|__zip_item| (__zip_item.0, __zip_item.1))).collect::<Vec<_>>());
    let bools: Vec<bool> = vec![true, false, true];
    println!("{}", (bools).iter().copied().any(|x| x));
    println!("{}", (bools).iter().copied().all(|x| x));
    println!("{}", (vec![true, true, true]).into_iter().all(|x| x));
}
