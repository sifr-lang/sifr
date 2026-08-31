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

fn countdown(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(__SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
    let mut i: SifrInt = n.clone();
    while &i > &SifrInt::from_i64(0) {
        __sifr_yielder.suspend(i.clone()).await;
        i = &i - &SifrInt::from_i64(1);
    }
}))
}

fn main() {
    let mut it: Box<dyn Iterator<Item = SifrInt>> = countdown(SifrInt::from_i64(3));
    let first: Option<SifrInt> = it.next();
    let second: Option<SifrInt> = it.next();
    let remaining: Vec<SifrInt> = it.collect::<Vec<_>>();
    let all_values: Vec<SifrInt> = countdown(SifrInt::from_i64(4)).collect::<Vec<_>>();
    assert!(first == Some(SifrInt::from_i64(3)));
    assert!(second == Some(SifrInt::from_i64(2)));
    assert!((remaining == vec![SifrInt::from_i64(1)]));
    assert!((all_values == vec![SifrInt::from_i64(4), SifrInt::from_i64(3), SifrInt::from_i64(2), SifrInt::from_i64(1)]));
    println!("{}", (first).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{}", (second).map_or("None".to_string().to_string(), |__v| format!("{}", __v)));
    println!("{:?}", remaining);
    println!("{:?}", all_values);
}
