// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{IndexError, ValueError};
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
    pub(super) fn repeat<T: Clone + 'static>(
        value: T,
        times: SifrInt,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                let holder: Vec<T> = vec![value];
                let mut i: SifrInt = SifrInt::from_i64(0);
                while i < times {
                    if holder.len() > SifrInt::from_i64(0) {
                        let current: Option<T> = {
                            let sifr_generated_checked_read_collection = &holder;
                            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        if let Some(current) = current {
                            sifr_generated_yielder.suspend(current).await;
                        }
                    }
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
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
    pub(super) fn count(start: SifrInt, step: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
                let mut current: SifrInt = start.clone();
                loop {
                    sifr_generated_yielder.suspend(current.clone()).await;
                    current = ::std::ops::Add::add(&current, &step);
                }
            },
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
}
mod sifr_generated_project_nominals {
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
    SifrGeneratedGenerator, SifrGeneratedYielder, chain, combinations, count, islice, product,
    repeat,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IndexError;
pub use sifr_generated_project_nominals::ValueError;
fn odds(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while i < limit {
                if i.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == SifrInt::from_i64(1) {
                    sifr_generated_yielder.suspend(i.clone()).await;
                }
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            }
        },
    ))
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
#[expect(
    clippy::assertions_on_constants,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    ];
    let mut it: Box<dyn Iterator<Item = SifrInt>> = Box::new(nums.iter().cloned());
    assert_eq!(it.next(), Some(SifrInt::from_i64(1)));
    assert_eq!(it.next(), Some(SifrInt::from_i64(2)));
    let mut doubled: Vec<SifrInt> = Vec::new();
    #[expect(
        clippy::explicit_iter_loop,
        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
    )]
    for n in nums.iter() {
        doubled.push(::std::ops::Mul::mul(n, &SifrInt::from_i64(2)));
    }
    assert_eq!(format!("{doubled:?}"), "[2, 4, 6, 8]");
    let mut odd_it: Box<dyn Iterator<Item = SifrInt>> = odds(SifrInt::from_i64(7));
    assert_eq!(odd_it.next(), Some(SifrInt::from_i64(1)));
    assert_eq!(odd_it.next(), Some(SifrInt::from_i64(3)));
    assert_eq!(odd_it.next(), Some(SifrInt::from_i64(5)));
    assert!(odd_it.next().is_none());
    assert_eq!(
        format!(
            "{:?}",
            Box::new(
                vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]
                    .into_iter()
                    .zip(vec!["a".to_string(), "b".to_string()])
                    .map(|sifr_generated_zip_item| (
                        sifr_generated_zip_item.0,
                        sifr_generated_zip_item.1
                    ))
            )
            .collect::<Vec<_>>()
        ),
        "[(1, \"a\"), (2, \"b\")]"
    );
    assert_eq!(
        format!(
            "{:?}",
            Box::new(
                vec!["x".to_string(), "y".to_string()]
                    .into_iter()
                    .enumerate()
                    .map(|sifr_generated_pair| (
                        ::std::ops::Add::add(
                            SifrInt::from(sifr_generated_pair.0),
                            SifrInt::from_i64(4)
                        ),
                        sifr_generated_pair.1
                    ))
            )
            .collect::<Vec<_>>()
        ),
        "[(4, \"x\"), (5, \"y\")]"
    );
    assert_eq!(
        format!(
            "{:?}",
            Box::new(
                vec![
                    SifrInt::from_i64(9),
                    SifrInt::from_i64(8),
                    SifrInt::from_i64(7)
                ]
                .into_iter()
                .rev()
            )
            .collect::<Vec<_>>()
        ),
        "[7, 8, 9]"
    );
    assert_eq!(
        format!(
            "{:?}",
            chain(&[
                vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
                vec![SifrInt::from_i64(3)]
            ])
            .collect::<Vec<_>>()
        ),
        "[1, 2, 3]"
    );
    assert_eq!(
        format!(
            "{:?}",
            repeat(SifrInt::from_i64(5), SifrInt::from_i64(3)).collect::<Vec<_>>()
        ),
        "[5, 5, 5]"
    );
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
        assert_eq!(format!("{:?}", sliced.collect::<Vec<_>>()), "[20, 40]");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        assert!(false, "{}", e.message);
    }
    let mut counter: Box<dyn Iterator<Item = SifrInt>> =
        count(SifrInt::from_i64(2), SifrInt::from_i64(3));
    assert_eq!(counter.next(), Some(SifrInt::from_i64(2)));
    assert_eq!(counter.next(), Some(SifrInt::from_i64(5)));
    assert_eq!(counter.next(), Some(SifrInt::from_i64(8)));
    let combos: Vec<Vec<SifrInt>> = combinations(
        Box::new(
            vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
            ]
            .into_iter(),
        ),
        SifrInt::from_i64(2),
    )
    .collect::<Vec<_>>();
    assert_eq!(format!("{combos:?}"), "[[1, 2], [1, 3], [2, 3]]");
    let prods: Vec<Vec<SifrInt>> = product(
        &[vec![SifrInt::from_i64(1), SifrInt::from_i64(2)]],
        SifrInt::from_i64(2),
    )
    .collect::<Vec<_>>();
    assert_eq!(format!("{prods:?}"), "[[1, 1], [1, 2], [2, 1], [2, 2]]");
    println!("iter_iterator_basics_closure_demo: ok");
}
