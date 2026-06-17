// --- stdlib: sifr.itertools ---
fn _prepend<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    head: T,
    tails: &Vec<Vec<T>>,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let head_holder: Vec<T> = vec![head];
    for tail in tails.iter().cloned() {
        let current: Option<T> = {
            let __sifr_index_list = &head_holder;
            let __sifr_index_i = 0 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(current) = current {
            let mut item: Vec<T> = vec![current];
            for value in tail.iter().cloned() {
                item.push(value.clone());
            }
            result.push(item);
        }
    }
    return result;
}
fn _product_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pools: &Vec<Vec<T>>,
    index: i64,
) -> Vec<Vec<T>> {
    if index >= (pools.len() as i64) {
        return vec![vec![]];
    }
    let suffixes: Vec<Vec<T>> = _product_impl(pools, index + (1 as i64));
    let mut result: Vec<Vec<T>> = vec![];
    let current_pool: Option<Vec<T>> = Some(pools[index as usize].clone());
    let Some(current_pool) = current_pool else {
        return result;
    };
    let mut i: i64 = 0 as i64;
    while i < (current_pool.len() as i64) {
        let mut j: i64 = 0 as i64;
        while j < (suffixes.len() as i64) {
            let value: Option<T> = Some(current_pool[i as usize].clone());
            let suffix: Option<Vec<T>> = Some(suffixes[j as usize].clone());
            if let Some(value) = value {
                if let Some(suffix) = suffix {
                    let mut entry: Vec<T> = vec![value];
                    for suffix_value in suffix.iter().cloned() {
                        entry.push(suffix_value.clone());
                    }
                    result.push(entry);
                }
            }
            j = j + (1 as i64);
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _combinations_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    start: i64,
    r: i64,
) -> Vec<Vec<T>> {
    if r == (0 as i64) {
        return vec![vec![]];
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut i: i64 = start;
    while i <= ((data.len() as i64) - r) {
        let current: Option<T> = {
            let __sifr_index_list = &data;
            let __sifr_index_i = i;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(current) = current {
            let tails: Vec<Vec<T>> = _combinations_impl(
                data,
                i + (1 as i64),
                r - (1 as i64),
            );
            for entry in _prepend(current, &tails).into_iter() {
                result.push(entry);
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _collect_iterable<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: Vec<T>,
) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data.iter().cloned() {
        collected.push(item.clone());
    }
    return collected;
}
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
fn repeat<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    value: T,
    times: i64,
) -> Box<dyn Iterator<Item = T>> {
    let holder: Vec<T> = vec![value];
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0 as i64;
    while i < times {
        if (holder.len() as i64) > (0 as i64) {
            result
                .push(
                    ({
                        let Some(__sifr_index_value) = ({
                            let __sifr_index_list = &holder;
                            let __sifr_index_i = 0 as i64;
                            let __sifr_index_norm = if __sifr_index_i < 0 {
                                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                            } else {
                                __sifr_index_i as usize
                            };
                            __sifr_index_list.get(__sifr_index_norm).cloned()
                        }) else {
                            unreachable!("compiler-verified index should be in range");
                        };
                        __sifr_index_value
                    })
                        .clone(),
                );
        }
        i = i + (1 as i64);
    }
    return Box::new((result).iter().cloned());
}
fn islice<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    start_or_stop: i64,
    stop: Option<i64>,
    step: i64,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut actual_start: i64 = 0 as i64;
    let mut actual_stop: i64 = start_or_stop;
    if let Some(stop) = stop {
        actual_start = start_or_stop;
        actual_stop = stop;
    }
    if actual_start < (0 as i64) {
        actual_start = 0 as i64;
    }
    if actual_stop < (0 as i64) {
        actual_stop = 0 as i64;
    }
    let mut stride: i64 = step;
    if stride <= (0 as i64) {
        stride = 1 as i64;
        actual_stop = actual_start;
    }
    let mut result: Vec<T> = vec![];
    let mut index: i64 = actual_start;
    while index < actual_stop {
        if index < (data_owned.len() as i64) {
            let value: Option<T> = Some(data_owned[index as usize].clone());
            if let Some(value) = value {
                result.push(value.clone());
            }
        } else {
            index = actual_stop;
        }
        index = index + stride;
    }
    return Box::new((result).iter().cloned());
}
fn count(start: i64, step: i64) -> Box<dyn Iterator<Item = i64>> {
    return count_from(start, step, 10000 as i64);
}
fn product<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
    repeat: i64,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let iterables = iterables.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<Vec<T>> = Vec::new().into_iter();
    return Box::new(
        std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<Vec<T>> = Vec::new();
                let mut result: Vec<Vec<T>> = vec![];
                if repeat >= (0 as i64) {
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
                    let mut i: i64 = 0 as i64;
                    while i < repeat {
                        for pool in base_pools.iter().cloned() {
                            pools.push(pool);
                        }
                        i = i + (1 as i64);
                    }
                    if (pools.len() as i64) == (0 as i64) {
                        result = vec![vec![]];
                    } else {
                        result = _product_impl(&pools, 0 as i64);
                    }
                }
                let mut i: i64 = 0 as i64;
                while i < (result.len() as i64) {
                    _yields.push(result[i as usize].clone());
                    i = i + (1 as i64);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            return __sifr_generator_iter.next();
        }),
    );
}
fn combinations<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    r: i64,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let data = data.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<Vec<T>> = Vec::new().into_iter();
    return Box::new(
        std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<Vec<T>> = Vec::new();
                let materialized: Vec<T> = _collect_iterable(
                    ((data).iter().cloned().collect::<Vec<_>>()).clone(),
                );
                let mut result: Vec<Vec<T>> = vec![];
                if ((r >= (0 as i64)) && (r <= (materialized.len() as i64))) {
                    if r == (0 as i64) {
                        result = vec![vec![]];
                    } else {
                        result = _combinations_impl(&materialized, 0 as i64, r);
                    }
                }
                let mut i: i64 = 0 as i64;
                while i < (result.len() as i64) {
                    _yields.push(result[i as usize].clone());
                    i = i + (1 as i64);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            return __sifr_generator_iter.next();
        }),
    );
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

fn odds(limit: i64) -> Box<dyn Iterator<Item = i64>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<i64> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
    if !__sifr_generator_initialized {
        let mut _yields: Vec<i64> = Vec::new();
        let mut i: i64 = 0 as i64;
        while i < limit {
            if (i % (2 as i64)) == (1 as i64) {
                _yields.push(i);
            }
            i = i + (1 as i64);
        }
        __sifr_generator_iter = _yields.into_iter();
        __sifr_generator_initialized = true;
    }
    return __sifr_generator_iter.next();
}));
}

