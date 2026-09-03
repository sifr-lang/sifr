// src/main.rs
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
}
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::ValueError;
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
fn chain<T: Clone + 'static>(iterables: &[Vec<T>]) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.to_vec();
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            for iterable in iterables.iter().cloned() {
                for item in iterable.iter().cloned() {
                    sifr_generated_yielder.suspend(item.clone()).await;
                }
            }
        },
    ))
}
fn pairwise<T: Clone + 'static>(data: &[T]) -> Vec<Vec<T>> {
    let mut result: Vec<Vec<T>> = Vec::new();
    let mut prev_values: Vec<T> = Vec::new();
    for value in data.iter().cloned() {
        if &SifrInt::from(prev_values.len()) > &SifrInt::from_i64(0) {
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
                pair.push(prev.clone());
            }
            pair.push(value.clone());
            result.push(pair.to_vec());
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
            prev_values.push(value.clone());
        }
    }
    result
}
fn batched<T: Clone + 'static>(data: &[T], n: SifrInt) -> Result<Vec<Vec<T>>, ValueError> {
    if &n <= &SifrInt::from_i64(0) {
        return Err(ValueError::new("batched: n must be > 0".to_string()));
    }
    let mut result: Vec<Vec<T>> = Vec::new();
    let mut current_batch: Vec<T> = Vec::new();
    for value in data.iter().cloned() {
        current_batch.push(value.clone());
        if &SifrInt::from(current_batch.len()) == &n {
            result.push(current_batch.to_vec());
            current_batch = Vec::new();
        }
    }
    if &SifrInt::from(current_batch.len()) > &SifrInt::from_i64(0) {
        result.push(current_batch.to_vec());
    }
    Ok(result)
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
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
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
        i = &i + &SifrInt::from_i64(1);
    }
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
fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![
        format!(
            "{:?}",
            chain(&vec![
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
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
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
        let e = sifr_generated_try_err.clone();
        invalid_batch_rejected = &SifrInt::from(e.message.chars().count()) > &SifrInt::from_i64(0);
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
