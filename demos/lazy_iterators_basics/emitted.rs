// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.itertools ---
fn chain<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<T> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<T> = Vec::new();
                for iterable in iterables.iter().cloned() {
                    for item in iterable.iter().cloned() {
                        _yields.push(item.clone());
                    }
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn count(start: SifrInt, step: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    count_from((start).clone(), (step).clone(), SifrInt::from_i64(10000))
}
fn count_from(
    start: SifrInt,
    step: SifrInt,
    n: SifrInt,
) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new()
        .into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<SifrInt> = Vec::new();
                let mut i: SifrInt = SifrInt::from_i64(0);
                let mut current: SifrInt = start.clone();
                while &i < &n {
                    _yields.push(current.clone());
                    current = &current + &step;
                    i = &i + &SifrInt::from_i64(1);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
// --- end stdlib ---

fn square(n: SifrInt) -> SifrInt {
    &n * &n
}

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)];
    let mut it: Box<dyn Iterator<Item = SifrInt>> = Box::new((nums).iter().cloned());
    assert!((it.next() == Some(SifrInt::from_i64(1))));
    assert!((it.next() == Some(SifrInt::from_i64(2))));
    assert!((format!("{:?}", Box::new(nums.iter().cloned().map(|__sifr_map_item| square(__sifr_map_item))).collect::<Vec<_>>()) == "[1, 4, 9, 16]"));
    assert!((format!("{:?}", Box::new((nums).iter().cloned().filter(move |__filter_item| (|x| (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)))(__filter_item.clone()))).collect::<Vec<_>>()) == "[2, 4]"));
    assert!((format!("{:?}", Box::new((nums).iter().cloned().zip((vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1))).collect::<Vec<_>>()) == "[(1, \"a\"), (2, \"b\"), (3, \"c\"), (4, \"d\")]"));
    assert!((format!("{:?}", Box::new((vec!["x".to_string(), "y".to_string()]).into_iter().enumerate().map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(10), __pair.1))).collect::<Vec<_>>()) == "[(10, \"x\"), (11, \"y\")]"));
    assert!((format!("{:?}", Box::new((nums).iter().cloned().rev()).collect::<Vec<_>>()) == "[4, 3, 2, 1]"));
    assert!((format!("{:?}", chain(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)], vec![SifrInt::from_i64(3)]]).collect::<Vec<_>>()) == "[1, 2, 3]"));
    let mut ticker: Box<dyn Iterator<Item = SifrInt>> = count(SifrInt::from_i64(3), SifrInt::from_i64(2));
    assert!((ticker.next() == Some(SifrInt::from_i64(3))));
    assert!((ticker.next() == Some(SifrInt::from_i64(5))));
    assert!((ticker.next() == Some(SifrInt::from_i64(7))));
    println!("iter_fix_lazy_iterators_basics_lock_demo: ok");
}
