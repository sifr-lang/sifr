// src/main.rs
// --- stdlib: sifr.itertools ---
fn _collect_iterable<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Vec<T>,
) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data.iter().cloned() {
        collected.push(item.clone().clone());
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
fn repeat<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    value: T,
    times: i64,
) -> Box<dyn Iterator<Item = T>> {
    let holder: Vec<T> = vec![value.clone()];
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0_i64;
    while i < times {
        if ((holder.len() as i64) > (0_i64)) {
            result
                .push(
                    ({
                        let Some(__sifr_index_value) = ({
                            let __sifr_index_list = &holder;
                            let __sifr_index_i = 0_i64;
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
        i += 1_i64;
    }
    Box::new(result.into_iter())
}
fn islice<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    start_or_stop: i64,
    stop: Option<i64>,
    step: i64,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut actual_start: i64 = 0_i64;
    let mut actual_stop: i64 = start_or_stop;
    if let Some(stop) = stop {
        actual_start = start_or_stop;
        actual_stop = stop;
    }
    if actual_start < (0_i64) {
        actual_start = 0_i64;
    }
    if actual_stop < (0_i64) {
        actual_stop = 0_i64;
    }
    let mut stride: i64 = step;
    if stride <= (0_i64) {
        stride = 1_i64;
        actual_stop = actual_start;
    }
    let mut result: Vec<T> = vec![];
    let mut index: i64 = actual_start;
    while index < actual_stop {
        if (index < (data_owned.len() as i64)) {
            let value: Option<T> = Some(data_owned[index as usize].clone());
            if let Some(value) = value {
                result.push(value.clone().clone());
            }
        } else {
            index = actual_stop;
        }
        index += stride;
    }
    Box::new(result.into_iter())
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

fn main() {
    let chained: Box<dyn Iterator<Item = i64>> = chain(&vec![vec![1_i64, 2_i64], vec![3_i64]]);
    println!("{:?}", chained.collect::<Vec<_>>());
    let repeated: Box<dyn Iterator<Item = i64>> = repeat(7_i64, 3_i64);
    println!("{:?}", repeated.collect::<Vec<_>>());
    let sliced: Box<dyn Iterator<Item = i64>> = islice(&(vec![10_i64, 20_i64, 30_i64, 40_i64, 50_i64]).into_iter().collect::<Vec<_>>(), 1_i64, Some(5_i64), 2_i64);
    println!("{:?}", sliced.collect::<Vec<_>>());
    let mut counter: Box<dyn Iterator<Item = i64>> = count(5_i64, 2_i64);
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}
