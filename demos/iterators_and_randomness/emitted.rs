// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{
        IndexError, SifrGeneratedStdlibSifrX2erandomX2eRandom,
        SifrGeneratedStdlibSifrX2erandomX2eRandomState, ValueError,
    };
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) struct SifrGeneratedYielder<T> {
        pub(super) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    }
    pub(super) struct SifrGeneratedYieldFuture<T> {
        pub(super) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(super) value: Option<T>,
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
        pub(super) fn suspend(&self, value: T) -> SifrGeneratedYieldFuture<T> {
            SifrGeneratedYieldFuture {
                slot: ::std::sync::Arc::clone(&self.slot),
                value: Some(value),
            }
        }
    }
    pub(super) fn sifr_generated_store_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        value: T,
    ) {
        match slot.lock() {
            Ok(mut state) => *state = Some(value),
            Err(poisoned) => *poisoned.into_inner() = Some(value),
        }
    }
    pub(super) fn sifr_generated_take_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    ) -> Option<T> {
        match slot.lock() {
            Ok(mut state) => state.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
    pub(super) struct SifrGeneratedGenerator<T> {
        pub(super) producer:
            Option<::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>>,
        pub(super) yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(super) complete: bool,
    }
    impl<T> SifrGeneratedGenerator<T> {
        pub(super) fn new<
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
    pub(super) fn sifr_generated_collect_iterator<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
    ) -> Vec<T> {
        let mut collected: Vec<T> = Vec::new();
        for item in data {
            collected.push(item);
        }
        collected
    }
    pub(super) fn chain<T: Clone + 'static>(iterables: &[Vec<T>]) -> Box<dyn Iterator<Item = T>> {
        let iterables = iterables.to_vec();
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                #[expect(
                    clippy::explicit_iter_loop,
                    reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                )]
                for iterable in iterables.iter() {
                    #[expect(
                        clippy::explicit_iter_loop,
                        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                    )]
                    for item in iterable.iter() {
                        sifr_generated_yielder.suspend(item.clone()).await;
                    }
                }
            },
        ))
    }
    pub(super) fn sifr_generated_islice_impl<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        start: SifrInt,
        stop: SifrInt,
        unbounded: bool,
        step_argument_af0b4e191da20cef: SifrInt,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                let mut index: SifrInt = SifrInt::from_i64(0);
                let mut next_yield: SifrInt = start.clone();
                for value in data {
                    if !unbounded && index >= stop {
                        return;
                    }
                    if index == next_yield {
                        sifr_generated_yielder.suspend(value.clone()).await;
                        next_yield =
                            ::std::ops::Add::add(&next_yield, &step_argument_af0b4e191da20cef);
                    }
                    index = ::std::ops::Add::add(&index, &SifrInt::from_i64(1));
                }
            },
        ))
    }
    pub(super) fn islice<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        start_or_stop: Option<&SifrInt>,
        slice_args: &[Option<SifrInt>],
    ) -> Result<Box<dyn Iterator<Item = T>>, ValueError> {
        let start_or_stop: Option<SifrInt> = start_or_stop.cloned();
        if slice_args.len() > SifrInt::from_i64(2) {
            return Err(ValueError::new(
                "islice: expected at most stop and step after start".to_string(),
            ));
        }
        let mut actual_start: SifrInt = SifrInt::from_i64(0);
        let mut actual_stop_value_351bdef5a4961be0: SifrInt = SifrInt::from_i64(0);
        let mut unbounded: bool = start_or_stop.is_none();
        if let Some(start_or_stop) = start_or_stop.clone() {
            actual_stop_value_351bdef5a4961be0.clone_from(&start_or_stop);
        }
        let mut actual_step_value_353dfaf5a4b331da: SifrInt = SifrInt::from_i64(1);
        let mut argument_index: SifrInt = SifrInt::from_i64(0);
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for argument in slice_args.iter() {
            if argument_index == SifrInt::from_i64(0) {
                let Some(start_or_stop) = start_or_stop.clone() else {
                    return Err(ValueError::new(
                        "islice: start must be an integer when stop is provided".to_string(),
                    ));
                };
                actual_start.clone_from(&start_or_stop);
                if argument.is_none() {
                    unbounded = true;
                } else if let Some(argument) = argument.clone() {
                    actual_stop_value_351bdef5a4961be0.clone_from(&argument);
                }
            } else if let Some(argument) = argument.clone() {
                actual_step_value_353dfaf5a4b331da.clone_from(&argument);
            }
            argument_index = ::std::ops::Add::add(&argument_index, &SifrInt::from_i64(1));
        }
        if actual_start < SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "islice: indices must be non-negative".to_string(),
            ));
        }
        if !unbounded && actual_stop_value_351bdef5a4961be0 < SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "islice: indices must be non-negative".to_string(),
            ));
        }
        if actual_step_value_353dfaf5a4b331da <= SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "islice: step must be greater than zero".to_string(),
            ));
        }
        Ok(sifr_generated_islice_impl(
            Box::new(data),
            actual_start,
            actual_stop_value_351bdef5a4961be0,
            unbounded,
            actual_step_value_353dfaf5a4b331da,
        ))
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one generated Rust function preserves one typed Sifr function"
    )]
    pub(super) fn product<T: Clone + 'static>(
        iterables: &[Vec<T>],
        repeat: SifrInt,
    ) -> Box<dyn Iterator<Item = Vec<T>>> {
        let iterables = iterables.to_vec();
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
                if repeat < SifrInt::from_i64(0) {
                    return;
                }
                let mut pools: Vec<Vec<T>> = Vec::new();
                let mut repetition: SifrInt = SifrInt::from_i64(0);
                while repetition < repeat {
                    for iterable in iterables.iter().cloned() {
                        pools.push(iterable);
                    }
                    repetition = ::std::ops::Add::add(&repetition, &SifrInt::from_i64(1));
                }
                if pools.len() == SifrInt::from_i64(0) {
                    sifr_generated_yielder.suspend(Vec::new()).await;
                    return;
                }
                #[expect(
                    clippy::explicit_iter_loop,
                    reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                )]
                for pool in pools.iter() {
                    if pool.len() == SifrInt::from_i64(0) {
                        return;
                    }
                }
                let mut indices: Vec<SifrInt> = Vec::new();
                #[expect(
                    clippy::explicit_iter_loop,
                    reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                )]
                for _ in pools.iter() {
                    indices.push(SifrInt::from_i64(0));
                }
                let mut finished: bool = false;
                while !finished {
                    let mut row: Vec<T> = Vec::new();
                    let mut pool_index: SifrInt = SifrInt::from_i64(0);
                    while pool_index < pools.len() {
                        let pool_value: Option<Vec<T>> = {
                            let sifr_generated_checked_read_collection = &pools;
                            let sifr_generated_checked_read_index = &pool_index;
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
                            let sifr_generated_checked_read_index = &pool_index;
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        let (Some(pool_value), Some(value_index_value_336ae61b280d8a15)) =
                            (pool_value, value_index)
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
                        row.push(value_value_7ce4fd9430e80cea);
                        pool_index = ::std::ops::Add::add(&pool_index, &SifrInt::from_i64(1));
                    }
                    sifr_generated_yielder.suspend(row).await;
                    let mut position: SifrInt =
                        ::std::ops::Sub::sub(&SifrInt::from(pools.len()), &SifrInt::from_i64(1));
                    let mut advanced: bool = false;
                    while position >= SifrInt::from_i64(0) && !advanced {
                        let current_pool: Option<Vec<T>> = {
                            let sifr_generated_checked_read_collection = &pools;
                            let sifr_generated_checked_read_index = &position;
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
                            let sifr_generated_checked_read_index = &position;
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
                        ) = (current_pool, current_index)
                        else {
                            return;
                        };
                        let next_index: SifrInt = ::std::ops::Add::add(
                            &current_index_value_57667e3202daa6c5,
                            &SifrInt::from_i64(1),
                        );
                        if next_index < current_pool_value_8d0aa685cb481a75.len() {
                            let sifr_generated_try_res: Result<(), IndexError> = (|| {
                                {
                                    let sifr_generated_assign_value = next_index;
                                    {
                                        let sifr_generated_index_raw = &position;
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
                                let _e = sifr_generated_try_err;
                                return;
                            }
                            advanced = true;
                        } else {
                            let sifr_generated_try_res: Result<(), IndexError> = (|| {
                                {
                                    let sifr_generated_assign_value = SifrInt::from_i64(0);
                                    {
                                        let sifr_generated_index_raw = &position;
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
                                let _e = sifr_generated_try_err;
                                return;
                            }
                            position = ::std::ops::Sub::sub(&position, &SifrInt::from_i64(1));
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
    pub(super) fn permutations<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        r: Option<&SifrInt>,
    ) -> Box<dyn Iterator<Item = Vec<T>>> {
        let r: Option<SifrInt> = r.cloned();
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
                let materialized: Vec<T> = sifr_generated_collect_iterator(Box::new(data));
                let target: SifrInt = r
                    .clone()
                    .unwrap_or_else(|| SifrInt::from(materialized.len()));
                let size: SifrInt = SifrInt::from(materialized.len());
                if target < SifrInt::from_i64(0) || target > size {
                    return;
                }
                if target == SifrInt::from_i64(0) {
                    sifr_generated_yielder.suspend(Vec::new()).await;
                    return;
                }
                let mut indices: Vec<SifrInt> = Vec::new();
                let mut index: SifrInt = SifrInt::from_i64(0);
                while index < size {
                    indices.push(index.clone());
                    index = ::std::ops::Add::add(&index, &SifrInt::from_i64(1));
                }
                let mut cycles: Vec<SifrInt> = Vec::new();
                index = SifrInt::from_i64(0);
                while index < target {
                    cycles.push(::std::ops::Sub::sub(&size, &index));
                    index = ::std::ops::Add::add(&index, &SifrInt::from_i64(1));
                }
                let mut first: Vec<T> = Vec::new();
                index = SifrInt::from_i64(0);
                while index < target {
                    let source_index: Option<SifrInt> = {
                        let sifr_generated_checked_read_collection = &indices;
                        let sifr_generated_checked_read_index = &index;
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(source_index_value_1402cfd57e11de6b) = source_index else {
                        return;
                    };
                    let value: Option<T> = {
                        let sifr_generated_checked_read_collection = &materialized;
                        let sifr_generated_checked_read_index = source_index_value_1402cfd57e11de6b;
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
                    first.push(value_value_7ce4fd9430e80cea);
                    index = ::std::ops::Add::add(&index, &SifrInt::from_i64(1));
                }
                sifr_generated_yielder.suspend(first).await;
                loop {
                    let mut position: SifrInt =
                        ::std::ops::Sub::sub(&target, &SifrInt::from_i64(1));
                    let mut produced: bool = false;
                    while position >= SifrInt::from_i64(0) && !produced {
                        let remaining: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &cycles;
                            let sifr_generated_checked_read_index = &position;
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        let Some(remaining_value_343f0419454a4243) = remaining else {
                            return;
                        };
                        let next_remaining: SifrInt = ::std::ops::Sub::sub(
                            &remaining_value_343f0419454a4243,
                            &SifrInt::from_i64(1),
                        );
                        let sifr_generated_try_res: Result<(), IndexError> = (|| {
                            {
                                let sifr_generated_assign_value = next_remaining.clone();
                                {
                                    let sifr_generated_index_raw = &position;
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
                            let _e = sifr_generated_try_err;
                            return;
                        }
                        if next_remaining == SifrInt::from_i64(0) {
                            let rotated: Option<SifrInt> = {
                                let sifr_generated_checked_read_collection = &indices;
                                let sifr_generated_checked_read_index = &position;
                                let sifr_generated_checked_read_normalized =
                                    sifr_generated_checked_read_index.normalize_index_or_len(
                                        sifr_generated_checked_read_collection.len(),
                                    );
                                sifr_generated_checked_read_collection
                                    .get(sifr_generated_checked_read_normalized)
                                    .cloned()
                            };
                            let Some(rotated_value_f64204a307abbb6a) = rotated else {
                                return;
                            };
                            let mut cursor: SifrInt = position.clone();
                            while cursor < ::std::ops::Sub::sub(&size, &SifrInt::from_i64(1)) {
                                let shifted: Option<SifrInt> = {
                                    let sifr_generated_checked_read_collection = &indices;
                                    let sifr_generated_checked_read_index =
                                        ::std::ops::Add::add(&cursor, &SifrInt::from_i64(1));
                                    let sifr_generated_checked_read_normalized =
                                        sifr_generated_checked_read_index.normalize_index_or_len(
                                            sifr_generated_checked_read_collection.len(),
                                        );
                                    sifr_generated_checked_read_collection
                                        .get(sifr_generated_checked_read_normalized)
                                        .cloned()
                                };
                                let Some(shifted_value_7540578f579f2e86) = shifted else {
                                    return;
                                };
                                let sifr_generated_try_res: Result<(), IndexError> = (|| {
                                    {
                                        let sifr_generated_assign_value =
                                            shifted_value_7540578f579f2e86;
                                        {
                                            let sifr_generated_index_raw = &cursor;
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
                                    let _e = sifr_generated_try_err;
                                    return;
                                }
                                cursor = ::std::ops::Add::add(&cursor, &SifrInt::from_i64(1));
                            }
                            let sifr_generated_try_res: Result<(), IndexError> = (|| {
                                {
                                    let sifr_generated_assign_value =
                                        rotated_value_f64204a307abbb6a;
                                    {
                                        let sifr_generated_index_raw =
                                            ::std::ops::Sub::sub(&size, &SifrInt::from_i64(1));
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
                                {
                                    let sifr_generated_assign_value =
                                        ::std::ops::Sub::sub(&size, &position);
                                    {
                                        let sifr_generated_index_raw = &position;
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
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
                                let _e = sifr_generated_try_err;
                                return;
                            }
                            position = ::std::ops::Sub::sub(&position, &SifrInt::from_i64(1));
                        } else {
                            let swap_position: SifrInt =
                                ::std::ops::Sub::sub(&size, &next_remaining);
                            let left_index: Option<SifrInt> = {
                                let sifr_generated_checked_read_collection = &indices;
                                let sifr_generated_checked_read_index = &position;
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
                                let sifr_generated_checked_read_index = &swap_position;
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
                            ) = (left_index, right_index)
                            else {
                                return;
                            };
                            let left_value: SifrInt = left_index_value_0cbf618cd64fdba3;
                            let right_value: SifrInt = right_index_value_0d20c76177571432;
                            let sifr_generated_try_res: Result<(), IndexError> = (|| {
                                {
                                    let sifr_generated_assign_value = right_value;
                                    {
                                        let sifr_generated_index_raw = &position;
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
                                {
                                    let sifr_generated_assign_value = left_value;
                                    {
                                        let sifr_generated_index_raw = &swap_position;
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
                                let _e = sifr_generated_try_err;
                                return;
                            }
                            let mut row: Vec<T> = Vec::new();
                            let mut row_index: SifrInt = SifrInt::from_i64(0);
                            while row_index < target {
                                let item_index: Option<SifrInt> = {
                                    let sifr_generated_checked_read_collection = &indices;
                                    let sifr_generated_checked_read_index = &row_index;
                                    let sifr_generated_checked_read_normalized =
                                        sifr_generated_checked_read_index.normalize_index_or_len(
                                            sifr_generated_checked_read_collection.len(),
                                        );
                                    sifr_generated_checked_read_collection
                                        .get(sifr_generated_checked_read_normalized)
                                        .cloned()
                                };
                                let Some(item_index_value_9a28a188f6a6f491) = item_index else {
                                    return;
                                };
                                let item: Option<T> = {
                                    let sifr_generated_checked_read_collection = &materialized;
                                    let sifr_generated_checked_read_index =
                                        item_index_value_9a28a188f6a6f491;
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
                                row.push(item_value_2841a0c596d6f426);
                                row_index = ::std::ops::Add::add(&row_index, &SifrInt::from_i64(1));
                            }
                            sifr_generated_yielder.suspend(row).await;
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
    pub(super) fn combinations<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        r: SifrInt,
    ) -> Box<dyn Iterator<Item = Vec<T>>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<Vec<T>>| {
                let materialized: Vec<T> = sifr_generated_collect_iterator(Box::new(data));
                let size: SifrInt = SifrInt::from(materialized.len());
                if r < SifrInt::from_i64(0) || r > size {
                    return;
                }
                if r == SifrInt::from_i64(0) {
                    sifr_generated_yielder.suspend(Vec::new()).await;
                    return;
                }
                let mut indices: Vec<SifrInt> = Vec::new();
                let mut index: SifrInt = SifrInt::from_i64(0);
                while index < r {
                    indices.push(index.clone());
                    index = ::std::ops::Add::add(&index, &SifrInt::from_i64(1));
                }
                loop {
                    let mut row: Vec<T> = Vec::new();
                    #[expect(
                        clippy::explicit_iter_loop,
                        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                    )]
                    for source_index in indices.iter() {
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
                        row.push(value_value_7ce4fd9430e80cea);
                    }
                    sifr_generated_yielder.suspend(row.clone()).await;
                    let mut position: SifrInt = ::std::ops::Sub::sub(&r, &SifrInt::from_i64(1));
                    while position >= SifrInt::from_i64(0) {
                        let current: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &indices;
                            let sifr_generated_checked_read_index = &position;
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
                        if current_value_2a2e8a5afcc8d89a
                            != ::std::ops::Sub::sub(&::std::ops::Add::add(&position, &size), &r)
                        {
                            break;
                        }
                        position = ::std::ops::Sub::sub(&position, &SifrInt::from_i64(1));
                    }
                    if position < SifrInt::from_i64(0) {
                        return;
                    }
                    let current: Option<SifrInt> = {
                        let sifr_generated_checked_read_collection = &indices;
                        let sifr_generated_checked_read_index = &position;
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    let Some(current_value_2a2e8a5afcc8d89a) = current else {
                        return;
                    };
                    let mut next_position: SifrInt = ::std::ops::Add::add(
                        &current_value_2a2e8a5afcc8d89a,
                        &SifrInt::from_i64(1),
                    );
                    let sifr_generated_try_res: Result<(), IndexError> = (|| {
                        {
                            let sifr_generated_assign_value = next_position.clone();
                            {
                                let sifr_generated_index_raw = &position;
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
                        let _e = sifr_generated_try_err;
                        return;
                    }
                    let mut cursor: SifrInt =
                        ::std::ops::Add::add(&position, &SifrInt::from_i64(1));
                    while cursor < r {
                        let previous: Option<SifrInt> = {
                            let sifr_generated_checked_read_collection = &indices;
                            let sifr_generated_checked_read_index =
                                ::std::ops::Sub::sub(&cursor, &SifrInt::from_i64(1));
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        let Some(previous_value_ec5f63ffe7e97248) = previous else {
                            return;
                        };
                        next_position = ::std::ops::Add::add(
                            &previous_value_ec5f63ffe7e97248,
                            &SifrInt::from_i64(1),
                        );
                        let sifr_generated_try_res: Result<(), IndexError> = (|| {
                            {
                                let sifr_generated_assign_value = next_position.clone();
                                {
                                    let sifr_generated_index_raw = &cursor;
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
                            let _e = sifr_generated_try_err;
                            return;
                        }
                        cursor = ::std::ops::Add::add(&cursor, &SifrInt::from_i64(1));
                    }
                }
            },
        ))
    }
    pub(super) fn starmap<A: Clone + 'static, B: Clone + 'static, R: Clone + 'static>(
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
    pub(super) fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
        ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
    }
    pub(super) fn random_seed() -> SifrInt {
        ::sifr_stdlib::random::random_seed().into_sifr_int()
    }
    pub(super) fn random_module_state_words() -> Vec<SifrInt> {
        ::sifr_stdlib::random::random_module_state_words()
            .into_iter()
            .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
            .collect()
    }
    pub(super) fn random_module_state_index() -> SifrInt {
        ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
    }
    pub(super) fn random_module_state_gauss_next() -> Option<f64> {
        ::sifr_stdlib::random::random_module_state_gauss_next()
    }
    pub(super) fn random_module_set_state(
        words: &[SifrInt],
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
            gauss_next,
        )
        .map_err(|sifr_generated_bridge_error| ValueError {
            message: sifr_generated_bridge_error,
        })
    }
    pub(super) const fn sifr_generated_const_5f4d545f4e() -> SifrInt {
        SifrInt::from_i64(624)
    }
    pub(super) const fn sifr_generated_const_5f4d545f4d() -> SifrInt {
        SifrInt::from_i64(397)
    }
    pub(super) const fn sifr_generated_const_5f4d545f4d41545249585f41() -> SifrInt {
        SifrInt::from_i64(2_567_483_615)
    }
    pub(super) const fn sifr_generated_const_5f4d545f55505045525f4d41534b() -> SifrInt {
        SifrInt::from_i64(2_147_483_648)
    }
    pub(super) const fn sifr_generated_const_5f4d545f4c4f5745525f4d41534b() -> SifrInt {
        SifrInt::from_i64(2_147_483_647)
    }
    pub(super) const fn sifr_generated_const_5f4d545f46() -> SifrInt {
        SifrInt::from_i64(1_812_433_253)
    }
    pub(super) const fn sifr_generated_const_5f4d545f574f52445f4d41534b() -> SifrInt {
        SifrInt::from_i64(4_294_967_295)
    }
    pub(super) fn sifr_generated_state_word_at(words: &[SifrInt], index: &SifrInt) -> SifrInt {
        let value: Option<SifrInt> = {
            let sifr_generated_checked_read_collection = &words;
            let sifr_generated_checked_read_index = (*index).clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        let Some(value_value_7ce4fd9430e80cea) = value else {
            return SifrInt::from_i64(0);
        };
        value_value_7ce4fd9430e80cea
    }
    pub(super) fn sifr_generated_clone_words(words: &[SifrInt]) -> Vec<SifrInt> {
        let mut copied: Vec<SifrInt> = Vec::new();
        for word in words.iter().cloned() {
            copied.push(word);
        }
        copied
    }
    pub(super) fn sifr_generated_normalize_seed_input(seed_value: Option<&SifrInt>) -> SifrInt {
        let seed_value: Option<SifrInt> = seed_value.cloned();
        let Some(seed_value) = seed_value else {
            return random_seed();
        };
        seed_value
    }
    pub(super) fn sifr_generated_seed_words_from_seed(seed_value: &SifrInt) -> Vec<SifrInt> {
        let mut words: Vec<SifrInt> = vec![::std::ops::BitAnd::bitand(
            seed_value,
            &sifr_generated_const_5f4d545f574f52445f4d41534b(),
        )];
        let mut i: SifrInt = SifrInt::from_i64(1);
        while i < sifr_generated_const_5f4d545f4e() {
            let prev: SifrInt = sifr_generated_state_word_at(
                &words,
                &::std::ops::Sub::sub(&i, &SifrInt::from_i64(1)),
            );
            let next_word: SifrInt = ::std::ops::BitAnd::bitand(
                &::std::ops::Add::add(
                    &::std::ops::Mul::mul(
                        &sifr_generated_const_5f4d545f46(),
                        &::std::ops::BitXor::bitxor(
                            &prev,
                            &prev.floor_div_known_nonzero(&SifrInt::from_i64(1_073_741_824)),
                        ),
                    ),
                    &i,
                ),
                &sifr_generated_const_5f4d545f574f52445f4d41534b(),
            );
            words.push(next_word);
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        words
    }
    pub(super) fn sifr_generated_build_state_from_module_storage()
    -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        SifrGeneratedStdlibSifrX2erandomX2eRandomState::new(
            &SifrInt::from_i64(3),
            random_module_state_words(),
            &random_module_state_index(),
            random_module_state_gauss_next(),
        )
    }
    pub(super) fn sifr_generated_store_state_into_module_storage(
        state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
    ) {
        let sifr_generated_set_result: Result<(), ValueError> = random_module_set_state(
            &sifr_generated_clone_words(&state.state_words.clone()),
            state.index.clone(),
            state.gauss_next,
        );
        let _ = sifr_generated_set_result;
    }
    pub(super) fn sifr_generated_ensure_module_state_initialized() {
        let words: Vec<SifrInt> = random_module_state_words();
        if words.len() == sifr_generated_const_5f4d545f4e() {
            return;
        }
        let bootstrap: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(&SifrInt::from_i64(5489)));
        sifr_generated_store_state_into_module_storage(&bootstrap.getstate());
    }
    pub(super) fn sifr_generated_module_random() -> SifrGeneratedStdlibSifrX2erandomX2eRandom {
        sifr_generated_ensure_module_state_initialized();
        let mut r: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            SifrGeneratedStdlibSifrX2erandomX2eRandom::new(Some(&SifrInt::from_i64(0)));
        let sifr_generated_set_result: Result<(), ValueError> =
            r.setstate(&sifr_generated_build_state_from_module_storage());
        let _ = sifr_generated_set_result;
        r
    }
    pub(super) fn sifr_generated_sync_module_random(
        generator: &SifrGeneratedStdlibSifrX2erandomX2eRandom,
    ) {
        sifr_generated_store_state_into_module_storage(&generator.getstate());
    }
    pub(super) fn randrange(
        start: &SifrInt,
        stop: Option<&SifrInt>,
        step_argument_af0b4e191da20cef: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        let stop: Option<SifrInt> = stop.cloned();
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let value: Result<SifrInt, ValueError> =
            generator.randrange(start, &stop, step_argument_af0b4e191da20cef);
        sifr_generated_sync_module_random(&generator);
        value
    }
    pub(super) fn choice<T: Clone + 'static>(items: &[T]) -> Result<T, ValueError> {
        let item_count: SifrInt = SifrInt::from(items.len());
        if item_count == SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "choice: items must not be empty".to_string(),
            ));
        }
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let index: SifrInt = generator
            .sifr_generated_next_u32()
            .floor_mod_known_nonzero(&item_count);
        let picked: Option<T> = {
            let sifr_generated_checked_read_collection = &items;
            let sifr_generated_checked_read_index = &index;
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        };
        sifr_generated_sync_module_random(&generator);
        let Some(picked_value_9fda901c871bd7d9) = picked else {
            return Err(ValueError::new("choice: index out of range".to_string()));
        };
        Ok(picked_value_9fda901c871bd7d9)
    }
    pub(super) fn choices<T: Clone + 'static>(
        items: &[T],
        k: &SifrInt,
    ) -> Result<Vec<T>, ValueError> {
        if k <= &SifrInt::from_i64(0) {
            return Ok(Vec::new());
        }
        let item_count: SifrInt = SifrInt::from(items.len());
        if item_count == SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "choices: items must not be empty".to_string(),
            ));
        }
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let mut result: Vec<T> = Vec::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < k {
            let index: SifrInt = generator
                .sifr_generated_next_u32()
                .floor_mod_known_nonzero(&item_count);
            let picked: Option<T> = {
                let sifr_generated_checked_read_collection = &items;
                let sifr_generated_checked_read_index = &index;
                let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                    .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                sifr_generated_checked_read_collection
                    .get(sifr_generated_checked_read_normalized)
                    .cloned()
            };
            if let Some(picked) = picked {
                result.push(picked);
            } else {
                return Err(ValueError::new("choices: index out of range".to_string()));
            }
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        sifr_generated_sync_module_random(&generator);
        Ok(result)
    }
    pub(super) fn shuffle<T: Clone + 'static>(items: &mut [T]) {
        let mut generator: SifrGeneratedStdlibSifrX2erandomX2eRandom =
            sifr_generated_module_random();
        let n: SifrInt = SifrInt::from(items.len());
        if n > SifrInt::from_i64(1) {
            let mut i: SifrInt = ::std::ops::Sub::sub(&n, &SifrInt::from_i64(1));
            while i > SifrInt::from_i64(0) {
                let divisor: SifrInt = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                if divisor == SifrInt::from_i64(0) {
                    return;
                }
                let j: SifrInt = generator
                    .sifr_generated_next_u32()
                    .floor_mod_known_nonzero(&divisor);
                let left: Option<T> = {
                    let sifr_generated_checked_read_collection = &items;
                    let sifr_generated_checked_read_index = &i;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                let right: Option<T> = {
                    let sifr_generated_checked_read_collection = &items;
                    let sifr_generated_checked_read_index = &j;
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(left) = left
                    && let Some(right) = right
                {
                    if SifrInt::from_i64(0) <= i && i < items.len() {
                        {
                            let sifr_generated_assign_value = right;
                            {
                                let sifr_generated_index_raw = &i;
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                                if let Some(sifr_generated_elem) =
                                    items.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                    if SifrInt::from_i64(0) <= j && j < items.len() {
                        {
                            let sifr_generated_assign_value = left;
                            {
                                let sifr_generated_index_raw = &j;
                                let sifr_generated_index_normalized =
                                    sifr_generated_index_raw.normalize_index_or_len(items.len());
                                if let Some(sifr_generated_elem) =
                                    items.get_mut(sifr_generated_index_normalized)
                                {
                                    *sifr_generated_elem = sifr_generated_assign_value;
                                }
                            }
                        }
                    }
                }
                i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
            }
        }
        sifr_generated_sync_module_random(&generator);
    }
    pub(super) fn compare_digest(a: &str, b: &str) -> bool {
        a == b
    }
    pub(super) fn token_hex(nbytes: &SifrInt) -> String {
        let hex_chars: String = "0123456789abcdef".to_string();
        let mut result: String = String::new();
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < ::std::ops::Mul::mul(nbytes, &SifrInt::from_i64(2)) {
            let idx: SifrInt = random_int(SifrInt::from_i64(0), SifrInt::from_i64(15));
            let ch: Option<String> = {
                let sifr_generated_string_chars = hex_chars.chars().collect::<Vec<char>>();
                let sifr_generated_string_index = &idx;
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_string_chars.len());
                sifr_generated_string_chars
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch {
                result.push_str(ch.as_str());
            }
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        result
    }
    pub(super) fn randbits(k: &SifrInt) -> Result<SifrInt, ValueError> {
        if k < &SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "randbits: number of bits must be >= 0".to_string(),
            ));
        }
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while &i < k {
            let bit: SifrInt = random_int(SifrInt::from_i64(0), SifrInt::from_i64(1));
            result =
                ::std::ops::Add::add(&::std::ops::Mul::mul(&result, &SifrInt::from_i64(2)), &bit);
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
        Ok(result)
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::{
        sifr_generated_clone_words, sifr_generated_const_5f4d545f4c4f5745525f4d41534b,
        sifr_generated_const_5f4d545f4d, sifr_generated_const_5f4d545f4d41545249585f41,
        sifr_generated_const_5f4d545f4e, sifr_generated_const_5f4d545f574f52445f4d41534b,
        sifr_generated_const_5f4d545f55505045525f4d41534b, sifr_generated_normalize_seed_input,
        sifr_generated_seed_words_from_seed, sifr_generated_state_word_at,
    };
    use ::sifr_runtime::SifrInt;
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        pub version: SifrInt,
        pub state_words: Vec<SifrInt>,
        pub index: SifrInt,
        pub gauss_next: Option<f64>,
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandomState {
        #[must_use]
        pub fn new(
            version: &SifrInt,
            state_words: Vec<SifrInt>,
            index: &SifrInt,
            gauss_next: Option<f64>,
        ) -> Self {
            let sifr_generated_field_value_bb62c62c9808ea37_76657273696f6e: SifrInt =
                (*version).clone();
            let sifr_generated_field_value_8e62ac2dd7162e8c_73746174655f776f726473: Vec<SifrInt> =
                state_words;
            let sifr_generated_field_value_83cf8e8f9081468b_696e646578: SifrInt = (*index).clone();
            let sifr_generated_field_value_edec7000e7b3eeaa_67617573735f6e657874: Option<f64> =
                gauss_next;
            Self {
                version: sifr_generated_field_value_bb62c62c9808ea37_76657273696f6e,
                state_words: sifr_generated_field_value_8e62ac2dd7162e8c_73746174655f776f726473,
                index: sifr_generated_field_value_83cf8e8f9081468b_696e646578,
                gauss_next: sifr_generated_field_value_edec7000e7b3eeaa_67617573735f6e657874,
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct SifrGeneratedStdlibSifrX2erandomX2eRandom {
        pub state_words_field: Vec<SifrInt>,
        pub index_field: SifrInt,
        pub gauss_next_field: Option<f64>,
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn new(seed_value: Option<&SifrInt>) -> Self {
            let seed_value: Option<SifrInt> = seed_value.cloned();
            let normalized_seed: SifrInt = sifr_generated_normalize_seed_input(seed_value.as_ref());
            let sifr_generated_field_value_7e372b502c45daad_5f73746174655f776f726473: Vec<SifrInt> =
                sifr_generated_seed_words_from_seed(&normalized_seed);
            let sifr_generated_field_value_497043933c8a2d12_5f696e646578: SifrInt =
                sifr_generated_const_5f4d545f4e();
            let sifr_generated_field_value_88c1b3a412b57c41_5f67617573735f6e657874: Option<f64> =
                None;
            Self {
                state_words_field:
                    sifr_generated_field_value_7e372b502c45daad_5f73746174655f776f726473,
                index_field: sifr_generated_field_value_497043933c8a2d12_5f696e646578,
                gauss_next_field:
                    sifr_generated_field_value_88c1b3a412b57c41_5f67617573735f6e657874,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        pub fn sifr_generated_twist(&mut self) {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while SifrInt::from_i64(0) <= i && i < self.state_words_field.len() {
                let y: SifrInt = ::std::ops::Add::add(
                    &::std::ops::BitAnd::bitand(
                        &sifr_generated_state_word_at(&self.state_words_field, &i),
                        &sifr_generated_const_5f4d545f55505045525f4d41534b(),
                    ),
                    &::std::ops::BitAnd::bitand(
                        &sifr_generated_state_word_at(
                            &self.state_words_field,
                            &::std::ops::Add::add(&i, &SifrInt::from_i64(1))
                                .floor_mod_known_nonzero(&sifr_generated_const_5f4d545f4e()),
                        ),
                        &sifr_generated_const_5f4d545f4c4f5745525f4d41534b(),
                    ),
                );
                let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
                if y.floor_mod_known_nonzero(&SifrInt::from_i64(2)) != SifrInt::from_i64(0) {
                    x_a = ::std::ops::BitXor::bitxor(
                        &x_a,
                        &sifr_generated_const_5f4d545f4d41545249585f41(),
                    );
                }
                let new_word: SifrInt = ::std::ops::BitXor::bitxor(
                    &sifr_generated_state_word_at(
                        &self.state_words_field,
                        &::std::ops::Add::add(&i, &sifr_generated_const_5f4d545f4d())
                            .floor_mod_known_nonzero(&sifr_generated_const_5f4d545f4e()),
                    ),
                    &x_a,
                );
                {
                    let sifr_generated_assign_value = ::std::ops::BitAnd::bitand(
                        &new_word,
                        &sifr_generated_const_5f4d545f574f52445f4d41534b(),
                    );
                    {
                        let sifr_generated_index_raw = &i;
                        let sifr_generated_index_normalized = sifr_generated_index_raw
                            .normalize_index_or_len(self.state_words_field.len());
                        if let Some(sifr_generated_elem) = self
                            .state_words_field
                            .get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            }
            self.index_field = SifrInt::from_i64(0);
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn sifr_generated_next_u32(&mut self) -> SifrInt {
            if self.index_field >= sifr_generated_const_5f4d545f4e() {
                self.sifr_generated_twist();
            }
            let mut y: SifrInt =
                sifr_generated_state_word_at(&self.state_words_field, &self.index_field);
            self.index_field =
                ::std::ops::Add::add(&self.index_field.clone(), &SifrInt::from_i64(1));
            y = ::std::ops::BitXor::bitxor(
                &y,
                &y.floor_div_known_nonzero(&SifrInt::from_i64(2048)),
            );
            y = ::std::ops::BitXor::bitxor(
                &y,
                &::std::ops::BitAnd::bitand(
                    &::std::ops::Mul::mul(&y, &SifrInt::from_i64(128)),
                    &SifrInt::from_i64(2_636_928_640),
                ),
            );
            y = ::std::ops::BitXor::bitxor(
                &y,
                &::std::ops::BitAnd::bitand(
                    &::std::ops::Mul::mul(&y, &SifrInt::from_i64(32768)),
                    &SifrInt::from_i64(4_022_730_752),
                ),
            );
            y = ::std::ops::BitXor::bitxor(
                &y,
                &y.floor_div_known_nonzero(&SifrInt::from_i64(262_144)),
            );
            ::std::ops::BitAnd::bitand(&y, &sifr_generated_const_5f4d545f574f52445f4d41534b())
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn randrange(
            &mut self,
            start: &SifrInt,
            stop: &Option<SifrInt>,
            step_argument_af0b4e191da20cef: &SifrInt,
        ) -> Result<SifrInt, ValueError> {
            if step_argument_af0b4e191da20cef == &SifrInt::from_i64(0) {
                return Err(ValueError::new(
                    "randrange: step must not be zero".to_string(),
                ));
            }
            let mut actual_start: SifrInt = (*start).clone();
            let mut actual_stop_value_351bdef5a4961be0: SifrInt = (*start).clone();
            if stop.is_none() {
                actual_start = SifrInt::from_i64(0);
            } else if let Some(stop) = stop.as_ref() {
                actual_stop_value_351bdef5a4961be0.clone_from(stop);
            }
            let width: SifrInt =
                ::std::ops::Sub::sub(&actual_stop_value_351bdef5a4961be0, &actual_start);
            if step_argument_af0b4e191da20cef > &SifrInt::from_i64(0) {
                if width <= SifrInt::from_i64(0) {
                    return Err(ValueError::new("randrange: empty range".to_string()));
                }
            } else if width >= SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let mut abs_width: SifrInt = width;
            if abs_width < SifrInt::from_i64(0) {
                abs_width = ::std::ops::Sub::sub(&SifrInt::from_i64(0), &abs_width);
            }
            let mut abs_step: SifrInt = (*step_argument_af0b4e191da20cef).clone();
            if abs_step < SifrInt::from_i64(0) {
                abs_step = ::std::ops::Sub::sub(&SifrInt::from_i64(0), &abs_step);
            }
            if abs_step == SifrInt::from_i64(0) {
                return Err(ValueError::new(
                    "randrange: step must not be zero".to_string(),
                ));
            }
            let count: SifrInt = ::std::ops::Sub::sub(
                &::std::ops::Add::add(&abs_width, &abs_step),
                &SifrInt::from_i64(1),
            )
            .floor_div_known_nonzero(&abs_step);
            if count <= SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            if count == SifrInt::from_i64(0) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
            let pick: SifrInt = self
                .sifr_generated_next_u32()
                .floor_mod_known_nonzero(&count);
            Ok(::std::ops::Add::add(
                &actual_start,
                &::std::ops::Mul::mul(&pick, step_argument_af0b4e191da20cef),
            ))
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        #[must_use]
        pub fn getstate(&self) -> SifrGeneratedStdlibSifrX2erandomX2eRandomState {
            SifrGeneratedStdlibSifrX2erandomX2eRandomState::new(
                &SifrInt::from_i64(3),
                sifr_generated_clone_words(&self.state_words_field),
                &self.index_field,
                self.gauss_next_field,
            )
        }
    }
    impl SifrGeneratedStdlibSifrX2erandomX2eRandom {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn setstate(
            &mut self,
            state: &SifrGeneratedStdlibSifrX2erandomX2eRandomState,
        ) -> Result<(), ValueError> {
            if state.version != SifrInt::from_i64(3) {
                return Err(ValueError::new("setstate: unsupported version".to_string()));
            }
            if state.state_words.len() != sifr_generated_const_5f4d545f4e() {
                return Err(ValueError::new(
                    "setstate: state_words must have length 624".to_string(),
                ));
            }
            if state.index < SifrInt::from_i64(0) || state.index > sifr_generated_const_5f4d545f4e()
            {
                return Err(ValueError::new(
                    "setstate: index must be in range [0, 624]".to_string(),
                ));
            }
            let mut normalized: Vec<SifrInt> = Vec::new();
            #[expect(
                clippy::explicit_iter_loop,
                reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
            )]
            for word in state.state_words.iter() {
                if word < &SifrInt::from_i64(0)
                    || word > &sifr_generated_const_5f4d545f574f52445f4d41534b()
                {
                    return Err(ValueError::new("setstate: word out of range".to_string()));
                }
                normalized.push(::std::ops::BitAnd::bitand(
                    word,
                    &sifr_generated_const_5f4d545f574f52445f4d41534b(),
                ));
            }
            self.state_words_field = normalized;
            self.index_field.clone_from(&state.index);
            self.gauss_next_field = state.gauss_next;
            Ok(())
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IndexError {
        pub message: String,
    }
    impl IndexError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for IndexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IndexError {}
}
use crate::sifr_generated_generated_support::{
    chain, choice, choices, combinations, compare_digest, islice, permutations, product, randbits,
    randrange, shuffle, starmap, token_hex,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IndexError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandom;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2erandomX2eRandomState;
pub use sifr_generated_project_nominals::ValueError;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Doubler {}
impl Doubler {
    const fn new() -> Self {
        Self {}
    }
}
impl ::std::default::Default for Doubler {
    fn default() -> Self {
        Self::new()
    }
}
impl Doubler {
    #[expect(
        clippy::unused_self,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    fn sifr_generated_call__(&self, x: &SifrInt) -> SifrInt {
        ::std::ops::Mul::mul(x, &SifrInt::from_i64(2))
    }
}
fn add(a: &SifrInt, b: &SifrInt) -> SifrInt {
    ::std::ops::Add::add(a, b)
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(0usize));
        sifr_generated_concat.push_str("chain(*iterables) = ");
        sifr_generated_concat.push_str(
            format!(
                "{:?}",
                chain(&[
                    vec![SifrInt::from_i64(1)],
                    vec![SifrInt::from_i64(2)],
                    vec![SifrInt::from_i64(3)],
                    vec![SifrInt::from_i64(4)]
                ])
                .collect::<Vec<_>>()
            )
            .as_str(),
        );
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let sliced: Box<dyn Iterator<Item = SifrInt>> = islice(
            Box::new(
                vec![
                    SifrInt::from_i64(10),
                    SifrInt::from_i64(20),
                    SifrInt::from_i64(30),
                    SifrInt::from_i64(40),
                    SifrInt::from_i64(50),
                ]
                .into_iter(),
            ),
            Some(&SifrInt::from_i64(1)),
            &[Some(SifrInt::from_i64(5)), Some(SifrInt::from_i64(2))],
        )?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(28usize.saturating_add(0usize));
            sifr_generated_concat.push_str("islice(start, stop, step) = ");
            sifr_generated_concat.push_str(format!("{:?}", sliced.collect::<Vec<_>>()).as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(14usize.saturating_add(0usize));
            sifr_generated_concat.push_str("islice error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(0usize));
        sifr_generated_concat.push_str("product(repeat=2) = ");
        sifr_generated_concat.push_str(
            format!(
                "{:?}",
                product(
                    &[vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]],
                    SifrInt::from_i64(2)
                )
                .collect::<Vec<_>>()
            )
            .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(0usize));
        sifr_generated_concat.push_str("permutations(r=2) = ");
        sifr_generated_concat.push_str(
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
                    Some(&SifrInt::from_i64(2))
                )
                .collect::<Vec<_>>()
            )
            .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(20usize.saturating_add(0usize));
        sifr_generated_concat.push_str("combinations(r=2) = ");
        sifr_generated_concat.push_str(
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
            )
            .as_str(),
        );
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(22usize.saturating_add(0usize));
        sifr_generated_concat.push_str("starmap(add, pairs) = ");
        sifr_generated_concat.push_str(
            format!(
                "{:?}",
                starmap(
                    add,
                    Box::new(
                        vec![
                            (SifrInt::from_i64(2), SifrInt::from_i64(3)),
                            (SifrInt::from_i64(4), SifrInt::from_i64(5))
                        ]
                        .into_iter()
                    )
                )
                .collect::<Vec<_>>()
            )
            .as_str(),
        );
        sifr_generated_concat
    });
    let doubler: Doubler = Doubler::new();
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(25usize.saturating_add(0usize));
        sifr_generated_concat.push_str("callable object direct = ");
        sifr_generated_concat.push_str(
            doubler
                .sifr_generated_call__(&SifrInt::from_i64(4))
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let mut items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    shuffle(&mut items);
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(25usize.saturating_add(0usize));
        sifr_generated_concat.push_str("shuffle(mut items) len = ");
        sifr_generated_concat.push_str(SifrInt::from(items.len()).to_string().as_str());
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let picked: SifrInt = choice(&items)?;
        let many: Vec<SifrInt> = choices(&items, &SifrInt::from_i64(3))?;
        let rr: SifrInt = randrange(&SifrInt::from_i64(10), None, &SifrInt::from_i64(1))?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(19usize.saturating_add(0usize));
            sifr_generated_concat.push_str("choice(items) ok = ");
            sifr_generated_concat.push_str(
                (picked >= SifrInt::from_i64(1) && picked <= SifrInt::from_i64(5))
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(26usize.saturating_add(0usize));
            sifr_generated_concat.push_str("choices(items, k=3) len = ");
            sifr_generated_concat.push_str(SifrInt::from(many.len()).to_string().as_str());
            sifr_generated_concat
        });
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(19usize.saturating_add(0usize));
            sifr_generated_concat.push_str("randrange(10) ok = ");
            sifr_generated_concat.push_str(
                (rr >= SifrInt::from_i64(0) && rr < SifrInt::from_i64(10))
                    .to_string()
                    .as_str(),
            );
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(14usize.saturating_add(0usize));
            sifr_generated_concat.push_str("random error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(25usize.saturating_add(0usize));
        sifr_generated_concat.push_str("secrets.compare_digest = ");
        sifr_generated_concat.push_str(compare_digest("abc", "abc").to_string().as_str());
        sifr_generated_concat
    });
    println!("{}", {
        let mut sifr_generated_concat: String =
            String::with_capacity(27usize.saturating_add(0usize));
        sifr_generated_concat.push_str("secrets.token_hex(4) len = ");
        sifr_generated_concat.push_str(
            SifrInt::from(token_hex(&SifrInt::from_i64(4)).chars().count())
                .to_string()
                .as_str(),
        );
        sifr_generated_concat
    });
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let bits: SifrInt = randbits(&SifrInt::from_i64(16))?;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(26usize.saturating_add(0usize));
            sifr_generated_concat.push_str("secrets.randbits(16) ok = ");
            sifr_generated_concat.push_str((bits >= SifrInt::from_i64(0)).to_string().as_str());
            sifr_generated_concat
        });
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(15usize.saturating_add(0usize));
            sifr_generated_concat.push_str("secrets error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
}
