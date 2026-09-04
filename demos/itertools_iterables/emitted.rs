// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::{IOError, IndexError, ValueError};
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(super) fn iterdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::iterdir(path).map_err(sifr_generated_io_err)
    }
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
    pub(super) trait SifrGeneratedAdd: Sized {
        #[must_use]
        fn sifr_generated_add(self, rhs: Self) -> Self;
    }
    impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {
        fn sifr_generated_add(self, rhs: Self) -> Self {
            ::std::ops::Add::add(self, rhs)
        }
    }
    impl SifrGeneratedAdd for String {
        #[expect(
            clippy::needless_pass_by_value,
            reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
        )]
        fn sifr_generated_add(mut self, rhs: Self) -> Self {
            self.push_str(&rhs);
            self
        }
    }
    pub(super) fn sifr_generated_islice_impl<T: Clone + 'static>(
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
                    if !unbounded && index >= stop {
                        return;
                    }
                    if index == next_yield {
                        sifr_generated_yielder.suspend(value.clone()).await;
                        next_yield =
                            ::std::ops::Add::add(&next_yield, &step_argument_af0b4e191da20cef);
                    }
                    index = ::std::ops::Add::add(&index, &SifrInt::from_i64(1));
                }
            },
        ))
    }
    #[expect(
        clippy::needless_pass_by_value,
        reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
    )]
    pub(super) fn islice<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        start_or_stop: Option<SifrInt>,
        slice_args: &[Option<SifrInt>],
    ) -> Result<Box<dyn Iterator<Item = T>>, ValueError> {
        if slice_args.len() > SifrInt::from_i64(2) {
            return Err(ValueError::new(
                "islice: expected at most stop and step after start".to_string(),
            ));
        }
        let mut actual_start: SifrInt = SifrInt::from_i64(0);
        let mut actual_stop_value_351bdef5a4961be0: SifrInt = SifrInt::from_i64(0);
        let mut unbounded: bool = start_or_stop.is_none();
        if let Some(start_or_stop) = start_or_stop.clone() {
            actual_stop_value_351bdef5a4961be0.clone_from(&start_or_stop);
        }
        let mut actual_step_value_353dfaf5a4b331da: SifrInt = SifrInt::from_i64(1);
        let mut argument_index: SifrInt = SifrInt::from_i64(0);
        #[expect(
            clippy::explicit_iter_loop,
            reason = "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime"
        )]
        for argument in slice_args.iter() {
            if argument_index == SifrInt::from_i64(0) {
                let Some(start_or_stop) = start_or_stop.clone() else {
                    return Err(ValueError::new(
                        "islice: start must be an integer when stop is provided".to_string(),
                    ));
                };
                actual_start.clone_from(&start_or_stop);
                if argument.is_none() {
                    unbounded = true;
                } else if let Some(argument) = argument.clone() {
                    actual_stop_value_351bdef5a4961be0.clone_from(&argument);
                }
            } else if let Some(argument) = argument.clone() {
                actual_step_value_353dfaf5a4b331da.clone_from(&argument);
            }
            argument_index = ::std::ops::Add::add(&argument_index, &SifrInt::from_i64(1));
        }
        if actual_start < SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "islice: indices must be non-negative".to_string(),
            ));
        }
        if !unbounded && actual_stop_value_351bdef5a4961be0 < SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "islice: indices must be non-negative".to_string(),
            ));
        }
        if actual_step_value_353dfaf5a4b331da <= SifrInt::from_i64(0) {
            return Err(ValueError::new(
                "islice: step must be greater than zero".to_string(),
            ));
        }
        Ok(sifr_generated_islice_impl(
            Box::new(data),
            actual_start,
            actual_stop_value_351bdef5a4961be0,
            unbounded,
            actual_step_value_353dfaf5a4b331da,
        ))
    }
    pub(super) fn accumulate<T: Clone + 'static + SifrGeneratedAdd>(
        data: Box<dyn Iterator<Item = T>>,
        initial: Option<T>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                let mut state: Vec<T> = Vec::new();
                if let Some(initial) = initial {
                    state.push(initial);
                    let initial_value: Option<T> = {
                        let sifr_generated_checked_read_collection = &state;
                        let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    };
                    if let Some(initial_value) = initial_value {
                        sifr_generated_yielder.suspend(initial_value).await;
                    }
                }
                for item in data {
                    if state.len() == SifrInt::from_i64(0) {
                        state.push(item);
                    } else {
                        let prev: Option<T> = {
                            let sifr_generated_checked_read_collection = &state;
                            let sifr_generated_checked_read_index = SifrInt::from_i64(0);
                            let sifr_generated_checked_read_normalized =
                                sifr_generated_checked_read_index.normalize_index_or_len(
                                    sifr_generated_checked_read_collection.len(),
                                );
                            sifr_generated_checked_read_collection
                                .get(sifr_generated_checked_read_normalized)
                                .cloned()
                        };
                        if let Some(prev) = prev {
                            let next_val: T = SifrGeneratedAdd::sifr_generated_add(prev, item);
                            let sifr_generated_try_res: Result<(), IndexError> = (|| {
                                {
                                    let sifr_generated_assign_value = next_val.clone();
                                    {
                                        let sifr_generated_index_raw = SifrInt::from_i64(0);
                                        let sifr_generated_index_normalized =
                                            sifr_generated_index_raw
                                                .normalize_index_or_len(state.len());
                                        if let Some(sifr_generated_elem) =
                                            state.get_mut(sifr_generated_index_normalized)
                                        {
                                            *sifr_generated_elem = sifr_generated_assign_value;
                                        } else {
                                            return Err(IndexError::new(
                                                "collection index out of range".to_string(),
                                            ));
                                        }
                                    }
                                }
                                Ok(())
                            })(
                            );
                            if let Err(sifr_generated_try_err) = sifr_generated_try_res {
                                let _ = sifr_generated_try_err;
                                return;
                            }
                        }
                    }
                    let current: Option<T> = {
                        let sifr_generated_checked_read_collection = &state;
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
                        sifr_generated_yielder.suspend(current).await;
                    }
                }
            },
        ))
    }
    pub(super) fn compress<T: Clone + 'static>(
        data: Box<dyn Iterator<Item = T>>,
        selectors: Box<dyn Iterator<Item = bool>>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                for (value, selector) in
                    Box::new(data.zip(selectors).map(|sifr_generated_zip_item| {
                        (sifr_generated_zip_item.0, sifr_generated_zip_item.1)
                    }))
                {
                    if selector {
                        sifr_generated_yielder.suspend(value.clone()).await;
                    }
                }
            },
        ))
    }
    pub(super) fn takewhile<T: Clone + 'static>(
        pred: impl Fn(&T) -> bool + Send + Sync + 'static,
        data: Box<dyn Iterator<Item = T>>,
    ) -> Box<dyn Iterator<Item = T>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<T>| {
                for val in data {
                    if !pred(&val) {
                        return;
                    }
                    sifr_generated_yielder.suspend(val.clone()).await;
                }
            },
        ))
    }
    pub(super) fn run_command(cmd: &str) -> Result<String, IOError> {
        ::sifr_stdlib::sys::run_command(cmd).map_err(sifr_generated_io_err)
    }
    pub(super) fn getpid() -> SifrInt {
        ::sifr_stdlib::sys::getpid().into_sifr_int()
    }
    pub(super) fn sifr_generated_iter_list_str(
        entries: Vec<String>,
    ) -> Box<dyn Iterator<Item = String>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<String>| {
                let mut i: SifrInt = SifrInt::from_i64(0);
                while i < entries.len() {
                    let Some(sifr_generated_checked_value_7) = ({
                        let sifr_generated_checked_read_collection = &entries;
                        let sifr_generated_checked_read_index = &i;
                        let sifr_generated_checked_read_normalized =
                            sifr_generated_checked_read_index.normalize_index_or_len(
                                sifr_generated_checked_read_collection.len(),
                            );
                        sifr_generated_checked_read_collection
                            .get(sifr_generated_checked_read_normalized)
                            .cloned()
                    }) else {
                        break;
                    };
                    sifr_generated_yielder
                        .suspend(sifr_generated_checked_value_7)
                        .await;
                    i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
                }
            },
        ))
    }
    pub(super) fn sifr_generated_iterdir_list(path: &str) -> Result<Vec<String>, IOError> {
        iterdir(path)
    }
    pub(super) fn sifr_generated_iterdir_to_iter(
        path: &str,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let sifr_generated_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = sifr_generated_iterdir_list(path)?;
            Ok(Ok(sifr_generated_iter_list_str(entries)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err;
            Err(e)
        })
    }
    pub(super) fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let sifr_generated_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match sifr_generated_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => "PermissionDenied".to_string(),
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => "DirectoryNotEmpty".to_string(),
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
}
mod sifr_generated_project_nominals {
    use crate::sifr_generated_generated_support::sifr_generated_iterdir_to_iter;
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2epathlibX2ePath {
        pub path: String,
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub const fn new(path: String) -> Self {
            let sifr_generated_field_value_0e74a76ec4f48c05_5f70617468: String = path;
            Self {
                path: sifr_generated_field_value_0e74a76ec4f48c05_5f70617468,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn iterdir(&self) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            sifr_generated_iterdir_to_iter(&self.path)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2epathlibX2ePath {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Path(_path={})", self.path)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IndexError {
        pub message: String,
    }
    impl IndexError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for IndexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IndexError {}
}
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::IndexError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2epathlibX2ePath;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0(
            crate::sifr_generated_project_nominals::IOError,
        ),
        SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::IOError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::IOError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ValueError) -> Self {
            Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                Self::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                Self::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use crate::sifr_generated_generated_support::{
    accumulate, compress, getpid, islice, run_command, takewhile, write_text,
};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0;
#[expect(
    clippy::needless_pass_by_value,
    reason = "language necessity: generated Rust preserves this exact typed Sifr source contract; owner Item 12; remove when the Rust ABI can differ without changing Sifr semantics"
)]
fn lt3(x: SifrInt) -> bool {
    x < SifrInt::from_i64(3)
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let nums: Vec<SifrInt> = vec![
        SifrInt::from_i64(1),
        SifrInt::from_i64(2),
        SifrInt::from_i64(3),
        SifrInt::from_i64(4),
    ];
    let sifr_generated_try_res: Result<(), ValueError> = (|| {
        let sliced: Box<dyn Iterator<Item = SifrInt>> = islice(
            Box::new(nums.clone().into_iter()),
            Some(SifrInt::from_i64(1)),
            &[Some(SifrInt::from_i64(4)), Some(SifrInt::from_i64(2))],
        )?;
        println!("{:?}", sliced.collect::<Vec<_>>());
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(14usize.saturating_add(0usize));
            sifr_generated_concat.push_str("islice error: ");
            sifr_generated_concat.push_str(e.message.as_str());
            sifr_generated_concat
        });
    }
    println!(
        "{:?}",
        accumulate(Box::new(nums.clone().into_iter()), None).collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        compress(
            Box::new(nums.clone().into_iter()),
            Box::new(vec![true, false, true, false].into_iter())
        )
        .collect::<Vec<_>>()
    );
    println!(
        "{:?}",
        takewhile(
            |sifr_generated_arg0| lt3(sifr_generated_arg0.clone()),
            Box::new(nums.clone().into_iter())
        )
        .collect::<Vec<_>>()
    );
    let base: String = {
        let mut sifr_generated_concat: String =
            String::with_capacity(29usize.saturating_add(0usize));
        sifr_generated_concat.push_str("/tmp/sifr_itertools_iterables");
        sifr_generated_concat.push_str(getpid().to_string().as_str());
        sifr_generated_concat
    };
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0,
    > = (|| {
        let _ = run_command(&format!("mkdir -p {base}"))
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0,
            )?;
        write_text(&format!("{base}/a.txt"), "demo")
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0,
            )?;
        write_text(&format!("{base}/b.txt"), "demo")
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0,
            )?;
        let root: SifrGeneratedStdlibSifrX2epathlibX2ePath = SifrGeneratedStdlibSifrX2epathlibX2ePath::new(
            base.clone(),
        );
        let entries_it: Box<dyn Iterator<Item = String>> = root
            .iterdir()
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0,
            )?;
        let sliced_entries: Box<dyn Iterator<Item = String>> = islice(
                Box::new(entries_it),
                Some(SifrInt::from_i64(1)),
                &Vec::new(),
            )
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0,
            )?;
        println!("{}", SifrInt::from(sliced_entries.count()));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error;
                println!(
                    "{}", { let mut sifr_generated_concat : String =
                    String::with_capacity(9usize.saturating_add(0usize));
                    sifr_generated_concat.push_str("ioerror: "); sifr_generated_concat
                    .push_str(e.message.to_string().as_str()); sifr_generated_concat }
                );
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a233X3a5X3aclass20X3asifrX2ebuiltinX2eIOError1X3a036X3a5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0::SifrGeneratedUnionVariant5X3aclass23X3asifrX2ebuiltinX2eValueError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error;
                println!(
                    "{}", { let mut sifr_generated_concat : String =
                    String::with_capacity(14usize.saturating_add(0usize));
                    sifr_generated_concat.push_str("islice error: ");
                    sifr_generated_concat.push_str(e.message.as_str());
                    sifr_generated_concat }
                );
            }
        }
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _ = run_command(&format!("rm -rf {base}"))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        println!("{}", {
            let mut sifr_generated_concat: String =
                String::with_capacity(17usize.saturating_add(0usize));
            sifr_generated_concat.push_str("cleanup ioerror: ");
            sifr_generated_concat.push_str(e.message.to_string().as_str());
            sifr_generated_concat
        });
    }
}