fn main() {
    let nums: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64, 4 as i64];
    let mut it: Box<dyn Iterator<Item = i64>> = Box::new((nums).iter().copied());
    assert!(it.next() == Some(1 as i64));
    assert!(it.next() == Some(2 as i64));
    let mut doubled: Vec<i64> = vec![];
    for n in nums.iter().copied() {
        doubled.push(n * (2 as i64));
    }
    assert!(format!("{:?}", doubled) == "[2, 4, 6, 8]".to_string());
    let mut odd_it: Box<dyn Iterator<Item = i64>> = odds(7 as i64);
    assert!(odd_it.next() == Some(1 as i64));
    assert!(odd_it.next() == Some(3 as i64));
    assert!(odd_it.next() == Some(5 as i64));
    assert!(odd_it.next() == None);
    assert!(format!("{:?}", Box::new((vec![1 as i64, 2 as i64]).into_iter().zip((vec!["a".to_string(), "b".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1))).collect::<Vec<_>>()) == "[(1, \"a\"), (2, \"b\")]".to_string());
    assert!(format!("{:?}", Box::new((vec!["x".to_string(), "y".to_string()]).into_iter().enumerate().map(|__pair| ((__pair.0 as i64) + (4 as i64), __pair.1))).collect::<Vec<_>>()) == "[(4, \"x\"), (5, \"y\")]".to_string());
    assert!(format!("{:?}", Box::new((vec![9 as i64, 8 as i64, 7 as i64]).into_iter().rev()).collect::<Vec<_>>()) == "[7, 8, 9]".to_string());
    assert!(format!("{:?}", chain(&vec![vec![1 as i64, 2 as i64], vec![3 as i64]]).collect::<Vec<_>>()) == "[1, 2, 3]".to_string());
    assert!(format!("{:?}", repeat(5 as i64, 3 as i64).collect::<Vec<_>>()) == "[5, 5, 5]".to_string());
    assert!(format!("{:?}", islice(&(vec![10 as i64, 20 as i64, 30 as i64, 40 as i64, 50 as i64]).into_iter().collect::<Vec<_>>(), 1 as i64, Some(5 as i64), 2 as i64).collect::<Vec<_>>()) == "[20, 40]".to_string());
    let mut counter: Box<dyn Iterator<Item = i64>> = count(2 as i64, 3 as i64);
    assert!(counter.next() == Some(2 as i64));
    assert!(counter.next() == Some(5 as i64));
    assert!(counter.next() == Some(8 as i64));
    let combos: Vec<Vec<i64>> = combinations(&(vec![1 as i64, 2 as i64, 3 as i64]).into_iter().collect::<Vec<_>>(), 2 as i64).collect::<Vec<_>>();
    assert!(format!("{:?}", combos) == "[[1, 2], [1, 3], [2, 3]]".to_string());
    let prods: Vec<Vec<i64>> = product(&vec![vec![1 as i64, 2 as i64]], 2 as i64).collect::<Vec<_>>();
    assert!(format!("{:?}", prods) == "[[1, 1], [1, 2], [2, 1], [2, 2]]".to_string());
    println!("iter_iterator_basics_closure_demo: ok");
}
