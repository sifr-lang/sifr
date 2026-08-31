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
fn _collect_iterator<T: Clone + 'static>(data: Box<dyn Iterator<Item = T>>) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data {
        collected.push(item.clone());
    }
    collected
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
fn repeat<T: Clone + 'static>(value: T, times: SifrInt) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let holder: Vec<T> = vec![value.clone()];
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&i < &times) {
                if (&SifrInt::from(holder.len()) > &SifrInt::from_i64(0)) {
                    let current: Option<T> = {
                        let __sifr_checked_read_collection = &holder;
                        let __sifr_checked_read_index = SifrInt::from_i64(0);
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current {
                        __sifr_yielder.suspend(current.clone()).await;
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn _islice_impl<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start: SifrInt,
    stop: SifrInt,
    unbounded: bool,
    step: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut index: SifrInt = SifrInt::from_i64(0);
            let mut next_yield: SifrInt = start.clone();
            for value in data {
                if !unbounded && (&index >= &stop) {
                    return;
                }
                if &index == &next_yield {
                    __sifr_yielder.suspend(value.clone()).await;
                    next_yield = &next_yield + &step;
                }
                index = &index + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn islice<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start_or_stop: SifrInt,
    slice_args: &[Option<SifrInt>],
) -> Result<Box<dyn Iterator<Item = T>>, ValueError> {
    if (&SifrInt::from(slice_args.len()) > &SifrInt::from_i64(2)) {
        return Err(
            ValueError::new(
                "islice: expected at most stop and step after start".to_string(),
            ),
        );
    }
    let mut actual_start: SifrInt = SifrInt::from_i64(0);
    let mut actual_stop: SifrInt = start_or_stop.clone();
    let mut unbounded: bool = false;
    let mut actual_step: SifrInt = SifrInt::from_i64(1);
    let mut argument_index: SifrInt = SifrInt::from_i64(0);
    for argument in slice_args.iter().cloned() {
        if (&argument_index == &SifrInt::from_i64(0)) {
            actual_start = start_or_stop.clone();
            if (argument.is_none()) {
                unbounded = true;
            } else {
                if let Some(argument) = argument.clone() {
                    actual_stop = argument.clone();
                }
            }
        } else {
            if let Some(argument) = argument.clone() {
                actual_step = argument.clone();
            }
        }
        argument_index = &argument_index + &SifrInt::from_i64(1);
    }
    if (&actual_start < &SifrInt::from_i64(0)) {
        return Err(ValueError::new("islice: indices must be non-negative".to_string()));
    }
    if !unbounded && (&actual_stop < &SifrInt::from_i64(0)) {
        return Err(ValueError::new("islice: indices must be non-negative".to_string()));
    }
    if (&actual_step <= &SifrInt::from_i64(0)) {
        return Err(
            ValueError::new("islice: step must be greater than zero".to_string()),
        );
    }
    Ok(
        _islice_impl(
            Box::new(data),
            actual_start.clone(),
            actual_stop.clone(),
            unbounded,
            actual_step.clone(),
        ),
    )
}
fn count(start: SifrInt, step: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
            let mut current: SifrInt = start.clone();
            loop {
                __sifr_yielder.suspend(current.clone()).await;
                current = &current + &step;
            }
        }),
    )
}
fn product<T: Clone + 'static>(
    iterables: &[Vec<T>],
    repeat: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let iterables = iterables.to_vec();
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<Vec<T>>| {
            if &repeat < &SifrInt::from_i64(0) {
                return;
            }
            let mut pools: Vec<Vec<T>> = vec![];
            let mut repetition: SifrInt = SifrInt::from_i64(0);
            while (&repetition < &repeat) {
                for iterable in iterables.iter().cloned() {
                    pools.push(iterable.to_vec());
                }
                repetition = &repetition + &SifrInt::from_i64(1);
            }
            if &SifrInt::from(pools.len()) == &SifrInt::from_i64(0) {
                __sifr_yielder.suspend(vec![]).await;
                return;
            }
            for pool in pools.iter().cloned() {
                if &SifrInt::from(pool.len()) == &SifrInt::from_i64(0) {
                    return;
                }
            }
            let mut indices: Vec<SifrInt> = vec![];
            for _pool in pools.iter().cloned() {
                indices.push(SifrInt::from_i64(0));
            }
            let mut finished: bool = false;
            while !finished {
                let mut row: Vec<T> = vec![];
                let mut pool_index: SifrInt = SifrInt::from_i64(0);
                while (&pool_index < &SifrInt::from(pools.len())) {
                    let pool_value: Option<Vec<T>> = {
                        let __sifr_checked_read_collection = &pools;
                        let __sifr_checked_read_index = pool_index.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let value_index: Option<SifrInt> = {
                        let __sifr_checked_read_collection = &indices;
                        let __sifr_checked_read_index = pool_index.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let (Some(pool_value), Some(value_index)) = (
                        pool_value,
                        value_index.clone(),
                    ) else {
                        return;
                    };
                    let value: Option<T> = {
                        let __sifr_checked_read_collection = &pool_value;
                        let __sifr_checked_read_index = value_index.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let Some(value) = value else {
                        return;
                    };
                    row.push(value.clone());
                    pool_index = &pool_index + &SifrInt::from_i64(1);
                }
                __sifr_yielder.suspend(row.to_vec()).await;
                let mut position: SifrInt = &SifrInt::from(pools.len())
                    - &SifrInt::from_i64(1);
                let mut advanced: bool = false;
                while (&position >= &SifrInt::from_i64(0)) && !advanced {
                    let current_pool: Option<Vec<T>> = {
                        let __sifr_checked_read_collection = &pools;
                        let __sifr_checked_read_index = position.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let current_index: Option<SifrInt> = {
                        let __sifr_checked_read_collection = &indices;
                        let __sifr_checked_read_index = position.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let (Some(current_pool), Some(current_index)) = (
                        current_pool,
                        current_index.clone(),
                    ) else {
                        return;
                    };
                    let next_index: SifrInt = &current_index + &SifrInt::from_i64(1);
                    if (&next_index < &SifrInt::from(current_pool.len())) {
                        let __sifr_try_res: Result<(), IndexError> = (|| {
                            {
                                let __assign_value = next_index.clone();
                                {
                                    let __index_raw = position.clone();
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(indices.len());
                                    if let Some(__elem) = indices.get_mut(__index_normalized) {
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
                        advanced = true;
                    } else {
                        let __sifr_try_res: Result<(), IndexError> = (|| {
                            {
                                let __assign_value = SifrInt::from_i64(0);
                                {
                                    let __index_raw = position.clone();
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(indices.len());
                                    if let Some(__elem) = indices.get_mut(__index_normalized) {
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
                        position = &position - &SifrInt::from_i64(1);
                    }
                }
                if !advanced {
                    finished = true;
                }
            }
        }),
    )
}
fn combinations<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    r: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<Vec<T>>| {
            let materialized: Vec<T> = _collect_iterator(Box::new(data));
            let size: SifrInt = SifrInt::from(materialized.len());
            if (&r < &SifrInt::from_i64(0)) || (&r > &size) {
                return;
            }
            if &r == &SifrInt::from_i64(0) {
                __sifr_yielder.suspend(vec![]).await;
                return;
            }
            let mut indices: Vec<SifrInt> = vec![];
            let mut index: SifrInt = SifrInt::from_i64(0);
            while (&index < &r) {
                indices.push(index.clone());
                index = &index + &SifrInt::from_i64(1);
            }
            loop {
                let mut row: Vec<T> = vec![];
                for source_index in indices.iter().cloned() {
                    let value: Option<T> = {
                        let __sifr_checked_read_collection = &materialized;
                        let __sifr_checked_read_index = source_index.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let Some(value) = value else {
                        return;
                    };
                    row.push(value.clone());
                }
                __sifr_yielder.suspend(row.to_vec()).await;
                let mut position: SifrInt = &r - &SifrInt::from_i64(1);
                while (&position >= &SifrInt::from_i64(0)) {
                    let current: Option<SifrInt> = {
                        let __sifr_checked_read_collection = &indices;
                        let __sifr_checked_read_index = position.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let Some(current) = current.clone() else {
                        return;
                    };
                    if (&current != &(&(&position + &size) - &r)) {
                        break;
                    }
                    position = &position - &SifrInt::from_i64(1);
                }
                if (&position < &SifrInt::from_i64(0)) {
                    return;
                }
                let current: Option<SifrInt> = {
                    let __sifr_checked_read_collection = &indices;
                    let __sifr_checked_read_index = position.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                let Some(current) = current.clone() else {
                    return;
                };
                let mut next_position: SifrInt = &current + &SifrInt::from_i64(1);
                let __sifr_try_res: Result<(), IndexError> = (|| {
                    {
                        let __assign_value = next_position.clone();
                        {
                            let __index_raw = position.clone();
                            let __index_normalized = __index_raw
                                .normalize_index_or_len(indices.len());
                            if let Some(__elem) = indices.get_mut(__index_normalized) {
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
                let mut cursor: SifrInt = &position.clone() + &SifrInt::from_i64(1);
                while (&cursor < &r) {
                    let previous: Option<SifrInt> = {
                        let __sifr_checked_read_collection = &indices;
                        let __sifr_checked_read_index = &cursor - &SifrInt::from_i64(1);
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let Some(previous) = previous.clone() else {
                        return;
                    };
                    next_position = &previous + &SifrInt::from_i64(1);
                    let __sifr_try_res: Result<(), IndexError> = (|| {
                        {
                            let __assign_value = next_position.clone();
                            {
                                let __index_raw = cursor.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(indices.len());
                                if let Some(__elem) = indices.get_mut(__index_normalized) {
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
                    cursor = &cursor + &SifrInt::from_i64(1);
                }
            }
        }),
    )
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
fn odds(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&i < &limit) {
                if (&i.floor_mod_known_nonzero(&SifrInt::from_i64(2))
                    == &SifrInt::from_i64(1))
                {
                    __sifr_yielder.suspend(i.clone()).await;
                }
                i = &i + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4)
    ];
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
    assert!((odd_it.next().is_none()));
    assert!(
        (format!("{:?}", Box::new((vec![SifrInt::from_i64(1), SifrInt::from_i64(2)])
        .into_iter().zip((vec!["a".to_string(), "b".to_string()]).into_iter()).map(|
        __zip_item | (__zip_item.0, __zip_item.1))).collect::< Vec < _ >> ()) ==
        "[(1, \"a\"), (2, \"b\")]")
    );
    assert!(
        (format!("{:?}", Box::new((vec!["x".to_string(), "y".to_string()]).into_iter()
        .enumerate().map(| __pair | (SifrInt::from(__pair.0) + SifrInt::from_i64(4),
        __pair.1))).collect::< Vec < _ >> ()) == "[(4, \"x\"), (5, \"y\")]")
    );
    assert!(
        (format!("{:?}", Box::new((vec![SifrInt::from_i64(9), SifrInt::from_i64(8),
        SifrInt::from_i64(7)]).into_iter().rev()).collect::< Vec < _ >> ()) ==
        "[7, 8, 9]")
    );
    assert!(
        (format!("{:?}", chain(& vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
        vec![SifrInt::from_i64(3)]]).collect::< Vec < _ >> ()) == "[1, 2, 3]")
    );
    assert!(
        (format!("{:?}", repeat(SifrInt::from_i64(5), SifrInt::from_i64(3)).collect::<
        Vec < _ >> ()) == "[5, 5, 5]")
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let sliced: Box<dyn Iterator<Item = SifrInt>> = islice(
            Box::new(
                vec![
                    SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30),
                    SifrInt::from_i64(40), SifrInt::from_i64(50)
                ]
                    .into_iter(),
            ),
            SifrInt::from_i64(1),
            &vec![Some(SifrInt::from_i64(5)), Some(SifrInt::from_i64(2))],
        )?;
        assert!((format!("{:?}", sliced.collect::< Vec < _ >> ()) == "[20, 40]"));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        assert!(false, "{}", e.message.clone());
    }
    let mut counter: Box<dyn Iterator<Item = SifrInt>> = count(
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
    );
    assert!((counter.next() == Some(SifrInt::from_i64(2))));
    assert!((counter.next() == Some(SifrInt::from_i64(5))));
    assert!((counter.next() == Some(SifrInt::from_i64(8))));
    let combos: Vec<Vec<SifrInt>> = combinations(
            Box::new(
                vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)]
                    .into_iter(),
            ),
            SifrInt::from_i64(2),
        )
        .collect::<Vec<_>>();
    assert!((format!("{:?}", combos) == "[[1, 2], [1, 3], [2, 3]]"));
    let prods: Vec<Vec<SifrInt>> = product(
            &vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]],
            SifrInt::from_i64(2),
        )
        .collect::<Vec<_>>();
    assert!((format!("{:?}", prods) == "[[1, 1], [1, 2], [2, 1], [2, 2]]"));
    println!("iter_iterator_basics_closure_demo: ok");
}
