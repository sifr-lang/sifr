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
fn gen_pairs(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &limit {
                sifr_generated_yielder.suspend(i.clone()).await;
                i = &i + &SifrInt::from_i64(1);
                if &i < &limit {
                    sifr_generated_yielder.suspend(i.clone()).await;
                    i = &i + &SifrInt::from_i64(1);
                }
            }
        },
    ))
}
fn gen_even(xs: &[SifrInt]) -> Box<dyn Iterator<Item = SifrInt>> {
    let xs = xs.to_vec();
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            for x in xs.iter().cloned() {
                if &x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0) {
                    sifr_generated_yielder.suspend(x.clone()).await;
                }
            }
        },
    ))
}
fn main() {
    let xs: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
        SifrInt::from_i64(5),
    ];
    let squares: Box<dyn Iterator<Item = SifrInt>> = Box::new(xs.iter().cloned().filter_map(|x| {
        if &x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0) {
            Some(&x * &x)
        } else {
            None
        }
    }));
    println!("{:?}", squares.collect::<Vec<_>>());
    println!("{:?}", gen_pairs(SifrInt::from_i64(5)).collect::<Vec<_>>());
    println!("{:?}", gen_even(&xs).collect::<Vec<_>>());
}
