// src/main.rs
mod sifr_generated_generated_support {
    use crate::{
        IOError, RegexError, SifrGeneratedStdlibSifrX2ereX2eMatch,
        SifrGeneratedStdlibSifrX2ereX2ePattern, SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
    };
    pub(crate) use ::sifr_runtime::SifrInt;
    pub(crate) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(crate) fn listdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::listdir(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn iterdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::iterdir(path).map_err(sifr_generated_io_err)
    }
    pub(crate) fn glob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::glob_pattern(dir, pattern).map_err(sifr_generated_io_err)
    }
    pub(crate) fn rglob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::rglob_pattern(dir, pattern).map_err(sifr_generated_io_err)
    }
    pub(crate) fn fnmatch(name: &str, pattern: &str) -> bool {
        sifr_generated_match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
    }
    pub(crate) fn sifr_generated_match(
        name: &str,
        mut ni: SifrInt,
        pattern: &str,
        mut pi: SifrInt,
    ) -> bool {
        while &pi < &SifrInt::from(pattern.chars().count()) {
            let pc: Option<String> = {
                let sifr_generated_string_chars = pattern.chars().collect::<Vec<char>>();
                let sifr_generated_string_index = pi.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_string_chars.len());
                sifr_generated_string_chars
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(pc) = pc {
                if pc == "*" {
                    pi = &pi + &SifrInt::from_i64(1);
                    if &pi == &SifrInt::from(pattern.chars().count()) {
                        return true;
                    }
                    let mut j: SifrInt = ni.clone();
                    while &j <= &SifrInt::from(name.chars().count()) {
                        if sifr_generated_match(name, j.clone(), pattern, pi.clone()) {
                            return true;
                        }
                        j = &j + &SifrInt::from_i64(1);
                    }
                    return false;
                }
                if &ni >= &SifrInt::from(name.chars().count()) {
                    return false;
                }
                if pc != "?" {
                    let nc: Option<String> = {
                        let sifr_generated_string_chars = name.chars().collect::<Vec<char>>();
                        let sifr_generated_string_index = ni.clone();
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_string_chars.len());
                        sifr_generated_string_chars
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(|character| character.to_string());
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                }
                ni = &ni + &SifrInt::from_i64(1);
                pi = &pi + &SifrInt::from_i64(1);
            } else {
                return false;
            }
        }
        &ni == &SifrInt::from(name.chars().count())
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
    pub(crate) fn glob(directory: &str, pattern: &str) -> Vec<String> {
        let sifr_generated_chars_pattern: Vec<char> = pattern.chars().collect::<Vec<char>>();
        let include_hidden: bool =
            &SifrInt::from(sifr_generated_chars_pattern.len()) > &SifrInt::from_i64(0) && {
                let sifr_generated_string_index = SifrInt::from_i64(0);
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_pattern.len());
                sifr_generated_chars_pattern
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string())
            .is_some_and(|_checked_value_0| {
                {
                    let sifr_generated_string_index = SifrInt::from_i64(0);
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_pattern.len());
                    sifr_generated_chars_pattern
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(Some)
                    == Some(Some('.'))
            });
        let mut matches: Vec<String> = Vec::new();
        let sifr_generated_try_res: Result<(), IOError> = (|| {
            let entries: Vec<String> = listdir(directory)?;
            for entry in entries.iter().cloned() {
                let sifr_generated_chars_entry: Vec<char> = entry.chars().collect::<Vec<char>>();
                if &SifrInt::from(sifr_generated_chars_entry.len()) == &SifrInt::from_i64(0) {
                    continue;
                }
                if !include_hidden && {
                    let sifr_generated_string_index = SifrInt::from_i64(0);
                    let sifr_generated_string_index_normalized = sifr_generated_string_index
                        .normalize_index_or_len(sifr_generated_chars_entry.len());
                    sifr_generated_chars_entry
                        .get(sifr_generated_string_index_normalized)
                        .copied()
                }
                .map(|character| character.to_string())
                .is_some_and(|_checked_value_1| {
                    {
                        let sifr_generated_string_index = SifrInt::from_i64(0);
                        let sifr_generated_string_index_normalized = sifr_generated_string_index
                            .normalize_index_or_len(sifr_generated_chars_entry.len());
                        sifr_generated_chars_entry
                            .get(sifr_generated_string_index_normalized)
                            .copied()
                    }
                    .map(Some)
                        == Some(Some('.'))
                }) {
                    continue;
                }
                if fnmatch(&entry, pattern) {
                    matches.push(entry);
                }
            }
            Ok(())
        })();
        if let Err(sifr_generated_try_err) = sifr_generated_try_res {
            let e = sifr_generated_try_err.clone();
            let _ = e.message.clone().to_string();
            return Vec::new();
        }
        {
            let mut sifr_generated_sorted_values = matches.iter().cloned().collect::<Vec<_>>();
            sifr_generated_sorted_values.sort_by(
                |sifr_generated_sorted_left, sifr_generated_sorted_right| {
                    sifr_generated_sorted_left.cmp(&sifr_generated_sorted_right)
                },
            );
            sifr_generated_sorted_values
        }
    }
    pub(crate) fn iglob(directory: &str, pattern: &str) -> Box<dyn Iterator<Item = String>> {
        let directory = directory.to_owned();
        let pattern = pattern.to_owned();
        Box::new(SifrGeneratedGenerator::new(
            async move |sifr_generated_yielder: SifrGeneratedYielder<String>| {
                let matches: Vec<String> = glob(&directory, &pattern);
                let mut i: SifrInt = SifrInt::from_i64(0);
                while &i < &SifrInt::from(matches.len()) {
                    let Some(sifr_generated_checked_value_2) = ({
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
                        .suspend(sifr_generated_checked_value_2.clone())
                        .await;
                    i = &i + &SifrInt::from_i64(1);
                }
            },
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
    pub(crate) fn sifr_generated_glob_list(
        path: &str,
        pattern: &str,
    ) -> Result<Vec<String>, IOError> {
        glob_pattern(path, pattern)
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
    pub(crate) fn sifr_generated_glob_to_iter(
        path: &str,
        pattern: &str,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let sifr_generated_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = sifr_generated_glob_list(path, pattern)?;
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
    pub(crate) trait SifrGeneratedOpaqueSifrStdlibSifrX2eregexX2eCompiledPatternMethods {
        fn search(&self, text: &str) -> Result<Option<String>, RegexError>;
        fn is_match(&self, text: &str) -> Result<bool, RegexError>;
        fn sub(&self, replacement: &str, text: &str) -> Result<String, RegexError>;
        fn findall(&self, text: &str) -> Result<Vec<String>, RegexError>;
        fn split(&self, text: &str) -> Result<Vec<String>, RegexError>;
        fn pattern(&self) -> Result<String, RegexError>;
        fn flags(&self) -> Result<SifrInt, RegexError>;
    }
    pub(crate) fn compile_pattern(
        pattern: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern, RegexError> {
        ::sifr_stdlib::regex::compile_pattern(pattern).map_err(|sifr_generated_bridge_error| {
            RegexError {
                message: sifr_generated_bridge_error.to_string(),
                detail: sifr_generated_bridge_error.to_string(),
            }
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
    pub(crate) fn compile(
        pattern: &str,
    ) -> Result<SifrGeneratedStdlibSifrX2ereX2ePattern, RegexError> {
        let sifr_generated_try_res: Result<
            Result<SifrGeneratedStdlibSifrX2ereX2ePattern, RegexError>,
            RegexError,
        > = (|| {
            let compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern =
                compile_pattern(pattern)?;
            Ok(Ok(SifrGeneratedStdlibSifrX2ereX2ePattern::new(
                compiled,
                pattern.to_owned(),
                SifrInt::from_i64(0),
            )))
        })();
        sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
            let error = sifr_generated_try_err.clone();
            Err(RegexError::new(error.message.clone()))
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
        pub fn glob(&self, pattern: &str) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            sifr_generated_glob_to_iter(&self.path, pattern)
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
    pub type SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern =
        ::sifr_runtime::interop::Handle<::sifr_stdlib::regex::CompiledPattern>;
    impl SifrGeneratedOpaqueSifrStdlibSifrX2eregexX2eCompiledPatternMethods
        for SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern
    {
        fn search(&self, text: &str) -> Result<Option<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_search(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn is_match(&self, text: &str) -> Result<bool, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_is_match(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn sub(&self, replacement: &str, text: &str) -> Result<String, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_replace(self, replacement, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn findall(&self, text: &str) -> Result<Vec<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_findall(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn split(&self, text: &str) -> Result<Vec<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_split(self, text).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn pattern(&self) -> Result<String, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_source(self).map_err(
                |sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                },
            )
        }
        fn flags(&self) -> Result<SifrInt, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_flags(self)
                .map(::sifr_runtime::interop::SifrIntBridge::into_sifr_int)
                .map_err(|sifr_generated_bridge_error| RegexError {
                    message: sifr_generated_bridge_error.to_string(),
                    detail: sifr_generated_bridge_error.to_string(),
                })
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
        pub fn group(&self) -> String {
            {
                let mut sifr_generated_concat: String = String::new();
                sifr_generated_concat.push_str(self.matched.clone().as_str());
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            }
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
    pub struct SifrGeneratedStdlibSifrX2ereX2ePattern {
        pub compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
        pub pattern: String,
        pub flags: SifrInt,
    }
    impl SifrGeneratedStdlibSifrX2ereX2ePattern {
        #[must_use]
        pub fn new(
            compiled: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern,
            pattern: String,
            flags: SifrInt,
        ) -> Self {
            let sifr_generated_field_value_fc19778909466d91_5f636f6d70696c6564: SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern = compiled;
            let sifr_generated_field_value_24bd37eb3fd1d0fc_5f7061747465726e: String = pattern;
            let sifr_generated_field_value_7e89da5111942f49_5f666c616773: SifrInt = flags.clone();
            Self {
                compiled: sifr_generated_field_value_fc19778909466d91_5f636f6d70696c6564,
                pattern: sifr_generated_field_value_24bd37eb3fd1d0fc_5f7061747465726e,
                flags: sifr_generated_field_value_7e89da5111942f49_5f666c616773,
            }
        }
    }
    impl SifrGeneratedStdlibSifrX2ereX2ePattern {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn findall(&self, text: &str) -> Result<Vec<String>, RegexError> {
            self.compiled.findall(text)
        }
    }
    impl SifrGeneratedStdlibSifrX2ereX2ePattern {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn finditer(
            &self,
            text: &str,
        ) -> Result<Box<dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>>, RegexError>
        {
            let sifr_generated_try_res: Result<
                Result<Box<dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>>, RegexError>,
                RegexError,
            > = (|| {
                let found_items: Vec<String> = self.compiled.findall(text)?;
                let matches: Vec<SifrGeneratedStdlibSifrX2ereX2eMatch> =
                    sifr_generated_finditer_from_items(&found_items, text);
                Ok(Ok(sifr_generated_iter_matches(matches)))
            })();
            sifr_generated_try_res.unwrap_or_else(|sifr_generated_try_err| {
                let e = sifr_generated_try_err.clone();
                Err(RegexError::new(e.message.clone()))
            })
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
use crate::sifr_generated_generated_support::*;
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::RegexError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2epathlibX2ePath;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ereX2eMatch;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2ereX2ePattern;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2eregexX2eCompiledPattern;
#[expect(
    clippy::assertions_on_constants,
    reason = "generated Rust preserves this exact typed Sifr source contract"
)]
fn main() {
    let sifr_generated_try_res: Result<(), RegexError> = (|| {
        let mut digits: Box<dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>> =
            finditer(&"\\d+".to_string(), &"v1 and v22".to_string())?;
        let first: Option<SifrGeneratedStdlibSifrX2ereX2eMatch> = digits.next();
        let second: Option<SifrGeneratedStdlibSifrX2ereX2eMatch> = digits.next();
        if let Some(first) = first {
            assert_eq!(first.group(), "1");
        }
        if let Some(second) = second {
            assert_eq!(second.group(), "22");
        }
        assert!(digits.next().is_none());
        let pat: SifrGeneratedStdlibSifrX2ereX2ePattern = compile(&"[a-z]+".to_string())?;
        let mut words: Vec<String> = Vec::new();
        let word_it_value_70fd338081233f6b: Box<
            dyn Iterator<Item = SifrGeneratedStdlibSifrX2ereX2eMatch>,
        > = pat.finditer(&"alpha 123 beta".to_string())?;
        for m in word_it_value_70fd338081233f6b {
            words.push(m.group());
        }
        assert_eq!(format!("{words:?}"), "[\"alpha\", \"beta\"]");
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        assert!(false);
    }
    let base: String = {
        let mut sifr_generated_concat: String = String::with_capacity(32usize);
        sifr_generated_concat.push_str("/tmp/sifr_regex_filesystem_demo_");
        sifr_generated_concat.push_str(getpid().to_string().as_str());
        sifr_generated_concat
    };
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _mk: String = run_command(&format!("mkdir -p {base}/sub"))?;
        write_text(&format!("{base}/a.txt"), &"a".to_string())?;
        write_text(&format!("{base}/sub/b.txt"), &"b".to_string())?;
        assert_eq!(
            format!(
                "{:?}",
                iglob(&base, &"*.txt".to_string()).collect::<Vec<_>>()
            ),
            "[\"a.txt\"]"
        );
        let root: SifrGeneratedStdlibSifrX2epathlibX2ePath =
            SifrGeneratedStdlibSifrX2epathlibX2ePath::new(base.to_string());
        let entries_it: Box<dyn Iterator<Item = String>> = root.iterdir()?;
        let entries: Vec<String> = entries_it.collect::<Vec<_>>();
        assert!(&SifrInt::from(entries.len()) >= &SifrInt::from_i64(2));
        let top_txt_it: Box<dyn Iterator<Item = String>> = root.glob(&"*.txt".to_string())?;
        let top_txt: Vec<String> = top_txt_it.collect::<Vec<_>>();
        assert_eq!(format!("{top_txt:?}"), {
            let mut sifr_generated_concat: String =
                String::with_capacity(2usize + base.len() + 8usize);
            sifr_generated_concat.push_str("[\"");
            sifr_generated_concat.push_str(base.as_str());
            sifr_generated_concat.push_str("/a.txt\"]");
            sifr_generated_concat
        });
        let recursive_it: Box<dyn Iterator<Item = String>> = root.rglob(&"*.txt".to_string())?;
        let recursive_txt_value_9e7000539eb63e40: Vec<String> = recursive_it.collect::<Vec<_>>();
        assert_eq!(
            &SifrInt::from(recursive_txt_value_9e7000539eb63e40.len()),
            &SifrInt::from_i64(2)
        );
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
        assert!(false);
    }
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _clean: String = run_command(&format!("rm -rf {base}"))?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err.clone();
        let _ = e.message.clone().to_string();
    }
    println!("parity_ext_regex_and_filesystem_filesystem_iterators_demo: ok");
}
