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
    pub type __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = ::sifr_runtime::interop::Handle<
        ::sifr_stdlib::regex::CompiledPattern,
    >;
    pub trait __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods {
        fn search(&self, text: &String) -> Result<Option<String>, RegexError>;
        fn is_match(&self, text: &String) -> Result<bool, RegexError>;
        fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError>;
        fn findall(&self, text: &String) -> Result<Vec<String>, RegexError>;
        fn split(&self, text: &String) -> Result<Vec<String>, RegexError>;
        fn pattern(&self) -> Result<String, RegexError>;
        fn flags(&self) -> Result<i64, RegexError>;
    }
    impl __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods
    for __SifrStdlib___sifr_x2eregex_x2eCompiledPattern {
        fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_search(self, text)
                .map(|__sifr_bridge_ok| __sifr_bridge_ok)
                .map_err(|__sifr_bridge_error| RegexError {
                    message: __sifr_bridge_error.to_string(),
                    detail: __sifr_bridge_error.to_string(),
                })
        }
        fn is_match(&self, text: &String) -> Result<bool, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_is_match(self, text)
                .map(|__sifr_bridge_ok| __sifr_bridge_ok)
                .map_err(|__sifr_bridge_error| RegexError {
                    message: __sifr_bridge_error.to_string(),
                    detail: __sifr_bridge_error.to_string(),
                })
        }
        fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_replace(self, replacement, text)
                .map(|__sifr_bridge_ok| __sifr_bridge_ok)
                .map_err(|__sifr_bridge_error| RegexError {
                    message: __sifr_bridge_error.to_string(),
                    detail: __sifr_bridge_error.to_string(),
                })
        }
        fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_findall(self, text)
                .map(|__sifr_bridge_ok| __sifr_bridge_ok)
                .map_err(|__sifr_bridge_error| RegexError {
                    message: __sifr_bridge_error.to_string(),
                    detail: __sifr_bridge_error.to_string(),
                })
        }
        fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_split(self, text)
                .map(|__sifr_bridge_ok| __sifr_bridge_ok)
                .map_err(|__sifr_bridge_error| RegexError {
                    message: __sifr_bridge_error.to_string(),
                    detail: __sifr_bridge_error.to_string(),
                })
        }
        fn pattern(&self) -> Result<String, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_source(self)
                .map(|__sifr_bridge_ok| __sifr_bridge_ok)
                .map_err(|__sifr_bridge_error| RegexError {
                    message: __sifr_bridge_error.to_string(),
                    detail: __sifr_bridge_error.to_string(),
                })
        }
        fn flags(&self) -> Result<i64, RegexError> {
            ::sifr_stdlib::regex::compiled_pattern_flags(self)
                .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
                .map_err(|__sifr_bridge_error| RegexError {
                    message: __sifr_bridge_error.to_string(),
                    detail: __sifr_bridge_error.to_string(),
                })
        }
    }
    pub fn compile_pattern(
        pattern: &String,
    ) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
        ::sifr_stdlib::regex::compile_pattern(pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn compile_pattern_flags(
        pattern: &String,
        flags: i64,
    ) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
        ::sifr_stdlib::regex::compile_pattern_flags(
                pattern,
                ::sifr_runtime::interop::SifrIntBridge::from(flags),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_match(pattern: &String, text: &String) -> Result<bool, RegexError> {
        ::sifr_stdlib::regex::re_match(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_find(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::re_find(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_replace(
        pattern: &String,
        replacement: &String,
        text: &String,
    ) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::re_replace(pattern, replacement, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_findall(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_split(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_find_start(pattern: &String, text: &String) -> Result<i64, RegexError> {
        ::sifr_stdlib::regex::re_find_start(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_find_end(pattern: &String, text: &String) -> Result<i64, RegexError> {
        ::sifr_stdlib::regex::re_find_end(pattern, text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_match_flags(
        pattern: &String,
        text: &String,
        flags: i64,
    ) -> Result<bool, RegexError> {
        ::sifr_stdlib::regex::re_match_flags(
                pattern,
                text,
                ::sifr_runtime::interop::SifrIntBridge::from(flags),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_find_flags(
        pattern: &String,
        text: &String,
        flags: i64,
    ) -> Result<Option<String>, RegexError> {
        ::sifr_stdlib::regex::re_find_flags(
                pattern,
                text,
                ::sifr_runtime::interop::SifrIntBridge::from(flags),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_replace_flags(
        pattern: &String,
        replacement: &String,
        text: &String,
        flags: i64,
    ) -> Result<String, RegexError> {
        ::sifr_stdlib::regex::re_replace_flags(
                pattern,
                replacement,
                text,
                ::sifr_runtime::interop::SifrIntBridge::from(flags),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_findall_flags(
        pattern: &String,
        text: &String,
        flags: i64,
    ) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_findall_flags(
                pattern,
                text,
                ::sifr_runtime::interop::SifrIntBridge::from(flags),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
    pub fn re_split_flags(
        pattern: &String,
        text: &String,
        flags: i64,
    ) -> Result<Vec<String>, RegexError> {
        ::sifr_stdlib::regex::re_split_flags(
                pattern,
                text,
                ::sifr_runtime::interop::SifrIntBridge::from(flags),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| RegexError {
                message: __sifr_bridge_error.to_string(),
                detail: __sifr_bridge_error.to_string(),
            })
    }
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
    pub fn _file_read_bytes(handle: &String) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::fs::file_read_bytes(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn open_file(
        path: &String,
        mode: &String,
    ) -> Result<__SifrIoNativeFileHandle, IOError> {
        let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
            let handle_id: String = _open_file(path, mode)?;
            return Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
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
    pub fn file_read_bytes(handle: &__SifrIoNativeFileHandle) -> Result<Vec<u8>, IOError> {
        _file_read_bytes(&handle._id.clone())
    }
    pub fn file_write_bytes(
        handle: &__SifrIoNativeFileHandle,
        data: &Vec<u8>,
    ) -> Result<(), IOError> {
        _file_write_bytes(&handle._id.clone(), data)
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
    pub fn stat_size(path: &String) -> Result<i64, IOError> {
        ::sifr_stdlib::fs::stat_size(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn disk_usage(path: &String) -> Vec<i64> {
        ::sifr_stdlib::fs::disk_usage(path)
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
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
            if parent == "" {
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
            if parent == "" {
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
        if ((__sifr_chars_base.len() as i64) == (0_i64)) {
            return {
                let mut __sifr_concat: String = String::with_capacity(child.len() + 0usize);
                __sifr_concat.push_str((child).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        let last: Option<String> = __sifr_chars_base
            .get(((base.chars().count() as i64) - (1_i64)) as usize)
            .map(|c| c.to_string());
        if let Some(last) = last {
            if last == "/" {
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
        let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
        while i >= (0_i64) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_path
                    .get(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "/" {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len_i64 = _slice_src.len() as i64;
                        let _slice_start_i64 = if (i + (1_i64)) < 0 {
                            (_slice_len_i64 + (i + (1_i64))).max(0)
                        } else {
                            (i + (1_i64)).min(_slice_len_i64)
                        };
                        let _slice_stop_i64 = _slice_len_i64;
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start_i64 as usize)
                                .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                                .copied(),
                        )
                    };
                }
            }
            i -= 1_i64;
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
        let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
        while i >= (0_i64) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_path
                    .get(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "/" {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len_i64 = _slice_src.len() as i64;
                        let _slice_start_i64 = 0;
                        let _slice_stop_i64 = if i < 0 {
                            (_slice_len_i64 + i).max(0)
                        } else {
                            i.min(_slice_len_i64)
                        };
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start_i64 as usize)
                                .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                                .copied(),
                        )
                    };
                }
            }
            i -= 1_i64;
        }
        "".to_string()
    }
    pub fn extension(path: &String) -> String {
        let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
        let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
        while i >= (0_i64) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_path
                    .get(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "." {
                    return {
                        let _slice_src = &__sifr_chars_path;
                        let _slice_len_i64 = _slice_src.len() as i64;
                        let _slice_start_i64 = if i < 0 {
                            (_slice_len_i64 + i).max(0)
                        } else {
                            i.min(_slice_len_i64)
                        };
                        let _slice_stop_i64 = _slice_len_i64;
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start_i64 as usize)
                                .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                                .copied(),
                        )
                    };
                }
                if ch == "/" {
                    return "".to_string();
                }
            }
            i -= 1_i64;
        }
        "".to_string()
    }
    pub fn stem(path: &String) -> String {
        let base: String = basename(path);
        let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
        let mut i: i64 = (__sifr_chars_base.len() as i64) - (1_i64);
        while i > (0_i64) {
            let ch: Option<String> = Some({
                let Some(__indexed_char) = __sifr_chars_base
                    .get(i as usize)
                    .map(|c| c.to_string()) else {
                    unreachable!("compiler-verified string index should be in range");
                };
                __indexed_char
            });
            if let Some(ch) = ch {
                if ch == "." {
                    return {
                        let _slice_src = &__sifr_chars_base;
                        let _slice_len_i64 = _slice_src.len() as i64;
                        let _slice_start_i64 = 0;
                        let _slice_stop_i64 = if i < 0 {
                            (_slice_len_i64 + i).max(0)
                        } else {
                            i.min(_slice_len_i64)
                        };
                        String::from_iter(
                            _slice_src
                                .iter()
                                .skip(_slice_start_i64 as usize)
                                .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                                .copied(),
                        )
                    };
                }
            }
            i -= 1_i64;
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
        if ((__sifr_chars_path.len() as i64) == (0_i64)) {
            return false;
        }
        if ((__sifr_chars_path.len() as i64) >= (3_i64)) {
            let colon: Option<String> = __sifr_chars_path
                .get((1_i64) as usize)
                .map(|c| c.to_string());
            let sep: Option<String> = __sifr_chars_path
                .get((2_i64) as usize)
                .map(|c| c.to_string());
            if let Some(colon) = colon {
                if let Some(sep) = sep {
                    if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                        return true;
                    }
                }
            }
        }
        let first: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get((0_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(first) = first {
            if (first == "/") || (first == "\\") {
                return true;
            }
        }
        false
    }
    pub fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
        let mut __sifr_generator_initialized: bool = false;
        let mut __sifr_generator_iter: ::std::vec::IntoIter<String> = Vec::new().into_iter();
        Box::new(
            ::std::iter::from_fn(move || {
                if !__sifr_generator_initialized {
                    let mut _yields: Vec<String> = Vec::new();
                    let mut i: i64 = 0_i64;
                    while (i < (entries.len() as i64)) {
                        _yields.push(entries[i as usize].clone());
                        i += 1_i64;
                    }
                    __sifr_generator_iter = _yields.into_iter();
                    __sifr_generator_initialized = true;
                }
                __sifr_generator_iter.next()
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
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
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
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
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
            return Ok(Ok(_iter_list_str(entries)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2ere_x2eMatch {
        pub _matched: String,
        pub _start: i64,
        pub _end: i64,
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn new(matched: String, start: i64, end: i64) -> Self {
            let __sifr_field_init_0: String = matched;
            let __sifr_field_init_1: i64 = start;
            let __sifr_field_init_2: i64 = end;
            Self {
                _matched: __sifr_field_init_0,
                _start: __sifr_field_init_1,
                _end: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn group(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._matched.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn start(&self) -> i64 {
            self._start
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn end(&self) -> i64 {
            self._end
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn span(&self) -> Vec<i64> {
            let result: Vec<i64> = vec![self._start, self._end];
            result
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2eMatch {
        pub fn to_str(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._matched.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2ere_x2eMatch {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "Match(_matched={}, _start={}, _end={})", self._matched, self._start, self
                ._end
            )
        }
    }
    pub struct __SifrStdlib_sifr_x2ere_x2ePattern {
        pub _compiled: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern,
        pub _pattern: String,
        pub _flags: i64,
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn new(
            compiled: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern,
            pattern: String,
            flags: i64,
        ) -> Self {
            let __sifr_field_init_0: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = compiled;
            let __sifr_field_init_1: String = pattern;
            let __sifr_field_init_2: i64 = flags;
            Self {
                _compiled: __sifr_field_init_0,
                _pattern: __sifr_field_init_1,
                _flags: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
            self._compiled.search(text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn is_match(&self, text: &String) -> Result<bool, RegexError> {
            self._compiled.is_match(text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn sub(
            &self,
            replacement: &String,
            text: &String,
        ) -> Result<String, RegexError> {
            self._compiled.sub(replacement, text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
            self._compiled.findall(text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn finditer(
            &self,
            text: &String,
        ) -> Result<Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>>, RegexError> {
            let __sifr_try_res: Result<
                Result<
                    Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>>,
                    RegexError,
                >,
                RegexError,
            > = (|| {
                let found_items: Vec<String> = self._compiled.findall(text)?;
                let matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = _finditer_from_items(
                    &found_items,
                    text,
                );
                return Ok(Ok(_iter_matches(matches)));
                unreachable!("sifr try/except return capture fell through");
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let e = __sifr_try_err.clone();
                    return Err(RegexError::new(e.message.clone()));
                }
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
            self._compiled.split(text)
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn pattern(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self._pattern.clone()).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2ere_x2ePattern {
        pub fn flags(&self) -> i64 {
            self._flags
        }
    }
    pub fn _iter_matches(
        matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch>,
    ) -> Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>> {
        let mut __sifr_generator_initialized: bool = false;
        let mut __sifr_generator_iter: ::std::vec::IntoIter<
            __SifrStdlib_sifr_x2ere_x2eMatch,
        > = Vec::new().into_iter();
        Box::new(
            ::std::iter::from_fn(move || {
                if !__sifr_generator_initialized {
                    let mut _yields: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = Vec::new();
                    let mut i: i64 = 0_i64;
                    while (i < (matches.len() as i64)) {
                        _yields.push(matches[i as usize].clone());
                        i += 1_i64;
                    }
                    __sifr_generator_iter = _yields.into_iter();
                    __sifr_generator_initialized = true;
                }
                __sifr_generator_iter.next()
            }),
        )
    }
    pub fn _find_index_from(text: &String, needle: &String, start: i64) -> i64 {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        let __sifr_chars_needle: Vec<char> = needle.chars().collect::<Vec<char>>();
        if start < (0_i64) {
            return -(1_i64);
        }
        if ((__sifr_chars_needle.len() as i64) == (0_i64)) {
            if (start <= (__sifr_chars_text.len() as i64)) {
                return start;
            }
            return -(1_i64);
        }
        let max_start: i64 = (__sifr_chars_text.len() as i64)
            - (__sifr_chars_needle.len() as i64);
        let mut i: i64 = start;
        while i <= max_start {
            if (({
                let _slice_src = &__sifr_chars_text;
                let _slice_len_i64 = _slice_src.len() as i64;
                let _slice_start_i64 = if i < 0 {
                    (_slice_len_i64 + i).max(0)
                } else {
                    i.min(_slice_len_i64)
                };
                let _slice_stop_i64 = if (i + (__sifr_chars_needle.len() as i64)) < 0 {
                    (_slice_len_i64 + (i + (__sifr_chars_needle.len() as i64))).max(0)
                } else {
                    (i + (__sifr_chars_needle.len() as i64)).min(_slice_len_i64)
                };
                String::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                        .copied(),
                )
            }) == needle.clone())
            {
                return i;
            }
            i += 1_i64;
        }
        -(1_i64)
    }
    pub fn _finditer_from_items(
        found_items: &Vec<String>,
        text: &String,
    ) -> Vec<__SifrStdlib_sifr_x2ere_x2eMatch> {
        let mut matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = vec![];
        let mut cursor: i64 = 0_i64;
        for found in found_items.iter().cloned() {
            let __sifr_chars_found: Vec<char> = found.chars().collect::<Vec<char>>();
            let mut start: i64 = _find_index_from(text, &found, cursor);
            if start < (0_i64) {
                start = cursor;
            }
            let found_len: i64 = __sifr_chars_found.len() as i64;
            let end: i64 = start + found_len;
            matches.push(__SifrStdlib_sifr_x2ere_x2eMatch::new(found, start, end));
            if found_len == (0_i64) {
                cursor = end + (1_i64);
            } else {
                cursor = end;
            }
        }
        matches
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct RegexError {
        pub message: String,
        pub detail: String,
    }
    impl RegexError {
        pub fn new(message: String) -> Self {
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
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::RegexError;
pub use __sifr_project_nominals::__SifrStdlib___sifr_x2eregex_x2eCompiledPattern;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2epathlib_x2ePath;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ere_x2eMatch;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ere_x2ePattern;
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
fn _file_read_bytes(handle: &String) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::fs::file_read_bytes(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write_bytes(handle: &String, data: &Vec<u8>) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write_bytes(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn open_file(path: &String, mode: &String) -> Result<__SifrIoNativeFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoNativeFileHandle, IOError>, IOError> = (|| {
        let handle_id: String = _open_file(path, mode)?;
        return Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message.clone()));
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
fn file_read_bytes(handle: &__SifrIoNativeFileHandle) -> Result<Vec<u8>, IOError> {
    _file_read_bytes(&handle._id.clone())
}
fn file_write_bytes(
    handle: &__SifrIoNativeFileHandle,
    data: &Vec<u8>,
) -> Result<(), IOError> {
    _file_write_bytes(&handle._id.clone(), data)
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
fn stat_size(path: &String) -> Result<i64, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn disk_usage(path: &String) -> Vec<i64> {
    ::sifr_stdlib::fs::disk_usage(path)
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
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
    _match(name, 0_i64, pattern, 0_i64)
}
fn _match(name: &String, mut ni: i64, pattern: &String, mut pi: i64) -> bool {
    while (pi < (pattern.chars().count() as i64)) {
        let pc: Option<String> = Some({
            let Some(__indexed_char) = pattern
                .chars()
                .nth(pi as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(pc) = pc {
            if pc == "*" {
                pi += 1_i64;
                if (pi == (pattern.chars().count() as i64)) {
                    return true;
                }
                let mut j: i64 = ni;
                while (j <= (name.chars().count() as i64)) {
                    if _match(name, j, pattern, pi) {
                        return true;
                    }
                    j += 1_i64;
                }
                return false;
            } else {
                if pc == "?" {
                    if (ni >= (name.chars().count() as i64)) {
                        return false;
                    }
                    ni += 1_i64;
                    pi += 1_i64;
                } else {
                    if (ni >= (name.chars().count() as i64)) {
                        return false;
                    }
                    let nc: Option<String> = Some({
                        let Some(__indexed_char) = name
                            .chars()
                            .nth(ni as usize)
                            .map(|c| c.to_string()) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char
                    });
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni += 1_i64;
                    pi += 1_i64;
                }
            }
        } else {
            return false;
        }
    }
    (ni == (name.chars().count() as i64))
}
fn filterfalse(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if !(fnmatch(&name, pattern)) {
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
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_pattern.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_pattern
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "*" {
                body.push_str(".*");
            } else {
                if ch == "?" {
                    body.push('.');
                } else {
                    body.push_str((_translate_literal(&ch)).as_str());
                }
            }
        }
        i += 1_i64;
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
    let include_hidden: bool = ((((__sifr_chars_pattern.len() as i64) > (0_i64)))
        && ((({
            let Some(__indexed_char) = __sifr_chars_pattern
                .get((0_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) == ".")));
    let mut matches: Vec<String> = vec![];
    let __sifr_try_res: Result<(), IOError> = (|| {
        let entries: Vec<String> = listdir(directory)?;
        for entry in entries.iter().cloned() {
            let __sifr_chars_entry: Vec<char> = entry.chars().collect::<Vec<char>>();
            if ((__sifr_chars_entry.len() as i64) == (0_i64)) {
                continue;
            }
            if !include_hidden
                && (({
                    let Some(__indexed_char) = __sifr_chars_entry
                        .get((0_i64) as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char
                }) == ".")
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
fn iglob(directory: &String, pattern: &String) -> Box<dyn Iterator<Item = String>> {
    let directory = directory.clone();
    let pattern = pattern.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<String> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<String> = Vec::new();
                let matches: Vec<String> = glob(&directory, &pattern);
                let mut i: i64 = 0_i64;
                while (i < (matches.len() as i64)) {
                    _yields.push(matches[i as usize].clone());
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
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
        let __sifr_field_init_3: Vec<u8> = vec![];
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
                self._pending = vec![];
                self._exhausted = true;
            }
            return Ok(Ok(outcome));
            unreachable!("sifr try/except return capture fell through");
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
            return Ok(Ok(outcome));
            unreachable!("sifr try/except return capture fell through");
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
        return Ok(Ok(value));
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(Ok(text));
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(Ok(recoveries));
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome::new(text, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(Ok(next_pending));
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(Ok(data));
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(Ok(recoveries));
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(
            Ok(__SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome::new(data, recoveries)),
        );
        unreachable!("sifr try/except return capture fell through");
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
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new((label).clone())
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
    > = (|| {
        return Ok(_encoding_decode_outcome(data, &enc.label.clone(), &handler_name));
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(Ok(outcome.get_text()));
        unreachable!("sifr try/except return capture fell through");
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
    > = (|| {
        return Ok(_encoding_encode_outcome(text, &enc.label.clone(), &handler_name));
        unreachable!("sifr try/except return capture fell through");
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
        return Ok(Ok(outcome.get_data()));
        unreachable!("sifr try/except return capture fell through");
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
fn sys_exit(code: i64) {
    ::sifr_stdlib::sys::sys_exit(::sifr_runtime::interop::SifrIntBridge::from(code));
}
fn sys_version() -> String {
    ::sifr_stdlib::sys::sys_version()
}
fn sys_platform() -> String {
    ::sifr_stdlib::sys::sys_platform()
}
fn sys_maxsize() -> i64 {
    ::sifr_stdlib::sys::sys_maxsize().to_i64_saturating()
}
fn getpid() -> i64 {
    ::sifr_stdlib::sys::getpid().to_i64_saturating()
}
fn cpu_count() -> i64 {
    ::sifr_stdlib::sys::cpu_count().to_i64_saturating()
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
fn join_path(base: &String, child: &String) -> String {
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    if ((__sifr_chars_base.len() as i64) == (0_i64)) {
        return {
            let mut __sifr_concat: String = String::with_capacity(child.len() + 0usize);
            __sifr_concat.push_str((child).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    let last: Option<String> = __sifr_chars_base
        .get(((base.chars().count() as i64) - (1_i64)) as usize)
        .map(|c| c.to_string());
    if let Some(last) = last {
        if last == "/" {
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
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "/" {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = if (i + (1_i64)) < 0 {
                        (_slice_len_i64 + (i + (1_i64))).max(0)
                    } else {
                        (i + (1_i64)).min(_slice_len_i64)
                    };
                    let _slice_stop_i64 = _slice_len_i64;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
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
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "/" {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = 0;
                    let _slice_stop_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
    }
    "".to_string()
}
fn extension(path: &String) -> String {
    let __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "." {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    let _slice_stop_i64 = _slice_len_i64;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
            if ch == "/" {
                return "".to_string();
            }
        }
        i -= 1_i64;
    }
    "".to_string()
}
fn stem(path: &String) -> String {
    let base: String = basename(path);
    let __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_base.len() as i64) - (1_i64);
    while i > (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_base
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch == "." {
                return {
                    let _slice_src = &__sifr_chars_base;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = 0;
                    let _slice_stop_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
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
    if ((__sifr_chars_path.len() as i64) == (0_i64)) {
        return false;
    }
    if ((__sifr_chars_path.len() as i64) >= (3_i64)) {
        let colon: Option<String> = __sifr_chars_path
            .get((1_i64) as usize)
            .map(|c| c.to_string());
        let sep: Option<String> = __sifr_chars_path
            .get((2_i64) as usize)
            .map(|c| c.to_string());
        if let Some(colon) = colon {
            if let Some(sep) = sep {
                if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = Some({
        let Some(__indexed_char) = __sifr_chars_path
            .get((0_i64) as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    });
    if let Some(first) = first {
        if (first == "/") || (first == "\\") {
            return true;
        }
    }
    false
}
fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<String> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<String> = Vec::new();
                let mut i: i64 = 0_i64;
                while (i < (entries.len() as i64)) {
                    _yields.push(entries[i as usize].clone());
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
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
        return Ok(Ok(_iter_list_str(entries)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message.clone()));
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
        return Ok(Ok(_iter_list_str(entries)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message.clone()));
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
        return Ok(Ok(_iter_list_str(entries)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message.clone()));
        }
    }
}
trait __SifrOpaque__SifrStdlib___sifr_x2eregex_x2eCompiledPatternMethods {
    fn search(&self, text: &String) -> Result<Option<String>, RegexError>;
    fn is_match(&self, text: &String) -> Result<bool, RegexError>;
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError>;
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError>;
    fn pattern(&self) -> Result<String, RegexError>;
    fn flags(&self) -> Result<i64, RegexError>;
}
fn compile_pattern(
    pattern: &String,
) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
    ::sifr_stdlib::regex::compile_pattern(pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn compile_pattern_flags(
    pattern: &String,
    flags: i64,
) -> Result<__SifrStdlib___sifr_x2eregex_x2eCompiledPattern, RegexError> {
    ::sifr_stdlib::regex::compile_pattern_flags(
            pattern,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match(pattern: &String, text: &String) -> Result<bool, RegexError> {
    ::sifr_stdlib::regex::re_match(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find(pattern: &String, text: &String) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_replace(
    pattern: &String,
    replacement: &String,
    text: &String,
) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace(pattern, replacement, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_findall(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_split(pattern: &String, text: &String) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_start(pattern: &String, text: &String) -> Result<i64, RegexError> {
    ::sifr_stdlib::regex::re_find_start(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_end(pattern: &String, text: &String) -> Result<i64, RegexError> {
    ::sifr_stdlib::regex::re_find_end(pattern, text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_match_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<bool, RegexError> {
    ::sifr_stdlib::regex::re_match_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_find_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Option<String>, RegexError> {
    ::sifr_stdlib::regex::re_find_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_replace_flags(
    pattern: &String,
    replacement: &String,
    text: &String,
    flags: i64,
) -> Result<String, RegexError> {
    ::sifr_stdlib::regex::re_replace_flags(
            pattern,
            replacement,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_findall_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_findall_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn re_split_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<String>, RegexError> {
    ::sifr_stdlib::regex::re_split_flags(
            pattern,
            text,
            ::sifr_runtime::interop::SifrIntBridge::from(flags),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| RegexError {
            message: __sifr_bridge_error.to_string(),
            detail: __sifr_bridge_error.to_string(),
        })
}
fn _iter_matches(
    matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch>,
) -> Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<
        __SifrStdlib_sifr_x2ere_x2eMatch,
    > = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = Vec::new();
                let mut i: i64 = 0_i64;
                while (i < (matches.len() as i64)) {
                    _yields.push(matches[i as usize].clone());
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn _find_index_from(text: &String, needle: &String, start: i64) -> i64 {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let __sifr_chars_needle: Vec<char> = needle.chars().collect::<Vec<char>>();
    if start < (0_i64) {
        return -(1_i64);
    }
    if ((__sifr_chars_needle.len() as i64) == (0_i64)) {
        if (start <= (__sifr_chars_text.len() as i64)) {
            return start;
        }
        return -(1_i64);
    }
    let max_start: i64 = (__sifr_chars_text.len() as i64)
        - (__sifr_chars_needle.len() as i64);
    let mut i: i64 = start;
    while i <= max_start {
        if (({
            let _slice_src = &__sifr_chars_text;
            let _slice_len_i64 = _slice_src.len() as i64;
            let _slice_start_i64 = if i < 0 {
                (_slice_len_i64 + i).max(0)
            } else {
                i.min(_slice_len_i64)
            };
            let _slice_stop_i64 = if (i + (__sifr_chars_needle.len() as i64)) < 0 {
                (_slice_len_i64 + (i + (__sifr_chars_needle.len() as i64))).max(0)
            } else {
                (i + (__sifr_chars_needle.len() as i64)).min(_slice_len_i64)
            };
            String::from_iter(
                _slice_src
                    .iter()
                    .skip(_slice_start_i64 as usize)
                    .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                    .copied(),
            )
        }) == needle.clone())
        {
            return i;
        }
        i += 1_i64;
    }
    -(1_i64)
}
fn _findall_for_finditer(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<String>, RegexError> {
    if flags != (0_i64) {
        return re_findall_flags(pattern, text, flags);
    }
    re_findall(pattern, text)
}
fn _finditer_from_items(
    found_items: &Vec<String>,
    text: &String,
) -> Vec<__SifrStdlib_sifr_x2ere_x2eMatch> {
    let mut matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = vec![];
    let mut cursor: i64 = 0_i64;
    for found in found_items.iter().cloned() {
        let __sifr_chars_found: Vec<char> = found.chars().collect::<Vec<char>>();
        let mut start: i64 = _find_index_from(text, &found, cursor);
        if start < (0_i64) {
            start = cursor;
        }
        let found_len: i64 = __sifr_chars_found.len() as i64;
        let end: i64 = start + found_len;
        matches.push(__SifrStdlib_sifr_x2ere_x2eMatch::new(found, start, end));
        if found_len == (0_i64) {
            cursor = end + (1_i64);
        } else {
            cursor = end;
        }
    }
    matches
}
fn _finditer_materialize(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<__SifrStdlib_sifr_x2ere_x2eMatch>, RegexError> {
    let __sifr_try_res: Result<
        Result<Vec<__SifrStdlib_sifr_x2ere_x2eMatch>, RegexError>,
        RegexError,
    > = (|| {
        let found_items: Vec<String> = _findall_for_finditer(pattern, text, flags)?;
        return Ok(Ok(_finditer_from_items(&found_items, text)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(RegexError::new(e.message.clone()));
        }
    }
}
fn finditer(
    pattern: &String,
    text: &String,
) -> Result<Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>>, RegexError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>>, RegexError>,
        RegexError,
    > = (|| {
        let matches: Vec<__SifrStdlib_sifr_x2ere_x2eMatch> = _finditer_materialize(
            pattern,
            text,
            0_i64,
        )?;
        return Ok(Ok(_iter_matches(matches)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(RegexError::new(e.message.clone()));
        }
    }
}
fn compile(pattern: &String) -> Result<__SifrStdlib_sifr_x2ere_x2ePattern, RegexError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2ere_x2ePattern, RegexError>,
        RegexError,
    > = (|| {
        let compiled: __SifrStdlib___sifr_x2eregex_x2eCompiledPattern = compile_pattern(
            pattern,
        )?;
        return Ok(
            Ok(
                __SifrStdlib_sifr_x2ere_x2ePattern::new(
                    compiled,
                    (pattern).clone(),
                    0_i64,
                ),
            ),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(RegexError::new(error.message.clone()));
        }
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
impl From<RegexError> for Error {
    fn from(err: RegexError) -> Self {
        Self::new(err.message)
    }
}
fn main() {
    let __sifr_try_res: Result<(), RegexError> = (|| {
        let mut digits: Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>> = finditer(
            &"\\d+".to_string(),
            &"v1 and v22".to_string(),
        )?;
        let first: Option<__SifrStdlib_sifr_x2ere_x2eMatch> = digits.next();
        let second: Option<__SifrStdlib_sifr_x2ere_x2eMatch> = digits.next();
        if let Some(first) = first {
            assert!((first.group() == "1"));
        }
        if let Some(second) = second {
            assert!((second.group() == "22"));
        }
        assert!((digits.next() == None));
        let pat: __SifrStdlib_sifr_x2ere_x2ePattern = compile(&"[a-z]+".to_string())?;
        let mut words: Vec<String> = vec![];
        let word_it: Box<dyn Iterator<Item = __SifrStdlib_sifr_x2ere_x2eMatch>> = pat
            .finditer(&"alpha 123 beta".to_string())?;
        for m in word_it {
            words.push(m.group());
        }
        assert!((format!("{:?}", words) == "[\"alpha\", \"beta\"]"));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        assert!(false);
    }
    let base: String = {
        let mut __sifr_concat: String = String::with_capacity(32usize + 0usize);
        __sifr_concat.push_str("/tmp/sifr_regex_filesystem_demo_");
        __sifr_concat.push_str((format!("{}", getpid())).as_str());
        __sifr_concat
    };
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _mk: String = run_command(
            &format!("{}{}", format!("{}{}", "mkdir -p ", base), "/sub"),
        )?;
        let _w1: () = write_text(&format!("{}{}", base, "/a.txt"), &"a".to_string())?;
        let _w2: () = write_text(
            &format!("{}{}", base, "/sub/b.txt"),
            &"b".to_string(),
        )?;
        assert!(
            (format!("{:?}", iglob(& base, & "*.txt".to_string()).collect::< Vec < _ >>
            ()) == "[\"a.txt\"]")
        );
        let root: __SifrStdlib_sifr_x2epathlib_x2ePath = __SifrStdlib_sifr_x2epathlib_x2ePath::new(
            format!("{}{}", base, ""),
        );
        let entries_it: Box<dyn Iterator<Item = String>> = root.iterdir()?;
        let entries: Vec<String> = entries_it.collect::<Vec<_>>();
        assert!((entries.len() as i64) >= (2_i64));
        let top_txt_it: Box<dyn Iterator<Item = String>> = root
            .glob(&"*.txt".to_string())?;
        let top_txt: Vec<String> = top_txt_it.collect::<Vec<_>>();
        assert!(
            (format!("{:?}", top_txt) == ({ let mut __sifr_concat : String =
            String::with_capacity((2usize + base.len()) + 8usize); __sifr_concat
            .push_str("[\""); __sifr_concat.push_str((base).as_str()); __sifr_concat
            .push_str("/a.txt\"]"); __sifr_concat }))
        );
        let recursive_it: Box<dyn Iterator<Item = String>> = root
            .rglob(&"*.txt".to_string())?;
        let recursive_txt: Vec<String> = recursive_it.collect::<Vec<_>>();
        assert!((recursive_txt.len() as i64) == (2_i64));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        assert!(false);
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _clean: String = run_command(&format!("{}{}", "rm -rf ", base))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    println!("parity_ext_regex_and_filesystem_filesystem_iterators_demo: ok");
}
