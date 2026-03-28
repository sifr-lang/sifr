// --- stdlib: sifr.itertools ---
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

fn main() {
    let mut chained: Box<dyn Iterator<Item = i64>> = chain(&vec![vec![1 as i64, 2 as i64], vec![3 as i64]]);
    println!("{:?}", chained.collect::<Vec<_>>());
    let mut repeated: Box<dyn Iterator<Item = i64>> = repeat(7 as i64, 3 as i64);
    println!("{:?}", repeated.collect::<Vec<_>>());
    let mut sliced: Box<dyn Iterator<Item = i64>> = islice(&(vec![10 as i64, 20 as i64, 30 as i64, 40 as i64, 50 as i64]).into_iter().collect::<Vec<_>>(), 1 as i64, Some(5 as i64), 2 as i64);
    println!("{:?}", sliced.collect::<Vec<_>>());
    let mut counter: Box<dyn Iterator<Item = i64>> = count(5 as i64, 2 as i64);
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}
