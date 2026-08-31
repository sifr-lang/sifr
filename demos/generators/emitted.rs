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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Timer {
    label: String,
}

impl Timer {
    fn new(label: String) -> Self {
        let __sifr_field_init_0: String = label;
        Self { label: __sifr_field_init_0 }
    }
}

impl Timer {
    fn __enter__(&self) -> Timer {
        self.clone()
    }
}

impl Timer {
    fn __exit__(&self) {
    }
}

impl ::std::fmt::Display for Timer {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Timer(label={})", self.label)
    }
}

fn fibonacci(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(__SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
    let mut a: SifrInt = SifrInt::from_i64(0);
    let mut b: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &n {
        __sifr_yielder.suspend(a.clone()).await;
        let temp: SifrInt = &a + &b;
        a = b.clone();
        b = temp.clone();
        i = &i + &SifrInt::from_i64(1);
    }
}))
}

fn evens(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(__SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &limit) {
        if (&i.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0)) {
            __sifr_yielder.suspend(i.clone()).await;
        }
        i = &i + &SifrInt::from_i64(1);
    }
}))
}

fn main() {
    let fibs: Vec<SifrInt> = fibonacci(SifrInt::from_i64(8)).collect::<Vec<_>>();
    println!("{:?}", fibs);
    let even_nums: Vec<SifrInt> = evens(SifrInt::from_i64(10)).collect::<Vec<_>>();
    println!("{:?}", even_nums);
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4), SifrInt::from_i64(5)];
    for x in nums.iter().cloned() {
        println!("{}", &x * &x);
    }
    {
        let mut __ctx_0 = Timer::new("work".to_string());
        struct __WithGuard0 { ctx: Timer }
        impl Drop for __WithGuard0 {
            fn drop(&mut self) { self.ctx.__exit__(); }
        }
        let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
        let _t = __guard_0.ctx.__enter__();
        println!("doing work");
    }
    println!("done");
}
