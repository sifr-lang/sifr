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
    pub(super) trait SifrGeneratedAdd: Sized {
        #[must_use]
        fn sifr_generated_add(self, rhs: Self) -> Self;
    }
    impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {
        fn sifr_generated_add(self, rhs: Self) -> Self {
            ::std::ops::Add::add(self, rhs)
        }
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
    pub(super) fn pairwise<T: Clone + 'static>(data: &[T]) -> Vec<Vec<T>> {
        let mut result: Vec<Vec<T>> = Vec::new();
        let mut prev_values: Vec<T> = Vec::new();
        for value in data.iter().cloned() {
            if prev_values.len() > SifrInt::from_i64(0) {
                let mut pair: Vec<T> = Vec::new();
                let prev: Option<T> = {
                    let sifr_generated_checked_read_collection = &prev_values;
                    let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                    let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                        .normalize_index_or_len(sifr_generated_checked_read_collection.len());
                    sifr_generated_checked_read_collection
                        .get(sifr_generated_checked_read_normalized)
                        .cloned()
                };
                if let Some(prev) = prev {
                    pair.push(prev);
                }
                pair.push(value.clone());
                result.push(pair);
                {
                    let sifr_generated_assign_value = value.clone();
                    {
                        let sifr_generated_index_raw = SifrInt::from_i64(0);
                        let sifr_generated_index_normalized =
                            sifr_generated_index_raw.normalize_index_or_len(prev_values.len());
                        if let Some(sifr_generated_elem) =
                            prev_values.get_mut(sifr_generated_index_normalized)
                        {
                            *sifr_generated_elem = sifr_generated_assign_value;
                        }
                    }
                }
            } else {
                prev_values.push(value);
            }
        }
        result
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn batched<T: Clone + 'static>(
        data: &[T],
        n: SifrInt,
    ) -> Result<Vec<Vec<T>>, ValueError> {
        if n <= SifrInt::from_i64(0) {
            return Err(ValueError::new("batched: n must be > 0".to_string()));
        }
        let mut result: Vec<Vec<T>> = Vec::new();
        let mut current_batch: Vec<T> = Vec::new();
        for value in data.iter().cloned() {
            current_batch.push(value);
            if current_batch.len() == n {
                result.push(current_batch.clone());
                current_batch = Vec::new();
            }
        }
        if current_batch.len() > SifrInt::from_i64(0) {
            result.push(current_batch);
        }
        Ok(result)
    }
    pub(super) fn accumulate<T: Clone + 'static + SifrGeneratedAdd>(
        data: Box<dyn Iterator<Item = T>>,
        initial: Option<T>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                let mut state: Vec<T> = Vec::new();
                if let Some(initial) = initial {
                    state.push(initial);
                    let initial_value: Option<T> = {
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
                    if let Some(initial_value) = initial_value {
                        sifr_generated_yielder.suspend(initial_value).await;
                    }
                }
                for item in data {
                    if state.len() == SifrInt::from_i64(0) {
                        state.push(item);
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
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
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
                                let _ = sifr_generated_try_err;
                                return;
                            }
                        }
                    }
                    let current: Option<T> = {
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
                    if let Some(current) = current {
                        sifr_generated_yielder.suspend(current).await;
                    }
                }
            },
        ))
    }
    pub(super) fn cycle<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        n: SifrInt,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                let mut saved: Vec<T> = Vec::new();
                let mut emitted: SifrInt = SifrInt::from_i64(0);
                if n <= SifrInt::from_i64(0) {
                    return;
                }
                for value in data {
                    saved.push(value.clone());
                    sifr_generated_yielder.suspend(value.clone()).await;
                    emitted = ::std::ops::Add::add(&emitted, &SifrInt::from_i64(1));
                    if emitted >= n {
                        return;
                    }
                }
                while emitted < n && saved.len() > SifrInt::from_i64(0) {
                    #[expect(
                        clippy::explicit_iter_loop,
                        reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
                    )]
                    for repeated in saved.iter() {
                        sifr_generated_yielder.suspend(repeated.clone()).await;
                        emitted = ::std::ops::Add::add(&emitted, &SifrInt::from_i64(1));
                        if emitted >= n {
                            return;
                        }
                    }
                }
            },
        ))
    }
    pub(super) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < actual.len() {
            assert_eq!(
                {
                    let sifr_generated_condition_list = &actual;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                },
                {
                    let sifr_generated_condition_list = &expected;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                }
            );
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
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
    accumulate, assert_bool_vector_eq, batched, chain, cycle, pairwise,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IndexError;
pub use sifr_generated_project_nominals::ValueError;
fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![
        format!(
            "{:?}",
            chain(&[
                vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
                vec![SifrInt::from_i64(3)]
            ])
            .collect::<Vec<_>>()
        )
        .as_str()
            == "[1, 2, 3]".to_string().as_str(),
        format!(
            "{:?}",
            pairwise(
                &vec![
                    SifrInt::from_i64(1),
                    SifrInt::from_i64(2),
                    SifrInt::from_i64(3),
                    SifrInt::from_i64(4)
                ]
                .into_iter()
                .collect::<Vec<_>>()
            )
        )
        .as_str()
            == "[[1, 2], [2, 3], [3, 4]]".to_string().as_str(),
    ];
    let mut batched_ok: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let bat: Vec<Vec<SifrInt>> = batched(
            &vec![
                SifrInt::from_i64(1),
                SifrInt::from_i64(2),
                SifrInt::from_i64(3),
                SifrInt::from_i64(4),
                SifrInt::from_i64(5),
            ]
            .into_iter()
            .collect::<Vec<_>>(),
            SifrInt::from_i64(2),
        )?;
        batched_ok = format!("{bat:?}") == "[[1, 2], [3, 4], [5]]";
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
    }
    actual.push(batched_ok);
    actual.push(
        format!(
            "{:?}",
            accumulate(
                Box::new(
                    vec![
                        SifrInt::from_i64(1),
                        SifrInt::from_i64(2),
                        SifrInt::from_i64(3)
                    ]
                    .into_iter()
                ),
                None
            )
            .collect::<Vec<_>>()
        )
        .as_str()
            == "[1, 3, 6]".to_string().as_str(),
    );
    actual.push(
        format!(
            "{:?}",
            cycle(
                Box::new(vec![SifrInt::from_i64(5), SifrInt::from_i64(6)].into_iter()),
                SifrInt::from_i64(5)
            )
            .collect::<Vec<_>>()
        )
        .as_str()
            == "[5, 6, 5, 6, 5]".to_string().as_str(),
    );
    actual
}
fn collect_negative_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut invalid_batch_rejected: bool = false;
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let sifr_generated_bad: Vec<Vec<SifrInt>> = batched(
            &vec![SifrInt::from_i64(1)].into_iter().collect::<Vec<_>>(),
            SifrInt::from_i64(0),
        )?;
        let _ = sifr_generated_bad;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        invalid_batch_rejected = e.message.chars().count() > SifrInt::from_i64(0);
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
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_core_actual());
    append_all(&mut actual, &collect_negative_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("itertools itertools parity demo: pass");
}
