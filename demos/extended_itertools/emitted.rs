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
fn _permutations_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    target: SifrInt,
) -> Vec<Vec<T>> {
    if &target == &SifrInt::from_i64(0) {
        return vec![vec![]];
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(data.len())) {
        let current: Option<T> = {
            let __sifr_checked_read_collection = &data;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(current) = current {
            let mut rest: Vec<T> = vec![];
            let mut j: SifrInt = SifrInt::from_i64(0);
            while (&j < &SifrInt::from(data.len())) {
                if (&j != &i) {
                    let item: Option<T> = {
                        let __sifr_checked_read_collection = &data;
                        let __sifr_checked_read_index = j.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(item) = item {
                        rest.push(item.clone());
                    }
                }
                j = &j + &SifrInt::from_i64(1);
            }
            let tails: Vec<Vec<T>> = _permutations_impl(
                &rest,
                &target - &SifrInt::from_i64(1),
            );
            for entry in _prepend(current, &tails).into_iter() {
                result.push(entry.clone());
            }
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
fn _combinations_with_replacement_impl<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static,
>(data: &Vec<T>, start: SifrInt, r: SifrInt) -> Vec<Vec<T>> {
    if &r == &SifrInt::from_i64(0) {
        return vec![vec![]];
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut i: SifrInt = start.clone();
    while (&i < &SifrInt::from(data.len())) {
        let current: Option<T> = {
            let __sifr_checked_read_collection = &data;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(current) = current {
            let tails: Vec<Vec<T>> = _combinations_with_replacement_impl(
                data,
                (i).clone(),
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
fn _compress_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    selectors: &Vec<bool>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(data.len())) {
        if (&i >= &SifrInt::from(selectors.len())) {
            i = SifrInt::from(data.len());
        } else {
            let sel: Option<bool> = {
                let __sifr_checked_read_collection = &selectors;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(sel) = sel {
                if let Some(val) = val {
                    if sel {
                        result.push(val.clone());
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
    }
    result
}
fn _takewhile_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(data.len())) {
        let val: Option<T> = {
            let __sifr_checked_read_collection = &data;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(val) = val {
            if pred(&val) {
                result.push(val.clone());
            } else {
                i = SifrInt::from(data.len());
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _zip_longest_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    b: &Vec<T>,
    fill: &T,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let len_a: SifrInt = SifrInt::from(a.len());
    let len_b: SifrInt = SifrInt::from(b.len());
    let mut max_len: SifrInt = len_a.clone();
    if (&len_b > &max_len) {
        max_len = len_b.clone();
    }
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &max_len) {
        let mut pair: Vec<T> = vec![];
        if (&i < &len_a) {
            let va: Option<T> = {
                let __sifr_checked_read_collection = &a;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(va) = va {
                pair.push(va.clone());
            } else {
                pair.push(fill.clone());
            }
        } else {
            pair.push(fill.clone());
        }
        if (&i < &len_b) {
            let vb: Option<T> = {
                let __sifr_checked_read_collection = &b;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(vb) = vb {
                pair.push(vb.clone());
            } else {
                pair.push(fill.clone());
            }
        } else {
            pair.push(fill.clone());
        }
        result.push(pair.clone());
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
fn permutations<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    r: Option<SifrInt>,
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
                let mut target: SifrInt = SifrInt::from(materialized.len());
                if let Some(r) = r.clone() {
                    target = r;
                }
                if (&target >= &SifrInt::from_i64(0))
                    && (&target <= &SifrInt::from(materialized.len()))
                {
                    if (&target == &SifrInt::from_i64(0)) {
                        result = vec![vec![]];
                    } else {
                        result = _permutations_impl(&materialized, (target).clone());
                    }
                }
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(result.len())) {
                    let Some(__sifr_checked_value_19) = ({
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
                    _yields.push(__sifr_checked_value_19.clone());
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
fn combinations_with_replacement<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
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
                if (&r >= &SifrInt::from_i64(0)) {
                    if (&r == &SifrInt::from_i64(0)) {
                        result = vec![vec![]];
                    } else {
                        if (&SifrInt::from(materialized.len()) > &SifrInt::from_i64(0)) {
                            result = _combinations_with_replacement_impl(
                                &materialized,
                                SifrInt::from_i64(0),
                                (r).clone(),
                            );
                        }
                    }
                }
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(result.len())) {
                    let Some(__sifr_checked_value_21) = ({
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
                    _yields.push(__sifr_checked_value_21.clone());
                    i = &i + &SifrInt::from_i64(1);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn starmap<
    A: Clone + ::std::fmt::Display + PartialOrd + 'static,
    B: Clone + ::std::fmt::Display + PartialOrd + 'static,
    R: Clone + ::std::fmt::Display + PartialOrd + 'static,
>(func: impl Fn(&A, &B) -> R, pairs: &Vec<(A, B)>) -> Box<dyn Iterator<Item = R>> {
    let mut result: Vec<R> = vec![];
    for (first, second) in pairs.iter().cloned() {
        result.push(func(&first, &second).clone());
    }
    Box::new(result.into_iter())
}
fn accumulate<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static + ::std::ops::Add<Output = T>,
>(data: &Vec<T>, initial: Option<T>) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    if let Some(initial) = initial {
        result.push(initial.clone());
    }
    for item in data.iter().cloned() {
        if (&SifrInt::from(result.len()) == &SifrInt::from_i64(0)) {
            result.push(item.clone());
        } else {
            let prev: Option<T> = {
                let __sifr_index_list = &result;
                let __sifr_index_i = SifrInt::from(result.len()) - SifrInt::from_i64(1);
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(prev) = prev {
                let next_val: T = prev + item;
                result.push(next_val.clone());
            }
        }
    }
    Box::new(result.into_iter())
}
fn compress<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    selectors: &Vec<bool>,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut selectors_owned: Vec<bool> = vec![];
    for selector in selectors.iter().copied() {
        selectors_owned.push(selector);
    }
    let result: Vec<T> = _compress_impl(&data_owned, &selectors_owned);
    Box::new(result.into_iter())
}
fn dropwhile<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    let mut dropping: bool = true;
    for val in data.iter().cloned() {
        if dropping {
            if !pred(&val) {
                dropping = false;
                result.push(val.clone());
            }
        } else {
            result.push(val.clone());
        }
    }
    Box::new(result.into_iter())
}
fn takewhile<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let result: Vec<T> = _takewhile_impl(pred, &data_owned);
    Box::new(result.into_iter())
}
fn filterfalse<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    for val in data.iter().cloned() {
        if !pred(&val) {
            result.push(val.clone());
        }
    }
    Box::new(result.into_iter())
}
fn zip_longest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    b: &Vec<T>,
    fill: &T,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let a_owned: Vec<T> = _collect_iterable(
        ((a).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let b_owned: Vec<T> = _collect_iterable(
        ((b).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let result: Vec<Vec<T>> = _zip_longest_impl(&a_owned, &b_owned, fill);
    Box::new(result.into_iter())
}
fn cycle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    let materialized: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut result: Vec<T> = vec![];
    let size: SifrInt = SifrInt::from(materialized.len());
    if (&size > &SifrInt::from_i64(0)) {
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &n) {
            let idx: SifrInt = i.floor_mod_known_nonzero(&size);
            let val: Option<T> = {
                let __sifr_checked_read_collection = &materialized;
                let __sifr_checked_read_index = idx.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(val) = val {
                result.push(val.clone());
            }
            i = &i + &SifrInt::from_i64(1);
        }
    }
    Box::new(result.into_iter())
}
// --- end stdlib ---

fn lt3(x: SifrInt) -> bool {
    &x < &SifrInt::from_i64(3)
}

fn add2(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}

fn main() {
    let mut acc_it: Box<dyn Iterator<Item = SifrInt>> = accumulate(&(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)]).into_iter().collect::<Vec<_>>(), None);
    assert!((acc_it.next() == Some(SifrInt::from_i64(1))));
    assert!((format!("{:?}", acc_it.collect::<Vec<_>>()) == "[3, 6, 10]"));
    assert!((format!("{:?}", compress(&(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)]).into_iter().collect::<Vec<_>>(), &(vec![true, false, true, false]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[1, 3]"));
    assert!((format!("{:?}", dropwhile(|__arg0| lt3((__arg0).clone()), &(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(1)]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[3, 1]"));
    assert!((format!("{:?}", takewhile(|__arg0| lt3((__arg0).clone()), &(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(1)]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[1, 2]"));
    assert!((format!("{:?}", filterfalse(|__arg0| lt3((__arg0).clone()), &(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(1)]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[3]"));
    assert!((format!("{:?}", zip_longest(&(vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]).into_iter().collect::<Vec<_>>(), &(vec![SifrInt::from_i64(9)]).into_iter().collect::<Vec<_>>(), &SifrInt::from_i64(0)).collect::<Vec<_>>()) == "[[1, 9], [2, 0]]"));
    let mut cyc: Box<dyn Iterator<Item = SifrInt>> = cycle(&(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]).into_iter().collect::<Vec<_>>(), SifrInt::from_i64(5));
    assert!((cyc.next() == Some(SifrInt::from_i64(1))));
    assert!((format!("{:?}", cyc.collect::<Vec<_>>()) == "[2, 3, 1, 2]"));
    assert!((format!("{:?}", starmap(|__arg0, __arg1| add2((__arg0).clone(), (__arg1).clone()), &(vec![(SifrInt::from_i64(2), SifrInt::from_i64(3)), (SifrInt::from_i64(4), SifrInt::from_i64(5))]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[5, 9]"));
    assert!((format!("{:?}", product(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]], SifrInt::from_i64(2)).collect::<Vec<_>>()) == "[[1, 1], [1, 2], [2, 1], [2, 2]]"));
    assert!((format!("{:?}", product(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]], -&SifrInt::from_i64(1)).collect::<Vec<_>>()) == "[]"));
    assert!((format!("{:?}", permutations(&(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]).into_iter().collect::<Vec<_>>(), Some(SifrInt::from_i64(2))).collect::<Vec<_>>()) == "[[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]]"));
    assert!((format!("{:?}", combinations(&(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]).into_iter().collect::<Vec<_>>(), SifrInt::from_i64(2)).collect::<Vec<_>>()) == "[[1, 2], [1, 3], [2, 3]]"));
    assert!((format!("{:?}", combinations_with_replacement(&(vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]).into_iter().collect::<Vec<_>>(), SifrInt::from_i64(2)).collect::<Vec<_>>()) == "[[1, 1], [1, 2], [2, 2]]"));
    println!("parity_ext_extended_itertools_lazy_surface_demo: ok");
}
