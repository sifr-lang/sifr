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
struct __SifrYielder<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
}
struct __SifrYieldFuture<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: Option<T>,
}
impl<T> Unpin for __SifrYieldFuture<T> {}
impl<T> ::std::future::Future for __SifrYieldFuture<T> {
    type Output = ();
    fn poll(
        self: ::std::pin::Pin<&mut Self>,
        _cx: &mut ::std::task::Context<'_>,
    ) -> ::std::task::Poll<()> {
        let state = self.get_mut();
        let Some(value) = state.value.take() else {
            return ::std::task::Poll::Ready(());
        };
        __sifr_store_suspended(&state.slot, value);
        ::std::task::Poll::Pending
    }
}
impl<T> __SifrYielder<T> {
    fn suspend(&self, value: T) -> __SifrYieldFuture<T> {
        __SifrYieldFuture {
            slot: ::std::sync::Arc::clone(&self.slot),
            value: Some(value),
        }
    }
}
fn __sifr_store_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: T,
) {
    match slot.lock() {
        Ok(mut state) => *state = Some(value),
        Err(poisoned) => *poisoned.into_inner() = Some(value),
    }
}
fn __sifr_take_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
) -> Option<T> {
    match slot.lock() {
        Ok(mut state) => state.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    }
}
struct __SifrGenerator<T> {
    producer: Option<
        ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>,
    >,
    yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    complete: bool,
}
impl<T> __SifrGenerator<T> {
    fn new<
        F: FnOnce(__SifrYielder<T>) -> Fut + 'static,
        Fut: ::std::future::Future<Output = ()> + 'static,
    >(factory: F) -> Self {
        let yielded = ::std::sync::Arc::new(::std::sync::Mutex::new(None));
        let producer = factory(__SifrYielder {
            slot: ::std::sync::Arc::clone(&yielded),
        });
        Self {
            producer: Some(Box::pin(producer)),
            yielded,
            complete: false,
        }
    }
}
impl<T> Iterator for __SifrGenerator<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        if self.complete {
            return None;
        }
        let completed = {
            let Some(producer) = self.producer.as_mut() else {
                self.complete = true;
                return None;
            };
            let mut context = ::std::task::Context::from_waker(
                ::std::task::Waker::noop(),
            );
            ::std::future::Future::poll(producer.as_mut(), &mut context).is_ready()
        };
        let yielded = __sifr_take_suspended(&self.yielded);
        if completed {
            self.complete = true;
            self.producer = None;
        }
        yielded
    }
}
pub trait __SifrAdd: Sized {
    fn __sifr_add(self, rhs: Self) -> Self;
}
impl __SifrAdd for ::sifr_runtime::SifrInt {
    fn __sifr_add(self, rhs: Self) -> Self {
        self + rhs
    }
}
impl __SifrAdd for f64 {
    fn __sifr_add(self, rhs: Self) -> Self {
        self + rhs
    }
}
impl __SifrAdd for String {
    fn __sifr_add(mut self, rhs: Self) -> Self {
        self.push_str(&rhs);
        self
    }
}
fn chain<T: Clone + 'static>(iterables: &[Vec<T>]) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.to_vec();
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for iterable in iterables.iter().cloned() {
                for item in iterable.iter().cloned() {
                    __sifr_yielder.suspend(item.clone()).await;
                }
            }
        }),
    )
}
fn pairwise<T: Clone + 'static>(data: &[T]) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = vec![];
    let mut prev_values: Vec<T> = vec![];
    for value in data.iter().cloned() {
        if (&SifrInt::from(prev_values.len()) > &SifrInt::from_i64(0)) {
            let mut pair: Vec<T> = vec![];
            let prev: Option<T> = {
                let __sifr_checked_read_collection = &prev_values;
                let __sifr_checked_read_index = SifrInt::from_i64(0);
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(prev) = prev {
                pair.push(prev.clone());
            }
            pair.push(value.clone());
            result.push(pair.to_vec());
            let __sifr_try_res: Result<(), IndexError> = (|| {
                {
                    let __assign_value = value.clone();
                    {
                        let __index_raw = SifrInt::from_i64(0);
                        let __index_normalized = __index_raw
                            .normalize_index_or_len(prev_values.len());
                        if let Some(__elem) = prev_values.get_mut(__index_normalized) {
                            *__elem = __assign_value;
                        }
                    }
                }
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let _e = __sifr_try_err.clone();
                return result;
            }
        } else {
            prev_values.push(value.clone());
        }
    }
    result
}
fn batched<T: Clone + 'static>(
    data: &[T],
    n: SifrInt,
) -> Result<Vec<Vec<T>>, ValueError> {
    if (&n <= &SifrInt::from_i64(0)) {
        return Err(ValueError::new("batched: n must be > 0".to_string()));
    }
    let mut result: Vec<Vec<T>> = vec![];
    let mut current_batch: Vec<T> = vec![];
    for value in data.iter().cloned() {
        current_batch.push(value.clone());
        if (&SifrInt::from(current_batch.len()) == &n) {
            result.push(current_batch.to_vec());
            current_batch = vec![];
        }
    }
    if (&SifrInt::from(current_batch.len()) > &SifrInt::from_i64(0)) {
        result.push(current_batch.to_vec());
    }
    Ok(result)
}
fn accumulate<T: Clone + 'static + __SifrAdd>(
    data: Box<dyn Iterator<Item = T>>,
    initial: Option<T>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut state: Vec<T> = vec![];
            if let Some(initial) = initial {
                state.push(initial.clone());
                let initial_value: Option<T> = {
                    let __sifr_checked_read_collection = &state;
                    let __sifr_checked_read_index = SifrInt::from_i64(0);
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(initial_value) = initial_value {
                    __sifr_yielder.suspend(initial_value.clone()).await;
                }
            }
            for item in data {
                if (&SifrInt::from(state.len()) == &SifrInt::from_i64(0)) {
                    state.push(item.clone());
                } else {
                    let prev: Option<T> = {
                        let __sifr_checked_read_collection = &state;
                        let __sifr_checked_read_index = SifrInt::from_i64(0);
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(prev) = prev {
                        let next_val: T = __SifrAdd::__sifr_add(prev, item);
                        let __sifr_try_res: Result<(), IndexError> = (|| {
                            {
                                let __assign_value = next_val.clone();
                                {
                                    let __index_raw = SifrInt::from_i64(0);
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(state.len());
                                    if let Some(__elem) = state.get_mut(__index_normalized) {
                                        *__elem = __assign_value;
                                    } else {
                                        return Err(
                                            IndexError::new("collection index out of range".to_string()),
                                        );
                                    }
                                }
                            }
                            Ok(())
                        })();
                        if let Err(__sifr_try_err) = __sifr_try_res {
                            let _e = __sifr_try_err.clone();
                            return;
                        }
                    }
                }
                let current: Option<T> = {
                    let __sifr_checked_read_collection = &state;
                    let __sifr_checked_read_index = SifrInt::from_i64(0);
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(current) = current {
                    __sifr_yielder.suspend(current.clone()).await;
                }
            }
        }),
    )
}
fn cycle<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    n: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut saved: Vec<T> = vec![];
            let mut emitted: SifrInt = SifrInt::from_i64(0);
            if &n <= &SifrInt::from_i64(0) {
                return;
            }
            for value in data {
                saved.push(value.clone());
                __sifr_yielder.suspend(value.clone()).await;
                emitted = &emitted + &SifrInt::from_i64(1);
                if (&emitted >= &n) {
                    return;
                }
            }
            while (&emitted < &n)
                && (&SifrInt::from(saved.len()) > &SifrInt::from_i64(0))
            {
                for repeated in saved.iter().cloned() {
                    __sifr_yielder.suspend(repeated.clone()).await;
                    emitted = &emitted + &SifrInt::from_i64(1);
                    if (&emitted >= &n) {
                        return;
                    }
                }
            }
        }),
    )
}
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() })
        );
        i = &i + &SifrInt::from_i64(1);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IndexError {
    message: String,
}
impl IndexError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for IndexError {}
fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual
        .push(
            format!(
                "{:?}", chain(& vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
                vec![SifrInt::from_i64(3)]]).collect::< Vec < _ >> ()
            )
                .as_str() == "[1, 2, 3]".to_string().as_str(),
        );
    actual
        .push(
            format!(
                "{:?}", pairwise(& (vec![SifrInt::from_i64(1), SifrInt::from_i64(2),
                SifrInt::from_i64(3), SifrInt::from_i64(4)]).into_iter().collect::< Vec <
                _ >> ())
            )
                .as_str() == "[[1, 2], [2, 3], [3, 4]]".to_string().as_str(),
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
            format!(
                "{:?}", accumulate(Box::new(vec![SifrInt::from_i64(1),
                SifrInt::from_i64(2), SifrInt::from_i64(3)] .into_iter()), None)
                .collect::< Vec < _ >> ()
            )
                .as_str() == "[1, 3, 6]".to_string().as_str(),
        );
    actual
        .push(
            format!(
                "{:?}", cycle(Box::new(vec![SifrInt::from_i64(5), SifrInt::from_i64(6)]
                .into_iter()), SifrInt::from_i64(5)).collect::< Vec < _ >> ()
            )
                .as_str() == "[5, 6, 5, 6, 5]".to_string().as_str(),
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
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
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
