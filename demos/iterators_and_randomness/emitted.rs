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
fn _collect_iterator<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data {
        collected.push(item.clone());
    }
    collected
}
fn chain<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.clone();
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
fn _islice_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut index: SifrInt = SifrInt::from_i64(0);
            let mut next_yield: SifrInt = start.clone();
            for value in data {
                if &index >= &stop {
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
fn islice<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start_or_stop: SifrInt,
    stop: Option<SifrInt>,
    step: SifrInt,
) -> Result<Box<dyn Iterator<Item = T>>, ValueError> {
    let mut actual_start: SifrInt = SifrInt::from_i64(0);
    let mut actual_stop: SifrInt = start_or_stop.clone();
    if let Some(stop) = stop.clone() {
        actual_start = start_or_stop.clone();
        actual_stop = stop.clone();
    }
    if (&actual_start < &SifrInt::from_i64(0)) || (&actual_stop < &SifrInt::from_i64(0))
    {
        return Err(ValueError::new("islice: indices must be non-negative".to_string()));
    }
    if (&step <= &SifrInt::from_i64(0)) {
        return Err(
            ValueError::new("islice: step must be greater than zero".to_string()),
        );
    }
    Ok(
        _islice_impl(
            Box::new(data),
            (actual_start).clone(),
            (actual_stop).clone(),
            (step).clone(),
        ),
    )
}
fn product<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
    repeat: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let iterables = iterables.clone();
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<Vec<T>>| {
            if &repeat < &SifrInt::from_i64(0) {
                return;
            }
            let mut pools: Vec<Vec<T>> = vec![];
            let mut repetition: SifrInt = SifrInt::from_i64(0);
            while (&repetition < &repeat) {
                for iterable in iterables.iter().cloned() {
                    pools.push(iterable.clone());
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
                __sifr_yielder.suspend(row.clone()).await;
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
fn permutations<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    r: Option<SifrInt>,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<Vec<T>>| {
            let materialized: Vec<T> = _collect_iterator(Box::new(data));
            let mut target: SifrInt = SifrInt::from(materialized.len());
            if let Some(r) = r.clone() {
                target = r;
            }
            let size: SifrInt = SifrInt::from(materialized.len());
            if (&target < &SifrInt::from_i64(0)) || (&target > &size) {
                return;
            }
            if &target == &SifrInt::from_i64(0) {
                __sifr_yielder.suspend(vec![]).await;
                return;
            }
            let mut indices: Vec<SifrInt> = vec![];
            let mut index: SifrInt = SifrInt::from_i64(0);
            while (&index < &size) {
                indices.push(index.clone());
                index = &index + &SifrInt::from_i64(1);
            }
            let mut cycles: Vec<SifrInt> = vec![];
            index = SifrInt::from_i64(0);
            while (&index < &target) {
                cycles.push(&size - &index);
                index = &index + &SifrInt::from_i64(1);
            }
            let mut first: Vec<T> = vec![];
            index = SifrInt::from_i64(0);
            while (&index < &target) {
                let source_index: Option<SifrInt> = {
                    let __sifr_checked_read_collection = &indices;
                    let __sifr_checked_read_index = index.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                let Some(source_index) = source_index.clone() else {
                    return;
                };
                let value: Option<T> = {
                    let __sifr_checked_read_collection = &materialized;
                    let __sifr_checked_read_index = source_index.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                let Some(value) = value else {
                    return;
                };
                first.push(value.clone());
                index = &index + &SifrInt::from_i64(1);
            }
            __sifr_yielder.suspend(first.clone()).await;
            loop {
                let mut position: SifrInt = &target - &SifrInt::from_i64(1);
                let mut produced: bool = false;
                while (&position >= &SifrInt::from_i64(0)) && !produced {
                    let remaining: Option<SifrInt> = {
                        let __sifr_checked_read_collection = &cycles;
                        let __sifr_checked_read_index = position.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    let Some(remaining) = remaining.clone() else {
                        return;
                    };
                    let next_remaining: SifrInt = &remaining - &SifrInt::from_i64(1);
                    let __sifr_try_res: Result<(), IndexError> = (|| {
                        {
                            let __assign_value = next_remaining.clone();
                            {
                                let __index_raw = position.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(cycles.len());
                                if let Some(__elem) = cycles.get_mut(__index_normalized) {
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
                    if (&next_remaining == &SifrInt::from_i64(0)) {
                        let rotated: Option<SifrInt> = {
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
                        let Some(rotated) = rotated.clone() else {
                            return;
                        };
                        let mut cursor: SifrInt = position.clone();
                        while (&cursor < &(&size - &SifrInt::from_i64(1))) {
                            let shifted: Option<SifrInt> = {
                                let __sifr_checked_read_collection = &indices;
                                let __sifr_checked_read_index = &cursor
                                    + &SifrInt::from_i64(1);
                                let __sifr_checked_read_normalized = __sifr_checked_read_index
                                    .normalize_index_or_len(
                                        __sifr_checked_read_collection.len(),
                                    );
                                __sifr_checked_read_collection
                                    .get(__sifr_checked_read_normalized)
                                    .cloned()
                            };
                            let Some(shifted) = shifted.clone() else {
                                return;
                            };
                            let __sifr_try_res: Result<(), IndexError> = (|| {
                                {
                                    let __assign_value = shifted.clone();
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
                        let __sifr_try_res: Result<(), IndexError> = (|| {
                            {
                                let __assign_value = rotated.clone();
                                {
                                    let __index_raw = &size - &SifrInt::from_i64(1);
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
                            {
                                let __assign_value = &size - &position;
                                {
                                    let __index_raw = position.clone();
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(cycles.len());
                                    if let Some(__elem) = cycles.get_mut(__index_normalized) {
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
                    } else {
                        let swap_position: SifrInt = &size - &next_remaining;
                        let left_index: Option<SifrInt> = {
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
                        let right_index: Option<SifrInt> = {
                            let __sifr_checked_read_collection = &indices;
                            let __sifr_checked_read_index = swap_position.clone();
                            let __sifr_checked_read_normalized = __sifr_checked_read_index
                                .normalize_index_or_len(
                                    __sifr_checked_read_collection.len(),
                                );
                            __sifr_checked_read_collection
                                .get(__sifr_checked_read_normalized)
                                .cloned()
                        };
                        let (Some(left_index), Some(right_index)) = (
                            left_index.clone(),
                            right_index.clone(),
                        ) else {
                            return;
                        };
                        let left_value: SifrInt = left_index;
                        let right_value: SifrInt = right_index;
                        let __sifr_try_res: Result<(), IndexError> = (|| {
                            {
                                let __assign_value = right_value.clone();
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
                            {
                                let __assign_value = left_value.clone();
                                {
                                    let __index_raw = swap_position.clone();
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
                        let mut row: Vec<T> = vec![];
                        let mut row_index: SifrInt = SifrInt::from_i64(0);
                        while (&row_index < &target) {
                            let item_index: Option<SifrInt> = {
                                let __sifr_checked_read_collection = &indices;
                                let __sifr_checked_read_index = row_index.clone();
                                let __sifr_checked_read_normalized = __sifr_checked_read_index
                                    .normalize_index_or_len(
                                        __sifr_checked_read_collection.len(),
                                    );
                                __sifr_checked_read_collection
                                    .get(__sifr_checked_read_normalized)
                                    .cloned()
                            };
                            let Some(item_index) = item_index.clone() else {
                                return;
                            };
                            let item: Option<T> = {
                                let __sifr_checked_read_collection = &materialized;
                                let __sifr_checked_read_index = item_index.clone();
                                let __sifr_checked_read_normalized = __sifr_checked_read_index
                                    .normalize_index_or_len(
                                        __sifr_checked_read_collection.len(),
                                    );
                                __sifr_checked_read_collection
                                    .get(__sifr_checked_read_normalized)
                                    .cloned()
                            };
                            let Some(item) = item else {
                                return;
                            };
                            row.push(item.clone());
                            row_index = &row_index + &SifrInt::from_i64(1);
                        }
                        __sifr_yielder.suspend(row.clone()).await;
                        produced = true;
                    }
                }
                if !produced {
                    return;
                }
            }
        }),
    )
}
fn combinations<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
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
                __sifr_yielder.suspend(row.clone()).await;
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
fn starmap<
    A: Clone + ::std::fmt::Display + PartialOrd + 'static,
    B: Clone + ::std::fmt::Display + PartialOrd + 'static,
    R: Clone + ::std::fmt::Display + PartialOrd + 'static,
>(
    func: impl Fn(&A, &B) -> R + Send + Sync + 'static,
    pairs: Box<dyn Iterator<Item = (A, B)>>,
) -> Box<dyn Iterator<Item = R>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<R>| {
            for (first, second) in pairs {
                __sifr_yielder.suspend(func(&first, &second)).await;
            }
        }),
    )
}
fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_word_to_unit_float(value: SifrInt) -> f64 {
    ::sifr_stdlib::random::random_word_to_unit_float(
        ::sifr_runtime::interop::SifrIntBridge::from(value),
    )
}
fn random_seed() -> SifrInt {
    ::sifr_stdlib::random::random_seed().into_sifr_int()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<SifrInt> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn random_module_state_index() -> SifrInt {
    ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .cloned()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode(s: &String) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &String,
    altchars: &String,
    wrapcol: SifrInt,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_encode_opts(
            s,
            altchars,
            ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_opts(
    s: &String,
    altchars: &String,
    validate: bool,
    ignorechars: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &String) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &String) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &String) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn ceil(x: f64) -> SifrInt {
    ::sifr_stdlib::math::ceil(x).into_sifr_int()
}
fn log(x: f64) -> f64 {
    ::sifr_stdlib::math::log(x)
}
fn cbrt(x: f64) -> f64 {
    ::sifr_stdlib::math::cbrt(x)
}
fn sin(x: f64) -> f64 {
    ::sifr_stdlib::math::sin(x)
}
fn cos(x: f64) -> f64 {
    ::sifr_stdlib::math::cos(x)
}
fn tan(x: f64) -> f64 {
    ::sifr_stdlib::math::tan(x)
}
fn pow_val(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::pow_val(x, y)
}
fn min_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::min_val(a, b)
}
fn max_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::max_val(a, b)
}
fn round_val(x: f64) -> SifrInt {
    ::sifr_stdlib::math::round_val(x).into_sifr_int()
}
fn asin(x: f64) -> f64 {
    ::sifr_stdlib::math::asin(x)
}
fn acos(x: f64) -> f64 {
    ::sifr_stdlib::math::acos(x)
}
fn atan(x: f64) -> f64 {
    ::sifr_stdlib::math::atan(x)
}
fn atan2(y: f64, x: f64) -> f64 {
    ::sifr_stdlib::math::atan2(y, x)
}
fn sinh(x: f64) -> f64 {
    ::sifr_stdlib::math::sinh(x)
}
fn cosh(x: f64) -> f64 {
    ::sifr_stdlib::math::cosh(x)
}
fn tanh(x: f64) -> f64 {
    ::sifr_stdlib::math::tanh(x)
}
fn log10(x: f64) -> f64 {
    ::sifr_stdlib::math::log10(x)
}
fn log2(x: f64) -> f64 {
    ::sifr_stdlib::math::log2(x)
}
fn exp2(x: f64) -> f64 {
    ::sifr_stdlib::math::exp2(x)
}
fn degrees(x: f64) -> f64 {
    ::sifr_stdlib::math::degrees(x)
}
fn radians(x: f64) -> f64 {
    ::sifr_stdlib::math::radians(x)
}
fn isnan(x: f64) -> bool {
    ::sifr_stdlib::math::isnan(x)
}
fn isinf(x: f64) -> bool {
    ::sifr_stdlib::math::isinf(x)
}
fn trunc(x: f64) -> SifrInt {
    ::sifr_stdlib::math::trunc(x).into_sifr_int()
}
fn copysign(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::copysign(x, y)
}
fn signbit(x: f64) -> bool {
    ::sifr_stdlib::math::signbit(x)
}
fn fmod(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmod(x, y)
}
fn remainder(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::remainder(x, y)
}
fn hypot(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::hypot(x, y)
}
fn fma(x: f64, y: f64, z: f64) -> f64 {
    ::sifr_stdlib::math::fma(x, y, z)
}
fn fmax(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmax(x, y)
}
fn fmin(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmin(x, y)
}
fn exp(x: f64) -> f64 {
    ::sifr_stdlib::math::exp(x)
}
fn expm1(x: f64) -> f64 {
    ::sifr_stdlib::math::expm1(x)
}
fn log1p(x: f64) -> f64 {
    ::sifr_stdlib::math::log1p(x)
}
fn fabs(x: f64) -> f64 {
    ::sifr_stdlib::math::fabs(x)
}
fn isfinite(x: f64) -> bool {
    ::sifr_stdlib::math::isfinite(x)
}
fn isnormal(x: f64) -> bool {
    ::sifr_stdlib::math::isnormal(x)
}
fn issubnormal(x: f64) -> bool {
    ::sifr_stdlib::math::issubnormal(x)
}
fn acosh(x: f64) -> f64 {
    ::sifr_stdlib::math::acosh(x)
}
fn asinh(x: f64) -> f64 {
    ::sifr_stdlib::math::asinh(x)
}
fn atanh(x: f64) -> f64 {
    ::sifr_stdlib::math::atanh(x)
}
fn isqrt(n: SifrInt) -> SifrInt {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .into_sifr_int()
}
fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::dist(p, q)
}
fn fsum_impl(data: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::fsum(data)
}
fn sumprod_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::sumprod(p, q)
}
fn erf(x: f64) -> f64 {
    ::sifr_stdlib::math::erf(x)
}
fn erfc(x: f64) -> f64 {
    ::sifr_stdlib::math::erfc(x)
}
fn gamma(x: f64) -> f64 {
    ::sifr_stdlib::math::gamma(x)
}
fn lgamma(x: f64) -> f64 {
    ::sifr_stdlib::math::lgamma(x)
}
fn frexp(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::frexp(x)
}
fn ldexp(m: f64, e: SifrInt) -> f64 {
    ::sifr_stdlib::math::ldexp(m, ::sifr_runtime::interop::SifrIntBridge::from(e))
}
fn modf(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::modf(x)
}
fn nextafter(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::nextafter(x, y)
}
fn ulp(x: f64) -> f64 {
    ::sifr_stdlib::math::ulp(x)
}
fn factorial(n: SifrInt) -> SifrInt {
    if &n < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(2);
    while &i <= &n {
        result = &result * &i;
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn gcd(a: SifrInt, b: SifrInt) -> SifrInt {
    let mut x: SifrInt = a.clone();
    let mut y: SifrInt = b.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    while (&y != &SifrInt::from_i64(0)) {
        let temp: SifrInt = y.clone();
        y = x.floor_mod_known_nonzero(&y);
        x = temp;
    }
    x.clone()
}
fn lcm(a: SifrInt, b: SifrInt) -> SifrInt {
    if &a == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &b == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let g: SifrInt = gcd((a).clone(), (b).clone());
    if &g == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut x: SifrInt = a.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    let mut y: SifrInt = b.clone();
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    &x.floor_div_known_nonzero(&g) * &y
}
fn comb(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    if &k == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    if &k == &n {
        return SifrInt::from_i64(1);
    }
    let mut r: SifrInt = k.clone();
    if &r > &(&n - &k) {
        r = &n - &k;
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &r) {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if (&divisor == &SifrInt::from_i64(0)) {
            return SifrInt::from_i64(0);
        }
        result = result.floor_div_known_nonzero(&divisor);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn perm(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &k {
        result = &result * &(&n - &i);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn log_base(x: f64, base: f64) -> f64 {
    log(x) / log(base)
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0_f64) {
        return false;
    }
    if abs_tol < (0.0_f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if isnan(a) || isnan(b) {
        return false;
    }
    if isinf(a) || isinf(b) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0_f64) {
        diff = (0.0_f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0_f64) {
        a_abs = (0.0_f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0_f64) {
        b_abs = (0.0_f64) - b_abs;
    }
    let mut larger_abs: f64 = a_abs;
    if b_abs > larger_abs {
        larger_abs = b_abs;
    }
    let mut rel_bound: f64 = rel_tol * larger_abs;
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    diff <= rel_bound
}
fn prod(data: &Vec<SifrInt>) -> SifrInt {
    let mut result: SifrInt = SifrInt::from_i64(1);
    for val in data.iter().cloned() {
        result = &result * &val;
    }
    result.clone()
}
fn _copy_float_list(data: &Vec<f64>) -> Vec<f64> {
    let mut out: Vec<f64> = vec![];
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    dist_impl(_copy_float_list(p), _copy_float_list(q))
}
fn fsum(data: &Vec<f64>) -> f64 {
    fsum_impl(_copy_float_list(data))
}
fn sumprod(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    sumprod_impl(_copy_float_list(p), _copy_float_list(q))
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = frexp(x);
    let m: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
fn __const__MT_N() -> SifrInt {
    SifrInt::from_i64(624)
}
fn __const__MT_M() -> SifrInt {
    SifrInt::from_i64(397)
}
fn __const__MT_MATRIX_A() -> SifrInt {
    SifrInt::from_i64(2567483615)
}
fn __const__MT_UPPER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483648)
}
fn __const__MT_LOWER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483647)
}
fn __const__MT_F() -> SifrInt {
    SifrInt::from_i64(1812433253)
}
fn __const__MT_WORD_MASK() -> SifrInt {
    SifrInt::from_i64(4294967295)
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandomState {
    version: SifrInt,
    state_words: Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {
    fn new(
        version: SifrInt,
        state_words: Vec<SifrInt>,
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Self {
        let __sifr_field_init_0: SifrInt = version.clone();
        let __sifr_field_init_1: Vec<SifrInt> = state_words;
        let __sifr_field_init_2: SifrInt = index.clone();
        let __sifr_field_init_3: Option<f64> = gauss_next;
        Self {
            version: __sifr_field_init_0,
            state_words: __sifr_field_init_1,
            index: __sifr_field_init_2,
            gauss_next: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandom {
    _state_words: Vec<SifrInt>,
    _index: SifrInt,
    _gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn new(seed_value: Option<SifrInt>) -> Self {
        let normalized_seed: SifrInt = _normalize_seed_input((seed_value).clone());
        let __sifr_field_init_0: Vec<SifrInt> = _seed_words_from_seed(
            (normalized_seed).clone(),
        );
        let __sifr_field_init_1: SifrInt = __const__MT_N().clone();
        let __sifr_field_init_2: Option<f64> = None;
        Self {
            _state_words: __sifr_field_init_0,
            _index: __sifr_field_init_1,
            _gauss_next: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn seed(&mut self, seed_value: &Option<SifrInt>) {
        let normalized_seed: SifrInt = _normalize_seed_input(
            (seed_value.clone()).clone(),
        );
        self._state_words = _seed_words_from_seed((normalized_seed).clone());
        self._index = __const__MT_N().clone();
        self._gauss_next = None;
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _twist(&mut self) {
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&SifrInt::from_i64(0) <= &i)
            && (&i < &SifrInt::from(self._state_words.len()))
        {
            let y: SifrInt = &(&_state_word_at(&self._state_words, (i).clone())
                & &__const__MT_UPPER_MASK())
                + &(&_state_word_at(
                    &self._state_words,
                    (&i + &SifrInt::from_i64(1))
                        .floor_mod_known_nonzero(&__const__MT_N()),
                ) & &__const__MT_LOWER_MASK());
            let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
            if (&y.floor_mod_known_nonzero(&SifrInt::from_i64(2))
                != &SifrInt::from_i64(0))
            {
                x_a = &x_a ^ &__const__MT_MATRIX_A();
            }
            let new_word: SifrInt = &_state_word_at(
                &self._state_words,
                (&i + &__const__MT_M()).floor_mod_known_nonzero(&__const__MT_N()),
            ) ^ &x_a;
            {
                let __assign_value = &new_word & &__const__MT_WORD_MASK();
                {
                    let __index_raw = i.clone();
                    let __index_normalized = __index_raw
                        .normalize_index_or_len(self._state_words.len());
                    if let Some(__elem) = self._state_words.get_mut(__index_normalized) {
                        *__elem = __assign_value;
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        self._index = SifrInt::from_i64(0);
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _next_u32(&mut self) -> SifrInt {
        if (&self._index.clone() >= &__const__MT_N()) {
            self._twist();
        }
        let mut y: SifrInt = _state_word_at(&self._state_words, self._index.clone());
        self._index = &self._index.clone() + &SifrInt::from_i64(1);
        y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(2048));
        y = &y ^ &(&(&y * &SifrInt::from_i64(128)) & &SifrInt::from_i64(2636928640));
        y = &y ^ &(&(&y * &SifrInt::from_i64(32768)) & &SifrInt::from_i64(4022730752));
        y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(262144));
        &y & &__const__MT_WORD_MASK()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn random(&mut self) -> f64 {
        random_word_to_unit_float(self._next_u32())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn uniform(&mut self, minimum: f64, maximum: f64) -> f64 {
        minimum + ((maximum - minimum) * self.random())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randrange(
        &mut self,
        start: &SifrInt,
        stop: &Option<SifrInt>,
        step: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if (&step.clone() == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let mut actual_start: SifrInt = start.clone();
        let mut actual_stop: SifrInt = start.clone();
        if (stop.clone() == None) {
            actual_start = SifrInt::from_i64(0);
        } else {
            if let Some(stop) = stop.as_ref() {
                actual_stop = stop.clone();
            }
        }
        let width: SifrInt = &actual_stop - &actual_start;
        if (&step.clone() > &SifrInt::from_i64(0)) {
            if (&width <= &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if (&width >= &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        }
        let mut abs_width: SifrInt = width.clone();
        if &abs_width < &SifrInt::from_i64(0) {
            abs_width = &SifrInt::from_i64(0) - &abs_width;
        }
        let mut abs_step: SifrInt = step.clone();
        if &abs_step < &SifrInt::from_i64(0) {
            abs_step = &SifrInt::from_i64(0) - &abs_step;
        }
        if (&abs_step == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
            .floor_div_known_nonzero(&abs_step);
        if (&count <= &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        if (&count == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        let pick: SifrInt = self._next_u32().floor_mod_known_nonzero(&count);
        Ok(&actual_start + &(&pick * step))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randint(
        &mut self,
        minimum: &SifrInt,
        maximum: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if *minimum > *maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        self.randrange(
            minimum,
            &Some((maximum + &SifrInt::from_i64(1)).clone()),
            &SifrInt::from_i64(1),
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getrandbits(&mut self, k: &SifrInt) -> Result<SifrInt, ValueError> {
        if (&k.clone() < &SifrInt::from_i64(0)) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut bits_left: SifrInt = k.clone();
        while (&bits_left > &SifrInt::from_i64(0)) {
            let word: SifrInt = self._next_u32();
            let mut take: SifrInt = SifrInt::from_i64(32);
            if (&bits_left < &SifrInt::from_i64(32)) {
                take = bits_left.clone();
            }
            let mut mask: SifrInt = SifrInt::from_i64(0);
            let mut shifted_result: SifrInt = result;
            let mut shift_index: SifrInt = SifrInt::from_i64(0);
            while (&shift_index < &take) {
                mask = &(&mask * &SifrInt::from_i64(2)) + &SifrInt::from_i64(1);
                shifted_result = &shifted_result * &SifrInt::from_i64(2);
                shift_index = &shift_index + &SifrInt::from_i64(1);
            }
            result = &shifted_result | &(&word & &mask);
            bits_left = &bits_left - &take;
        }
        Ok(result.clone())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randbytes(&mut self, n: &SifrInt) -> Result<Vec<u8>, ValueError> {
        if (&n.clone() < &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *n {
            let byte_value: SifrInt = &self._next_u32() & &SifrInt::from_i64(255);
            values.push(byte_value.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                __out
                    .push(
                        __pair
                            .1
                            .try_to_u8()
                            .map_err(|_error| ValueError {
                                message: format!(
                                    "byte out of range at index {}: {}", __pair.0, * __pair.1
                                ),
                            })?,
                    );
            }
            Ok::<Vec<u8>, ValueError>(__out)
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let cached: Option<f64> = self._gauss_next;
        if let Some(cached) = cached {
            self._gauss_next = None;
            return mu + (sigma * cached);
        }
        let mut u1: f64 = self.random();
        if u1 <= (0.0_f64) {
            u1 = 0.000000000001_f64;
        }
        let u2: f64 = self.random();
        let radius: f64 = sqrt(-(2.0_f64) * log(u1));
        let theta: f64 = ((2.0_f64) * PI) * u2;
        let z0: f64 = radius * cos(theta);
        let z1: f64 = radius * sin(theta);
        let next_cached: Option<f64> = Some(z1);
        self._gauss_next = next_cached;
        mu + (sigma * z0)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getstate(&self) -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
        __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
            SifrInt::from_i64(3),
            _clone_words(&self._state_words),
            self._index.clone(),
            self._gauss_next,
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn setstate(
        &mut self,
        state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        if (&state.version.clone() != &SifrInt::from_i64(3)) {
            return Err(ValueError::new("setstate: unsupported version".to_string()));
        }
        if (&SifrInt::from(state.state_words.len()) != &__const__MT_N()) {
            return Err(
                ValueError::new("setstate: state_words must have length 624".to_string()),
            );
        }
        if (&state.index.clone() < &SifrInt::from_i64(0))
            || (&state.index.clone() > &__const__MT_N())
        {
            return Err(
                ValueError::new("setstate: index must be in range [0, 624]".to_string()),
            );
        }
        let mut normalized: Vec<SifrInt> = vec![];
        for word in state.state_words.clone().iter().cloned() {
            if (&word < &SifrInt::from_i64(0)) || (&word > &__const__MT_WORD_MASK()) {
                return Err(ValueError::new("setstate: word out of range".to_string()));
            }
            normalized.push(&word & &__const__MT_WORD_MASK());
        }
        self._state_words = normalized;
        self._index = state.index.clone();
        self._gauss_next = state.gauss_next;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2erandom_x2eSystemRandom {}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn new() -> Self {
        Self {}
    }
}
impl ::std::default::Default for __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn default() -> Self {
        Self::new()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn seed(&self, _seed_value: &Option<SifrInt>) {}
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn getstate(
        &self,
    ) -> Result<__SifrStdlib_sifr_x2erandom_x2eRandomState, ValueError> {
        Err(ValueError::new("SystemRandom does not support getstate".to_string()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn setstate(
        &self,
        _state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        Err(ValueError::new("SystemRandom does not support setstate".to_string()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn random(&self) -> f64 {
        random_float()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn uniform(&self, minimum: f64, maximum: f64) -> f64 {
        random_uniform(minimum, maximum)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randrange(
        &self,
        start: &SifrInt,
        stop: &Option<SifrInt>,
        step: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        let mut actual_start: SifrInt = start.clone();
        let mut actual_stop: SifrInt = start.clone();
        if (stop.clone() == None) {
            actual_start = SifrInt::from_i64(0);
        } else {
            if let Some(stop) = stop.as_ref() {
                actual_stop = stop.clone();
            }
        }
        random_randrange(
            (actual_start).clone(),
            (actual_stop).clone(),
            (step.clone()).clone(),
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randint(
        &self,
        minimum: &SifrInt,
        maximum: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if *minimum > *maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        Ok(random_int((minimum.clone()).clone(), (maximum.clone()).clone()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn getrandbits(&self, k: &SifrInt) -> Result<SifrInt, ValueError> {
        if (&k.clone() < &SifrInt::from_i64(0)) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *k {
            let mut bit: SifrInt = SifrInt::from_i64(0);
            if (random_float() >= (0.5_f64)) {
                bit = SifrInt::from_i64(1);
            }
            result = &(&result * &SifrInt::from_i64(2)) + &bit;
            i = &i + &SifrInt::from_i64(1);
        }
        Ok(result.clone())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn gauss(&self, mu: f64, sigma: f64) -> f64 {
        random_gauss(mu, sigma)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randbytes(&self, n: &SifrInt) -> Result<Vec<u8>, ValueError> {
        if (&n.clone() < &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *n {
            let value: SifrInt = random_int(
                SifrInt::from_i64(0),
                SifrInt::from_i64(255),
            );
            values.push(value.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                __out
                    .push(
                        __pair
                            .1
                            .try_to_u8()
                            .map_err(|_error| ValueError {
                                message: format!(
                                    "byte out of range at index {}: {}", __pair.0, * __pair.1
                                ),
                            })?,
                    );
            }
            Ok::<Vec<u8>, ValueError>(__out)
        }
    }
}
fn _state_word_at(words: &Vec<SifrInt>, index: SifrInt) -> SifrInt {
    let value: Option<SifrInt> = {
        let __sifr_checked_read_collection = &words;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(value) = value.clone() {
        return value;
    }
    SifrInt::from_i64(0)
}
fn _clone_words(words: &Vec<SifrInt>) -> Vec<SifrInt> {
    let mut copied: Vec<SifrInt> = vec![];
    for word in words.iter().cloned() {
        copied.push(word.clone());
    }
    copied
}
fn _normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
    if let Some(seed_value) = seed_value.clone() {
        return seed_value.clone();
    }
    random_seed()
}
fn _seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
    let mut words: Vec<SifrInt> = vec![];
    words.push(&seed_value & &__const__MT_WORD_MASK());
    let mut i: SifrInt = SifrInt::from_i64(1);
    while (&i < &__const__MT_N()) {
        let prev: SifrInt = _state_word_at(&words, &i - &SifrInt::from_i64(1));
        let next_word: SifrInt = &(&(&__const__MT_F()
            * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1073741824))))
            + &i) & &__const__MT_WORD_MASK();
        words.push(next_word.clone());
        i = &i + &SifrInt::from_i64(1);
    }
    words
}
fn _build_state_from_module_storage() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
        SifrInt::from_i64(3),
        random_module_state_words(),
        random_module_state_index(),
        random_module_state_gauss_next(),
    )
}
fn _store_state_into_module_storage(state: &__SifrStdlib_sifr_x2erandom_x2eRandomState) {
    let _set_result: Result<(), ValueError> = random_module_set_state(
        &_clone_words(&state.state_words.clone()),
        state.index.clone(),
        state.gauss_next,
    );
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<SifrInt> = random_module_state_words();
    if &SifrInt::from(words.len()) == &__const__MT_N() {
        return;
    }
    let bootstrap: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(5489)),
    );
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> __SifrStdlib_sifr_x2erandom_x2eRandom {
    _ensure_module_state_initialized();
    let mut r: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r
            .setstate(&_build_state_from_module_storage());
        let _ = _set_result;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    r
}
fn _sync_module_random(generator: &mut __SifrStdlib_sifr_x2erandom_x2eRandom) {
    _store_state_into_module_storage(&generator.getstate());
}
fn seed(seed_value: Option<SifrInt>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        (seed_value).clone(),
    );
    _sync_module_random(&mut generator);
}
fn getstate() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    _ensure_module_state_initialized();
    _build_state_from_module_storage()
}
fn setstate(
    state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
) -> Result<(), ValueError> {
    let mut probe: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let result: Result<(), ValueError> = probe.setstate(state);
    _sync_module_random(&mut probe);
    result
}
fn randint(minimum: SifrInt, maximum: SifrInt) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.randint(&minimum, &maximum);
    _sync_module_random(&mut generator);
    value
}
fn random() -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.random();
    _sync_module_random(&mut generator);
    value
}
fn uniform(minimum: f64, maximum: f64) -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.uniform(minimum, maximum);
    _sync_module_random(&mut generator);
    value
}
fn randrange(
    start: SifrInt,
    stop: Option<SifrInt>,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.randrange(&start, &stop, &step);
    _sync_module_random(&mut generator);
    value
}
fn getrandbits(k: SifrInt) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.getrandbits(&k);
    _sync_module_random(&mut generator);
    value
}
fn gauss(mu: f64, sigma: f64) -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.gauss(mu, sigma);
    _sync_module_random(&mut generator);
    value
}
fn choice<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
) -> Result<T, ValueError> {
    let item_count: SifrInt = SifrInt::from(items.len());
    if (&item_count == &SifrInt::from_i64(0)) {
        return Err(ValueError::new("choice: items must not be empty".to_string()));
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let index: SifrInt = generator._next_u32().floor_mod_known_nonzero(&item_count);
    let picked: Option<T> = {
        let __sifr_checked_read_collection = &items;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    _sync_module_random(&mut generator);
    if let Some(picked) = picked {
        return Ok(picked);
    }
    Err(ValueError::new("choice: index out of range".to_string()))
}
fn choices<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    k: SifrInt,
) -> Result<Vec<T>, ValueError> {
    if &k <= &SifrInt::from_i64(0) {
        return Ok(vec![]);
    }
    let item_count: SifrInt = SifrInt::from(items.len());
    if (&item_count == &SifrInt::from_i64(0)) {
        return Err(ValueError::new("choices: items must not be empty".to_string()));
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &k) {
        let index: SifrInt = generator._next_u32().floor_mod_known_nonzero(&item_count);
        let picked: Option<T> = {
            let __sifr_checked_read_collection = &items;
            let __sifr_checked_read_index = index.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone());
        } else {
            return Err(ValueError::new("choices: index out of range".to_string()));
        }
        i = &i + &SifrInt::from_i64(1);
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn sample<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    k: SifrInt,
) -> Result<Vec<T>, ValueError> {
    if (&k < &SifrInt::from_i64(0)) {
        return Err(ValueError::new("sample: k must be >= 0".to_string()));
    }
    if (&k > &SifrInt::from(items.len())) {
        return Err(ValueError::new("sample larger than population".to_string()));
    }
    let mut pool: Vec<T> = vec![];
    for item in items.iter().cloned() {
        pool.push(item.clone());
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut remaining: SifrInt = SifrInt::from(pool.len());
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &k) {
        if (&remaining == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("sample larger than population".to_string()));
        }
        let pick_index: SifrInt = generator
            ._next_u32()
            .floor_mod_known_nonzero(&remaining);
        let picked: Option<T> = {
            let __sifr_checked_read_collection = &pool;
            let __sifr_checked_read_index = pick_index.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone());
        }
        let last: Option<T> = {
            let __sifr_checked_read_collection = &pool;
            let __sifr_checked_read_index = &remaining - &SifrInt::from_i64(1);
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(last) = last {
            if (&SifrInt::from_i64(0) <= &pick_index)
                && (&pick_index < &SifrInt::from(pool.len()))
            {
                {
                    let __assign_value = last.clone();
                    {
                        let __index_raw = pick_index.clone();
                        let __index_normalized = __index_raw
                            .normalize_index_or_len(pool.len());
                        if let Some(__elem) = pool.get_mut(__index_normalized) {
                            *__elem = __assign_value;
                        }
                    }
                }
            }
        }
        remaining = &remaining - &SifrInt::from_i64(1);
        i = &i + &SifrInt::from_i64(1);
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn shuffle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(items: &mut Vec<T>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let n: SifrInt = SifrInt::from(items.len());
    if (&n > &SifrInt::from_i64(1)) {
        let mut i: SifrInt = &n - &SifrInt::from_i64(1);
        while (&i > &SifrInt::from_i64(0)) {
            let divisor: SifrInt = &i + &SifrInt::from_i64(1);
            if (&divisor == &SifrInt::from_i64(0)) {
                return;
            }
            let j: SifrInt = generator._next_u32().floor_mod_known_nonzero(&divisor);
            let left: Option<T> = {
                let __sifr_checked_read_collection = &items;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let right: Option<T> = {
                let __sifr_checked_read_collection = &items;
                let __sifr_checked_read_index = j.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(left) = left {
                if let Some(right) = right {
                    if (&SifrInt::from_i64(0) <= &i)
                        && (&i < &SifrInt::from(items.len()))
                    {
                        {
                            let __assign_value = right.clone();
                            {
                                let __index_raw = i.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(items.len());
                                if let Some(__elem) = items.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                    if (&SifrInt::from_i64(0) <= &j)
                        && (&j < &SifrInt::from(items.len()))
                    {
                        {
                            let __assign_value = left.clone();
                            {
                                let __index_raw = j.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(items.len());
                                if let Some(__elem) = items.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
    }
    _sync_module_random(&mut generator);
}
fn randbytes(n: SifrInt) -> Result<Vec<u8>, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<Vec<u8>, ValueError> = generator.randbytes(&n);
    _sync_module_random(&mut generator);
    value
}
fn compare_digest(a: &String, b: &String) -> bool {
    a == b
}
fn token_hex(nbytes: SifrInt) -> String {
    let hex_chars: String = "0123456789abcdef".to_string();
    let mut result: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &(&nbytes * &SifrInt::from_i64(2))) {
        let idx: SifrInt = random_int(SifrInt::from_i64(0), SifrInt::from_i64(15));
        let ch: Option<String> = ({
            let __sifr_string_source = &hex_chars;
            let __sifr_string_index = idx.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn randbits(k: SifrInt) -> Result<SifrInt, ValueError> {
    if (&k < &SifrInt::from_i64(0)) {
        return Err(ValueError::new("randbits: number of bits must be >= 0".to_string()));
    }
    let mut result: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &k) {
        let bit: SifrInt = random_int(SifrInt::from_i64(0), SifrInt::from_i64(1));
        result = &(&result * &SifrInt::from_i64(2)) + &bit;
        i = &i + &SifrInt::from_i64(1);
    }
    Ok(result.clone())
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParseError {
    message: String,
}
impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ParseError {}
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Doubler {}
impl Doubler {
    fn new() -> Self {
        Self {}
    }
}
impl ::std::default::Default for Doubler {
    fn default() -> Self {
        Self::new()
    }
}
impl Doubler {
    fn __call__(&self, x: &SifrInt) -> SifrInt {
        x * &SifrInt::from_i64(2)
    }
}
fn add(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}
fn main() {
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("chain(*iterables) = "); __sifr_concat
        .push_str((format!("{:?}", chain(& vec![vec![SifrInt::from_i64(1)],
        vec![SifrInt::from_i64(2)], vec![SifrInt::from_i64(3)],
        vec![SifrInt::from_i64(4)]]).collect::< Vec < _ >> ())).as_str()); __sifr_concat
        }
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
            Some(SifrInt::from_i64(5)),
            SifrInt::from_i64(2),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(28usize +
            0usize); __sifr_concat.push_str("islice(start, stop, step) = ");
            __sifr_concat.push_str((format!("{:?}", sliced.collect::< Vec < _ >> ()))
            .as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("islice error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("product(repeat=2) = "); __sifr_concat
        .push_str((format!("{:?}", product(& vec![vec![SifrInt::from_i64(1),
        SifrInt::from_i64(2)]], SifrInt::from_i64(2)).collect::< Vec < _ >> ()))
        .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("permutations(r=2) = "); __sifr_concat
        .push_str((format!("{:?}", permutations(Box::new(vec![SifrInt::from_i64(1),
        SifrInt::from_i64(2), SifrInt::from_i64(3)] .into_iter()),
        Some(SifrInt::from_i64(2))).collect::< Vec < _ >> ())).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("combinations(r=2) = "); __sifr_concat
        .push_str((format!("{:?}", combinations(Box::new(vec![SifrInt::from_i64(1),
        SifrInt::from_i64(2), SifrInt::from_i64(3)] .into_iter()), SifrInt::from_i64(2))
        .collect::< Vec < _ >> ())).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(22usize + 0usize);
        __sifr_concat.push_str("starmap(add, pairs) = "); __sifr_concat
        .push_str((format!("{:?}", starmap(| __arg0, __arg1 | add((__arg0).clone(),
        (__arg1).clone()), Box::new(vec![(SifrInt::from_i64(2), SifrInt::from_i64(3)),
        (SifrInt::from_i64(4), SifrInt::from_i64(5))] .into_iter())).collect::< Vec < _
        >> ())).as_str()); __sifr_concat }
    );
    let doubler: Doubler = Doubler::new();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(25usize + 0usize);
        __sifr_concat.push_str("callable object direct = "); __sifr_concat
        .push_str((format!("{}", doubler.__call__(& SifrInt::from_i64(4)))).as_str());
        __sifr_concat }
    );
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5)
    ];
    shuffle(&mut items);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(25usize + 0usize);
        __sifr_concat.push_str("shuffle(mut items) len = "); __sifr_concat
        .push_str((format!("{}", SifrInt::from(items.len()))).as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let picked: SifrInt = choice(&items)?;
        let many: Vec<SifrInt> = choices(&items, SifrInt::from_i64(3))?;
        let rr: SifrInt = randrange(SifrInt::from_i64(10), None, SifrInt::from_i64(1))?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
            0usize); __sifr_concat.push_str("choice(items) ok = "); __sifr_concat
            .push_str((format!("{}", (& picked >= & SifrInt::from_i64(1)) && (& picked <=
            & SifrInt::from_i64(5)))).as_str()); __sifr_concat }
        );
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(26usize +
            0usize); __sifr_concat.push_str("choices(items, k=3) len = "); __sifr_concat
            .push_str((format!("{}", SifrInt::from(many.len()))).as_str()); __sifr_concat
            }
        );
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
            0usize); __sifr_concat.push_str("randrange(10) ok = "); __sifr_concat
            .push_str((format!("{}", (& rr >= & SifrInt::from_i64(0)) && (& rr < &
            SifrInt::from_i64(10)))).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("random error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(25usize + 0usize);
        __sifr_concat.push_str("secrets.compare_digest = "); __sifr_concat
        .push_str((format!("{}", compare_digest(& "abc".to_string(), & "abc"
        .to_string()))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(27usize + 0usize);
        __sifr_concat.push_str("secrets.token_hex(4) len = "); __sifr_concat
        .push_str((format!("{}", SifrInt::from(token_hex(SifrInt::from_i64(4)).chars()
        .count()))).as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let bits: SifrInt = randbits(SifrInt::from_i64(16))?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(26usize +
            0usize); __sifr_concat.push_str("secrets.randbits(16) ok = "); __sifr_concat
            .push_str((format!("{}", & bits >= & SifrInt::from_i64(0))).as_str());
            __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(15usize +
            0usize); __sifr_concat.push_str("secrets error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
}
