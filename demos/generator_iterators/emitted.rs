// src/main.rs
use ::sifr_runtime::SifrInt;

fn gen_pairs(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &limit {
            _yields.push(i.clone());
            i = &i + &SifrInt::from_i64(1);
            if &i < &limit {
                _yields.push(i.clone());
                i = &i + &SifrInt::from_i64(1);
            }
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn gen_even(xs: &Vec<SifrInt>) -> Box<dyn Iterator<Item = SifrInt>> {
    let xs = xs.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        for x in xs.iter().cloned() {
            if (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
                _yields.push(x.clone());
            }
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn main() {
    let xs: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)];
    let squares: Box<dyn Iterator<Item = SifrInt>> = Box::new(xs.iter().cloned().filter_map(|x| if (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) { Some(&x * &x) } else { None }));
    println!("{:?}", squares.collect::<Vec<_>>());
    println!("{:?}", gen_pairs(SifrInt::from_i64(5)).collect::<Vec<_>>());
    println!("{:?}", gen_even(&xs).collect::<Vec<_>>());
}
