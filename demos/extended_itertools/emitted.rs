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
fn _permutations_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    target: i64,
) -> Vec<Vec<T>> {
    if target == (0 as i64) {
        return vec![vec![]];
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut i: i64 = 0 as i64;
    while i < (data.len() as i64) {
        let current: Option<T> = Some(data[i as usize].clone());
        if let Some(current) = current {
            let mut rest: Vec<T> = vec![];
            let mut j: i64 = 0 as i64;
            while j < (data.len() as i64) {
                if j != i {
                    let item: Option<T> = Some(data[j as usize].clone());
                    if let Some(item) = item {
                        rest.push(item.clone());
                    }
                }
                j = j + (1 as i64);
            }
            let tails: Vec<Vec<T>> = _permutations_impl(&rest, target - (1 as i64));
            for entry in _prepend(current, &tails).into_iter() {
                result.push(entry);
            }
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
fn _combinations_with_replacement_impl<
    T: Clone + std::fmt::Display + PartialOrd + 'static,
>(data: &Vec<T>, start: i64, r: i64) -> Vec<Vec<T>> {
    if r == (0 as i64) {
        return vec![vec![]];
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut i: i64 = start;
    while i < (data.len() as i64) {
        let current: Option<T> = Some(data[i as usize].clone());
        if let Some(current) = current {
            let tails: Vec<Vec<T>> = _combinations_with_replacement_impl(
                data,
                i,
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
fn _compress_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    selectors: &Vec<bool>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0 as i64;
    while i < (data.len() as i64) {
        if i >= (selectors.len() as i64) {
            i = data.len() as i64;
        } else {
            let sel: Option<bool> = {
                let __sifr_index_list = &selectors;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            let val: Option<T> = Some(data[i as usize].clone());
            if let Some(sel) = sel {
                if let Some(val) = val {
                    if sel {
                        result.push(val.clone());
                    }
                }
            }
            i = i + (1 as i64);
        }
    }
    return result;
}
fn _takewhile_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0 as i64;
    while i < (data.len() as i64) {
        let val: Option<T> = Some(data[i as usize].clone());
        if let Some(val) = val {
            if pred(&val) {
                result.push(val.clone());
            } else {
                i = data.len() as i64;
            }
        }
        i = i + (1 as i64);
    }
    return result;
}
fn _zip_longest_impl<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    b: &Vec<T>,
    fill: &T,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let len_a: i64 = a.len() as i64;
    let len_b: i64 = b.len() as i64;
    let mut max_len: i64 = len_a;
    if len_b > max_len {
        max_len = len_b;
    }
    let mut i: i64 = 0 as i64;
    while i < max_len {
        let mut pair: Vec<T> = vec![];
        if i < len_a {
            let va: Option<T> = {
                let __sifr_index_list = &a;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(va) = va {
                pair.push(va.clone());
            } else {
                pair.push(fill.clone());
            }
        } else {
            pair.push(fill.clone());
        }
        if i < len_b {
            let vb: Option<T> = {
                let __sifr_index_list = &b;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(vb) = vb {
                pair.push(vb.clone());
            } else {
                pair.push(fill.clone());
            }
        } else {
            pair.push(fill.clone());
        }
        result.push(pair);
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
fn permutations<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    r: Option<i64>,
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
                let mut target: i64 = materialized.len() as i64;
                if let Some(r) = r {
                    target = r;
                }
                if ((target >= (0 as i64)) && (target <= (materialized.len() as i64))) {
                    if target == (0 as i64) {
                        result = vec![vec![]];
                    } else {
                        result = _permutations_impl(&materialized, target);
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
fn combinations_with_replacement<T: Clone + std::fmt::Display + PartialOrd + 'static>(
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
                if r >= (0 as i64) {
                    if r == (0 as i64) {
                        result = vec![vec![]];
                    } else {
                        if (materialized.len() as i64) > (0 as i64) {
                            result = _combinations_with_replacement_impl(
                                &materialized,
                                0 as i64,
                                r,
                            );
                        }
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
fn starmap<
    A: Clone + std::fmt::Display + PartialOrd + 'static,
    B: Clone + std::fmt::Display + PartialOrd + 'static,
    R: Clone + std::fmt::Display + PartialOrd + 'static,
>(func: impl Fn(&A, &B) -> R, pairs: &Vec<(A, B)>) -> Box<dyn Iterator<Item = R>> {
    let mut result: Vec<R> = vec![];
    for (first, second) in pairs.iter().cloned() {
        result.push(func(&first, &second).clone());
    }
    return Box::new((result).iter().cloned());
}
fn accumulate<
    T: Clone + std::fmt::Display + PartialOrd + 'static + std::ops::Add<Output = T>,
>(data: &Vec<T>, initial: Option<T>) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    if let Some(initial) = initial {
        result.push(initial.clone());
    }
    for item in data.iter().cloned() {
        if (result.len() as i64) == (0 as i64) {
            result.push(item.clone());
        } else {
            let prev: Option<T> = {
                let __sifr_index_list = &result;
                let __sifr_index_i = (result.len() as i64) - (1 as i64);
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(prev) = prev {
                let next_val: T = prev + item;
                result.push(next_val.clone());
            }
        }
    }
    return Box::new((result).iter().cloned());
}
fn compress<T: Clone + std::fmt::Display + PartialOrd + 'static>(
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
    return Box::new((result).iter().cloned());
}
fn dropwhile<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    let mut dropping: bool = true;
    for val in data.iter().cloned() {
        if dropping {
            if !(pred(&val)) {
                dropping = false;
                result.push(val.clone());
            }
        } else {
            result.push(val.clone());
        }
    }
    return Box::new((result).iter().cloned());
}
fn takewhile<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let result: Vec<T> = _takewhile_impl(pred, &data_owned);
    return Box::new((result).iter().cloned());
}
fn filterfalse<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool,
    data: &Vec<T>,
) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    for val in data.iter().cloned() {
        if !(pred(&val)) {
            result.push(val.clone());
        }
    }
    return Box::new((result).iter().cloned());
}
fn zip_longest<T: Clone + std::fmt::Display + PartialOrd + 'static>(
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
    return Box::new((result).iter().cloned());
}
fn cycle<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: i64,
) -> Box<dyn Iterator<Item = T>> {
    let materialized: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut result: Vec<T> = vec![];
    if (materialized.len() as i64) > (0 as i64) {
        let mut i: i64 = 0 as i64;
        while i < n {
            let idx: i64 = i % (materialized.len() as i64);
            let val: Option<T> = {
                let __sifr_index_list = &materialized;
                let __sifr_index_i = idx;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(val) = val {
                result.push(val.clone());
            }
            i = i + (1 as i64);
        }
    }
    return Box::new((result).iter().cloned());
}

fn lt3(x: i64) -> bool {
    return x < (3 as i64);
}

fn add2(a: i64, b: i64) -> i64 {
    return a + b;
}

fn main() {
    let mut acc_it: Box<dyn Iterator<Item = i64>> = accumulate(&(vec![1 as i64, 2 as i64, 3 as i64, 4 as i64]).into_iter().collect::<Vec<_>>(), None);
    assert!(acc_it.next() == Some(1 as i64));
    assert!(format!("{:?}", acc_it.collect::<Vec<_>>()) == "[3, 6, 10]".to_string());
    assert!(format!("{:?}", compress(&(vec![1 as i64, 2 as i64, 3 as i64, 4 as i64]).into_iter().collect::<Vec<_>>(), &(vec![true, false, true, false]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[1, 3]".to_string());
    assert!(format!("{:?}", dropwhile(|__arg0| lt3((__arg0).clone()), &(vec![1 as i64, 2 as i64, 3 as i64, 1 as i64]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[3, 1]".to_string());
    assert!(format!("{:?}", takewhile(|__arg0| lt3((__arg0).clone()), &(vec![1 as i64, 2 as i64, 3 as i64, 1 as i64]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[1, 2]".to_string());
    assert!(format!("{:?}", filterfalse(|__arg0| lt3((__arg0).clone()), &(vec![1 as i64, 2 as i64, 3 as i64, 1 as i64]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[3]".to_string());
    assert!(format!("{:?}", zip_longest(&(vec![1 as i64, 2 as i64]).into_iter().collect::<Vec<_>>(), &(vec![9 as i64]).into_iter().collect::<Vec<_>>(), &(0 as i64)).collect::<Vec<_>>()) == "[[1, 9], [2, 0]]".to_string());
    let mut cyc: Box<dyn Iterator<Item = i64>> = cycle(&(vec![1 as i64, 2 as i64, 3 as i64]).into_iter().collect::<Vec<_>>(), 5 as i64);
    assert!(cyc.next() == Some(1 as i64));
    assert!(format!("{:?}", cyc.collect::<Vec<_>>()) == "[2, 3, 1, 2]".to_string());
    assert!(format!("{:?}", starmap(|__arg0, __arg1| add2((__arg0).clone(), (__arg1).clone()), &(vec![(2 as i64, 3 as i64), (4 as i64, 5 as i64)]).into_iter().collect::<Vec<_>>()).collect::<Vec<_>>()) == "[5, 9]".to_string());
    assert!(format!("{:?}", product(&vec![vec![1 as i64, 2 as i64]], 2 as i64).collect::<Vec<_>>()) == "[[1, 1], [1, 2], [2, 1], [2, 2]]".to_string());
    assert!(format!("{:?}", product(&vec![vec![1 as i64, 2 as i64]], -(1 as i64)).collect::<Vec<_>>()) == "[]".to_string());
    assert!(format!("{:?}", permutations(&(vec![1 as i64, 2 as i64, 3 as i64]).into_iter().collect::<Vec<_>>(), Some(2 as i64)).collect::<Vec<_>>()) == "[[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]]".to_string());
    assert!(format!("{:?}", combinations(&(vec![1 as i64, 2 as i64, 3 as i64]).into_iter().collect::<Vec<_>>(), 2 as i64).collect::<Vec<_>>()) == "[[1, 2], [1, 3], [2, 3]]".to_string());
    assert!(format!("{:?}", combinations_with_replacement(&(vec![1 as i64, 2 as i64]).into_iter().collect::<Vec<_>>(), 2 as i64).collect::<Vec<_>>()) == "[[1, 1], [1, 2], [2, 2]]".to_string());
    println!("parity_ext_extended_itertools_lazy_surface_demo: ok");
}
