// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: sifr.itertools ---
struct __SifrYielder<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
}
struct __SifrYieldFuture<T> {
    slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: Option<T>,
}
impl<T> Unpin for __SifrYieldFuture<T> {}
impl<T> ::std::future::Future for __SifrYieldFuture<T> {
    type Output = ();
    fn poll(
        self: ::std::pin::Pin<&mut Self>,
        _cx: &mut ::std::task::Context<'_>,
    ) -> ::std::task::Poll<()> {
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
        __SifrYieldFuture {
            slot: ::std::sync::Arc::clone(&self.slot),
            value: Some(value),
        }
    }
}
fn __sifr_store_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    value: T,
) {
    match slot.lock() {
        Ok(mut state) => *state = Some(value),
        Err(poisoned) => *poisoned.into_inner() = Some(value),
    }
}
fn __sifr_take_suspended<T>(
    slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
) -> Option<T> {
    match slot.lock() {
        Ok(mut state) => state.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    }
}
struct __SifrGenerator<T> {
    producer: Option<
        ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>,
    >,
    yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    complete: bool,
}
impl<T> __SifrGenerator<T> {
    fn new<
        F: FnOnce(__SifrYielder<T>) -> Fut + 'static,
        Fut: ::std::future::Future<Output = ()> + 'static,
    >(factory: F) -> Self {
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
            let mut context = ::std::task::Context::from_waker(
                ::std::task::Waker::noop(),
            );
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
fn chain<T: Clone + 'static>(iterables: &Vec<Vec<T>>) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.clone();
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for iterable in iterables.iter().cloned() {
                for item in iterable.iter().cloned() {
                    __sifr_yielder.suspend(item.clone()).await;
                }
            }
        }),
    )
}
fn count(start: SifrInt, step: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<SifrInt>| {
            let mut current: SifrInt = start.clone();
            loop {
                __sifr_yielder.suspend(current.clone()).await;
                current = &current + &step;
            }
        }),
    )
}
// --- end stdlib ---

fn square(n: SifrInt) -> SifrInt {
    &n * &n
}

fn main() {
    let nums: Vec<SifrInt> = vec![SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3), SifrInt::from_i64(4)];
    let mut it: Box<dyn Iterator<Item = SifrInt>> = Box::new((nums).iter().cloned());
    assert!((it.next() == Some(SifrInt::from_i64(1))));
    assert!((it.next() == Some(SifrInt::from_i64(2))));
    assert!((format!("{:?}", Box::new(nums.iter().cloned().map(|__sifr_map_item| square(__sifr_map_item))).collect::<Vec<_>>()) == "[1, 4, 9, 16]"));
    assert!((format!("{:?}", Box::new((nums).iter().cloned().filter(move |__filter_item| {
    let x = __filter_item.clone();
    (&x.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0))
})).collect::<Vec<_>>()) == "[2, 4]"));
    assert!((format!("{:?}", Box::new((nums).iter().cloned().zip((vec!["a".to_string(), "b".to_string(), "c".to_string(), "d".to_string()]).into_iter()).map(|__zip_item| (__zip_item.0, __zip_item.1))).collect::<Vec<_>>()) == "[(1, \"a\"), (2, \"b\"), (3, \"c\"), (4, \"d\")]"));
    assert!((format!("{:?}", Box::new((vec!["x".to_string(), "y".to_string()]).into_iter().enumerate().map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(10), __pair.1))).collect::<Vec<_>>()) == "[(10, \"x\"), (11, \"y\")]"));
    assert!((format!("{:?}", Box::new((nums).iter().cloned().rev()).collect::<Vec<_>>()) == "[4, 3, 2, 1]"));
    assert!((format!("{:?}", chain(&vec![vec![SifrInt::from_i64(1), SifrInt::from_i64(2)], vec![SifrInt::from_i64(3)]]).collect::<Vec<_>>()) == "[1, 2, 3]"));
    let mut ticker: Box<dyn Iterator<Item = SifrInt>> = count(SifrInt::from_i64(3), SifrInt::from_i64(2));
    assert!((ticker.next() == Some(SifrInt::from_i64(3))));
    assert!((ticker.next() == Some(SifrInt::from_i64(5))));
    assert!((ticker.next() == Some(SifrInt::from_i64(7))));
    println!("iter_fix_lazy_iterators_basics_lock_demo: ok");
}
