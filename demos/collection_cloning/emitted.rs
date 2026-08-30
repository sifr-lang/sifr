// src/main.rs
use ::sifr_runtime::SifrInt;

fn double(n: SifrInt) -> SifrInt {
    &n * &SifrInt::from_i64(2)
}

fn is_even(n: SifrInt) -> bool {
    (&n.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0))
}

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)];
    let mapped: Vec<SifrInt> = Box::new(nums.iter().cloned().map(|__sifr_map_item| double(__sifr_map_item))).collect::<Vec<_>>();
    let filtered: Vec<SifrInt> = Box::new(nums.iter().cloned().filter(|__filter_item| {
    let __filter_value = __filter_item.clone();
    is_even(__filter_value)
})).collect::<Vec<_>>();
    let first: SifrInt = SifrInt::from_i64(0);
    let rest: Vec<SifrInt> = vec![];
    let _star_tmp = &nums;
    let first = _star_tmp[0].clone();
    let rest = _star_tmp[1.._star_tmp.len()].to_vec();
    println!("{}", format!("{:?}", mapped));
    println!("{}", format!("{:?}", filtered));
    println!("{}", format!("{}", first));
    println!("{}", format!("{:?}", rest));
    println!("clone_collection_cloning_lock_demo: pass");
}
