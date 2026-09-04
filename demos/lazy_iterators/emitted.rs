// src/main.rs
mod sifr_generated_generated_support {
    pub(crate) struct SifrGeneratedYielder<T> {
        pub(crate) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    }
    pub(crate) struct SifrGeneratedYieldFuture<T> {
        pub(crate) slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(crate) value: Option<T>,
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
        pub(crate) fn suspend(&self, value: T) -> SifrGeneratedYieldFuture<T> {
            SifrGeneratedYieldFuture {
                slot: ::std::sync::Arc::clone(&self.slot),
                value: Some(value),
            }
        }
    }
    pub(crate) fn sifr_generated_store_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        value: T,
    ) {
        match slot.lock() {
            Ok(mut state) => *state = Some(value),
            Err(poisoned) => *poisoned.into_inner() = Some(value),
        }
    }
    pub(crate) fn sifr_generated_take_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    ) -> Option<T> {
        match slot.lock() {
            Ok(mut state) => state.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
    pub(crate) struct SifrGeneratedGenerator<T> {
        pub(crate) producer:
            Option<::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>>,
        pub(crate) yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub(crate) complete: bool,
    }
    impl<T> SifrGeneratedGenerator<T> {
        pub(crate) fn new<
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
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
fn fibonacci(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut a: SifrInt = SifrInt::from_i64(0);
            let mut b: SifrInt = SifrInt::from_i64(1);
            let mut count: SifrInt = SifrInt::from_i64(0);
            while &count < &n {
                sifr_generated_yielder.suspend(a.clone()).await;
                let temp: SifrInt = &a + &b;
                a = b.clone();
                b = temp.clone();
                count = &count + &SifrInt::from_i64(1);
            }
        },
    ))
}
fn squares(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &n {
                sifr_generated_yielder.suspend(&i * &i).await;
                i = &i + &SifrInt::from_i64(1);
            }
        },
    ))
}
fn evens(limit: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &limit {
                if &i.floor_mod_known_nonzero(&SifrInt::from_i64(2)) == &SifrInt::from_i64(0) {
                    sifr_generated_yielder.suspend(i.clone()).await;
                }
                i = &i + &SifrInt::from_i64(1);
            }
        },
    ))
}
fn count_up(n: SifrInt) -> Box<dyn Iterator<Item = SifrInt>> {
    Box::new(SifrGeneratedGenerator::new(
        async move |sifr_generated_yielder: SifrGeneratedYielder<SifrInt>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while &i < &n {
                sifr_generated_yielder.suspend(i.clone()).await;
                i = &i + &SifrInt::from_i64(1);
            }
        },
    ))
}
fn format_int_list(values: &[SifrInt]) -> String {
    if &SifrInt::from(values.len()) == &SifrInt::from_i64(0) {
        return "[]".to_string();
    }
    let mut formatted: String = "[".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(values.len()) {
        let Some(sifr_generated_checked_value_0) = ({
            let sifr_generated_checked_read_collection = &values;
            let sifr_generated_checked_read_index = i.clone();
            let sifr_generated_checked_read_normalized = sifr_generated_checked_read_index
                .normalize_index_or_len(sifr_generated_checked_read_collection.len());
            sifr_generated_checked_read_collection
                .get(sifr_generated_checked_read_normalized)
                .cloned()
        }) else {
            break;
        };
        formatted.push_str(sifr_generated_checked_value_0.clone().to_string().as_str());
        if &(&i + &SifrInt::from_i64(1)) < &SifrInt::from(values.len()) {
            formatted.push_str(", ");
        }
        i = &i + &SifrInt::from_i64(1);
    }
    formatted.push(']');
    formatted
}
fn main() {
    let mut output: Vec<String> = vec!["=== Fibonacci (lazy for loop) ===".to_string()];
    for fib in fibonacci(SifrInt::from_i64(8)) {
        output.push(fib.to_string());
    }
    output.push("=== Squares (collected) ===".to_string());
    let sq: Vec<SifrInt> = squares(SifrInt::from_i64(5)).collect::<Vec<_>>();
    output.push(format_int_list(&sq));
    output.push("=== Evens (conditional yield) ===".to_string());
    for e in evens(SifrInt::from_i64(10)) {
        output.push(e.to_string());
    }
    output.push("=== Count (lazy) ===".to_string());
    for c in count_up(SifrInt::from_i64(3)) {
        output.push(c.to_string());
    }
    output.push("=== Count (collected) ===".to_string());
    let nums: Vec<SifrInt> = count_up(SifrInt::from_i64(5)).collect::<Vec<_>>();
    output.push(format_int_list(&nums));
    assert_eq!(
        output,
        vec![
            "=== Fibonacci (lazy for loop) ===".to_string(),
            "0".to_string(),
            "1".to_string(),
            "1".to_string(),
            "2".to_string(),
            "3".to_string(),
            "5".to_string(),
            "8".to_string(),
            "13".to_string(),
            "=== Squares (collected) ===".to_string(),
            "[0, 1, 4, 9, 16]".to_string(),
            "=== Evens (conditional yield) ===".to_string(),
            "0".to_string(),
            "2".to_string(),
            "4".to_string(),
            "6".to_string(),
            "8".to_string(),
            "=== Count (lazy) ===".to_string(),
            "0".to_string(),
            "1".to_string(),
            "2".to_string(),
            "=== Count (collected) ===".to_string(),
            "[0, 1, 2, 3, 4]".to_string()
        ]
    );
    println!("Lazy iterator demo output:");
    for item in output.iter().cloned() {
        println!("{item}");
    }
}
