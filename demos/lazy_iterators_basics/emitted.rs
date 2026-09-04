// src/main.rs
pub mod sifr_generated_generated_support {
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
    pub(super) trait SifrGeneratedAdd: Sized {}
    impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {}
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
}
use crate::sifr_generated_generated_support::{chain, count};
use ::sifr_runtime::SifrInt;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn square(n: SifrInt) -> SifrInt {
    ::std::ops::Mul::mul(&n, &n)
}
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
    assert_eq!(
        format!(
            "{:?}",
            Box::new(nums.iter().cloned().map(square)).collect::<Vec<_>>()
        ),
        "[1, 4, 9, 16]"
    );
    assert_eq!(
        format!(
            "{:?}",
            Box::new(
                nums.iter()
                    .filter(move |&sifr_generated_filter_item| {
                        let x = sifr_generated_filter_item.clone();
                        x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == SifrInt::from_i64(0)
                    })
                    .cloned()
            )
            .collect::<Vec<_>>()
        ),
        "[2, 4]"
    );
    assert_eq!(
        format!(
            "{:?}",
            Box::new(
                nums.iter()
                    .cloned()
                    .zip(vec![
                        "a".to_string(),
                        "b".to_string(),
                        "c".to_string(),
                        "d".to_string()
                    ])
                    .map(|sifr_generated_zip_item| (
                        sifr_generated_zip_item.0,
                        sifr_generated_zip_item.1
                    ))
            )
            .collect::<Vec<_>>()
        ),
        "[(1, \"a\"), (2, \"b\"), (3, \"c\"), (4, \"d\")]"
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
                            SifrInt::from_i64(10)
                        ),
                        sifr_generated_pair.1
                    ))
            )
            .collect::<Vec<_>>()
        ),
        "[(10, \"x\"), (11, \"y\")]"
    );
    assert_eq!(
        format!(
            "{:?}",
            Box::new(nums.iter().cloned().rev()).collect::<Vec<_>>()
        ),
        "[4, 3, 2, 1]"
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
    let mut ticker: Box<dyn Iterator<Item = SifrInt>> =
        count(SifrInt::from_i64(3), SifrInt::from_i64(2));
    assert_eq!(ticker.next(), Some(SifrInt::from_i64(3)));
    assert_eq!(ticker.next(), Some(SifrInt::from_i64(5)));
    assert_eq!(ticker.next(), Some(SifrInt::from_i64(7)));
    println!("iter_fix_lazy_iterators_basics_lock_demo: ok");
}
