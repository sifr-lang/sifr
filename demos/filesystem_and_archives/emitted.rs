// src/main.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct __SifrIoNativeFileHandle {
    pub _id: String,
}
impl __SifrIoNativeFileHandle {
    pub fn new(id: String) -> Self {
        let __sifr_field_init_0: String = id;
        Self { _id: __sifr_field_init_0 }
    }
}
impl __SifrIoNativeFileHandle {}
impl ::std::fmt::Display for __SifrIoNativeFileHandle {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "NativeFileHandle(_id={})", self._id)
    }
}

mod __sifr_project_nominals {
    use crate::__SifrIoNativeFileHandle;
    pub use ::sifr_runtime::SifrInt;
    pub fn read_text(path: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn write_text(path: &String, content: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn exists(path: &String) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub fn read_lines(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::read_lines(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn append_text(path: &String, content: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::append_text(path, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _open_file(path: &String, mode: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_read(handle: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::file_read(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write(handle: &String, data: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_readline(handle: &String) -> Result<Option<String>, IOError> {
        ::sifr_stdlib::fs::file_readline(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_readlines(handle: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::file_readlines(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_close(handle: &String) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub fn _file_read_bytes(
        handle: &String,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::fs::file_read_bytes(
                handle,
                size.map(::sifr_runtime::interop::SifrIntBridge::from),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_flush(handle: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_flush(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_seek(
        handle: &String,
        offset: SifrInt,
        whence: SifrInt,
    ) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::file_seek(
                handle,
                ::sifr_runtime::interop::SifrIntBridge::from(offset),
                ::sifr_runtime::interop::SifrIntBridge::from(whence),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_tell(handle: &String) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::file_tell(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn open_file(
        path: &String,
        mode: &String,
    ) -> Result<__SifrIoNativeFileHandle, IOError> {
        let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
            let handle_id: String = _open_file(path, mode)?;
            Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn file_read(handle: &__SifrIoNativeFileHandle) -> Result<String, IOError> {
        _file_read(&handle._id.clone())
    }
    pub fn file_write(
        handle: &__SifrIoNativeFileHandle,
        data: &String,
    ) -> Result<(), IOError> {
        _file_write(&handle._id.clone(), data)
    }
    pub fn file_readline(
        handle: &__SifrIoNativeFileHandle,
    ) -> Result<Option<String>, IOError> {
        _file_readline(&handle._id.clone())
    }
    pub fn file_readlines(
        handle: &__SifrIoNativeFileHandle,
    ) -> Result<Vec<String>, IOError> {
        _file_readlines(&handle._id.clone())
    }
    pub fn file_close(handle: &__SifrIoNativeFileHandle) {
        _file_close(&handle._id.clone());
    }
    pub fn file_read_bytes(
        handle: &__SifrIoNativeFileHandle,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        _file_read_bytes(&handle._id.clone(), (size).clone())
    }
    pub fn file_write_bytes(
        handle: &__SifrIoNativeFileHandle,
        data: &Vec<u8>,
    ) -> Result<(), IOError> {
        _file_write_bytes(&handle._id.clone(), data)
    }
    pub fn file_flush(handle: &__SifrIoNativeFileHandle) -> Result<(), IOError> {
        _file_flush(&handle._id.clone())
    }
    pub fn file_seek(
        handle: &__SifrIoNativeFileHandle,
        offset: SifrInt,
        whence: SifrInt,
    ) -> Result<SifrInt, IOError> {
        _file_seek(&handle._id.clone(), (offset).clone(), (whence).clone())
    }
    pub fn file_tell(handle: &__SifrIoNativeFileHandle) -> Result<SifrInt, IOError> {
        _file_tell(&handle._id.clone())
    }
    pub fn getcwd() -> Result<String, IOError> {
        ::sifr_stdlib::fs::getcwd()
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn listdir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::listdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn mkdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rmdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn remove_file(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rename(src: &String, dst: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rename(src, dst)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn chdir(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::chdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn stat_size(path: &String) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::stat_size(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn disk_usage(path: &String) -> Vec<SifrInt> {
        ::sifr_stdlib::fs::disk_usage(path)
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn is_file(path: &String) -> bool {
        ::sifr_stdlib::fs::is_file(path)
    }
    pub fn is_dir(path: &String) -> bool {
        ::sifr_stdlib::fs::is_dir(path)
    }
    pub fn copy_file(src: &String, dst: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::copy_file(src, dst)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn walk_dir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::walk_dir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rmdir_all(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir_all(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn gettempdir() -> String {
        ::sifr_stdlib::fs::gettempdir()
    }
    pub fn makedirs(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::makedirs(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn touch(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::fs::touch(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn resolve_path(path: &String) -> Result<String, IOError> {
        ::sifr_stdlib::fs::resolve_path(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn iterdir(path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::iterdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn glob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::glob_pattern(dir, pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rglob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub struct __SifrYielder<T> {
        pub slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    }
    pub struct __SifrYieldFuture<T> {
        pub slot: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub value: Option<T>,
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
        pub fn suspend(&self, value: T) -> __SifrYieldFuture<T> {
            __SifrYieldFuture {
                slot: ::std::sync::Arc::clone(&self.slot),
                value: Some(value),
            }
        }
    }
    pub fn __sifr_store_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        value: T,
    ) {
        match slot.lock() {
            Ok(mut state) => *state = Some(value),
            Err(poisoned) => *poisoned.into_inner() = Some(value),
        }
    }
    pub fn __sifr_take_suspended<T>(
        slot: &::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
    ) -> Option<T> {
        match slot.lock() {
            Ok(mut state) => state.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        }
    }
    pub struct __SifrGenerator<T> {
        pub producer: Option<
            ::std::pin::Pin<Box<dyn ::std::future::Future<Output = ()> + 'static>>,
        >,
        pub yielded: ::std::sync::Arc<::std::sync::Mutex<Option<T>>>,
        pub complete: bool,
    }
    impl<T> __SifrGenerator<T> {
        pub fn new<
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub _path: String,
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn new(path: String) -> Self {
            let __sifr_field_init_0: String = path;
            Self { _path: __sifr_field_init_0 }
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn name(&self) -> String {
            basename(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn parent(&self) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
            __SifrStdlib_sifr_x2epathlib_x2ePath::new(dirname(&self._path))
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn suffix(&self) -> String {
            extension(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn stem(&self) -> String {
            stem(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn exists(&self) -> bool {
            exists(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn is_file(&self) -> bool {
            is_file(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn is_dir(&self) -> bool {
            is_dir(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn is_absolute(&self) -> bool {
            is_absolute(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn read_text(&self) -> Result<String, IOError> {
            read_text(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn write_text(&self, content: &String) -> Result<(), IOError> {
            write_text(&self._path, content)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn mkdir(&self) -> Result<(), IOError> {
            mkdir(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn joinpath(&self, child: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
            __SifrStdlib_sifr_x2epathlib_x2ePath::new(join_path(&self._path, child))
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn to_str(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._path.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn touch(&self) -> Result<(), IOError> {
            touch(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn unlink(&self) -> Result<(), IOError> {
            remove_file(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn rmdir(&self) -> Result<(), IOError> {
            rmdir(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn resolve(&self) -> Result<String, IOError> {
            resolve_path(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn iterdir(&self) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            _iterdir_to_iter(&self._path)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn with_name(&self, name: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
            let parent: String = dirname(&self._path);
            if (parent == "") {
                return __SifrStdlib_sifr_x2epathlib_x2ePath::new(format!("{}{}", name, ""));
            }
            __SifrStdlib_sifr_x2epathlib_x2ePath::new(
                format!("{}{}", format!("{}{}", parent, "/"), name),
            )
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn with_suffix(&self, suffix: &String) -> __SifrStdlib_sifr_x2epathlib_x2ePath {
            let s: String = stem(&self._path);
            let parent: String = dirname(&self._path);
            if (parent == "") {
                return __SifrStdlib_sifr_x2epathlib_x2ePath::new(format!("{}{}", s, suffix));
            }
            __SifrStdlib_sifr_x2epathlib_x2ePath::new(
                format!("{}{}", format!("{}{}", format!("{}{}", parent, "/"), s), suffix),
            )
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn glob(
            &self,
            pattern: &String,
        ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            _glob_to_iter(&self._path, pattern)
        }
    }
    impl __SifrStdlib_sifr_x2epathlib_x2ePath {
        pub fn rglob(
            &self,
            pattern: &String,
        ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
            _rglob_to_iter(&self._path, pattern)
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2epathlib_x2ePath {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Path(_path={})", self._path)
        }
    }
    pub fn join_path(base: &String, child: &String) -> String {
        let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        if (&SifrInt::from(__sifr_chars_base.len()) == &SifrInt::from_i64(0)) {
            return {
                let mut __sifr_concat: String = String::with_capacity(child.len() + 0usize);
                __sifr_concat.push_str((child).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        let last: Option<String> = ({
            let __sifr_string_index = SifrInt::from(base.chars().count())
                - SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_base.len());
            __sifr_chars_base.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(last) = last {
            if (last).as_str() == ("/".to_string()).as_str() {
                return {
                    let mut __sifr_concat: String = String::with_capacity(
                        base.len() + child.len(),
                    );
                    __sifr_concat.push_str((base).as_str());
                    __sifr_concat.push_str((child).as_str());
                    __sifr_concat
                };
            }
        }
        {
            let mut __sifr_concat: String = String::with_capacity(
                (base.len() + 1usize) + child.len(),
            );
            __sifr_concat.push_str((base).as_str());
            __sifr_concat.push('/');
            __sifr_concat.push_str((child).as_str());
            __sifr_concat
        }
    }
    pub fn basename(path: &String) -> String {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
        while (&i >= &SifrInt::from_i64(0)) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                if (ch == "/") {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len = _slice_src.len();
                        let _slice_start = (&i + &SifrInt::from_i64(1))
                            .clamp_slice_bound(_slice_len);
                        let _slice_stop = _slice_len;
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    };
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    pub fn dirname(path: &String) -> String {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
        while (&i >= &SifrInt::from_i64(0)) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                if (ch == "/") {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len = _slice_src.len();
                        let _slice_start = 0;
                        let _slice_stop = i.clamp_slice_bound(_slice_len);
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    };
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        "".to_string()
    }
    pub fn extension(path: &String) -> String {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
        while (&i >= &SifrInt::from_i64(0)) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                if (ch == ".") {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len = _slice_src.len();
                        let _slice_start = i.clamp_slice_bound(_slice_len);
                        let _slice_stop = _slice_len;
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    };
                }
                if (ch == "/") {
                    return "".to_string();
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        "".to_string()
    }
    pub fn stem(path: &String) -> String {
        let base: String = basename(path);
        let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut i: SifrInt = &SifrInt::from(__sifr_chars_base.len()) - &SifrInt::from_i64(1);
        while (&i > &SifrInt::from_i64(0)) {
            let ch: Option<String> = ({
                let __sifr_string_index = i.clone();
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_base.len());
                __sifr_chars_base.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(ch) = ch {
                if (ch == ".") {
                    return {
                        let _slice_src = &__sifr_chars_base;
                        let _slice_len = _slice_src.len();
                        let _slice_start = 0;
                        let _slice_stop = i.clamp_slice_bound(_slice_len);
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start)
                                .take(_slice_stop.saturating_sub(_slice_start))
                                .copied(),
                        )
                    };
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
        {
            let mut __sifr_concat: String = String::with_capacity(base.len() + 0usize);
            __sifr_concat.push_str((base).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    pub fn is_absolute(path: &String) -> bool {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        if (&SifrInt::from(__sifr_chars_path.len()) == &SifrInt::from_i64(0)) {
            return false;
        }
        if (&SifrInt::from(__sifr_chars_path.len()) >= &SifrInt::from_i64(3)) {
            let colon: Option<String> = ({
                let __sifr_string_index = SifrInt::from_i64(1);
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            let sep: Option<String> = ({
                let __sifr_string_index = SifrInt::from_i64(2);
                let __sifr_string_index_normalized = __sifr_string_index
                    .normalize_index_or_len(__sifr_chars_path.len());
                __sifr_chars_path.get(__sifr_string_index_normalized)
            })
                .map(|c| c.to_string());
            if let Some(colon) = colon {
                if let Some(sep) = sep {
                    if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                        return true;
                    }
                }
            }
        }
        let first: Option<String> = ({
            let __sifr_string_index = SifrInt::from_i64(0);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(first) = first {
            if (first == "/") || (first == "\\") {
                return true;
            }
        }
        false
    }
    pub fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
        Box::new(
            __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<String>| {
                let mut i: SifrInt = SifrInt::from_i64(0);
                while (&i < &SifrInt::from(entries.len())) {
                    let Some(__sifr_checked_value_7) = ({
                        let __sifr_checked_read_collection = &entries;
                        let __sifr_checked_read_index = i.clone();
                        let __sifr_checked_read_normalized = __sifr_checked_read_index
                            .normalize_index_or_len(__sifr_checked_read_collection.len());
                        __sifr_checked_read_collection
                            .get(__sifr_checked_read_normalized)
                            .cloned()
                    }) else {
                        break;
                    };
                    __sifr_yielder.suspend(__sifr_checked_value_7.clone()).await;
                    i = &i + &SifrInt::from_i64(1);
                }
            }),
        )
    }
    pub fn _iterdir_list(path: &String) -> Result<Vec<String>, IOError> {
        iterdir(path)
    }
    pub fn _glob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        glob_pattern(path, pattern)
    }
    pub fn _rglob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
        rglob_pattern(path, pattern)
    }
    pub fn _iterdir_to_iter(
        path: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let __sifr_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = _iterdir_list(path)?;
            Ok(Ok(_iter_list_str(entries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn _glob_to_iter(
        path: &String,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let __sifr_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = _glob_list(path, pattern)?;
            Ok(Ok(_iter_list_str(entries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn _rglob_to_iter(
        path: &String,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        let __sifr_try_res: Result<
            Result<Box<dyn Iterator<Item = String>>, IOError>,
            IOError,
        > = (|| {
            let entries: Vec<String> = _rglob_list(path, pattern)?;
            Ok(Ok(_iter_list_str(entries)))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
    }
    pub fn _gzip_compress_bytes_impl(data: &String) -> Vec<u8> {
        ::sifr_stdlib::gzip::gzip_compress_bytes(data)
    }
    pub fn _gzip_decompress_bytes_impl(data: &Vec<u8>) -> Result<String, IOError> {
        ::sifr_stdlib::gzip::gzip_decompress_bytes(data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_create(path: &String) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_create(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_add_file(
        zip_path: &String,
        name: &String,
        content: &String,
    ) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_add_file(zip_path, name, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_add_file_bytes(
        zip_path: &String,
        name: &String,
        content: &Vec<u8>,
    ) -> Result<(), IOError> {
        ::sifr_stdlib::zipfile::zip_add_file_bytes(zip_path, name, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_read_file(zip_path: &String, name: &String) -> Result<String, IOError> {
        ::sifr_stdlib::zipfile::zip_read_file(zip_path, name)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_read_file_bytes(
        zip_path: &String,
        name: &String,
    ) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::zipfile::zip_read_file_bytes(zip_path, name)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn zip_namelist(zip_path: &String) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::zipfile::zip_namelist(zip_path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ezipfile_x2eZipInfo {
        pub filename: String,
        pub file_size: SifrInt,
        pub compress_type: SifrInt,
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipInfo {
        pub fn new(filename: String, file_size: SifrInt, compress_type: SifrInt) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    filename.len() + 0usize,
                );
                __sifr_concat.push_str((filename).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: SifrInt = file_size.clone();
            let __sifr_field_init_2: SifrInt = compress_type.clone();
            Self {
                filename: __sifr_field_init_0,
                file_size: __sifr_field_init_1,
                compress_type: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipInfo {}
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2ezipfile_x2eZipInfo {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "ZipInfo(filename={}, file_size={}, compress_type={})", self.filename,
                self.file_size, self.compress_type
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub _data: Vec<u8>,
        pub _cursor: SifrInt,
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub fn new(data: Vec<u8>) -> Self {
            let __sifr_field_init_0: Vec<u8> = data;
            let __sifr_field_init_1: SifrInt = SifrInt::from_i64(0);
            let __sifr_field_init_2: bool = false;
            Self {
                _data: __sifr_field_init_0,
                _cursor: __sifr_field_init_1,
                _closed: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle {
        pub fn read_bytes(&mut self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let mut end: SifrInt = SifrInt::from(self._data.len());
            if let Some(size) = size.as_ref() {
                let requested_size: SifrInt = size.clone();
                if (&requested_size < &SifrInt::from_i64(0)) {
                    end = SifrInt::from(self._data.len());
                } else {
                    let requested_end: SifrInt = &self._cursor.clone() + &requested_size;
                    if (&requested_end < &end) {
                        end = requested_end;
                    }
                }
            }
            let out: Vec<u8> = {
                let _slice_src = &self._data.clone();
                let _slice_len = _slice_src.len();
                let _slice_start = self._cursor.clone().clamp_slice_bound(_slice_len);
                let _slice_stop = end.clamp_slice_bound(_slice_len);
                Vec::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start))
                        .cloned(),
                )
            };
            self._cursor = end.clone();
            Ok(out)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub path: String,
        pub mode: String,
        pub compression: SifrInt,
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn new(path: String, mode: String, compression: SifrInt) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
                __sifr_concat.push_str((path).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: String = {
                let mut __sifr_concat: String = String::with_capacity(mode.len() + 0usize);
                __sifr_concat.push_str((mode).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_2: SifrInt = compression.clone();
            Self {
                path: __sifr_field_init_0,
                mode: __sifr_field_init_1,
                compression: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn _writable_mode(&self) -> bool {
            (((((self.mode.clone() == "w")) || ((self.mode.clone() == "a")))
                || ((self.mode.clone() == "wb"))) || ((self.mode.clone() == "ab")))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn create(&self) -> Result<(), IOError> {
            zip_create(&self.path)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn write(&self, name: &String, content: &String) -> Result<(), IOError> {
            if !self._writable_mode() {
                return Err(IOError::new(_zip_read_only_error()));
            }
            zip_add_file(&self.path, name, content)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn write_bytes(&self, name: &String, content: &Vec<u8>) -> Result<(), IOError> {
            if !self._writable_mode() {
                return Err(IOError::new(_zip_read_only_error()));
            }
            zip_add_file_bytes(&self.path, name, content)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn read(&self, name: &String) -> Result<String, IOError> {
            zip_read_file(&self.path, name)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn read_bytes(&self, name: &String) -> Result<Vec<u8>, IOError> {
            zip_read_file_bytes(&self.path, name)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn namelist(&self) -> Result<Vec<String>, IOError> {
            zip_namelist(&self.path)
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn infolist(
            &self,
        ) -> Result<Vec<__SifrStdlib_sifr_x2ezipfile_x2eZipInfo>, IOError> {
            Err(IOError::new(_zip_unimplemented_error(&"infolist".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn getinfo(
            &self,
            name: &String,
        ) -> Result<__SifrStdlib_sifr_x2ezipfile_x2eZipInfo, IOError> {
            let _ = (name).clone();
            Err(IOError::new(_zip_unimplemented_error(&"getinfo".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn open(
            &self,
            name: &String,
            mode: &String,
        ) -> Result<__SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle, IOError> {
            let _ = (name).clone();
            if ((mode).as_str() != "r") && ((mode).as_str() != "rb") {
                return Err(IOError::new(_zip_open_mode_error(mode)));
            }
            Err(IOError::new(_zip_unimplemented_error(&"open".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn extract(&self, name: &String, path: &String) -> Result<String, IOError> {
            let _ = (name).clone();
            let _ = (path).clone();
            Err(IOError::new(_zip_unimplemented_error(&"extract".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn extractall(&self, path: &String) -> Result<Vec<String>, IOError> {
            let _ = (path).clone();
            Err(IOError::new(_zip_unimplemented_error(&"extractall".to_string())))
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn __enter__(&self) -> __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
            self.clone()
        }
    }
    impl __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        pub fn __exit__(&self) {}
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2ezipfile_x2eZipFile {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "ZipFile(path={}, mode={}, compression={})", self.path, self.mode, self
                .compression
            )
        }
    }
    pub fn _zip_read_only_error() -> String {
        "zipfile operation requires write or append mode".to_string()
    }
    pub fn _zip_open_mode_error(mode: &String) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(48usize + mode.len());
            __sifr_concat.push_str("zipfile open supports read-only mode only, got: ");
            __sifr_concat.push_str((mode).as_str());
            __sifr_concat
        }
    }
    pub fn _closed_stream_error() -> String {
        "I/O operation on closed stream".to_string()
    }
    pub fn _zip_unimplemented_error(feature: &String) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(
                (8usize + feature.len()) + 49usize,
            );
            __sifr_concat.push_str("zipfile ");
            __sifr_concat.push_str((feature).as_str());
            __sifr_concat.push_str(" is not implemented in this compatibility surface");
            __sifr_concat
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct IOError {
        pub message: String,
        pub kind: String,
    }
    impl IOError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                kind: "Other".to_string(),
            }
        }
    }
    impl ::std::fmt::Display for IOError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for IOError {}
    pub fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
        let msg = e.to_string();
        let kind = {
            let __sifr_io_kind = (&e as &dyn ::std::any::Any)
                .downcast_ref::<std::io::Error>()
                .map(::std::io::Error::kind);
            match __sifr_io_kind {
                Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
                Some(::std::io::ErrorKind::PermissionDenied) => {
                    "PermissionDenied".to_string()
                }
                Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
                Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
                Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
                Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                    "DirectoryNotEmpty".to_string()
                }
                _ => "Other".to_string(),
            }
        };
        IOError { message: msg, kind }
    }
}
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2epathlib_x2ePath;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipFile;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipInfo;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle;
use ::sifr_runtime::SifrInt;
fn read_text(path: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::read_text(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn write_text(path: &String, content: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn exists(path: &String) -> bool {
    ::sifr_stdlib::fs::exists(path)
}
fn read_lines(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::read_lines(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn append_text(path: &String, content: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::append_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _open_file(path: &String, mode: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::open_file(path, mode)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_read(handle: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::file_read(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write(handle: &String, data: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readline(handle: &String) -> Result<Option<String>, IOError> {
    ::sifr_stdlib::fs::file_readline(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readlines(handle: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::file_readlines(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_close(handle: &String) {
    ::sifr_stdlib::fs::file_close(handle);
}
fn _file_read_bytes(handle: &String, size: Option<SifrInt>) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::fs::file_read_bytes(
            handle,
            size.map(::sifr_runtime::interop::SifrIntBridge::from),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write_bytes(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_flush(handle: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_flush(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_seek(
    handle: &String,
    offset: SifrInt,
    whence: SifrInt,
) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::file_seek(
            handle,
            ::sifr_runtime::interop::SifrIntBridge::from(offset),
            ::sifr_runtime::interop::SifrIntBridge::from(whence),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_tell(handle: &String) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::file_tell(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn open_file(path: &String, mode: &String) -> Result<__SifrIoNativeFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
        let handle_id: String = _open_file(path, mode)?;
        Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn file_read(handle: &__SifrIoNativeFileHandle) -> Result<String, IOError> {
    _file_read(&handle._id.clone())
}
fn file_write(handle: &__SifrIoNativeFileHandle, data: &String) -> Result<(), IOError> {
    _file_write(&handle._id.clone(), data)
}
fn file_readline(handle: &__SifrIoNativeFileHandle) -> Result<Option<String>, IOError> {
    _file_readline(&handle._id.clone())
}
fn file_readlines(handle: &__SifrIoNativeFileHandle) -> Result<Vec<String>, IOError> {
    _file_readlines(&handle._id.clone())
}
fn file_close(handle: &__SifrIoNativeFileHandle) {
    _file_close(&handle._id.clone());
}
fn file_read_bytes(
    handle: &__SifrIoNativeFileHandle,
    size: Option<SifrInt>,
) -> Result<Vec<u8>, IOError> {
    _file_read_bytes(&handle._id.clone(), (size).clone())
}
fn file_write_bytes(
    handle: &__SifrIoNativeFileHandle,
    data: &Vec<u8>,
) -> Result<(), IOError> {
    _file_write_bytes(&handle._id.clone(), data)
}
fn file_flush(handle: &__SifrIoNativeFileHandle) -> Result<(), IOError> {
    _file_flush(&handle._id.clone())
}
fn file_seek(
    handle: &__SifrIoNativeFileHandle,
    offset: SifrInt,
    whence: SifrInt,
) -> Result<SifrInt, IOError> {
    _file_seek(&handle._id.clone(), (offset).clone(), (whence).clone())
}
fn file_tell(handle: &__SifrIoNativeFileHandle) -> Result<SifrInt, IOError> {
    _file_tell(&handle._id.clone())
}
fn getcwd() -> Result<String, IOError> {
    ::sifr_stdlib::fs::getcwd()
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn listdir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn mkdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::mkdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn remove_file(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::remove_file(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rename(src: &String, dst: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rename(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn chdir(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::chdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn stat_size(path: &String) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn disk_usage(path: &String) -> Vec<SifrInt> {
    ::sifr_stdlib::fs::disk_usage(path)
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn is_file(path: &String) -> bool {
    ::sifr_stdlib::fs::is_file(path)
}
fn is_dir(path: &String) -> bool {
    ::sifr_stdlib::fs::is_dir(path)
}
fn copy_file(src: &String, dst: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::copy_file(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn walk_dir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::walk_dir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir_all(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir_all(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn gettempdir() -> String {
    ::sifr_stdlib::fs::gettempdir()
}
fn makedirs(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::makedirs(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn touch(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::fs::touch(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn resolve_path(path: &String) -> Result<String, IOError> {
    ::sifr_stdlib::fs::resolve_path(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn iterdir(path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::iterdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn glob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::glob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rglob_pattern(dir: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn fnmatch(name: &String, pattern: &String) -> bool {
    _match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
}
fn _match(name: &String, mut ni: SifrInt, pattern: &String, mut pi: SifrInt) -> bool {
    while (&pi < &SifrInt::from(pattern.chars().count())) {
        let pc: Option<String> = ({
            let __sifr_string_source = &pattern;
            let __sifr_string_index = pi.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(pc) = pc {
            if (pc == "*") {
                pi = &pi + &SifrInt::from_i64(1);
                if (&pi == &SifrInt::from(pattern.chars().count())) {
                    return true;
                }
                let mut j: SifrInt = ni.clone();
                while (&j <= &SifrInt::from(name.chars().count())) {
                    if _match(name, (j).clone(), pattern, (pi).clone()) {
                        return true;
                    }
                    j = &j + &SifrInt::from_i64(1);
                }
                return false;
            } else {
                if (pc == "?") {
                    if (&ni >= &SifrInt::from(name.chars().count())) {
                        return false;
                    }
                    ni = &ni + &SifrInt::from_i64(1);
                    pi = &pi + &SifrInt::from_i64(1);
                } else {
                    if (&ni >= &SifrInt::from(name.chars().count())) {
                        return false;
                    }
                    let nc: Option<String> = ({
                        let __sifr_string_source = &name;
                        let __sifr_string_index = ni.clone();
                        let __sifr_string_index_normalized = __sifr_string_index
                            .normalize_index_or_len(
                                __sifr_string_source.chars().count(),
                            );
                        __sifr_string_source.chars().nth(__sifr_string_index_normalized)
                    })
                        .map(|c| c.to_string());
                    if let Some(nc) = nc {
                        if (nc != pc) {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni = &ni + &SifrInt::from_i64(1);
                    pi = &pi + &SifrInt::from_i64(1);
                }
            }
        } else {
            return false;
        }
    }
    (&ni == &SifrInt::from(name.chars().count()))
}
fn filterfalse(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if !fnmatch(&name, pattern) {
            result.push(name.clone());
        }
    }
    result
}
fn _translate_literal(ch: &String) -> String {
    if (ch).as_str() == "." {
        return "\\.".to_string();
    }
    if (ch).as_str() == "^" {
        return "\\^".to_string();
    }
    if (ch).as_str() == "$" {
        return "\\$".to_string();
    }
    if (ch).as_str() == "+" {
        return "\\+".to_string();
    }
    if (ch).as_str() == "(" {
        return "\\(".to_string();
    }
    if (ch).as_str() == ")" {
        return "\\)".to_string();
    }
    if (ch).as_str() == "{" {
        return "\\{".to_string();
    }
    if (ch).as_str() == "}" {
        return "\\}".to_string();
    }
    if (ch).as_str() == "[" {
        return "\\[".to_string();
    }
    if (ch).as_str() == "]" {
        return "\\]".to_string();
    }
    if (ch).as_str() == "|" {
        return "\\|".to_string();
    }
    if (ch).as_str() == "\\" {
        return "\\\\".to_string();
    }
    {
        let mut __sifr_concat: String = String::with_capacity(ch.len() + 0usize);
        __sifr_concat.push_str((ch).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn translate(pattern: &String) -> String {
    let __sifr_chars_pattern: Vec<char> = pattern.chars().collect::<Vec<char>>();
    let mut body: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_pattern.len())) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_pattern.len());
            __sifr_chars_pattern.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == "*") {
                body.push_str(".*");
            } else {
                if (ch == "?") {
                    body.push('.');
                } else {
                    body.push_str((_translate_literal(&ch)).as_str());
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            (4usize + body.len()) + 3usize,
        );
        __sifr_concat.push_str("(?s:");
        __sifr_concat.push_str((body).as_str());
        __sifr_concat.push_str(")\\z");
        __sifr_concat
    }
}
fn filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name.clone());
        }
    }
    result
}
fn glob(directory: &String, pattern: &String) -> Vec<String> {
    let __sifr_chars_pattern: Vec<char> = pattern.chars().collect::<Vec<char>>();
    let include_hidden: bool = (((&SifrInt::from(__sifr_chars_pattern.len())
        > &SifrInt::from_i64(0)))
        && (({
            let __sifr_string_index = SifrInt::from_i64(0);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_pattern.len());
            __sifr_chars_pattern.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string())
            .is_some_and(|__sifr_checked_value_0| {
                (__sifr_checked_value_0.clone() == ".")
            })));
    let mut matches: Vec<String> = vec![];
    let __sifr_try_res: Result<(), IOError> = (|| {
        let entries: Vec<String> = listdir(directory)?;
        for entry in entries.iter().cloned() {
            let __sifr_chars_entry: Vec<char> = entry.chars().collect::<Vec<char>>();
            if (&SifrInt::from(__sifr_chars_entry.len()) == &SifrInt::from_i64(0)) {
                continue;
            }
            if !include_hidden
                && ({
                    let __sifr_string_index = SifrInt::from_i64(0);
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_entry.len());
                    __sifr_chars_entry.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string())
                    .is_some_and(|__sifr_checked_value_1| {
                        (__sifr_checked_value_1.clone() == ".")
                    })
            {
                continue;
            }
            if fnmatch(&entry, pattern) {
                matches.push(entry.clone());
            }
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        return vec![];
    }
    {
        let mut __sifr_sorted_v = (matches).iter().cloned().collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        __sifr_sorted_v
    }
}
fn _gzip_compress_bytes_impl(data: &String) -> Vec<u8> {
    ::sifr_stdlib::gzip::gzip_compress_bytes(data)
}
fn _gzip_decompress_bytes_impl(data: &Vec<u8>) -> Result<String, IOError> {
    ::sifr_stdlib::gzip::gzip_decompress_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_create(path: &String) -> Result<(), IOError> {
    ::sifr_stdlib::zipfile::zip_create(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_add_file(
    zip_path: &String,
    name: &String,
    content: &String,
) -> Result<(), IOError> {
    ::sifr_stdlib::zipfile::zip_add_file(zip_path, name, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_add_file_bytes(
    zip_path: &String,
    name: &String,
    content: &Vec<u8>,
) -> Result<(), IOError> {
    ::sifr_stdlib::zipfile::zip_add_file_bytes(zip_path, name, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_read_file(zip_path: &String, name: &String) -> Result<String, IOError> {
    ::sifr_stdlib::zipfile::zip_read_file(zip_path, name)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_read_file_bytes(zip_path: &String, name: &String) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::zipfile::zip_read_file_bytes(zip_path, name)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn zip_namelist(zip_path: &String) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::zipfile::zip_namelist(zip_path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn compress(data: &String) -> Vec<u8> {
    _gzip_compress_bytes_impl(data)
}
fn decompress(data: &Vec<u8>) -> Result<String, IOError> {
    _gzip_decompress_bytes_impl(data)
}
fn _encoding_is_supported_impl(label: &String) -> bool {
    ::sifr_stdlib::encoding::encoding_is_supported(label)
}
fn _encoding_canonical_label_impl(label: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_canonical_label(label)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_text_impl(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_text(data, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_recoveries_impl(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_recoveries(data, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_text_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_text(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_recoveries_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_recoveries(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_pending_impl(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    r#final: bool,
) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_incremental_pending(
            data,
            pending,
            encoding,
            r#final,
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_encode_bytes_impl(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::encoding::encoding_encode_bytes(text, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_encode_recoveries_impl(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_encode_recoveries(text, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn __const_ENCODING_UTF8() -> String {
    "utf-8".to_string().to_string()
}
fn __const_ENCODING_UTF8_SIG() -> String {
    "utf-8-sig".to_string().to_string()
}
fn __const_ENCODING_ASCII() -> String {
    "ascii".to_string().to_string()
}
fn __const_ENCODING_LATIN1() -> String {
    "latin-1".to_string().to_string()
}
fn __const_ENCODING_UTF16_LE() -> String {
    "utf-16-le".to_string().to_string()
}
fn __const_ENCODING_UTF16_BE() -> String {
    "utf-16-be".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1250() -> String {
    "windows-1250".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1251() -> String {
    "windows-1251".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1252() -> String {
    "windows-1252".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1253() -> String {
    "windows-1253".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1254() -> String {
    "windows-1254".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1255() -> String {
    "windows-1255".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1256() -> String {
    "windows-1256".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1257() -> String {
    "windows-1257".to_string().to_string()
}
fn __const_ENCODING_WINDOWS_1258() -> String {
    "windows-1258".to_string().to_string()
}
fn __const_DECODE_ERRORS_STRICT() -> String {
    "strict".to_string().to_string()
}
fn __const_DECODE_ERRORS_REPLACE() -> String {
    "replace".to_string().to_string()
}
fn __const_DECODE_ERRORS_IGNORE() -> String {
    "ignore".to_string().to_string()
}
fn __const_DECODE_ERRORS_BACKSLASH_REPLACE() -> String {
    "backslashreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_STRICT() -> String {
    "strict".to_string().to_string()
}
fn __const_ENCODE_ERRORS_REPLACE() -> String {
    "replace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_IGNORE() -> String {
    "ignore".to_string().to_string()
}
fn __const_ENCODE_ERRORS_BACKSLASH_REPLACE() -> String {
    "backslashreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_XMLCHARREF_REPLACE() -> String {
    "xmlcharrefreplace".to_string().to_string()
}
fn __const_ENCODE_ERRORS_NAME_REPLACE() -> String {
    "namereplace".to_string().to_string()
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    message: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("DecodeError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2eencoding_x2eDecodeError {}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    message: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("EncodeError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2eencoding_x2eEncodeError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    label: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn new(label: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(label.len() + 0usize);
            __sifr_concat.push_str((label).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { label: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn canonical_label(
        &self,
    ) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
        _encoding_canonical_label(&self.label)
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn is_supported(&self) -> bool {
        _encoding_is_supported(&self.label)
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "Encoding(label={})", self.label)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    name: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
            __sifr_concat.push_str((name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { name: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "DecodeErrorHandler(name={})", self.name)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    name: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    fn new(name: String) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
            __sifr_concat.push_str((name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        Self { name: __sifr_field_init_0 }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "EncodeErrorHandler(name={})", self.name)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    text: String,
    recoveries: Vec<String>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn new(text: String, recoveries: Vec<String>) -> Self {
        let __sifr_field_init_0: String = {
            let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
            __sifr_concat.push_str((text).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        let __sifr_field_init_1: Vec<String> = recoveries;
        Self {
            text: __sifr_field_init_0,
            recoveries: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn get_text(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self.text.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn get_recoveries(&self) -> Vec<String> {
        self.recoveries.clone()
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    data: Vec<u8>,
    recoveries: Vec<String>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn new(data: Vec<u8>, recoveries: Vec<String>) -> Self {
        let __sifr_field_init_0: Vec<u8> = data;
        let __sifr_field_init_1: Vec<String> = recoveries;
        Self {
            data: __sifr_field_init_0,
            recoveries: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
    fn get_recoveries(&self) -> Vec<String> {
        self.recoveries.clone()
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
    _errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
    _exhausted: bool,
    _pending: Vec<u8>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    fn new(
        enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        errors: Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> Self {
        let __sifr_field_init_0: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = _decode_handler_or_strict(
            &errors,
        );
        let __sifr_field_init_2: bool = false;
        let __sifr_field_init_3: Vec<u8> = {
            let __sifr_empty_bytes_literal: Vec<u8> = vec![];
            __sifr_empty_bytes_literal
        };
        Self {
            _encoding: __sifr_field_init_0,
            _errors: __sifr_field_init_1,
            _exhausted: __sifr_field_init_2,
            _pending: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
    fn decode(
        &mut self,
        data: &Vec<u8>,
        r#final: bool,
    ) -> Result<
        __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > {
        if self._exhausted {
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(
                    "decoder is exhausted".to_string(),
                ),
            );
        }
        let __sifr_try_res: Result<
            Result<
                __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
            >,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        > = (|| {
            let outcome: __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome = _encoding_decode_incremental_outcome(
                data,
                &self._pending,
                &self._encoding.clone().label.clone(),
                &self._errors.clone().name.clone(),
                r#final,
            )?;
            let next_pending: Vec<u8> = _encoding_decode_incremental_pending(
                data,
                &self._pending,
                &self._encoding.clone().label.clone(),
                r#final,
            )?;
            self._pending = next_pending;
            if r#final {
                self._pending = {
                    let __sifr_empty_bytes_literal: Vec<u8> = vec![];
                    __sifr_empty_bytes_literal
                };
                self._exhausted = true;
            }
            Ok(Ok(outcome))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
                );
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
    _errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
    _exhausted: bool,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn new(
        enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        errors: Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
    ) -> Self {
        let __sifr_field_init_0: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
        let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler = _encode_handler_or_strict(
            &errors,
        );
        let __sifr_field_init_2: bool = false;
        Self {
            _encoding: __sifr_field_init_0,
            _errors: __sifr_field_init_1,
            _exhausted: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn encode(
        &mut self,
        text: &String,
        r#final: bool,
    ) -> Result<
        __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > {
        if self._exhausted {
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(
                    "encoder is exhausted".to_string(),
                ),
            );
        }
        let __sifr_try_res: Result<
            Result<
                __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
            >,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        > = (|| {
            let outcome: __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome = encode_outcome(
                text,
                &self._encoding,
                &Some((self._errors.clone()).clone()),
            )?;
            if r#final {
                self._exhausted = true;
            }
            Ok(Ok(outcome))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(
                    __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
                );
            }
        }
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncoder {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f, "Encoder(_encoding={}, _errors={}, _exhausted={})", self._encoding, self
            ._errors, self._exhausted
        )
    }
}
fn _encoding_is_supported(label: &String) -> bool {
    _encoding_is_supported_impl(label)
}
fn _encoding_canonical_label(
    label: &String,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let value: String = _encoding_canonical_label_impl(label)?;
        Ok(Ok(value))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_text(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_text_impl(data, encoding, errors)?;
        Ok(Ok(text))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_recoveries(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let recoveries: Vec<String> = _encoding_decode_recoveries_impl(
            data,
            encoding,
            errors,
        )?;
        Ok(Ok(recoveries))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_outcome(
    data: &Vec<u8>,
    encoding: &String,
    errors: &String,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_text_impl(data, encoding, errors)?;
        let recoveries: Vec<String> = _encoding_decode_recoveries_impl(
            data,
            encoding,
            errors,
        )?;
        Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_incremental_outcome(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    errors: &String,
    r#final: bool,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        ParseError,
    > = (|| {
        let text: String = _encoding_decode_incremental_text_impl(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )?;
        let recoveries: Vec<String> = _encoding_decode_incremental_recoveries_impl(
            data,
            pending,
            encoding,
            errors,
            r#final,
        )?;
        Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_decode_incremental_pending(
    data: &Vec<u8>,
    pending: &Vec<u8>,
    encoding: &String,
    r#final: bool,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        ParseError,
    > = (|| {
        let next_pending: Vec<u8> = _encoding_decode_incremental_pending_impl(
            data,
            pending,
            encoding,
            r#final,
        )?;
        Ok(Ok(next_pending))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_bytes(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        ParseError,
    > = (|| {
        let data: Vec<u8> = _encoding_encode_bytes_impl(text, encoding, errors)?;
        Ok(Ok(data))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_recoveries(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<String>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        ParseError,
    > = (|| {
        let recoveries: Vec<String> = _encoding_encode_recoveries_impl(
            text,
            encoding,
            errors,
        )?;
        Ok(Ok(recoveries))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn _encoding_encode_outcome(
    text: &String,
    encoding: &String,
    errors: &String,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
> {
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        >,
        ParseError,
    > = (|| {
        let data: Vec<u8> = _encoding_encode_bytes_impl(text, encoding, errors)?;
        let recoveries: Vec<String> = _encoding_encode_recoveries_impl(
            text,
            encoding,
            errors,
        )?;
        Ok(Ok(__SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome::new(data, recoveries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn encoding(label: &String) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new((label.clone()).clone())
}
fn utf8() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8())
}
fn utf8_sig() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8_SIG())
}
fn ascii() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_ASCII())
}
fn latin1() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_LATIN1())
}
fn utf16_le() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_LE())
}
fn utf16_be() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_BE())
}
fn windows1252() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_WINDOWS_1252())
}
fn strict_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_STRICT(),
    )
}
fn replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_REPLACE(),
    )
}
fn ignore_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_IGNORE(),
    )
}
fn backslash_replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
        __const_DECODE_ERRORS_BACKSLASH_REPLACE(),
    )
}
fn strict_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_STRICT(),
    )
}
fn replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_REPLACE(),
    )
}
fn ignore_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_IGNORE(),
    )
}
fn backslash_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_BACKSLASH_REPLACE(),
    )
}
fn xmlcharref_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_XMLCHARREF_REPLACE(),
    )
}
fn name_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        __const_ENCODE_ERRORS_NAME_REPLACE(),
    )
}
fn _decode_handler_name(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> String {
    if let Some(errors) = errors.as_ref() {
        return {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((errors.name.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    __const_DECODE_ERRORS_STRICT()
}
fn _encode_handler_name(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> String {
    if let Some(errors) = errors.as_ref() {
        return {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((errors.name.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    __const_ENCODE_ERRORS_STRICT()
}
fn _decode_handler_or_strict(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        );
    }
    strict_decode_handler()
}
fn _encode_handler_or_strict(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        );
    }
    strict_encode_handler()
}
fn decode_outcome(
    data: &Vec<u8>,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
> {
    let handler_name: String = _decode_handler_name(errors);
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        >,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > = (|| { Ok(_encoding_decode_outcome(data, &enc.label.clone(), &handler_name)) })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn decode(
    data: &Vec<u8>,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
    let __sifr_try_res: Result<
        Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError>,
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    > = (|| {
        let outcome: __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome = decode_outcome(
            data,
            enc,
            errors,
        )?;
        Ok(Ok(outcome.get_text()))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message.clone()),
            );
        }
    }
}
fn encode_outcome(
    text: &String,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> Result<
    __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
    __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
> {
    let handler_name: String = _encode_handler_name(errors);
    let __sifr_try_res: Result<
        Result<
            __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome,
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        >,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > = (|| { Ok(_encoding_encode_outcome(text, &enc.label.clone(), &handler_name)) })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn encode(
    text: &String,
    enc: &__SifrStdlib_sifr_x2eencoding_x2eEncoding,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
) -> Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError> {
    let __sifr_try_res: Result<
        Result<Vec<u8>, __SifrStdlib_sifr_x2eencoding_x2eEncodeError>,
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    > = (|| {
        let outcome: __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome = encode_outcome(
            text,
            enc,
            errors,
        )?;
        Ok(Ok(outcome.get_data()))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message.clone()),
            );
        }
    }
}
fn run_command(cmd: &String) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn env_get(key: &String) -> Option<String> {
    ::sifr_stdlib::sys::env_get(key)
}
fn env_keys() -> Vec<String> {
    ::sifr_stdlib::sys::env_keys()
}
fn env_values() -> Vec<String> {
    ::sifr_stdlib::sys::env_values()
}
fn env_items() -> Vec<String> {
    ::sifr_stdlib::sys::env_items()
}
fn get_args() -> Vec<String> {
    ::sifr_stdlib::sys::get_args()
}
fn sys_exit(code: SifrInt) {
    ::sifr_stdlib::sys::sys_exit(::sifr_runtime::interop::SifrIntBridge::from(code));
}
fn sys_version() -> String {
    ::sifr_stdlib::sys::sys_version()
}
fn sys_platform() -> String {
    ::sifr_stdlib::sys::sys_platform()
}
fn sys_maxsize() -> SifrInt {
    ::sifr_stdlib::sys::sys_maxsize().into_sifr_int()
}
fn getpid() -> SifrInt {
    ::sifr_stdlib::sys::getpid().into_sifr_int()
}
fn cpu_count() -> SifrInt {
    ::sifr_stdlib::sys::cpu_count().into_sifr_int()
}
fn which(name: &String) -> Option<String> {
    ::sifr_stdlib::sys::which(name)
}
fn os_sep() -> String {
    ::sifr_stdlib::sys::os_sep()
}
fn os_linesep() -> String {
    ::sifr_stdlib::sys::os_linesep()
}
fn os_name() -> String {
    ::sifr_stdlib::sys::os_name()
}
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
fn join_path(base: &String, child: &String) -> String {
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_base.len()) == &SifrInt::from_i64(0)) {
        return {
            let mut __sifr_concat: String = String::with_capacity(child.len() + 0usize);
            __sifr_concat.push_str((child).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    let last: Option<String> = ({
        let __sifr_string_index = SifrInt::from(base.chars().count())
            - SifrInt::from_i64(1);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_base.len());
        __sifr_chars_base.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string());
    if let Some(last) = last {
        if (last).as_str() == ("/".to_string()).as_str() {
            return {
                let mut __sifr_concat: String = String::with_capacity(
                    base.len() + child.len(),
                );
                __sifr_concat.push_str((base).as_str());
                __sifr_concat.push_str((child).as_str());
                __sifr_concat
            };
        }
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            (base.len() + 1usize) + child.len(),
        );
        __sifr_concat.push_str((base).as_str());
        __sifr_concat.push('/');
        __sifr_concat.push_str((child).as_str());
        __sifr_concat
    }
}
fn basename(path: &String) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == "/") {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len = _slice_src.len();
                    let _slice_start = (&i + &SifrInt::from_i64(1))
                        .clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
        }
        i = &i - &SifrInt::from_i64(1);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
        __sifr_concat.push_str((path).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn dirname(path: &String) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == "/") {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len = _slice_src.len();
                    let _slice_start = 0;
                    let _slice_stop = i.clamp_slice_bound(_slice_len);
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
        }
        i = &i - &SifrInt::from_i64(1);
    }
    "".to_string()
}
fn extension(path: &String) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_path.len()) - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == ".") {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len = _slice_src.len();
                    let _slice_start = i.clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
            if (ch == "/") {
                return "".to_string();
            }
        }
        i = &i - &SifrInt::from_i64(1);
    }
    "".to_string()
}
fn stem(path: &String) -> String {
    let base: String = basename(path);
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    let mut i: SifrInt = &SifrInt::from(__sifr_chars_base.len()) - &SifrInt::from_i64(1);
    while (&i > &SifrInt::from_i64(0)) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_base.len());
            __sifr_chars_base.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch == ".") {
                return {
                    let _slice_src = &__sifr_chars_base;
                    let _slice_len = _slice_src.len();
                    let _slice_start = 0;
                    let _slice_stop = i.clamp_slice_bound(_slice_len);
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .copied(),
                    )
                };
            }
        }
        i = &i - &SifrInt::from_i64(1);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 0usize);
        __sifr_concat.push_str((base).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn is_absolute(path: &String) -> bool {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_path.len()) == &SifrInt::from_i64(0)) {
        return false;
    }
    if (&SifrInt::from(__sifr_chars_path.len()) >= &SifrInt::from_i64(3)) {
        let colon: Option<String> = ({
            let __sifr_string_index = SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let sep: Option<String> = ({
            let __sifr_string_index = SifrInt::from_i64(2);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_path.len());
            __sifr_chars_path.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(colon) = colon {
            if let Some(sep) = sep {
                if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = ({
        let __sifr_string_index = SifrInt::from_i64(0);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_path.len());
        __sifr_chars_path.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string());
    if let Some(first) = first {
        if (first == "/") || (first == "\\") {
            return true;
        }
    }
    false
}
fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<String>| {
            let mut i: SifrInt = SifrInt::from_i64(0);
            while (&i < &SifrInt::from(entries.len())) {
                let Some(__sifr_checked_value_7) = ({
                    let __sifr_checked_read_collection = &entries;
                    let __sifr_checked_read_index = i.clone();
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                }) else {
                    break;
                };
                __sifr_yielder.suspend(__sifr_checked_value_7.clone()).await;
                i = &i + &SifrInt::from_i64(1);
            }
        }),
    )
}
fn _iterdir_list(path: &String) -> Result<Vec<String>, IOError> {
    iterdir(path)
}
fn _glob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    glob_pattern(path, pattern)
}
fn _rglob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    rglob_pattern(path, pattern)
}
fn _iterdir_to_iter(path: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _iterdir_list(path)?;
        Ok(Ok(_iter_list_str(entries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn _glob_to_iter(
    path: &String,
    pattern: &String,
) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _glob_list(path, pattern)?;
        Ok(Ok(_iter_list_str(entries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn _rglob_to_iter(
    path: &String,
    pattern: &String,
) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _rglob_list(path, pattern)?;
        Ok(Ok(_iter_list_str(entries)))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(e);
        }
    }
}
fn copy(src: &String, dst: &String) -> Result<(), IOError> {
    copy_file(src, dst)
}
fn move_file(src: &String, dst: &String) -> Result<(), IOError> {
    rename(src, dst)
}
fn rmtree(path: &String) -> Result<(), IOError> {
    rmdir_all(path)
}
fn random_int(min: SifrInt, max: SifrInt) -> SifrInt {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .into_sifr_int()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_word_to_unit_float(value: SifrInt) -> f64 {
    ::sifr_stdlib::random::random_word_to_unit_float(
        ::sifr_runtime::interop::SifrIntBridge::from(value),
    )
}
fn random_seed() -> SifrInt {
    ::sifr_stdlib::random::random_seed().into_sifr_int()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(
    start: SifrInt,
    stop: SifrInt,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<SifrInt> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn random_module_state_index() -> SifrInt {
    ::sifr_stdlib::random::random_module_state_index().into_sifr_int()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .cloned()
                .map(::sifr_runtime::interop::SifrIntBridge::from)
                .collect::<Vec<_>>(),
            ::sifr_runtime::interop::SifrIntBridge::from(index),
            gauss_next.map(|__sifr_bridge_item_0| __sifr_bridge_item_0),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode(s: &String) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &String,
    altchars: &String,
    wrapcol: SifrInt,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_encode_opts(
            s,
            altchars,
            ::sifr_runtime::interop::SifrIntBridge::from(wrapcol),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_opts(
    s: &String,
    altchars: &String,
    validate: bool,
    ignorechars: &String,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &String) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &Vec<u8>) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &String) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &String) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &String) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &Vec<u8>) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}
fn _random_suffix() -> String {
    let n: SifrInt = random_int(SifrInt::from_i64(100000), SifrInt::from_i64(999999));
    format!("{}", n)
}
fn mktemp_path(prefix: &String) -> String {
    let suffix: String = _random_suffix();
    let mut root: String = gettempdir();
    let mut __sifr_chars_root: Vec<char> = root.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_root.len()) == &SifrInt::from_i64(0)) {
        root = "/tmp".to_string();
        __sifr_chars_root = root.chars().collect::<Vec<char>>();
    } else {
        let last: Option<String> = ({
            let __sifr_string_index = SifrInt::from(root.chars().count())
                - SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_root.len());
            __sifr_chars_root.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(last) = last {
            if (last == "/") {
                return {
                    let mut __sifr_concat: String = String::with_capacity(
                        (root.len() + prefix.len()) + suffix.len(),
                    );
                    __sifr_concat.push_str((root).as_str());
                    __sifr_concat.push_str((prefix).as_str());
                    __sifr_concat.push_str((suffix).as_str());
                    __sifr_concat
                };
            }
        }
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            ((root.len() + 1usize) + prefix.len()) + suffix.len(),
        );
        __sifr_concat.push_str((root).as_str());
        __sifr_concat.push('/');
        __sifr_concat.push_str((prefix).as_str());
        __sifr_concat.push_str((suffix).as_str());
        __sifr_concat
    }
}
fn _next_candidate(prefix: &String) -> String {
    mktemp_path(prefix)
}
fn _collision_message(kind: &String, attempts: SifrInt) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(
            (((9usize + kind.len()) + 37usize) + 0usize) + 9usize,
        );
        __sifr_concat.push_str("tempfile.");
        __sifr_concat.push_str((kind).as_str());
        __sifr_concat.push_str(": failed to create unique path after ");
        __sifr_concat.push_str((format!("{}", attempts)).as_str());
        __sifr_concat.push_str(" attempts");
        __sifr_concat
    }
}
fn mkstemp(prefix: &String) -> Result<String, IOError> {
    let mut attempts: SifrInt = SifrInt::from_i64(0);
    let max_attempts: SifrInt = SifrInt::from_i64(64);
    while (&attempts < &max_attempts) {
        let path: String = _next_candidate(prefix);
        let path_for_check: String = {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        if exists(&path) {
            attempts = &attempts + &SifrInt::from_i64(1);
            continue;
        }
        let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
            let wrt: () = write_text(&path, &"".to_string())?;
            Ok(Ok(path))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                if exists(&path_for_check) {
                    attempts = &attempts + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(IOError::new(_collision_message(&"mkstemp".to_string(), (max_attempts).clone())))
}
fn mkdtemp(prefix: &String) -> Result<String, IOError> {
    let mut attempts: SifrInt = SifrInt::from_i64(0);
    let max_attempts: SifrInt = SifrInt::from_i64(64);
    while (&attempts < &max_attempts) {
        let path: String = _next_candidate(prefix);
        let path_for_check: String = {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
        if exists(&path) {
            attempts = &attempts + &SifrInt::from_i64(1);
            continue;
        }
        let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
            let md: () = mkdir(&path)?;
            Ok(Ok(path))
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                if exists(&path_for_check) {
                    attempts = &attempts + &SifrInt::from_i64(1);
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(IOError::new(_collision_message(&"mkdtemp".to_string(), (max_attempts).clone())))
}
fn _zip_read_only_error() -> String {
    "zipfile operation requires write or append mode".to_string()
}
fn _zip_open_mode_error(mode: &String) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(48usize + mode.len());
        __sifr_concat.push_str("zipfile open supports read-only mode only, got: ");
        __sifr_concat.push_str((mode).as_str());
        __sifr_concat
    }
}
fn _closed_stream_error() -> String {
    "I/O operation on closed stream".to_string()
}
fn _zip_unimplemented_error(feature: &String) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(
            (8usize + feature.len()) + 49usize,
        );
        __sifr_concat.push_str("zipfile ");
        __sifr_concat.push_str((feature).as_str());
        __sifr_concat.push_str(" is not implemented in this compatibility surface");
        __sifr_concat
    }
}
fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
        let __sifr_io_kind = (&e as &dyn ::std::any::Any)
            .downcast_ref::<std::io::Error>()
            .map(::std::io::Error::kind);
        match __sifr_io_kind {
            Some(::std::io::ErrorKind::NotFound) => "FileNotFound".to_string(),
            Some(::std::io::ErrorKind::PermissionDenied) => {
                "PermissionDenied".to_string()
            }
            Some(::std::io::ErrorKind::AlreadyExists) => "FileExists".to_string(),
            Some(::std::io::ErrorKind::IsADirectory) => "IsADirectory".to_string(),
            Some(::std::io::ErrorKind::NotADirectory) => "NotADirectory".to_string(),
            Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
                "DirectoryNotEmpty".to_string()
            }
            _ => "Other".to_string(),
        }
    };
    IOError { message: msg, kind }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Error {
    message: String,
}
impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for Error {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for Error {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParseError {
    message: String,
}
impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ParseError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ValueError {
    message: String,
}
impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for ValueError {}
impl From<IOError> for Error {
    fn from(err: IOError) -> Self {
        Self::new(err.message)
    }
}
impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Self::new(err.message)
    }
}
impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self {
        Self::new(err.message)
    }
}
fn main() {
    let base: String = {
        let mut __sifr_concat: String = String::with_capacity(42usize + 0usize);
        __sifr_concat.push_str("/tmp/sifr_filesystem_archive_surface_demo_");
        __sifr_concat.push_str((format!("{}", getpid())).as_str());
        __sifr_concat
    };
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _mk: String = run_command(&format!("{}{}", "mkdir -p ", base))?;
        let source: String = {
            let mut __sifr_concat: String = String::with_capacity(base.len() + 9usize);
            __sifr_concat.push_str((base).as_str());
            __sifr_concat.push_str("/note.txt");
            __sifr_concat
        };
        let _w: () = write_text(&source, &"hello d1".to_string())?;
        let __sifr_try_res: Result<(), IOError> = (|| {
            let note_content: String = read_text(&source)?;
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(15usize +
                note_content.len()); __sifr_concat.push_str("io.read_text = ");
                __sifr_concat.push_str((note_content).as_str()); __sifr_concat }
            );
            Ok(())
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(20usize +
                0usize); __sifr_concat.push_str("io.read_text error: "); __sifr_concat
                .push_str((e.message.clone()).as_str()); __sifr_concat }
            );
        }
        let note_path: __SifrStdlib_sifr_x2epathlib_x2ePath = __SifrStdlib_sifr_x2epathlib_x2ePath::new(
            format!("{}{}", source, ""),
        );
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(15usize +
            0usize); __sifr_concat.push_str("pathlib.stem = "); __sifr_concat
            .push_str((note_path.stem()).as_str()); __sifr_concat }
        );
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(16usize +
            0usize); __sifr_concat.push_str("glob(\"*.txt\") = "); __sifr_concat
            .push_str((format!("{:?}", glob(& base, & "*.txt".to_string()))).as_str());
            __sifr_concat }
        );
        let copied: String = {
            let mut __sifr_concat: String = String::with_capacity(base.len() + 11usize);
            __sifr_concat.push_str((base).as_str());
            __sifr_concat.push_str("/copied.txt");
            __sifr_concat
        };
        let moved: String = {
            let mut __sifr_concat: String = String::with_capacity(base.len() + 10usize);
            __sifr_concat.push_str((base).as_str());
            __sifr_concat.push_str("/moved.txt");
            __sifr_concat
        };
        let _cp: () = copy(&source, &copied)?;
        let _mv: () = move_file(&copied, &moved)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(26usize +
            0usize); __sifr_concat.push_str("shutil.move_file exists = "); __sifr_concat
            .push_str((format!("{}", __SifrStdlib_sifr_x2epathlib_x2ePath::new(moved)
            .exists())).as_str()); __sifr_concat }
        );
        let temp_file: String = mkstemp(
            &"sifr_filesystem_archive_surface_demo_".to_string(),
        )?;
        let temp_dir: String = mkdtemp(
            &"sifr_filesystem_archive_surface_demo_".to_string(),
        )?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
            temp_file.len()); __sifr_concat.push_str("tempfile.mkstemp = ");
            __sifr_concat.push_str((temp_file).as_str()); __sifr_concat }
        );
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
            temp_dir.len()); __sifr_concat.push_str("tempfile.mkdtemp = "); __sifr_concat
            .push_str((temp_dir).as_str()); __sifr_concat }
        );
        let compressed: Vec<u8> = compress(&"archive sample".to_string());
        let __sifr_try_res: Result<(), IOError> = (|| {
            let restored: String = decompress(&compressed)?;
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
                restored.len()); __sifr_concat.push_str("gzip roundtrip = ");
                __sifr_concat.push_str((restored).as_str()); __sifr_concat }
            );
            Ok(())
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(12usize +
                0usize); __sifr_concat.push_str("gzip error: "); __sifr_concat
                .push_str((e.message.clone()).as_str()); __sifr_concat }
            );
        }
        let zip_path: String = {
            let mut __sifr_concat: String = String::with_capacity(base.len() + 9usize);
            __sifr_concat.push_str((base).as_str());
            __sifr_concat.push_str("/demo.zip");
            __sifr_concat
        };
        let archive: __SifrStdlib_sifr_x2ezipfile_x2eZipFile = __SifrStdlib_sifr_x2ezipfile_x2eZipFile::new(
            zip_path,
            "a".to_string(),
            SifrInt::from_i64(0),
        );
        let __sifr_try_res: Result<(), IOError> = (|| {
            let _zc: () = archive.create()?;
            let _zw: () = archive
                .write(&"inside.txt".to_string(), &"inside-zip".to_string())?;
            let inside: String = archive.read(&"inside.txt".to_string())?;
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(15usize +
                inside.len()); __sifr_concat.push_str("zipfile.read = "); __sifr_concat
                .push_str((inside).as_str()); __sifr_concat }
            );
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
                0usize); __sifr_concat.push_str("zipfile.namelist = "); __sifr_concat
                .push_str((format!("{:?}", archive.namelist())).as_str()); __sifr_concat
                }
            );
            Ok(())
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            println!(
                "{}", { let mut __sifr_concat : String = String::with_capacity(15usize +
                0usize); __sifr_concat.push_str("zipfile error: "); __sifr_concat
                .push_str((e.message.clone()).as_str()); __sifr_concat }
            );
        }
        let _rm_temp_file: String = run_command(&format!("{}{}", "rm -f ", temp_file))?;
        let _rm_temp_dir: () = rmtree(&temp_dir)?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(39usize +
            0usize); __sifr_concat.push_str("filesystem_archive_surface demo error: ");
            __sifr_concat.push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _cleanup: String = run_command(&format!("{}{}", "rm -rf ", base))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(42usize +
            0usize); __sifr_concat
            .push_str("filesystem_archive_surface cleanup error: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
}
