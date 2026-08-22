// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
}
pub use __sifr_project_nominals::ValueError;
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
fn pairwise<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let mut prev_values: Vec<T> = vec![];
    for value in data.iter().cloned() {
        if ((prev_values.len() as i64) > (0_i64)) {
            let mut pair: Vec<T> = vec![];
            let prev: Option<T> = Some(prev_values[(0_i64) as usize].clone());
            if let Some(prev) = prev {
                pair.push(prev.clone().clone());
            }
            pair.push(value.clone().clone());
            result.push(pair.clone());
            {
                let __idx_raw = 0_i64;
                let __idx_norm = if __idx_raw < 0 {
                    (prev_values.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = prev_values.get_mut(__idx_norm as usize) {
                        *__elem = value.clone();
                    }
                }
            }
        } else {
            prev_values.push(value.clone().clone());
        }
    }
    result
}
fn batched<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: i64,
) -> Result<Vec<Vec<T>>, ValueError> {
    if n <= (0_i64) {
        return Err(ValueError::new("batched: n must be > 0".to_string()));
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut current_batch: Vec<T> = vec![];
    for value in data.iter().cloned() {
        current_batch.push(value.clone().clone());
        if ((current_batch.len() as i64) == n) {
            result.push(current_batch.clone());
            current_batch = vec![];
        }
    }
    if ((current_batch.len() as i64) > (0_i64)) {
        result.push(current_batch.clone());
    }
    Ok(result)
}
fn accumulate<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static + ::std::ops::Add<Output = T>,
>(data: &Vec<T>, initial: Option<T>) -> Box<dyn Iterator<Item = T>> {
    let mut result: Vec<T> = vec![];
    if let Some(initial) = initial {
        result.push(initial.clone().clone());
    }
    for item in data.iter().cloned() {
        if ((result.len() as i64) == (0_i64)) {
            result.push(item.clone().clone());
        } else {
            let prev: Option<T> = {
                let __sifr_index_list = &result;
                let __sifr_index_i = (result.len() as i64) - (1_i64);
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(prev) = prev {
                let next_val: T = prev + item;
                result.push(next_val.clone().clone());
            }
        }
    }
    Box::new(result.into_iter())
}
fn cycle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: i64,
) -> Box<dyn Iterator<Item = T>> {
    let materialized: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut result: Vec<T> = vec![];
    if ((materialized.len() as i64) > (0_i64)) {
        let mut i: i64 = 0_i64;
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
                result.push(val.clone().clone());
            }
            i += 1_i64;
        }
    }
    Box::new(result.into_iter())
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual
        .push(
            (format!(
                "{:?}", chain(& vec![vec![1_i64, 2_i64], vec![3_i64]]).collect::< Vec < _
                >> ()
            ))
                .as_str() == ("[1, 2, 3]".to_string()).as_str(),
        );
    actual
        .push(
            (format!(
                "{:?}", pairwise(& (vec![1_i64, 2_i64, 3_i64, 4_i64]).into_iter()
                .collect::< Vec < _ >> ())
            ))
                .as_str() == ("[[1, 2], [2, 3], [3, 4]]".to_string()).as_str(),
        );
    let mut batched_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let bat: Vec<Vec<i64>> = batched(
            &(vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64]).into_iter().collect::<Vec<_>>(),
            2_i64,
        )?;
        batched_ok = (format!("{:?}", bat) == "[[1, 2], [3, 4], [5]]");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    actual.push(batched_ok);
    actual
        .push(
            (format!(
                "{:?}", accumulate(& (vec![1_i64, 2_i64, 3_i64]).into_iter().collect::<
                Vec < _ >> (), None).collect::< Vec < _ >> ()
            ))
                .as_str() == ("[1, 3, 6]".to_string()).as_str(),
        );
    actual
        .push(
            (format!(
                "{:?}", cycle(& (vec![5_i64, 6_i64]).into_iter().collect::< Vec < _ >>
                (), 5_i64).collect::< Vec < _ >> ()
            ))
                .as_str() == ("[5, 6, 5, 6, 5]".to_string()).as_str(),
        );
    actual
}
fn collect_negative_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut invalid_batch_rejected: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _bad: Vec<Vec<i64>> = batched(
            &(vec![1_i64]).into_iter().collect::<Vec<_>>(),
            0_i64,
        )?;
        let _ = _bad;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        invalid_batch_rejected = ((e.message.chars().count() as i64) > (0_i64));
    }
    actual.push(invalid_batch_rejected);
    actual
}
fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_core_actual());
    append_all(&mut actual, &collect_negative_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("itertools itertools parity demo: pass");
}
