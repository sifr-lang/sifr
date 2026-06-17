// --- stdlib: sifr.itertools ---
fn chain<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<T> = Vec::new().into_iter();
    return Box::new(
        std::iter::from_fn(move || {
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
            return __sifr_generator_iter.next();
        }),
    );
}
fn count(start: i64, step: i64) -> Box<dyn Iterator<Item = i64>> {
    return count_from(start, step, 10000 as i64);
}
fn count_from(start: i64, step: i64, n: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<i64> = Vec::new().into_iter();
    return Box::new(
        std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<i64> = Vec::new();
                let mut i: i64 = 0 as i64;
                let mut current: i64 = start;
                while i < n {
                    _yields.push(current);
                    current = current + step;
                    i = i + (1 as i64);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            return __sifr_generator_iter.next();
        }),
    );
}

fn square(n: i64) -> i64 {
    return n * n;
}

fn main() {
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64];
    let mut it: Box<dyn Iterator<Item = i64>> = Box::new((nums).iter().copied());
    assert!(it.next() == Some(1 as i64));
    assert!(it.next() == Some(2 as i64));
    assert!(format!("{:?}", Box::new(nums.iter().copied().map(square)).collect::<Vec<_>>()) == "[1, 4, 9, 16]".to_string());
    assert!(format!("{:?}", Box::new(nums.iter().copied().filter(|__filter_item| {
    let __filter_value = *__filter_item;
    return {
    let x = __filter_value;
    (x % (2 as i64)) == (0 as i64)
};
})).collect::<Vec<_>>()) == "[2, 4]".to_string());
    assert!(format!("{:?}", Box::new((nums).iter().copied().zip((vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1))).collect::<Vec<_>>()) == "[(1, \"a\"), (2, \"b\"), (3, \"c\"), (4, \"d\")]".to_string());
    assert!(format!("{:?}", Box::new((vec!["x".to_string(), "y".to_string()]).into_iter().enumerate().map(|__pair| ((__pair.0 as i64) + (10 as i64), __pair.1))).collect::<Vec<_>>()) == "[(10, \"x\"), (11, \"y\")]".to_string());
    assert!(format!("{:?}", Box::new((nums).iter().copied().rev()).collect::<Vec<_>>()) == "[4, 3, 2, 1]".to_string());
    assert!(format!("{:?}", chain(&vec![vec![1 as i64, 2 as i64], vec![3 as i64]]).collect::<Vec<_>>()) == "[1, 2, 3]".to_string());
    let mut ticker: Box<dyn Iterator<Item = i64>> = count(3 as i64, 2 as i64);
    assert!(ticker.next() == Some(3 as i64));
    assert!(ticker.next() == Some(5 as i64));
    assert!(ticker.next() == Some(7 as i64));
    println!("iter_fix_lazy_iterators_basics_lock_demo: ok");
}
