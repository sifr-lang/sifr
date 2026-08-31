// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.itertools ---
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
fn combinations_with_replacement<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    r: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<Vec<T>>| {
            let materialized: Vec<T> = _collect_iterator(Box::new(data));
            let size: SifrInt = SifrInt::from(materialized.len());
            if &r < &SifrInt::from_i64(0) {
                return;
            }
            if &r == &SifrInt::from_i64(0) {
                __sifr_yielder.suspend(vec![]).await;
                return;
            }
            if &size == &SifrInt::from_i64(0) {
                return;
            }
            let mut indices: Vec<SifrInt> = vec![];
            let mut index: SifrInt = SifrInt::from_i64(0);
            while (&index < &r) {
                indices.push(SifrInt::from_i64(0));
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
                    if (&current != &(&size - &SifrInt::from_i64(1))) {
                        break;
                    }
                    position = &position - &SifrInt::from_i64(1);
                }
                if (&position < &SifrInt::from_i64(0)) {
                    return;
                }
                let next_index: Option<SifrInt> = {
                    let __sifr_checked_read_collection = &indices;
                    let __sifr_checked_read_index = position.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                let Some(next_index) = next_index.clone() else {
                    return;
                };
                let next_value: SifrInt = &next_index + &SifrInt::from_i64(1);
                let mut cursor: SifrInt = position.clone();
                while (&cursor < &r) {
                    let __sifr_try_res: Result<(), IndexError> = (|| {
                        {
                            let __assign_value = next_value.clone();
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
fn accumulate<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static + ::std::ops::Add<Output = T>,
>(data: Box<dyn Iterator<Item = T>>, initial: Option<T>) -> Box<dyn Iterator<Item = T>> {
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
                        let next_val: T = prev + item;
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
fn compress<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    selectors: Box<dyn Iterator<Item = bool>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for (value, selector) in Box::new(
                data.zip(selectors).map(|__zip_item| (__zip_item.0, __zip_item.1)),
            ) {
                if selector {
                    __sifr_yielder.suspend(value.clone()).await;
                }
            }
        }),
    )
}
fn dropwhile<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut dropping: bool = true;
            for val in data {
                if dropping {
                    if !pred(&val) {
                        dropping = false;
                        __sifr_yielder.suspend(val.clone()).await;
                    }
                } else {
                    __sifr_yielder.suspend(val.clone()).await;
                }
            }
        }),
    )
}
fn takewhile<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for val in data {
                if !pred(&val) {
                    return;
                }
                __sifr_yielder.suspend(val.clone()).await;
            }
        }),
    )
}
fn filterfalse<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for val in data {
                if !pred(&val) {
                    __sifr_yielder.suspend(val.clone()).await;
                }
            }
        }),
    )
}
fn zip_longest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: Box<dyn Iterator<Item = T>>,
    b: Box<dyn Iterator<Item = T>>,
    fill: &T,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let fill = fill.clone();
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<Vec<T>>| {
            let mut left: Box<dyn Iterator<Item = T>> = a;
            let mut right: Box<dyn Iterator<Item = T>> = b;
            loop {
                let left_value: Option<T> = left.next();
                let right_value: Option<T> = right.next();
                if (left_value == None) && (right_value == None) {
                    return;
                }
                let mut pair: Vec<T> = vec![];
                if let Some(left_value) = left_value {
                    pair.push(left_value.clone());
                } else {
                    pair.push(fill.clone());
                }
                if let Some(right_value) = right_value {
                    pair.push(right_value.clone());
                } else {
                    pair.push(fill.clone());
                }
                __sifr_yielder.suspend(pair.clone()).await;
            }
        }),
    )
}
fn cycle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    n: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut saved: Vec<T> = vec![];
            let mut emitted: SifrInt = SifrInt::from_i64(0);
            for value in data {
                if (&emitted >= &n) {
                    return;
                }
                saved.push(value.clone());
                let current: Option<T> = {
                    let __sifr_index_list = &saved;
                    let __sifr_index_i = SifrInt::from(saved.len())
                        - SifrInt::from_i64(1);
                    let __sifr_index_norm = __sifr_index_i
                        .normalize_index_or_len(__sifr_index_list.len());
                    __sifr_index_list.get(__sifr_index_norm).cloned()
                };
                if let Some(current) = current {
                    __sifr_yielder.suspend(current.clone()).await;
                    emitted = &emitted + &SifrInt::from_i64(1);
                }
            }
            let size: SifrInt = SifrInt::from(saved.len());
            while (&emitted < &n) && (&size > &SifrInt::from_i64(0)) {
                let index: SifrInt = emitted.floor_mod_known_nonzero(&size);
                let repeated: Option<T> = {
                    let __sifr_checked_read_collection = &saved;
                    let __sifr_checked_read_index = index.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(repeated) = repeated {
                    __sifr_yielder.suspend(repeated.clone()).await;
                }
                emitted = &emitted + &SifrInt::from_i64(1);
            }
        }),
    )
}
// --- end stdlib ---

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

