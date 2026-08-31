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

fn fibonacci(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(__SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
    let mut a: SifrInt = SifrInt::from_i64(0);
    let mut b: SifrInt = SifrInt::from_i64(1);
    let mut count: SifrInt = SifrInt::from_i64(0);
    while &count < &n {
        __sifr_yielder.suspend(a.clone()).await;
        let temp: SifrInt = &a + &b;
        a = b.clone();
        b = temp.clone();
        count = &count + &SifrInt::from_i64(1);
    }
}))
}

fn squares(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(__SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &n {
        __sifr_yielder.suspend(&i * &i).await;
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

fn count_up(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(__SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &n {
        __sifr_yielder.suspend(i.clone()).await;
        i = &i + &SifrInt::from_i64(1);
    }
}))
}

fn format_int_list(values: &Vec<SifrInt>) -> String {
    if &SifrInt::from(values.len()) == &SifrInt::from_i64(0) {
        return "[]".to_string();
    }
    let mut formatted: String = "[".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(values.len())) {
        let Some(__sifr_checked_value_0) = ({
    let __sifr_checked_read_collection = &values;
    let __sifr_checked_read_index = i.clone();
    let __sifr_checked_read_normalized = __sifr_checked_read_index.normalize_index_or_len(__sifr_checked_read_collection.len());
    __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
}) else {
            break;
        };
        formatted.push_str((format!("{}", __sifr_checked_value_0.clone())).as_str());
        if (&(&i + &SifrInt::from_i64(1)) < &SifrInt::from(values.len())) {
            formatted.push_str(", ");
        }
        i = &i + &SifrInt::from_i64(1);
    }
    formatted.push(']');
    formatted
}

fn main() {
    let mut output: Vec<String> = vec![];
    output.push("=== Fibonacci (lazy for loop) ===".to_string());
    for fib in fibonacci(SifrInt::from_i64(8)) {
        output.push(format!("{}", fib));
    }
    output.push("=== Squares (collected) ===".to_string());
    let sq: Vec<SifrInt> = squares(SifrInt::from_i64(5)).collect::<Vec<_>>();
    output.push(format_int_list(&sq));
    output.push("=== Evens (conditional yield) ===".to_string());
    for e in evens(SifrInt::from_i64(10)) {
        output.push(format!("{}", e));
    }
    output.push("=== Count (lazy) ===".to_string());
    for c in count_up(SifrInt::from_i64(3)) {
        output.push(format!("{}", c));
    }
    output.push("=== Count (collected) ===".to_string());
    let nums: Vec<SifrInt> = count_up(SifrInt::from_i64(5)).collect::<Vec<_>>();
    output.push(format_int_list(&nums));
    assert!((output == vec!["=== Fibonacci (lazy for loop) ===".to_string(), "0".to_string(), "1".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "5".to_string(), "8".to_string(), "13".to_string(), "=== Squares (collected) ===".to_string(), "[0, 1, 4, 9, 16]".to_string(), "=== Evens (conditional yield) ===".to_string(), "0".to_string(), "2".to_string(), "4".to_string(), "6".to_string(), "8".to_string(), "=== Count (lazy) ===".to_string(), "0".to_string(), "1".to_string(), "2".to_string(), "=== Count (collected) ===".to_string(), "[0, 1, 2, 3, 4]".to_string()]));
    println!("Lazy iterator demo output:");
    for item in output.iter().cloned() {
        println!("{}", item);
    }
}
