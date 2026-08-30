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
use ::sifr_runtime::SifrInt;
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
fn pairwise<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let mut prev_values: Vec<T> = vec![];
    for value in data.iter().cloned() {
        if (&SifrInt::from(prev_values.len()) > &SifrInt::from_i64(0)) {
            let mut pair: Vec<T> = vec![];
            let prev: Option<T> = Some(
                prev_values[::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(0)))]
                    .clone(),
            );
            if let Some(prev) = prev {
                pair.push(prev.clone());
            }
            pair.push(value.clone());
            result.push(pair.clone());
            {
                let __idx_raw = SifrInt::from_i64(0);
                let __idx_norm = __idx_raw.normalize_index_or_len(prev_values.len());
                if let Some(__elem) = prev_values.get_mut(__idx_norm) {
                    *__elem = value.clone();
                }
            }
        } else {
            prev_values.push(value.clone());
        }
    }
    result
}
fn batched<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: SifrInt,
) -> Result<Vec<Vec<T>>, ValueError> {
    if &n <= &SifrInt::from_i64(0) {
        return Err(ValueError::new("batched: n must be > 0".to_string()));
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut current_batch: Vec<T> = vec![];
    for value in data.iter().cloned() {
        current_batch.push(value.clone());
        if (&SifrInt::from(current_batch.len()) == &n) {
            result.push(current_batch.clone());
            current_batch = vec![];
        }
    }
    if (&SifrInt::from(current_batch.len()) > &SifrInt::from_i64(0)) {
        result.push(current_batch.clone());
    }
    Ok(result)
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
fn cycle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    n: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    let materialized: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut result: Vec<T> = vec![];
    let size: SifrInt = SifrInt::from(materialized.len());
    if &size > &SifrInt::from_i64(0) {
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < &n {
            let idx: SifrInt = i.floor_mod_known_nonzero(&size);
            let val: Option<T> = {
                let __sifr_index_list = &materialized;
                let __sifr_index_i = idx.clone();
                let __sifr_index_norm = __sifr_index_i
                    .normalize_index_or_len(__sifr_index_list.len());
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(val) = val {
                result.push(val.clone());
            }
            i = &i + &SifrInt::from_i64(1);
        }
    }
    Box::new(result.into_iter())
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            Some(actual[::sifr_runtime::to_usize_proven(& (i))]) == expected
            .get(::sifr_runtime::to_usize_proven(& (i))).copied()
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual
        .push(
            (format!(
                "{:?}", chain(& vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
                vec![SifrInt::from_i64(3)]]).collect::< Vec < _ >> ()
            ))
                .as_str() == ("[1, 2, 3]".to_string()).as_str(),
        );
    actual
        .push(
            (format!(
                "{:?}", pairwise(& (vec![SifrInt::from_i64(1), SifrInt::from_i64(2),
                SifrInt::from_i64(3), SifrInt::from_i64(4)]).into_iter().collect::< Vec <
                _ >> ())
            ))
                .as_str() == ("[[1, 2], [2, 3], [3, 4]]".to_string()).as_str(),
        );
    let mut batched_ok: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let bat: Vec<Vec<SifrInt>> = batched(
            &(vec![
                SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
                SifrInt::from_i64(4), SifrInt::from_i64(5)
            ])
                .into_iter()
                .collect::<Vec<_>>(),
            SifrInt::from_i64(2),
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
                "{:?}", accumulate(& (vec![SifrInt::from_i64(1), SifrInt::from_i64(2),
                SifrInt::from_i64(3)]).into_iter().collect::< Vec < _ >> (), None)
                .collect::< Vec < _ >> ()
            ))
                .as_str() == ("[1, 3, 6]".to_string()).as_str(),
        );
    actual
        .push(
            (format!(
                "{:?}", cycle(& (vec![SifrInt::from_i64(5), SifrInt::from_i64(6)])
                .into_iter().collect::< Vec < _ >> (), SifrInt::from_i64(5)).collect::<
                Vec < _ >> ()
            ))
                .as_str() == ("[5, 6, 5, 6, 5]".to_string()).as_str(),
        );
    actual
}
fn collect_negative_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut invalid_batch_rejected: bool = false;
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _bad: Vec<Vec<SifrInt>> = batched(
            &(vec![SifrInt::from_i64(1)]).into_iter().collect::<Vec<_>>(),
            SifrInt::from_i64(0),
        )?;
        let _ = _bad;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        invalid_batch_rejected = (&SifrInt::from(e.message.chars().count())
            > &SifrInt::from_i64(0));
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
