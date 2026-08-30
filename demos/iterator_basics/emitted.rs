// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.itertools ---
fn _prepend<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    head: T,
    tails: &Vec<Vec<T>>,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let head_holder: Vec<T> = vec![head.clone()];
    for tail in tails.iter().cloned() {
        let current: Option<T> = {
            let __sifr_checked_read_collection = &head_holder;
            let __sifr_checked_read_index = SifrInt::from_i64(0);
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(current) = current {
            let mut item: Vec<T> = vec![current.clone()];
            for value in tail.iter().cloned() {
                item.push(value.clone());
            }
            result.push(item.clone());
        }
    }
    result
}
fn _product_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pools: &Vec<Vec<T>>,
    index: SifrInt,
) -> Vec<Vec<T>> {
    if &index >= &SifrInt::from(pools.len()) {
        return vec![vec![]];
    }
    let suffixes: Vec<Vec<T>> = _product_impl(pools, &index + &SifrInt::from_i64(1));
    let mut result: Vec<Vec<T>> = vec![];
    let current_pool: Option<Vec<T>> = {
        let __sifr_checked_read_collection = &pools;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(current_pool) = current_pool else {
        return result;
    };
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(current_pool.len())) {
        let Some(__sifr_checked_value_2) = ({
            let __sifr_checked_read_collection = &current_pool;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        }) else {
            break;
        };
        let mut j: SifrInt = SifrInt::from_i64(0);
        while (&j < &SifrInt::from(suffixes.len())) {
            let value: Option<T> = {
                let __sifr_checked_read_collection = &current_pool;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let suffix: Option<Vec<T>> = {
                let __sifr_checked_read_collection = &suffixes;
                let __sifr_checked_read_index = j.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(value) = value {
                if let Some(suffix) = suffix {
                    let mut entry: Vec<T> = vec![value.clone()];
                    for suffix_value in suffix.iter().cloned() {
                        entry.push(suffix_value.clone());
                    }
                    result.push(entry.clone());
                }
            }
            j = &j + &SifrInt::from_i64(1);
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _combinations_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    start: SifrInt,
    r: SifrInt,
) -> Vec<Vec<T>> {
    if &r == &SifrInt::from_i64(0) {
        return vec![vec![]];
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut i: SifrInt = start.clone();
    while (&i <= &(&SifrInt::from(data.len()) - &r)) {
        let current: Option<T> = {
            let __sifr_checked_read_collection = &data;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(current) = current {
            let tails: Vec<Vec<T>> = _combinations_impl(
                data,
                &i + &SifrInt::from_i64(1),
                &r - &SifrInt::from_i64(1),
            );
            for entry in _prepend(current, &tails).into_iter() {
                result.push(entry.clone());
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _collect_iterable<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Vec<T>,
) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data.iter().cloned() {
        collected.push(item.clone());
    }
    collected
}
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
fn repeat<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    value: T,
    times: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    let holder: Vec<T> = vec![value.clone()];
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &times) {
        if (&SifrInt::from(holder.len()) > &SifrInt::from_i64(0)) {
            if let Some(__sifr_checked_value_15) = {
                let __sifr_checked_read_collection = &holder;
                let __sifr_checked_read_index = SifrInt::from_i64(0);
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            } {
                result.push(__sifr_checked_value_15.clone().clone());
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    Box::new(result.into_iter())
}
fn islice<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    start_or_stop: SifrInt,
    stop: Option<SifrInt>,
    step: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut actual_start: SifrInt = SifrInt::from_i64(0);
    let mut actual_stop: SifrInt = start_or_stop.clone();
    if let Some(stop) = stop.clone() {
        actual_start = start_or_stop.clone();
        actual_stop = stop.clone();
    }
    if &actual_start < &SifrInt::from_i64(0) {
        actual_start = SifrInt::from_i64(0);
    }
    if &actual_stop < &SifrInt::from_i64(0) {
        actual_stop = SifrInt::from_i64(0);
    }
    let mut stride: SifrInt = step.clone();
    if (&stride <= &SifrInt::from_i64(0)) {
        stride = SifrInt::from_i64(1);
        actual_stop = actual_start.clone();
    }
    let mut result: Vec<T> = vec![];
    let mut index: SifrInt = actual_start.clone();
    while (&index < &actual_stop) {
        if (&index < &SifrInt::from(data_owned.len())) {
            let value: Option<T> = {
                let __sifr_checked_read_collection = &data_owned;
                let __sifr_checked_read_index = index.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(value) = value {
                result.push(value.clone());
            }
        } else {
            index = actual_stop.clone();
        }
        index = &index + &stride;
    }
    Box::new(result.into_iter())
}
fn count(start: SifrInt, step: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    count_from((start).clone(), (step).clone(), SifrInt::from_i64(10000))
}
fn product<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
    repeat: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let iterables = iterables.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<Vec<T>> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<Vec<T>> = Vec::new();
                let mut result: Vec<Vec<T>> = vec![];
                if (&repeat >= &SifrInt::from_i64(0)) {
                    let mut base_pools: Vec<Vec<T>> = vec![];
                    for iterable in iterables.iter().cloned() {
                        base_pools
                            .push(
                                _collect_iterable(
                                    (iterable).iter().cloned().collect::<Vec<_>>(),
                                ),
                            );
                    }
                    let mut pools: Vec<Vec<T>> = vec![];
                    let mut i: SifrInt = SifrInt::from_i64(0);
                    while (&i < &repeat) {
                        for pool in base_pools.iter().cloned() {
                            pools.push(pool.clone());
                        }
                        i = &i + &SifrInt::from_i64(1);
                    }
                    if (&SifrInt::from(pools.len()) == &SifrInt::from_i64(0)) {
                        result = vec![vec![]];
                    } else {
                        result = _product_impl(&pools, SifrInt::from_i64(0));
                    }
                }
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(result.len())) {
                    let Some(__sifr_checked_value_18) = ({
                        let __sifr_checked_read_collection = &result;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    }) else {
                        break;
                    };
                    _yields.push(__sifr_checked_value_18.clone());
                    i = &i + &SifrInt::from_i64(1);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn combinations<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    r: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let data = data.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<Vec<T>> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<Vec<T>> = Vec::new();
                let materialized: Vec<T> = _collect_iterable(
                    (data).iter().cloned().collect::<Vec<_>>(),
                );
                let mut result: Vec<Vec<T>> = vec![];
                if (&r >= &SifrInt::from_i64(0))
                    && (&r <= &SifrInt::from(materialized.len()))
                {
                    if (&r == &SifrInt::from_i64(0)) {
                        result = vec![vec![]];
                    } else {
                        result = _combinations_impl(
                            &materialized,
                            SifrInt::from_i64(0),
                            (r).clone(),
                        );
                    }
                }
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(result.len())) {
                    let Some(__sifr_checked_value_20) = ({
                        let __sifr_checked_read_collection = &result;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    }) else {
                        break;
                    };
                    _yields.push(__sifr_checked_value_20.clone());
                    i = &i + &SifrInt::from_i64(1);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
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

fn odds(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<SifrInt> = Vec::new().into_iter();
    Box::new(::std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<SifrInt> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &limit) {
            if (&i.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(1)) {
                _yields.push(i.clone());
            }
            i = &i + &SifrInt::from_i64(1);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    __sifr_generator_iter.next()
}))
}

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)];
    let mut it: Box<dyn Iterator<Item = SifrInt>> = Box::new((nums).iter().cloned());
    assert!((it.next() == Some(SifrInt::from_i64(1))));
    assert!((it.next() == Some(SifrInt::from_i64(2))));
    let mut doubled: Vec<SifrInt> = vec![];
    for n in nums.iter().cloned() {
        doubled.push(&n * &SifrInt::from_i64(2));
    }
    assert!((format!("{:?}", doubled) == "[2, 4, 6, 8]"));
    let mut odd_it: Box<dyn Iterator<Item = SifrInt>> = odds(SifrInt::from_i64(7));
    assert!((odd_it.next() == Some(SifrInt::from_i64(1))));
    assert!((odd_it.next() == Some(SifrInt::from_i64(3))));
    assert!((odd_it.next() == Some(SifrInt::from_i64(5))));
    assert!((odd_it.next() == None));
    assert!((format!("{:?}", Box::new((vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]).into_iter().zip((vec!["a".to_string(), "b".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1))).collect::<Vec<_>>()) == "[(1, \"a\"), (2, \"b\")]"));
    assert!((format!("{:?}", Box::new((vec!["x".to_string(), "y".to_string()]).into_iter().enumerate().map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(4), __pair.1))).collect::<Vec<_>>()) == "[(4, \"x\"), (5, \"y\")]"));
    assert!((format!("{:?}", Box::new((vec![SifrInt::from_i64(9), SifrInt::from_i64(8), SifrInt::from_i64(7)]).into_iter().rev()).collect::<Vec<_>>()) == "[7, 8, 9]"));
    assert!((format!("{:?}", chain(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)], vec![SifrInt::from_i64(3)]]).collect::<Vec<_>>()) == "[1, 2, 3]"));
    assert!((format!("{:?}", repeat(SifrInt::from_i64(5), SifrInt::from_i64(3)).collect::<Vec<_>>()) == "[5, 5, 5]"));
    assert!((format!("{:?}", islice(&(vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30), SifrInt::from_i64(40), SifrInt::from_i64(50)]).into_iter().collect::<Vec<_>>(), SifrInt::from_i64(1), Some(SifrInt::from_i64(5)), SifrInt::from_i64(2)).collect::<Vec<_>>()) == "[20, 40]"));
    let mut counter: Box<dyn Iterator<Item = SifrInt>> = count(SifrInt::from_i64(2), SifrInt::from_i64(3));
    assert!((counter.next() == Some(SifrInt::from_i64(2))));
    assert!((counter.next() == Some(SifrInt::from_i64(5))));
    assert!((counter.next() == Some(SifrInt::from_i64(8))));
    let combos: Vec<Vec<SifrInt>> = combinations(&(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]).into_iter().collect::<Vec<_>>(), SifrInt::from_i64(2)).collect::<Vec<_>>();
    assert!((format!("{:?}", combos) == "[[1, 2], [1, 3], [2, 3]]"));
    let prods: Vec<Vec<SifrInt>> = product(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]], SifrInt::from_i64(2)).collect::<Vec<_>>();
    assert!((format!("{:?}", prods) == "[[1, 1], [1, 2], [2, 1], [2, 2]]"));
    println!("iter_iterator_basics_closure_demo: ok");
}
