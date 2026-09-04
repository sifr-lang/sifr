// src/main.rs
pub mod sifr_generated_generated_support {
    use crate::IOError;
    pub(super) use ::sifr_runtime::SifrInt;
    pub(super) fn read_text(path: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content).map_err(sifr_generated_io_err)
    }
    pub(super) fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub(super) fn mkdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn rmdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn remove_file(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path).map_err(sifr_generated_io_err)
    }
    pub(super) fn is_file(path: &str) -> bool {
        ::sifr_stdlib::fs::is_file(path)
    }
    pub(super) fn glob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::glob_pattern(dir, pattern).map_err(sifr_generated_io_err)
    }
    pub(super) fn getpid() -> SifrInt {
        ::sifr_stdlib::sys::getpid().into_sifr_int()
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
    pub(super) fn join_path(base: &str, child: &str) -> String {
        let sifr_generated_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        if sifr_generated_chars_base.len() == SifrInt::from_i64(0) {
            return {
                let mut sifr_generated_concat: String =
                    String::with_capacity(child.len().saturating_add(0usize));
                sifr_generated_concat.push_str(child);
                sifr_generated_concat.push_str("");
                sifr_generated_concat
            };
        }
        let last: Option<String> = {
            let sifr_generated_string_index =
                ::std::ops::Sub::sub(SifrInt::from(base.chars().count()), SifrInt::from_i64(1));
            let sifr_generated_string_index_normalized =
                sifr_generated_string_index.normalize_index_or_len(sifr_generated_chars_base.len());
            sifr_generated_chars_base
                .get(sifr_generated_string_index_normalized)
                .copied()
        }
        .map(|character| character.to_string());
        if let Some(last) = last
            && last.as_str() == "/".to_string().as_str()
        {
            return {
                let mut sifr_generated_concat: String =
                    String::with_capacity(base.len().saturating_add(child.len()));
                sifr_generated_concat.push_str(base.as_ref());
                sifr_generated_concat.push_str(child.as_ref());
                sifr_generated_concat
            };
        }
        {
            let mut sifr_generated_concat: String = String::with_capacity(
                base.len()
                    .saturating_add(1usize)
                    .saturating_add(child.len()),
            );
            sifr_generated_concat.push_str(base);
            sifr_generated_concat.push('/');
            sifr_generated_concat.push_str(child);
            sifr_generated_concat
        }
    }
    pub(super) fn basename(path: &str) -> String {
        let sifr_generated_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = ::std::ops::Sub::sub(
            &SifrInt::from(sifr_generated_chars_path.len()),
            &SifrInt::from_i64(1),
        );
        while i >= SifrInt::from_i64(0) {
            let ch: Option<String> = {
                let sifr_generated_string_index = i.clone();
                let sifr_generated_string_index_normalized = sifr_generated_string_index
                    .normalize_index_or_len(sifr_generated_chars_path.len());
                sifr_generated_chars_path
                    .get(sifr_generated_string_index_normalized)
                    .copied()
            }
            .map(|character| character.to_string());
            if let Some(ch) = ch
                && ch == "/"
            {
                return {
                    let sifr_generated_slice_src = &sifr_generated_chars_path;
                    let sifr_generated_slice_len = sifr_generated_slice_src.len();
                    let sifr_generated_slice_start =
                        ::std::ops::Add::add(&i, &SifrInt::from_i64(1))
                            .clamp_slice_bound(sifr_generated_slice_len);
                    let sifr_generated_slice_stop = sifr_generated_slice_len;
                    String::from_iter(
                        sifr_generated_slice_src
                            .iter()
                            .skip(sifr_generated_slice_start)
                            .take(
                                sifr_generated_slice_stop
                                    .saturating_sub(sifr_generated_slice_start),
                            )
                            .copied(),
                    )
                };
            }
            i = ::std::ops::Sub::sub(&i, &SifrInt::from_i64(1));
        }
        {
            let mut sifr_generated_concat: String =
                String::with_capacity(path.len().saturating_add(0usize));
            sifr_generated_concat.push_str(path);
            sifr_generated_concat.push_str("");
            sifr_generated_concat
        }
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
    pub(super) fn sifr_generated_glob_list(
        path: &str,
        pattern: &str,
    ) -> Result<Vec<String>, IOError> {
        glob_pattern(path, pattern)
    }
    pub(super) fn sifr_generated_glob_to_iter(
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
            let e = sifr_generated_try_err;
            Err(e)
        })
    }
    pub(super) fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
        assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < actual.len() {
            assert_eq!(
                {
                    let sifr_generated_condition_list = &actual;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                },
                {
                    let sifr_generated_condition_list = &expected;
                    let sifr_generated_condition_index = i.clone();
                    let sifr_generated_condition_normalized = sifr_generated_condition_index
                        .normalize_index_or_len(sifr_generated_condition_list.len());
                    sifr_generated_condition_list
                        .get(sifr_generated_condition_normalized)
                        .copied()
                }
            );
            i = ::std::ops::Add::add(&i, &SifrInt::from_i64(1));
        }
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
    use crate::sifr_generated_generated_support::{
        exists, is_file, mkdir, read_text, remove_file, rmdir, sifr_generated_glob_to_iter,
        write_text,
    };
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
        #[must_use]
        pub fn exists(&self) -> bool {
            exists(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        #[must_use]
        pub fn is_file(&self) -> bool {
            is_file(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn read_text(&self) -> Result<String, IOError> {
            read_text(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn write_text(&self, content: &str) -> Result<(), IOError> {
            write_text(&self.path, content)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn mkdir(&self) -> Result<(), IOError> {
            mkdir(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn unlink(&self) -> Result<(), IOError> {
            remove_file(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn rmdir(&self) -> Result<(), IOError> {
            rmdir(&self.path)
        }
    }
    impl SifrGeneratedStdlibSifrX2epathlibX2ePath {
        ///# Errors
        ///Returns the typed error produced by this operation.
        pub fn glob(&self, pattern: &str) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            sifr_generated_glob_to_iter(&self.path, pattern)
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
}
use crate::sifr_generated_generated_support::{assert_bool_vector_eq, basename, getpid, join_path};
use ::sifr_runtime::SifrInt;
pub use sifr_generated_project_nominals::IOError;
pub use sifr_generated_project_nominals::SifrGeneratedStdlibSifrX2epathlibX2ePath;
fn collect_path_helpers_actual() -> Vec<bool> {
    vec![
        basename("/tmp/demo.txt").as_str() == "demo.txt".to_string().as_str(),
        join_path("/tmp", "demo.txt").as_str() == "/tmp/demo.txt".to_string().as_str(),
    ]
}
fn collect_path_class_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let base: String = {
        let mut sifr_generated_concat: String =
            String::with_capacity(31usize.saturating_add(0usize));
        sifr_generated_concat.push_str("/tmp/sifr_pathlib_pathlib_demo_");
        sifr_generated_concat.push_str(getpid().to_string().as_str());
        sifr_generated_concat
    };
    let filep: SifrGeneratedStdlibSifrX2epathlibX2ePath =
        SifrGeneratedStdlibSifrX2epathlibX2ePath::new(format!("{base}/demo.txt"));
    let dirp: SifrGeneratedStdlibSifrX2epathlibX2ePath =
        SifrGeneratedStdlibSifrX2epathlibX2ePath::new(base);
    let mut path_flow_ok: bool = false;
    let mut glob_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        dirp.mkdir()?;
        filep.write_text("hello")?;
        let content: String = filep.read_text()?;
        path_flow_ok = filep.exists() && filep.is_file() && content == "hello";
        let matches_it: Box<dyn Iterator<Item = String>> = dirp.glob("*.txt")?;
        glob_ok = matches_it.count() >= SifrInt::from_i64(1);
        filep.unlink()?;
        dirp.rmdir()?;
        cleanup_ok = !dirp.exists();
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
    }
    actual.push(path_flow_ok);
    actual.push(glob_ok);
    actual.push(cleanup_ok);
    actual
}
fn collect_missing_path_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = Vec::new();
    let mut missing_rejected: bool = false;
    let sifr_generated_try_res: Result<(), IOError> = (|| {
        let _ = SifrGeneratedStdlibSifrX2epathlibX2ePath::new(
            "/tmp/sifr_pathlib_pathlib_demo_missing.txt".to_string(),
        )
        .read_text()?;
        Ok(())
    })();
    if let Err(sifr_generated_try_err) = sifr_generated_try_res {
        let e = sifr_generated_try_err;
        let _ = e.message;
        missing_rejected = true;
    }
    actual.push(missing_rejected);
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = Vec::new();
    append_all(&mut actual, &collect_path_helpers_actual());
    append_all(&mut actual, &collect_path_class_actual());
    append_all(&mut actual, &collect_missing_path_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("pathlib pathlib parity demo: pass");
}
