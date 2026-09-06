// src/main.rs
mod sifr_generated_generated_support {
    use crate::{IOError, RegexError, SifrGeneratedStdlibSifrX2ereX2eMatch, ValueError};
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(crate) fn iterdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::iterdir(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn rglob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::rglob_pattern(dir, pattern).map_err(sifr_generated_io_err)
    }
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
    pub(crate) trait SifrGeneratedAdd: Sized {}
    impl SifrGeneratedAdd for ::sifr_runtime::SifrInt {}
    impl SifrGeneratedAdd for String {}
    pub(crate) fn sifr_generated_islice_impl<T: Clone + 'static>(
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
    pub(crate) fn islice<T: Clone + 'static>(
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
    pub(crate) fn run_command(cmd: &str) -> Result<String, IOError> {
        ::sifr_stdlib::sys::run_command(cmd).map_err(sifr_generated_io_err)
    }
    pub(crate) fn getpid() -> SifrInt {
        ::sifr_stdlib::sys::getpid().into_sifr_int()
    }
    pub(crate) fn sifr_generated_iter_list_str(
        entries: Vec<String>,
    ) -> Box<dyn Iterator<Item = String>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<String>| {
                let mut i: SifrInt = SifrInt::from_i64(0);
                while &i < &SifrInt::from(entries.len()) {
                    let Some(sifr_generated_checked_value_7) = ({
                        let sifr_generated_checked_read_collection = &entries;
                        let sifr_generated_checked_read_index = i.clone();
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
                        .suspend(sifr_generated_checked_value_7.clone())
                        .await;
                    i = &i + &SifrInt::from_i64(1);
                }
            },
        ))
    }
    pub(crate) fn sifr_generated_iterdir_list(path: &str) -> Result<Vec<String>, IOError> {
        iterdir(path)
    }
    pub(crate) fn sifr_generated_rglob_list(
        path: &str,
        pattern: &str,
    ) -> Result<Vec<String>, IOError> {
        rglob_pattern(path, pattern)
    }
    pub(crate) fn sifr_generated_iterdir_to_iter(
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
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn sifr_generated_rglob_to_iter(
        path: &str,
        pattern: &str,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let sifr_generated_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = sifr_generated_rglob_list(path, pattern)?;
            Ok(Ok(sifr_generated_iter_list_str(entries)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(e)
        })
    }
    pub(crate) fn re_findall(pattern: &str, text: &str) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_findall(pattern, text).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
        })
    }
    pub(crate) fn re_findall_flags(
        pattern: &str,
        text: &str,
        flags: SifrInt,
    ) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_findall_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map_err(|sifr_generated_bridge_error| RegexError {
            message: sifr_generated_bridge_error.to_string(),
            detail: sifr_generated_bridge_error.to_string(),
        })
    }
    pub(crate) fn sifr_generated_iter_matches(
        matches: Vec<SifrGeneratedStdlibSifrX2ereX2eMatch>,
    ) -> Box<dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>> {
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<
                SifrGeneratedStdlibSifrX2ereX2eMatch,
            >| {
                let mut i: SifrInt = SifrInt::from_i64(0);
                while &i < &SifrInt::from(matches.len()) {
                    let Some(sifr_generated_checked_value_0) = ({
                        let sifr_generated_checked_read_collection = &matches;
                        let sifr_generated_checked_read_index = i.clone();
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
                        .suspend(sifr_generated_checked_value_0.clone())
                        .await;
                    i = &i + &SifrInt::from_i64(1);
                }
            },
        ))
    }
    pub(crate) fn sifr_generated_find_index_from(
        text: &str,
        needle: &str,
        start: SifrInt,
    ) -> SifrInt {
        let sifr_generated_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let sifr_generated_chars_needle: Vec<char> = needle.chars().collect::<Vec<char>>();
        if &start < &SifrInt::from_i64(0) {
            return -&SifrInt::from_i64(1);
        }
        if &SifrInt::from(sifr_generated_chars_needle.len()) == &SifrInt::from_i64(0) {
            if &start <= &SifrInt::from(sifr_generated_chars_text.len()) {
                return start.clone();
            }
            return -&SifrInt::from_i64(1);
        }
        let max_start: SifrInt = &SifrInt::from(sifr_generated_chars_text.len())
            - &SifrInt::from(sifr_generated_chars_needle.len());
        let mut i: SifrInt = start.clone();
        while &i <= &max_start {
            if &{
                let sifr_generated_slice_src = &sifr_generated_chars_text;
                let sifr_generated_slice_len = sifr_generated_slice_src.len();
                let sifr_generated_slice_start = i.clamp_slice_bound(sifr_generated_slice_len);
                let sifr_generated_slice_stop = (&i
                    + &SifrInt::from(sifr_generated_chars_needle.len()))
                    .clamp_slice_bound(sifr_generated_slice_len);
                String::from_iter(
                    sifr_generated_slice_src
                        .iter()
                        .skip(sifr_generated_slice_start)
                        .take(sifr_generated_slice_stop.saturating_sub(sifr_generated_slice_start))
                        .copied(),
                )
            } == needle
            {
                return i.clone();
            }
            i = &i + &SifrInt::from_i64(1);
        }
        -&SifrInt::from_i64(1)
    }
    pub(crate) fn sifr_generated_findall_for_finditer(
        pattern: &str,
        text: &str,
        flags: SifrInt,
    ) -> Result<Vec<String>, RegexError> {
        if &flags != &SifrInt::from_i64(0) {
            return re_findall_flags(pattern, text, flags.clone());
        }
        re_findall(pattern, text)
    }
    pub(crate) fn sifr_generated_finditer_from_items(
        found_items: &[String],
        text: &str,
    ) -> Vec<SifrGeneratedStdlibSifrX2ereX2eMatch> {
        let mut matches: Vec<SifrGeneratedStdlibSifrX2ereX2eMatch> = Vec::new();
        let mut cursor: SifrInt = SifrInt::from_i64(0);
        for found in found_items.iter().cloned() {
            let sifr_generated_chars_found: Vec<char> = found.chars().collect::<Vec<char>>();
            let mut start: SifrInt = sifr_generated_find_index_from(text, &found, cursor.clone());
            if &start < &SifrInt::from_i64(0) {
                start = cursor.clone();
            }
            let found_len: SifrInt = SifrInt::from(sifr_generated_chars_found.len());
            let end: SifrInt = &start + &found_len;
            matches.push(SifrGeneratedStdlibSifrX2ereX2eMatch::new(
                found,
                start.clone(),
                end.clone(),
            ));
            if &found_len == &SifrInt::from_i64(0) {
                cursor = &end + &SifrInt::from_i64(1);
            } else {
                cursor = end;
            }
        }
        matches
    }
    pub(crate) fn sifr_generated_finditer_materialize(
        pattern: &str,
        text: &str,
        flags: SifrInt,
    ) -> Result<Vec<SifrGeneratedStdlibSifrX2ereX2eMatch>, RegexError> {
        let sifr_generated_try_res: Result<
            Result<Vec<SifrGeneratedStdlibSifrX2ereX2eMatch>, RegexError>,
            RegexError,
        > = (|| {
            let found_items: Vec<String> =
                sifr_generated_findall_for_finditer(pattern, text, flags.clone())?;
            Ok(Ok(sifr_generated_finditer_from_items(&found_items, text)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(RegexError::new(e.message.clone()))
        })
    }
    pub(crate) fn finditer(
        pattern: &str,
        text: &str,
    ) -> Result<Box<dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>>, RegexError> {
        let sifr_generated_try_res: Result<
            Result<Box<dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>>, RegexError>,
            RegexError,
        > = (|| {
            let matches: Vec<SifrGeneratedStdlibSifrX2ereX2eMatch> =
                sifr_generated_finditer_materialize(pattern, text, SifrInt::from_i64(0))?;
            Ok(Ok(sifr_generated_iter_matches(matches)))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let e = sifr_generated_try_err.clone();
            Err(RegexError::new(e.message.clone()))
        })
    }
    pub(crate) fn sifr_generated_io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
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
    use crate::sifr_generated_generated_support::*;
    use ::sifr_runtime::SifrInt;
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
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn rglob(&self, pattern: &str) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            sifr_generated_rglob_to_iter(&self.path, pattern)
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2epathlibX2ePath {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Path(_path={})", self.path)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct SifrGeneratedStdlibSifrX2ereX2eMatch {
        pub matched: String,
        pub start: SifrInt,
        pub end: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ereX2eMatch {
        #[must_use]
        pub fn new(matched: String, start: SifrInt, end: SifrInt) -> Self {
            let sifr_generated_field_value_057487a7ae39ff66_5f6d617463686564: String = matched;
            let sifr_generated_field_value_f46e9a817c26293e_5f7374617274: SifrInt = start.clone();
            let sifr_generated_field_value_3daa7443932b3d2b_5f656e64: SifrInt = end.clone();
            Self {
                matched: sifr_generated_field_value_057487a7ae39ff66_5f6d617463686564,
                start: sifr_generated_field_value_f46e9a817c26293e_5f7374617274,
                end: sifr_generated_field_value_3daa7443932b3d2b_5f656e64,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ereX2eMatch {
        #[must_use]
        pub fn start(&self) -> SifrInt {
            self.start.clone()
        }
    }
    impl ::std::fmt::Display for SifrGeneratedStdlibSifrX2ereX2eMatch {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "Match(_matched={}, _start={}, _end={})",
                self.matched, self.start, self.end
            )
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
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl RegexError {
        #[must_use]
        pub const fn new(message: String) -> Self {
            Self {
                message,
                detail: String::new(),
            }
        }
    }
    impl ::std::fmt::Display for RegexError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for RegexError {}
}
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::RegexError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2epathlibX2ePath;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ereX2eMatch;
pub use sifr_generated_project_nominals::ValueError;
mod sifr_generated_project_unions {
    #[derive(Debug, Clone)]
    pub enum SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0
    {
        SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
            crate::sifr_generated_project_nominals::IOError,
        ),
        SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
            crate::sifr_generated_project_nominals::ValueError,
        ),
    }
    impl From<crate::sifr_generated_project_nominals::IOError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::IOError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                value,
            )
        }
    }
    impl From<crate::sifr_generated_project_nominals::ValueError>
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn from(value: crate::sifr_generated_project_nominals::ValueError) -> Self {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                    v,
                ) => write!(f, "{v}"),
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                    v,
                ) => write!(f, "{v}"),
            }
        }
    }
}
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_unions::SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0;
fn adapt_to_iterable(it: Box<dyn Iterator<Item = SifrInt>>) -> Vec<SifrInt> {
    it.collect::<Vec<_>>()
}
fn collect_starts(
    it: Box<dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>>,
) -> Vec<SifrInt> {
    let mut starts: Vec<SifrInt> = Vec::new();
    for m in it {
        starts.push(m.start());
    }
    starts
}
#[expect(
    clippy::too_many_lines,
    reason = "one generated Rust function preserves one typed Sifr function"
)]
fn main() {
    let nums: Box<dyn Iterator<Item = SifrInt>> = Box::new(
        vec![
            SifrInt::from_i64(3),
            SifrInt::from_i64(4),
            SifrInt::from_i64(5),
        ]
        .into_iter(),
    );
    let via_binding: Vec<SifrInt> = nums.collect::<Vec<_>>();
    println!("{:?}", via_binding.iter().cloned().collect::<Vec<_>>());
    let via_return: Vec<SifrInt> = adapt_to_iterable(Box::new(
        vec![SifrInt::from_i64(7), SifrInt::from_i64(8)].into_iter(),
    ))
    .into_iter()
    .collect::<Vec<_>>();
    println!("{:?}", via_return.iter().cloned().collect::<Vec<_>>());
    let payload: Vec<u8> = vec![65u8, 90u8];
    let byte_iter: Box<dyn Iterator<Item = u8>> = Box::new(
        payload
            .iter()
            .map(|sifr_generated_byte| *sifr_generated_byte as u8),
    );
    let mapped_bytes: Vec<SifrInt> =
        Box::new(byte_iter.map(|b| SifrInt::from(b) + SifrInt::from_i64(1))).collect::<Vec<_>>();
    println!("{mapped_bytes:?}");
    let sifr_generated_try_res: Result<(), RegexError> = (|| {
        let matches: Box<dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>> =
            finditer(&"\\d+".to_string(), &"x11y222".to_string())?;
        println!("{:?}", collect_starts(Box::new(matches)));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", e.message.clone());
    }
    let base: String = {
        let mut sifr_generated_concat: String = String::with_capacity(30usize);
        sifr_generated_concat.push_str("/tmp/sifr_iterator_integration");
        sifr_generated_concat.push_str(getpid().to_string().as_str());
        sifr_generated_concat
    };
    let sifr_generated_try_res: Result<
        (),
        SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0,
    > = (|| {
        let _mk: String = run_command(&format!("mkdir -p {base}/nested"))
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
            )?;
        write_text(&format!("{base}/a.txt"), &"a".to_string())
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
            )?;
        write_text(&format!("{base}/nested/b.txt"), &"b".to_string())
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
            )?;
        let root: SifrGeneratedStdlibSifrX2epathlibX2ePath = SifrGeneratedStdlibSifrX2epathlibX2ePath::new(
            base.to_string(),
        );
        let entries_it: Box<dyn Iterator<Item = String>> = root
            .iterdir()
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
            )?;
        let sliced_entries: Box<dyn Iterator<Item = String>> = islice(
                Box::new(entries_it),
                SifrInt::from_i64(2),
                &Vec::<Option<SifrInt>>::new(),
            )
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0,
            )?;
        println!("{}", SifrInt::from(sliced_entries.collect:: < Vec < _ > > ().len()));
        let recursive_it: Box<dyn Iterator<Item = String>> = root
            .rglob(&"*.txt".to_string())
            .map_err(
                SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0,
            )?;
        println!("{}", SifrInt::from(recursive_it.collect:: < Vec < _ > > ().len()));
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        match sifr_generated_try_err {
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass7X3aIOError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error.clone();
                println!("{}", e.message.clone());
            }
            SifrGeneratedUnion8X3asequence5X3aunion1X3a223X3a5X3aclass10X3aValueError1X3a019X3a5X3aclass7X3aIOError1X3a0::SifrGeneratedUnionVariant5X3aclass10X3aValueError1X3a0(
                sifr_generated_try_variant_error,
            ) => {
                let e = sifr_generated_try_variant_error.clone();
                println!("{}", e.message.clone());
            }
        }
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _rm: String = run_command(&format!("rm -rf {base}"))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        println!("{}", e.message.clone());
    }
}
