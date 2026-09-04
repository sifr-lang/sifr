// src/main.rs
pub mod sifr_generated_generated_support {
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
}
use crate::sifr_generated_generated_support::{SifrGeneratedGenerator, SifrGeneratedYielder};
use ::sifr_runtime::SifrInt;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Timer {
    label: String,
}
impl Timer {
    const fn new(label: String) -> Self {
        let sifr_generated_field_value_39f7fcec8fcb623d_6c6162656c: String = label;
        Self {
            label: sifr_generated_field_value_39f7fcec8fcb623d_6c6162656c,
        }
    }
}
impl Timer {
    fn sifr_generated_enter__(&self) -> Self {
        self.clone()
    }
}
impl Timer {
    #[expect(
        clippy::unused_self,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    const fn sifr_generated_exit__(&self) {}
}
impl ::std::fmt::Display for Timer {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Timer(label={})", self.label)
    }
}
fn fibonacci(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut a: SifrInt = SifrInt::from_i64(0);
            let mut b: SifrInt = SifrInt::from_i64(1);
            let mut i: SifrInt = SifrInt::from_i64(0);
            while i < n {
                sifr_generated_yielder.suspend(a.clone()).await;
                let temp: SifrInt = ::std::ops::Add::add(&a, &b);
                a.clone_from(&b);
                b.clone_from(&temp);
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            }
        },
    ))
}
fn evens(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while i < limit {
                if i.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == SifrInt::from_i64(0) {
                    sifr_generated_yielder.suspend(i.clone()).await;
                }
                i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
            }
        },
    ))
}
fn main() {
    let fibs: Vec<SifrInt> = fibonacci(SifrInt::from_i64(8)).collect::<Vec<_>>();
    println!("{fibs:?}");
    let even_nums: Vec<SifrInt> = evens(SifrInt::from_i64(10)).collect::<Vec<_>>();
    println!("{even_nums:?}");
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    for x in nums.iter().cloned() {
        println!("{}", ::std::ops::Mul::mul(&x, &x));
    }
    {
        struct SifrGeneratedWithGuard0 {
            ctx: Timer,
        }
        impl Drop for SifrGeneratedWithGuard0 {
            fn drop(&mut self) {
                self.ctx.sifr_generated_exit__();
            }
        }
        let sifr_generated_ctx_0 = Timer::new("work".to_string());
        let sifr_generated_guard_0 = SifrGeneratedWithGuard0 {
            ctx: sifr_generated_ctx_0,
        };
        let _ = sifr_generated_guard_0.ctx.sifr_generated_enter__();
        println!("doing work");
    }
    println!("done");
}
