// src/main.rs
use ::sifr_runtime::SifrInt;

fn countdown(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = n.clone();
        while &i > &SifrInt::from_i64(0) {
            _yields.push(i.clone());
            i = &i - &SifrInt::from_i64(1);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn main() {
    let mut it: Box<dyn Iterator<Item = SifrInt>> = countdown(SifrInt::from_i64(3));
    let first: Option<SifrInt> = it.next();
    let second: Option<SifrInt> = it.next();
    let remaining: Vec<SifrInt> = it.collect::<Vec<_>>();
    let all_values: Vec<SifrInt> = countdown(SifrInt::from_i64(4)).collect::<Vec<_>>();
    assert!(first == Some(SifrInt::from_i64(3)));
    assert!(second == Some(SifrInt::from_i64(2)));
    assert!((remaining == vec![SifrInt::from_i64(1)]));
    assert!((all_values == vec![SifrInt::from_i64(4), SifrInt::from_i64(3), SifrInt::from_i64(2), SifrInt::from_i64(1)]));
    println!("{}", (first).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (second).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{:?}", remaining);
    println!("{:?}", all_values);
}
