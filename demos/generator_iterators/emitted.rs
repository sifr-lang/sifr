// src/main.rs
use ::sifr_runtime::SifrInt;

struct __SifrYielder<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
}

struct __SifrYieldFuture<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: Option<T>,
}

impl<T> Unpin for __SifrYieldFuture<T> {
}

impl<T> ::std::future::Future for __SifrYieldFuture<T> {
    type Output = ();
    fn poll(self: ::std::pin::Pin<&mut Self>, _cx: &mut ::std::task::Context<'_>) -> ::std::task::Poll<()> {
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
        __SifrYieldFuture { slot: ::std::sync::Arc::clone(&self.slot), value: Some(value) }
    }
}

fn __sifr_store_suspended<T>(slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>, value: T) {
    match slot.lock() {
    Ok(mut state) => *state = Some(value),
    Err(poisoned) => *poisoned.into_inner() = Some(value),
}
}

fn __sifr_take_suspended<T>(slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>) -> Option<T> {
    match slot.lock() {
    Ok(mut state) => state.take(),
    Err(poisoned) => poisoned.into_inner().take(),
}
}

struct __SifrGenerator<T> {
    producer: Option<::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>>,
    yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    complete: bool,
}

impl<T> __SifrGenerator<T> {
    fn new<F: FnOnce(__SifrYielder<T>) -> Fut + 'static, Fut: ::std::future::Future<Output = ()> + 'static>(factory: F) -> Self {
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
    let mut context = ::std::task::Context::from_waker(::std::task::Waker::noop());
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

fn gen_pairs(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(__SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &limit {
        __sifr_yielder.suspend(i.clone()).await;
        i = &i + &SifrInt::from_i64(1);
        if &i < &limit {
            __sifr_yielder.suspend(i.clone()).await;
            i = &i + &SifrInt::from_i64(1);
        }
    }
}))
}

fn gen_even(xs: &Vec<SifrInt>) -> Box<dyn Iterator<Item = SifrInt>> {
    let xs = xs.clone();
    Box::new(__SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
    for x in xs.iter().cloned() {
        if (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
            __sifr_yielder.suspend(x.clone()).await;
        }
    }
}))
}

fn main() {
    let xs: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)];
    let squares: Box<dyn Iterator<Item = SifrInt>> = Box::new(xs.iter().cloned().filter_map(|x| if (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) { Some(&x * &x) } else { None }));
    println!("{:?}", squares.collect::<Vec<_>>());
    println!("{:?}", gen_pairs(SifrInt::from_i64(5)).collect::<Vec<_>>());
    println!("{:?}", gen_even(&xs).collect::<Vec<_>>());
}
