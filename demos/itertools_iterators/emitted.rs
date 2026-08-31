// src/main.rs
mod __sifr_project_nominals {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
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
pub use __sifr_project_nominals::ValueError;
use ::sifr_runtime::SifrInt;
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
fn chain<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
) -> Box<dyn Iterator<Item = T>> {
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
fn repeat<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    value: T,
    times: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let holder: Vec<T> = vec![value.clone()];
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&i < &times) {
                if (&SifrInt::from(holder.len()) > &SifrInt::from_i64(0)) {
                    let current: Option<T> = {
                        let __sifr_checked_read_collection = &holder;
                        let __sifr_checked_read_index = SifrInt::from_i64(0);
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(
                                __sifr_checked_read_collection.len(),
                            );
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current {
                        __sifr_yielder.suspend(current.clone()).await;
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn _islice_impl<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            let mut index: SifrInt = SifrInt::from_i64(0);
            let mut next_yield: SifrInt = start.clone();
            for value in data {
                if &index >= &stop {
                    return;
                }
                if &index == &next_yield {
                    __sifr_yielder.suspend(value.clone()).await;
                    next_yield = &next_yield + &step;
                }
                index = &index + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn islice<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start_or_stop: SifrInt,
    stop: Option<SifrInt>,
    step: SifrInt,
) -> Result<Box<dyn Iterator<Item = T>>, ValueError> {
    let mut actual_start: SifrInt = SifrInt::from_i64(0);
    let mut actual_stop: SifrInt = start_or_stop.clone();
    if let Some(stop) = stop.clone() {
        actual_start = start_or_stop.clone();
        actual_stop = stop.clone();
    }
    if (&actual_start < &SifrInt::from_i64(0)) || (&actual_stop < &SifrInt::from_i64(0))
    {
        return Err(ValueError::new("islice: indices must be non-negative".to_string()));
    }
    if (&step <= &SifrInt::from_i64(0)) {
        return Err(
            ValueError::new("islice: step must be greater than zero".to_string()),
        );
    }
    Ok(
        _islice_impl(
            Box::new(data),
            (actual_start).clone(),
            (actual_stop).clone(),
            (step).clone(),
        ),
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
fn main() {
    let chained: Box<dyn Iterator<Item = SifrInt>> = chain(
        &vec![
            vec![SifrInt::from_i64(1), SifrInt::from_i64(2)], vec![SifrInt::from_i64(3)]
        ],
    );
    println!("{:?}", chained.collect::< Vec < _ >> ());
    let repeated: Box<dyn Iterator<Item = SifrInt>> = repeat(
        SifrInt::from_i64(7),
        SifrInt::from_i64(3),
    );
    println!("{:?}", repeated.collect::< Vec < _ >> ());
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let sliced: Box<dyn Iterator<Item = SifrInt>> = islice(
            Box::new(
                vec![
                    SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30),
                    SifrInt::from_i64(40), SifrInt::from_i64(50)
                ]
                    .into_iter(),
            ),
            SifrInt::from_i64(1),
            Some(SifrInt::from_i64(5)),
            SifrInt::from_i64(2),
        )?;
        println!("{:?}", sliced.collect::< Vec < _ >> ());
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("islice error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    let mut counter: Box<dyn Iterator<Item = SifrInt>> = count(
        SifrInt::from_i64(5),
        SifrInt::from_i64(2),
    );
    println!(
        "{}", (counter.next()).map_or("None".to_string().to_string(), | __v |
        format!("{}", __v))
    );
    println!(
        "{}", (counter.next()).map_or("None".to_string().to_string(), | __v |
        format!("{}", __v))
    );
    println!(
        "{}", (counter.next()).map_or("None".to_string().to_string(), | __v |
        format!("{}", __v))
    );
    println!(
        "{}", (counter.next()).map_or("None".to_string().to_string(), | __v |
        format!("{}", __v))
    );
}
