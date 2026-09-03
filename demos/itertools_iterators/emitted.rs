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
pub trait SifrGeneratedAdd: Sized {}
impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {}
impl SifrGeneratedAdd for String {}
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
fn repeat<T: Clone + 'static>(value: T, times: SifrInt) -> Box<dyn Iterator<Item = T>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            let holder: Vec<T> = vec![value];
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &times {
                if &SifrInt::from(holder.len()) > &SifrInt::from_i64(0) {
                    let current: Option<T> = {
                        let sifr_generated_checked_read_collection = &holder;
                        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(current) = current {
                        sifr_generated_yielder.suspend(current.clone()).await;
                    }
                }
                i = &i + &SifrInt::from_i64(1);
            }
        },
    ))
}
fn sifr_generated_islice_impl<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start: SifrInt,
    stop: SifrInt,
    unbounded: bool,
    step_argument_af0b4e191da20cef: SifrInt,
) -> Box<dyn Iterator<Item = T>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
            let mut index: SifrInt = SifrInt::from_i64(0);
            let mut next_yield: SifrInt = start.clone();
            for value in data {
                if !unbounded && &index >= &stop {
                    return;
                }
                if &index == &next_yield {
                    sifr_generated_yielder.suspend(value.clone()).await;
                    next_yield = &next_yield + &step_argument_af0b4e191da20cef;
                }
                index = &index + &SifrInt::from_i64(1);
            }
        },
    ))
}
fn islice<T: Clone + 'static>(
    data: Box<dyn Iterator<Item = T>>,
    start_or_stop: SifrInt,
    slice_args: &[Option<SifrInt>],
) -> Result<Box<dyn Iterator<Item = T>>, ValueError> {
    if &SifrInt::from(slice_args.len()) > &SifrInt::from_i64(2) {
        return Err(ValueError::new(
            "islice: expected at most stop and step after start".to_string(),
        ));
    }
    let mut actual_start: SifrInt = SifrInt::from_i64(0);
    let mut actual_stop_value_351bdef5a4961be0: SifrInt = start_or_stop.clone();
    let mut unbounded: bool = false;
    let mut actual_step_value_353dfaf5a4b331da: SifrInt = SifrInt::from_i64(1);
    let mut argument_index: SifrInt = SifrInt::from_i64(0);
    for argument in slice_args.iter().cloned() {
        if &argument_index == &SifrInt::from_i64(0) {
            actual_start = start_or_stop.clone();
            if argument.is_none() {
                unbounded = true;
            } else if let Some(argument) = argument.clone() {
                actual_stop_value_351bdef5a4961be0 = argument.clone();
            }
        } else if let Some(argument) = argument.clone() {
            actual_step_value_353dfaf5a4b331da = argument.clone();
        }
        argument_index = &argument_index + &SifrInt::from_i64(1);
    }
    if &actual_start < &SifrInt::from_i64(0) {
        return Err(ValueError::new(
            "islice: indices must be non-negative".to_string(),
        ));
    }
    if !unbounded && &actual_stop_value_351bdef5a4961be0 < &SifrInt::from_i64(0) {
        return Err(ValueError::new(
            "islice: indices must be non-negative".to_string(),
        ));
    }
    if &actual_step_value_353dfaf5a4b331da <= &SifrInt::from_i64(0) {
        return Err(ValueError::new(
            "islice: step must be greater than zero".to_string(),
        ));
    }
    Ok(sifr_generated_islice_impl(
        Box::new(data),
        actual_start.clone(),
        actual_stop_value_351bdef5a4961be0.clone(),
        unbounded,
        actual_step_value_353dfaf5a4b331da.clone(),
    ))
}
fn count(start: SifrInt, step: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut current: SifrInt = start.clone();
            loop {
                sifr_generated_yielder.suspend(current.clone()).await;
                current = &current + &step;
            }
        },
    ))
}
fn main() {
    let chained: Box<dyn Iterator<Item = SifrInt>> = chain(&vec![
        vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
        vec![SifrInt::from_i64(3)],
    ]);
    println!("{:?}", chained.collect::<Vec<_>>());
    let repeated: Box<dyn Iterator<Item = SifrInt>> =
        repeat(SifrInt::from_i64(7), SifrInt::from_i64(3));
    println!("{:?}", repeated.collect::<Vec<_>>());
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let sliced: Box<dyn Iterator<Item = SifrInt>> = islice(
            Box::new(
                vec![
                    SifrInt::from_i64(10),
                    SifrInt::from_i64(20),
                    SifrInt::from_i64(30),
                    SifrInt::from_i64(40),
                    SifrInt::from_i64(50),
                ]
                .into_iter(),
            ),
            SifrInt::from_i64(1),
            &vec![Some(SifrInt::from_i64(5)), Some(SifrInt::from_i64(2))],
        )?;
        println!("{:?}", sliced.collect::<Vec<_>>());
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", {
            let mut sifr_generated_concat: String = String::with_capacity(14usize);
            sifr_generated_concat.push_str("islice error: ");
            sifr_generated_concat.push_str(e.message.clone().as_str());
            sifr_generated_concat
        });
    }
    let mut counter: Box<dyn Iterator<Item = SifrInt>> =
        count(SifrInt::from_i64(5), SifrInt::from_i64(2));
    println!(
        "{}",
        counter.next().map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        counter.next().map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        counter.next().map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
    println!(
        "{}",
        counter.next().map_or_else(
            || "None".to_string(),
            |sifr_generated_v| sifr_generated_v.to_string()
        )
    );
}
