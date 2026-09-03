// src/main.rs
use ::sifr_runtime::SifrInt;
struct SifrGeneratedYielder<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
}
struct SifrGeneratedYieldFuture<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: Option<T>,
}
impl<T> Unpin for SifrGeneratedYieldFuture<T> {}
impl<T> ::std::future::Future for SifrGeneratedYieldFuture<T> {
    type Output = ();
    fn poll(
        self: ::std::pin::Pin<&mut Self>,
        _: &mut ::std::task::Context<'_>,
    ) -> ::std::task::Poll<()> {
        let state = self.get_mut();
        let Some(value) = state.value.take() else {
            return ::std::task::Poll::Ready(());
        };
        sifr_generated_store_suspended(&state.slot, value);
        ::std::task::Poll::Pending
    }
}
impl<T> SifrGeneratedYielder<T> {
    fn suspend(&self, value: T) -> SifrGeneratedYieldFuture<T> {
        SifrGeneratedYieldFuture {
            slot: ::std::sync::Arc::clone(&self.slot),
            value: Some(value),
        }
    }
}
fn sifr_generated_store_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: T,
) {
    match slot.lock() {
        Ok(mut state) => *state = Some(value),
        Err(poisoned) => *poisoned.into_inner() = Some(value),
    }
}
fn sifr_generated_take_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
) -> Option<T> {
    match slot.lock() {
        Ok(mut state) => state.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    }
}
struct SifrGeneratedGenerator<T> {
    producer: Option<::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>>,
    yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    complete: bool,
}
impl<T> SifrGeneratedGenerator<T> {
    fn new<
        F: FnOnce(SifrGeneratedYielder<T>) -> Fut + 'static,
        Fut: ::std::future::Future<Output = ()> + 'static,
    >(
        factory: F,
    ) -> Self {
        let yielded = ::std::sync::Arc::new(::std::sync::Mutex::new(None));
        let producer = factory(SifrGeneratedYielder {
            slot: ::std::sync::Arc::clone(&yielded),
        });
        Self {
            producer: Some(Box::pin(producer)),
            yielded,
            complete: false,
        }
    }
}
impl<T> Iterator for SifrGeneratedGenerator<T> {
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
            let mut context = ::std::task::Context::from_waker(::std::task::Waker::noop());
            ::std::future::Future::poll(producer.as_mut(), &mut context).is_ready()
        };
        let yielded = sifr_generated_take_suspended(&self.yielded);
        if completed {
            self.complete = true;
            self.producer = None;
        }
        yielded
    }
}
pub trait SifrGeneratedAdd: Sized {
    #[must_use]
    fn sifr_generated_add(self, rhs: Self) -> Self;
}
impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {
    fn sifr_generated_add(self, rhs: Self) -> Self {
        self + rhs
    }
}
impl SifrGeneratedAdd for String {
    fn sifr_generated_add(mut self, rhs: Self) -> Self {
        self.push_str(&rhs);
        self
    }
}
fn sifr_generated_collect_iterator<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
) -> Vec<T> {
    let mut collected: Vec<T> = Vec::new();
    for item in data {
        collected.push(item.clone());
    }
    collected
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn product<T: Clone + 'static>(
    iterables: &[Vec<T>],
    repeat: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let iterables = iterables.to_vec();
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
            if &repeat < &SifrInt::from_i64(0) {
                return;
            }
            let mut pools: Vec<Vec<T>> = Vec::new();
            let mut repetition: SifrInt = SifrInt::from_i64(0);
            while &repetition < &repeat {
                for iterable in iterables.iter().cloned() {
                    pools.push(iterable.to_vec());
                }
                repetition = &repetition + &SifrInt::from_i64(1);
            }
            if &SifrInt::from(pools.len()) == &SifrInt::from_i64(0) {
                sifr_generated_yielder.suspend(Vec::new()).await;
                return;
            }
            for pool in pools.iter().cloned() {
                if &SifrInt::from(pool.len()) == &SifrInt::from_i64(0) {
                    return;
                }
            }
            let mut indices: Vec<SifrInt> = Vec::new();
            for _pool in pools.iter().cloned() {
                indices.push(SifrInt::from_i64(0));
            }
            let mut finished: bool = false;
            while !finished {
                let mut row: Vec<T> = Vec::new();
                let mut pool_index: SifrInt = SifrInt::from_i64(0);
                while &pool_index < &SifrInt::from(pools.len()) {
                    let pool_value: Option<Vec<T>> = {
                        let sifr_generated_checked_read_collection = &pools;
                        let sifr_generated_checked_read_index = pool_index.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let value_index: Option<SifrInt> = {
                        let sifr_generated_checked_read_collection = &indices;
                        let sifr_generated_checked_read_index = pool_index.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let (Some(pool_value), Some(value_index_value_336ae61b280d8a15)) =
                        (pool_value, value_index.clone())
                    else {
                        return;
                    };
                    let value: Option<T> = {
                        let sifr_generated_checked_read_collection = &pool_value;
                        let sifr_generated_checked_read_index =
                            value_index_value_336ae61b280d8a15.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(value_value_7ce4fd9430e80cea) = value else {
                        return;
                    };
                    row.push(value_value_7ce4fd9430e80cea.clone());
                    pool_index = &pool_index + &SifrInt::from_i64(1);
                }
                sifr_generated_yielder.suspend(row.to_vec()).await;
                let mut position: SifrInt = &SifrInt::from(pools.len()) - &SifrInt::from_i64(1);
                let mut advanced: bool = false;
                while &position >= &SifrInt::from_i64(0) && !advanced {
                    let current_pool: Option<Vec<T>> = {
                        let sifr_generated_checked_read_collection = &pools;
                        let sifr_generated_checked_read_index = position.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let current_index: Option<SifrInt> = {
                        let sifr_generated_checked_read_collection = &indices;
                        let sifr_generated_checked_read_index = position.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let (
                        Some(current_pool_value_8d0aa685cb481a75),
                        Some(current_index_value_57667e3202daa6c5),
                    ) = (current_pool, current_index.clone())
                    else {
                        return;
                    };
                    let next_index: SifrInt =
                        &current_index_value_57667e3202daa6c5 + &SifrInt::from_i64(1);
                    if &next_index < &SifrInt::from(current_pool_value_8d0aa685cb481a75.len()) {
                        let sifr_generated_try_res: Result<(), IndexError> = (|| {
                            {
                                let sifr_generated_assign_value = next_index.clone();
                                {
                                    let sifr_generated_index_raw = position.clone();
                                    let sifr_generated_index_normalized = sifr_generated_index_raw
                                        .normalize_index_or_len(indices.len());
                                    if let Some(sifr_generated_elem) =
                                        indices.get_mut(sifr_generated_index_normalized)
                                    {
                                        *sifr_generated_elem = sifr_generated_assign_value;
                                    } else {
                                        return Err(IndexError::new(
                                            "collection index out of range".to_string(),
                                        ));
                                    }
                                }
                            }
                            Ok(())
                        })(
                        );
                        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                            let _e = sifr_generated_try_err.clone();
                            return;
                        }
                        advanced = true;
                    } else {
                        let sifr_generated_try_res: Result<(), IndexError> = (|| {
                            {
                                let sifr_generated_assign_value = SifrInt::from_i64(0);
                                {
                                    let sifr_generated_index_raw = position.clone();
                                    let sifr_generated_index_normalized = sifr_generated_index_raw
                                        .normalize_index_or_len(indices.len());
                                    if let Some(sifr_generated_elem) =
                                        indices.get_mut(sifr_generated_index_normalized)
                                    {
                                        *sifr_generated_elem = sifr_generated_assign_value;
                                    } else {
                                        return Err(IndexError::new(
                                            "collection index out of range".to_string(),
                                        ));
                                    }
                                }
                            }
                            Ok(())
                        })(
                        );
                        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                            let _e = sifr_generated_try_err.clone();
                            return;
                        }
                        position = &position - &SifrInt::from_i64(1);
                    }
                }
                if !advanced {
                    finished = true;
                }
            }
        },
    ))
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn permutations<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    r: Option<SifrInt>,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
            let materialized: Vec<T> = sifr_generated_collect_iterator(Box::new(data));
            let target: SifrInt = r
                .clone()
                .unwrap_or_else(|| SifrInt::from(materialized.len()));
            let size: SifrInt = SifrInt::from(materialized.len());
            if &target < &SifrInt::from_i64(0) || &target > &size {
                return;
            }
            if &target == &SifrInt::from_i64(0) {
                sifr_generated_yielder.suspend(Vec::new()).await;
                return;
            }
            let mut indices: Vec<SifrInt> = Vec::new();
            let mut index: SifrInt = SifrInt::from_i64(0);
            while &index < &size {
                indices.push(index.clone());
                index = &index + &SifrInt::from_i64(1);
            }
            let mut cycles: Vec<SifrInt> = Vec::new();
            index = SifrInt::from_i64(0);
            while &index < &target {
                cycles.push(&size - &index);
                index = &index + &SifrInt::from_i64(1);
            }
            let mut first: Vec<T> = Vec::new();
            index = SifrInt::from_i64(0);
            while &index < &target {
                let source_index: Option<SifrInt> = {
                    let sifr_generated_checked_read_collection = &indices;
                    let sifr_generated_checked_read_index = index.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(source_index_value_1402cfd57e11de6b) = source_index.clone() else {
                    return;
                };
                let value: Option<T> = {
                    let sifr_generated_checked_read_collection = &materialized;
                    let sifr_generated_checked_read_index =
                        source_index_value_1402cfd57e11de6b.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(value_value_7ce4fd9430e80cea) = value else {
                    return;
                };
                first.push(value_value_7ce4fd9430e80cea.clone());
                index = &index + &SifrInt::from_i64(1);
            }
            sifr_generated_yielder.suspend(first.to_vec()).await;
            loop {
                let mut position: SifrInt = &target - &SifrInt::from_i64(1);
                let mut produced: bool = false;
                while &position >= &SifrInt::from_i64(0) && !produced {
                    let remaining: Option<SifrInt> = {
                        let sifr_generated_checked_read_collection = &cycles;
                        let sifr_generated_checked_read_index = position.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(remaining_value_343f0419454a4243) = remaining.clone() else {
                        return;
                    };
                    let next_remaining: SifrInt =
                        &remaining_value_343f0419454a4243 - &SifrInt::from_i64(1);
                    let sifr_generated_try_res: Result<(), IndexError> = (|| {
                        {
                            let sifr_generated_assign_value = next_remaining.clone();
                            {
                                let sifr_generated_index_raw = position.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(cycles.len());
                                if let Some(sifr_generated_elem) =
                                    cycles.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                } else {
                                    return Err(IndexError::new(
                                        "collection index out of range".to_string(),
                                    ));
                                }
                            }
                        }
                        Ok(())
                    })();
                    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                        let _e = sifr_generated_try_err.clone();
                        return;
                    }
                    if &next_remaining == &SifrInt::from_i64(0) {
                        let rotated: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &indices;
                            let sifr_generated_checked_read_index = position.clone();
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        let Some(rotated_value_f64204a307abbb6a) = rotated.clone() else {
                            return;
                        };
                        let mut cursor: SifrInt = position.clone();
                        while &cursor < &(&size - &SifrInt::from_i64(1)) {
                            let shifted: Option<SifrInt> = {
                                let sifr_generated_checked_read_collection = &indices;
                                let sifr_generated_checked_read_index =
                                    &cursor + &SifrInt::from_i64(1);
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            let Some(shifted_value_7540578f579f2e86) = shifted.clone() else {
                                return;
                            };
                            let sifr_generated_try_res: Result<(), IndexError> = (|| {
                                {
                                    let sifr_generated_assign_value =
                                        shifted_value_7540578f579f2e86.clone();
                                    {
                                        let sifr_generated_index_raw = cursor.clone();
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
                                                .normalize_index_or_len(indices.len());
                                        if let Some(sifr_generated_elem) =
                                            indices.get_mut(sifr_generated_index_normalized)
                                        {
                                            *sifr_generated_elem = sifr_generated_assign_value;
                                        } else {
                                            return Err(IndexError::new(
                                                "collection index out of range".to_string(),
                                            ));
                                        }
                                    }
                                }
                                Ok(())
                            })(
                            );
                            if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                                let _e = sifr_generated_try_err.clone();
                                return;
                            }
                            cursor = &cursor + &SifrInt::from_i64(1);
                        }
                        let sifr_generated_try_res: Result<(), IndexError> = (|| {
                            {
                                let sifr_generated_assign_value =
                                    rotated_value_f64204a307abbb6a.clone();
                                {
                                    let sifr_generated_index_raw = &size - &SifrInt::from_i64(1);
                                    let sifr_generated_index_normalized = sifr_generated_index_raw
                                        .normalize_index_or_len(indices.len());
                                    if let Some(sifr_generated_elem) =
                                        indices.get_mut(sifr_generated_index_normalized)
                                    {
                                        *sifr_generated_elem = sifr_generated_assign_value;
                                    } else {
                                        return Err(IndexError::new(
                                            "collection index out of range".to_string(),
                                        ));
                                    }
                                }
                            }
                            {
                                let sifr_generated_assign_value = &size - &position;
                                {
                                    let sifr_generated_index_raw = position.clone();
                                    let sifr_generated_index_normalized = sifr_generated_index_raw
                                        .normalize_index_or_len(cycles.len());
                                    if let Some(sifr_generated_elem) =
                                        cycles.get_mut(sifr_generated_index_normalized)
                                    {
                                        *sifr_generated_elem = sifr_generated_assign_value;
                                    } else {
                                        return Err(IndexError::new(
                                            "collection index out of range".to_string(),
                                        ));
                                    }
                                }
                            }
                            Ok(())
                        })(
                        );
                        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                            let _e = sifr_generated_try_err.clone();
                            return;
                        }
                        position = &position - &SifrInt::from_i64(1);
                    } else {
                        let swap_position: SifrInt = &size - &next_remaining;
                        let left_index: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &indices;
                            let sifr_generated_checked_read_index = position.clone();
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        let right_index: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &indices;
                            let sifr_generated_checked_read_index = swap_position.clone();
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        let (
                            Some(left_index_value_0cbf618cd64fdba3),
                            Some(right_index_value_0d20c76177571432),
                        ) = (left_index.clone(), right_index.clone())
                        else {
                            return;
                        };
                        let left_value: SifrInt = left_index_value_0cbf618cd64fdba3;
                        let right_value: SifrInt = right_index_value_0d20c76177571432;
                        let sifr_generated_try_res: Result<(), IndexError> = (|| {
                            {
                                let sifr_generated_assign_value = right_value.clone();
                                {
                                    let sifr_generated_index_raw = position.clone();
                                    let sifr_generated_index_normalized = sifr_generated_index_raw
                                        .normalize_index_or_len(indices.len());
                                    if let Some(sifr_generated_elem) =
                                        indices.get_mut(sifr_generated_index_normalized)
                                    {
                                        *sifr_generated_elem = sifr_generated_assign_value;
                                    } else {
                                        return Err(IndexError::new(
                                            "collection index out of range".to_string(),
                                        ));
                                    }
                                }
                            }
                            {
                                let sifr_generated_assign_value = left_value.clone();
                                {
                                    let sifr_generated_index_raw = swap_position.clone();
                                    let sifr_generated_index_normalized = sifr_generated_index_raw
                                        .normalize_index_or_len(indices.len());
                                    if let Some(sifr_generated_elem) =
                                        indices.get_mut(sifr_generated_index_normalized)
                                    {
                                        *sifr_generated_elem = sifr_generated_assign_value;
                                    } else {
                                        return Err(IndexError::new(
                                            "collection index out of range".to_string(),
                                        ));
                                    }
                                }
                            }
                            Ok(())
                        })(
                        );
                        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                            let _e = sifr_generated_try_err.clone();
                            return;
                        }
                        let mut row: Vec<T> = Vec::new();
                        let mut row_index: SifrInt = SifrInt::from_i64(0);
                        while &row_index < &target {
                            let item_index: Option<SifrInt> = {
                                let sifr_generated_checked_read_collection = &indices;
                                let sifr_generated_checked_read_index = row_index.clone();
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            let Some(item_index_value_9a28a188f6a6f491) = item_index.clone() else {
                                return;
                            };
                            let item: Option<T> = {
                                let sifr_generated_checked_read_collection = &materialized;
                                let sifr_generated_checked_read_index =
                                    item_index_value_9a28a188f6a6f491.clone();
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            let Some(item_value_2841a0c596d6f426) = item else {
                                return;
                            };
                            row.push(item_value_2841a0c596d6f426.clone());
                            row_index = &row_index + &SifrInt::from_i64(1);
                        }
                        sifr_generated_yielder.suspend(row.to_vec()).await;
                        produced = true;
                    }
                }
                if !produced {
                    return;
                }
            }
        },
    ))
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn combinations<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    r: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
            let materialized: Vec<T> = sifr_generated_collect_iterator(Box::new(data));
            let size: SifrInt = SifrInt::from(materialized.len());
            if &r < &SifrInt::from_i64(0) || &r > &size {
                return;
            }
            if &r == &SifrInt::from_i64(0) {
                sifr_generated_yielder.suspend(Vec::new()).await;
                return;
            }
            let mut indices: Vec<SifrInt> = Vec::new();
            let mut index: SifrInt = SifrInt::from_i64(0);
            while &index < &r {
                indices.push(index.clone());
                index = &index + &SifrInt::from_i64(1);
            }
            loop {
                let mut row: Vec<T> = Vec::new();
                for source_index in indices.iter().cloned() {
                    let value: Option<T> = {
                        let sifr_generated_checked_read_collection = &materialized;
                        let sifr_generated_checked_read_index = source_index.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(value_value_7ce4fd9430e80cea) = value else {
                        return;
                    };
                    row.push(value_value_7ce4fd9430e80cea.clone());
                }
                sifr_generated_yielder.suspend(row.to_vec()).await;
                let mut position: SifrInt = &r - &SifrInt::from_i64(1);
                while &position >= &SifrInt::from_i64(0) {
                    let current: Option<SifrInt> = {
                        let sifr_generated_checked_read_collection = &indices;
                        let sifr_generated_checked_read_index = position.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(current_value_2a2e8a5afcc8d89a) = current.clone() else {
                        return;
                    };
                    if &current_value_2a2e8a5afcc8d89a != &(&(&position + &size) - &r) {
                        break;
                    }
                    position = &position - &SifrInt::from_i64(1);
                }
                if &position < &SifrInt::from_i64(0) {
                    return;
                }
                let current: Option<SifrInt> = {
                    let sifr_generated_checked_read_collection = &indices;
                    let sifr_generated_checked_read_index = position.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(current_value_2a2e8a5afcc8d89a) = current.clone() else {
                    return;
                };
                let mut next_position: SifrInt =
                    &current_value_2a2e8a5afcc8d89a + &SifrInt::from_i64(1);
                let sifr_generated_try_res: Result<(), IndexError> = (|| {
                    {
                        let sifr_generated_assign_value = next_position.clone();
                        {
                            let sifr_generated_index_raw = position.clone();
                            let sifr_generated_index_normalized =
                                sifr_generated_index_raw.normalize_index_or_len(indices.len());
                            if let Some(sifr_generated_elem) =
                                indices.get_mut(sifr_generated_index_normalized)
                            {
                                *sifr_generated_elem = sifr_generated_assign_value;
                            } else {
                                return Err(IndexError::new(
                                    "collection index out of range".to_string(),
                                ));
                            }
                        }
                    }
                    Ok(())
                })();
                if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                    let _e = sifr_generated_try_err.clone();
                    return;
                }
                let mut cursor: SifrInt = &position.clone() + &SifrInt::from_i64(1);
                while &cursor < &r {
                    let previous: Option<SifrInt> = {
                        let sifr_generated_checked_read_collection = &indices;
                        let sifr_generated_checked_read_index = &cursor - &SifrInt::from_i64(1);
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(previous_value_ec5f63ffe7e97248) = previous.clone() else {
                        return;
                    };
                    next_position = &previous_value_ec5f63ffe7e97248 + &SifrInt::from_i64(1);
                    let sifr_generated_try_res: Result<(), IndexError> = (|| {
                        {
                            let sifr_generated_assign_value = next_position.clone();
                            {
                                let sifr_generated_index_raw = cursor.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(indices.len());
                                if let Some(sifr_generated_elem) =
                                    indices.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                } else {
                                    return Err(IndexError::new(
                                        "collection index out of range".to_string(),
                                    ));
                                }
                            }
                        }
                        Ok(())
                    })();
                    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                        let _e = sifr_generated_try_err.clone();
                        return;
                    }
                    cursor = &cursor + &SifrInt::from_i64(1);
                }
            }
        },
    ))
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn combinations_with_replacement<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    r: SifrInt,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
            let materialized: Vec<T> = sifr_generated_collect_iterator(Box::new(data));
            let size: SifrInt = SifrInt::from(materialized.len());
            if &r < &SifrInt::from_i64(0) {
                return;
            }
            if &r == &SifrInt::from_i64(0) {
                sifr_generated_yielder.suspend(Vec::new()).await;
                return;
            }
            if &size == &SifrInt::from_i64(0) {
                return;
            }
            let mut indices: Vec<SifrInt> = Vec::new();
            let mut index: SifrInt = SifrInt::from_i64(0);
            while &index < &r {
                indices.push(SifrInt::from_i64(0));
                index = &index + &SifrInt::from_i64(1);
            }
            loop {
                let mut row: Vec<T> = Vec::new();
                for source_index in indices.iter().cloned() {
                    let value: Option<T> = {
                        let sifr_generated_checked_read_collection = &materialized;
                        let sifr_generated_checked_read_index = source_index.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(value_value_7ce4fd9430e80cea) = value else {
                        return;
                    };
                    row.push(value_value_7ce4fd9430e80cea.clone());
                }
                sifr_generated_yielder.suspend(row.to_vec()).await;
                let mut position: SifrInt = &r - &SifrInt::from_i64(1);
                while &position >= &SifrInt::from_i64(0) {
                    let current: Option<SifrInt> = {
                        let sifr_generated_checked_read_collection = &indices;
                        let sifr_generated_checked_read_index = position.clone();
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(current_value_2a2e8a5afcc8d89a) = current.clone() else {
                        return;
                    };
                    if &current_value_2a2e8a5afcc8d89a != &(&size - &SifrInt::from_i64(1)) {
                        break;
                    }
                    position = &position - &SifrInt::from_i64(1);
                }
                if &position < &SifrInt::from_i64(0) {
                    return;
                }
                let next_index: Option<SifrInt> = {
                    let sifr_generated_checked_read_collection = &indices;
                    let sifr_generated_checked_read_index = position.clone();
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let Some(next_index_value_7472c0a0a5867a1b) = next_index.clone() else {
                    return;
                };
                let next_value: SifrInt =
                    &next_index_value_7472c0a0a5867a1b + &SifrInt::from_i64(1);
                let mut cursor: SifrInt = position.clone();
                while &cursor < &r {
                    let sifr_generated_try_res: Result<(), IndexError> = (|| {
                        {
                            let sifr_generated_assign_value = next_value.clone();
                            {
                                let sifr_generated_index_raw = cursor.clone();
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(indices.len());
                                if let Some(sifr_generated_elem) =
                                    indices.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                } else {
                                    return Err(IndexError::new(
                                        "collection index out of range".to_string(),
                                    ));
                                }
                            }
                        }
                        Ok(())
                    })();
                    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                        let _e = sifr_generated_try_err.clone();
                        return;
                    }
                    cursor = &cursor + &SifrInt::from_i64(1);
                }
            }
        },
    ))
}
fn starmap<A: Clone + 'static, B: Clone + 'static, R: Clone + 'static>(
    func: impl Fn(&A, &B) -> R + Send + Sync + 'static,
    pairs: Box<dyn Iterator<Item = (A, B)>>,
) -> Box<dyn Iterator<Item = R>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<R>| {
            for (first, second) in pairs {
                sifr_generated_yielder.suspend(func(&first, &second)).await;
            }
        },
    ))
}
fn accumulate<T: Clone + 'static + SifrGeneratedAdd>(
    data: Box<dyn Iterator<Item = T>>,
    initial: Option<T>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            let mut state: Vec<T> = Vec::new();
            if let Some(initial) = initial {
                state.push(initial.clone());
                let initial_value: Option<T> = {
                    let sifr_generated_checked_read_collection = &state;
                    let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(initial_value) = initial_value {
                    sifr_generated_yielder.suspend(initial_value.clone()).await;
                }
            }
            for item in data {
                if &SifrInt::from(state.len()) == &SifrInt::from_i64(0) {
                    state.push(item.clone());
                } else {
                    let prev: Option<T> = {
                        let sifr_generated_checked_read_collection = &state;
                        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(prev) = prev {
                        let next_val: T = SifrGeneratedAdd::sifr_generated_add(prev, item);
                        let sifr_generated_try_res: Result<(), IndexError> = (|| {
                            {
                                let sifr_generated_assign_value = next_val.clone();
                                {
                                    let sifr_generated_index_raw = SifrInt::from_i64(0);
                                    let sifr_generated_index_normalized = sifr_generated_index_raw
                                        .normalize_index_or_len(state.len());
                                    if let Some(sifr_generated_elem) =
                                        state.get_mut(sifr_generated_index_normalized)
                                    {
                                        *sifr_generated_elem = sifr_generated_assign_value;
                                    } else {
                                        return Err(IndexError::new(
                                            "collection index out of range".to_string(),
                                        ));
                                    }
                                }
                            }
                            Ok(())
                        })(
                        );
                        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                            let _e = sifr_generated_try_err.clone();
                            return;
                        }
                    }
                }
                let current: Option<T> = {
                    let sifr_generated_checked_read_collection = &state;
                    let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(current) = current {
                    sifr_generated_yielder.suspend(current.clone()).await;
                }
            }
        },
    ))
}
fn compress<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    selectors: Box<dyn Iterator<Item = bool>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            for (value, selector) in Box::new(data.zip(selectors).map(|sifr_generated_zip_item| {
                (sifr_generated_zip_item.0, sifr_generated_zip_item.1)
            })) {
                if selector {
                    sifr_generated_yielder.suspend(value.clone()).await;
                }
            }
        },
    ))
}
fn dropwhile<T: Clone + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            let mut dropping: bool = true;
            for val in data {
                if dropping {
                    if !pred(&val) {
                        dropping = false;
                        sifr_generated_yielder.suspend(val.clone()).await;
                    }
                } else {
                    sifr_generated_yielder.suspend(val.clone()).await;
                }
            }
        },
    ))
}
fn takewhile<T: Clone + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            for val in data {
                if !pred(&val) {
                    return;
                }
                sifr_generated_yielder.suspend(val.clone()).await;
            }
        },
    ))
}
fn filterfalse<T: Clone + 'static>(
    pred: impl Fn(&T) -> bool + Send + Sync + 'static,
    data: Box<dyn Iterator<Item = T>>,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            for val in data {
                if !pred(&val) {
                    sifr_generated_yielder.suspend(val.clone()).await;
                }
            }
        },
    ))
}
fn zip_longest<T: Clone + 'static>(
    a: Box<dyn Iterator<Item = T>>,
    b: Box<dyn Iterator<Item = T>>,
    fill: &T,
) -> Box<dyn Iterator<Item = Vec<T>>> {
    let fill = fill.clone();
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
            let mut left: Box<dyn Iterator<Item = T>> = a;
            let mut right: Box<dyn Iterator<Item = T>> = b;
            loop {
                let left_value: Option<T> = left.next();
                let right_value: Option<T> = right.next();
                if left_value.is_none() && right_value.is_none() {
                    return;
                }
                let mut pair: Vec<T> = Vec::new();
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
                sifr_generated_yielder.suspend(pair.to_vec()).await;
            }
        },
    ))
}
fn cycle<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    n: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            let mut saved: Vec<T> = Vec::new();
            let mut emitted: SifrInt = SifrInt::from_i64(0);
            if &n <= &SifrInt::from_i64(0) {
                return;
            }
            for value in data {
                saved.push(value.clone());
                sifr_generated_yielder.suspend(value.clone()).await;
                emitted = &emitted + &SifrInt::from_i64(1);
                if &emitted >= &n {
                    return;
                }
            }
            while &emitted < &n && &SifrInt::from(saved.len()) > &SifrInt::from_i64(0) {
                for repeated in saved.iter().cloned() {
                    sifr_generated_yielder.suspend(repeated.clone()).await;
                    emitted = &emitted + &SifrInt::from_i64(1);
                    if &emitted >= &n {
                        return;
                    }
                }
            }
        },
    ))
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IndexError {
    message: String,
}
impl IndexError {
    const fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for IndexError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for IndexError {}
fn lt3(x: SifrInt) -> bool {
    &x < &SifrInt::from_i64(3)
}
fn add2(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let mut acc_it: Box<dyn Iterator<Item = SifrInt>> = accumulate(
        Box::new(
            vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(4),
            ]
            .into_iter(),
        ),
        None,
    );
    assert_eq!(acc_it.next(), Some(SifrInt::from_i64(1)));
    assert_eq!(format!("{:?}", acc_it.collect::<Vec<_>>()), "[3, 6, 10]");
    assert_eq!(
        format!(
            "{:?}",
            compress(
                Box::new(
                    vec![
                        SifrInt::from_i64(1),
                        SifrInt::from_i64(2),
                        SifrInt::from_i64(3),
                        SifrInt::from_i64(4)
                    ]
                    .into_iter()
                ),
                Box::new(vec![true, false, true, false].into_iter())
            )
            .collect::<Vec<_>>()
        ),
        "[1, 3]"
    );
    assert_eq!(
        format!(
            "{:?}",
            dropwhile(
                |sifr_generated_arg0| lt3(sifr_generated_arg0.clone()),
                Box::new(
                    vec![
                        SifrInt::from_i64(1),
                        SifrInt::from_i64(2),
                        SifrInt::from_i64(3),
                        SifrInt::from_i64(1)
                    ]
                    .into_iter()
                )
            )
            .collect::<Vec<_>>()
        ),
        "[3, 1]"
    );
    assert_eq!(
        format!(
            "{:?}",
            takewhile(
                |sifr_generated_arg0| lt3(sifr_generated_arg0.clone()),
                Box::new(
                    vec![
                        SifrInt::from_i64(1),
                        SifrInt::from_i64(2),
                        SifrInt::from_i64(3),
                        SifrInt::from_i64(1)
                    ]
                    .into_iter()
                )
            )
            .collect::<Vec<_>>()
        ),
        "[1, 2]"
    );
    assert_eq!(
        format!(
            "{:?}",
            filterfalse(
                |sifr_generated_arg0| lt3(sifr_generated_arg0.clone()),
                Box::new(
                    vec![
                        SifrInt::from_i64(1),
                        SifrInt::from_i64(2),
                        SifrInt::from_i64(3),
                        SifrInt::from_i64(1)
                    ]
                    .into_iter()
                )
            )
            .collect::<Vec<_>>()
        ),
        "[3]"
    );
    assert_eq!(
        format!(
            "{:?}",
            zip_longest(
                Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2)].into_iter()),
                Box::new(vec![SifrInt::from_i64(9)].into_iter()),
                &SifrInt::from_i64(0)
            )
            .collect::<Vec<_>>()
        ),
        "[[1, 9], [2, 0]]"
    );
    let mut cyc: Box<dyn Iterator<Item = SifrInt>> = cycle(
        Box::new(
            vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
            ]
            .into_iter(),
        ),
        SifrInt::from_i64(5),
    );
    assert_eq!(cyc.next(), Some(SifrInt::from_i64(1)));
    assert_eq!(format!("{:?}", cyc.collect::<Vec<_>>()), "[2, 3, 1, 2]");
    assert_eq!(
        format!(
            "{:?}",
            starmap(
                |sifr_generated_arg0, sifr_generated_arg1| add2(
                    sifr_generated_arg0.clone(),
                    sifr_generated_arg1.clone()
                ),
                Box::new(
                    vec![
                        (SifrInt::from_i64(2), SifrInt::from_i64(3)),
                        (SifrInt::from_i64(4), SifrInt::from_i64(5))
                    ]
                    .into_iter()
                )
            )
            .collect::<Vec<_>>()
        ),
        "[5, 9]"
    );
    assert_eq!(
        format!(
            "{:?}",
            product(
                &vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]],
                SifrInt::from_i64(2)
            )
            .collect::<Vec<_>>()
        ),
        "[[1, 1], [1, 2], [2, 1], [2, 2]]"
    );
    assert_eq!(
        format!(
            "{:?}",
            product(
                &vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]],
                -&SifrInt::from_i64(1)
            )
            .collect::<Vec<_>>()
        ),
        "[]"
    );
    assert_eq!(
        format!(
            "{:?}",
            permutations(
                Box::new(
                    vec![
                        SifrInt::from_i64(1),
                        SifrInt::from_i64(2),
                        SifrInt::from_i64(3)
                    ]
                    .into_iter()
                ),
                Some(SifrInt::from_i64(2))
            )
            .collect::<Vec<_>>()
        ),
        "[[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]]"
    );
    assert_eq!(
        format!(
            "{:?}",
            combinations(
                Box::new(
                    vec![
                        SifrInt::from_i64(1),
                        SifrInt::from_i64(2),
                        SifrInt::from_i64(3)
                    ]
                    .into_iter()
                ),
                SifrInt::from_i64(2)
            )
            .collect::<Vec<_>>()
        ),
        "[[1, 2], [1, 3], [2, 3]]"
    );
    assert_eq!(
        format!(
            "{:?}",
            combinations_with_replacement(
                Box::new(vec![SifrInt::from_i64(1), SifrInt::from_i64(2)].into_iter()),
                SifrInt::from_i64(2)
            )
            .collect::<Vec<_>>()
        ),
        "[[1, 1], [1, 2], [2, 2]]"
    );
    println!("parity_ext_extended_itertools_lazy_surface_demo: ok");
}
