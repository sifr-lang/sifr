// src/main.rs
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
                        _yields.push(item);
                    }
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn count(start: i64, step: i64) -> Box<dyn Iterator<Item = i64>> {
    count_from(start, step, 10000_i64)
}
fn count_from(start: i64, step: i64, n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<i64> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<i64> = Vec::new();
                let mut i: i64 = 0_i64;
                let mut current: i64 = start;
                while i < n {
                    _yields.push(current);
                    current += step;
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
// --- end stdlib ---

fn square(n: i64) -> i64 {
    n * n
}

fn main() {
    let nums: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64];
    let mut it: Box<dyn Iterator<Item = i64>> = Box::new((nums).iter().copied());
    assert!((it.next() == Some(1_i64)));
    assert!((it.next() == Some(2_i64)));
    assert!((format!("{:?}", Box::new(nums.iter().copied().map(|__sifr_map_item| square(__sifr_map_item))).collect::<Vec<_>>()) == "[1, 4, 9, 16]"));
    assert!((format!("{:?}", Box::new(nums.iter().copied().filter(|__filter_item| {
    let __filter_value = *__filter_item;
    {
    let x = __filter_value;
    (x % (2_i64)) == (0_i64)
}
})).collect::<Vec<_>>()) == "[2, 4]"));
    assert!((format!("{:?}", Box::new((nums).iter().copied().zip((vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1))).collect::<Vec<_>>()) == "[(1, \"a\"), (2, \"b\"), (3, \"c\"), (4, \"d\")]"));
    assert!((format!("{:?}", Box::new((vec!["x".to_string(), "y".to_string()]).into_iter().enumerate().map(|__pair| ((__pair.0 as i64) + (10_i64), __pair.1))).collect::<Vec<_>>()) == "[(10, \"x\"), (11, \"y\")]"));
    assert!((format!("{:?}", Box::new((nums).iter().copied().rev()).collect::<Vec<_>>()) == "[4, 3, 2, 1]"));
    assert!((format!("{:?}", chain(&vec![vec![1_i64, 2_i64], vec![3_i64]]).collect::<Vec<_>>()) == "[1, 2, 3]"));
    let mut ticker: Box<dyn Iterator<Item = i64>> = count(3_i64, 2_i64);
    assert!((ticker.next() == Some(3_i64)));
    assert!((ticker.next() == Some(5_i64)));
    assert!((ticker.next() == Some(7_i64)));
    println!("iter_fix_lazy_iterators_basics_lock_demo: ok");
}
