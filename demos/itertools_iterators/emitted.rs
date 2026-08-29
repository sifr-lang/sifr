// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.itertools ---
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
    let holder: Vec<T> = vec![value];
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &times {
        if (&SifrInt::from(holder.len()) > &SifrInt::from_i64(0)) {
            result
                .push(
                    ({
                        let Some(__sifr_index_value) = ({
                            let __sifr_index_list = &holder;
                            let __sifr_index_i = SifrInt::from_i64(0);
                            let __sifr_index_norm = __sifr_index_i
                                .normalize_index_or_len(__sifr_index_list.len());
                            __sifr_index_list.get(__sifr_index_norm).cloned()
                        }) else {
                            unreachable!("compiler-verified index should be in range");
                        };
                        __sifr_index_value
                    })
                        .clone(),
                );
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
    if &stride <= &SifrInt::from_i64(0) {
        stride = SifrInt::from_i64(1);
        actual_stop = actual_start.clone();
    }
    let mut result: Vec<T> = vec![];
    let mut index: SifrInt = actual_start.clone();
    while &index < &actual_stop {
        if (&index < &SifrInt::from(data_owned.len())) {
            let value: Option<T> = Some(
                data_owned[::sifr_runtime::to_usize_proven(&(index))].clone(),
            );
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

fn main() {
    let chained: Box<dyn Iterator<Item = SifrInt>> = chain(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)], vec![SifrInt::from_i64(3)]]);
    println!("{:?}", chained.collect::<Vec<_>>());
    let repeated: Box<dyn Iterator<Item = SifrInt>> = repeat(SifrInt::from_i64(7), SifrInt::from_i64(3));
    println!("{:?}", repeated.collect::<Vec<_>>());
    let sliced: Box<dyn Iterator<Item = SifrInt>> = islice(&(vec![SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30), SifrInt::from_i64(40), SifrInt::from_i64(50)]).into_iter().collect::<Vec<_>>(), SifrInt::from_i64(1), Some(SifrInt::from_i64(5)), SifrInt::from_i64(2));
    println!("{:?}", sliced.collect::<Vec<_>>());
    let mut counter: Box<dyn Iterator<Item = SifrInt>> = count(SifrInt::from_i64(5), SifrInt::from_i64(2));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (counter.next()).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
}
