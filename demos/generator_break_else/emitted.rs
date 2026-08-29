// src/main.rs
use ::sifr_runtime::SifrInt;

fn r#gen(flag: bool) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &SifrInt::from_i64(2) {
            if flag && (&i == &SifrInt::from_i64(0)) {
                break;
            }
            _yields.push(i.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        if !flag && (&i == &SifrInt::from_i64(2)) {
            _yields.push(SifrInt::from_i64(99));
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn main() {
    println!("generator_break_else yield/loop-path coverage demo:");
    for v in r#gen(false) {
        println!("{}", v);
    }
    for v in r#gen(true) {
        println!("{}", v);
    }
}