impl ::std::error::Error for IndexError {
}

fn lt3(x: SifrInt) -> bool {
    &x < &SifrInt::from_i64(3)
}

fn add2(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}

fn main() {
    let mut acc_it: Box<dyn Iterator<Item = SifrInt>> = accumulate(Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)].into_iter()), None);
    assert!((acc_it.next() == Some(SifrInt::from_i64(1))));
    assert!((format!("{:?}", acc_it.collect::<Vec<_>>()) == "[3, 6, 10]"));
    assert!((format!("{:?}", compress(Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)].into_iter()), Box::new(vec![true, false, true, false].into_iter())).collect::<Vec<_>>()) == "[1, 3]"));
    assert!((format!("{:?}", dropwhile(|__arg0| lt3((__arg0).clone()), Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(1)].into_iter())).collect::<Vec<_>>()) == "[3, 1]"));
    assert!((format!("{:?}", takewhile(|__arg0| lt3((__arg0).clone()), Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(1)].into_iter())).collect::<Vec<_>>()) == "[1, 2]"));
    assert!((format!("{:?}", filterfalse(|__arg0| lt3((__arg0).clone()), Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(1)].into_iter())).collect::<Vec<_>>()) == "[3]"));
    assert!((format!("{:?}", zip_longest(Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2)].into_iter()), Box::new(vec![SifrInt::from_i64(9)].into_iter()), &SifrInt::from_i64(0)).collect::<Vec<_>>()) == "[[1, 9], [2, 0]]"));
    let mut cyc: Box<dyn Iterator<Item = SifrInt>> = cycle(Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)].into_iter()), SifrInt::from_i64(5));
    assert!((cyc.next() == Some(SifrInt::from_i64(1))));
    assert!((format!("{:?}", cyc.collect::<Vec<_>>()) == "[2, 3, 1, 2]"));
    assert!((format!("{:?}", starmap(|__arg0, __arg1| add2((__arg0).clone(), (__arg1).clone()), Box::new(vec![(SifrInt::from_i64(2), SifrInt::from_i64(3)), (SifrInt::from_i64(4), SifrInt::from_i64(5))].into_iter())).collect::<Vec<_>>()) == "[5, 9]"));
    assert!((format!("{:?}", product(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]], SifrInt::from_i64(2)).collect::<Vec<_>>()) == "[[1, 1], [1, 2], [2, 1], [2, 2]]"));
    assert!((format!("{:?}", product(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]], -&SifrInt::from_i64(1)).collect::<Vec<_>>()) == "[]"));
    assert!((format!("{:?}", permutations(Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)].into_iter()), Some(SifrInt::from_i64(2))).collect::<Vec<_>>()) == "[[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]]"));
    assert!((format!("{:?}", combinations(Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)].into_iter()), SifrInt::from_i64(2)).collect::<Vec<_>>()) == "[[1, 2], [1, 3], [2, 3]]"));
    assert!((format!("{:?}", combinations_with_replacement(Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2)].into_iter()), SifrInt::from_i64(2)).collect::<Vec<_>>()) == "[[1, 1], [1, 2], [2, 2]]"));
    println!("parity_ext_extended_itertools_lazy_surface_demo: ok");
}
