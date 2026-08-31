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
    pub fn _encoding_is_supported_impl(label: &str) -> bool {
        ::sifr_stdlib::encoding::encoding_is_supported(label)
    }
    pub fn _encoding_canonical_label_impl(label: &str) -> Result<String, ParseError> {
        ::sifr_stdlib::encoding::encoding_canonical_label(label)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _encoding_decode_text_impl(
        data: &[u8],
        encoding: &str,
        errors: &str,
    ) -> Result<String, ParseError> {
        ::sifr_stdlib::encoding::encoding_decode_text(data, encoding, errors)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _encoding_decode_recoveries_impl(
        data: &[u8],
        encoding: &str,
        errors: &str,
    ) -> Result<Vec<String>, ParseError> {
        ::sifr_stdlib::encoding::encoding_decode_recoveries(data, encoding, errors)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _encoding_decode_incremental_text_impl(
        data: &[u8],
        pending: &[u8],
        encoding: &str,
        errors: &str,
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
    pub fn _encoding_decode_incremental_recoveries_impl(
        data: &[u8],
        pending: &[u8],
        encoding: &str,
        errors: &str,
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
    pub fn _encoding_decode_incremental_pending_impl(
        data: &[u8],
        pending: &[u8],
        encoding: &str,
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
    pub fn _encoding_encode_bytes_impl(
        text: &str,
        encoding: &str,
        errors: &str,
    ) -> Result<Vec<u8>, ParseError> {
        ::sifr_stdlib::encoding::encoding_encode_bytes(text, encoding, errors)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _encoding_encode_recoveries_impl(
        text: &str,
        encoding: &str,
        errors: &str,
    ) -> Result<Vec<String>, ParseError> {
        ::sifr_stdlib::encoding::encoding_encode_recoveries(text, encoding, errors)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn read_text(path: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::read_text(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn write_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::write_text(path, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn exists(path: &str) -> bool {
        ::sifr_stdlib::fs::exists(path)
    }
    pub fn read_lines(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::read_lines(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn append_text(path: &str, content: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::append_text(path, content)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _open_file(path: &str, mode: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::open_file(path, mode)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_read(handle: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::file_read(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write(handle: &str, data: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_readline(handle: &str) -> Result<Option<String>, IOError> {
        ::sifr_stdlib::fs::file_readline(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_readlines(handle: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::file_readlines(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_close(handle: &str) {
        ::sifr_stdlib::fs::file_close(handle);
    }
    pub fn _file_read_bytes(
        handle: &str,
        size: Option<SifrInt>,
    ) -> Result<Vec<u8>, IOError> {
        ::sifr_stdlib::fs::file_read_bytes(
                handle,
                size.map(::sifr_runtime::interop::SifrIntBridge::from),
            )
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_write_bytes(handle: &str, data: &[u8]) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_write_bytes(handle, data)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_flush(handle: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::file_flush(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn _file_seek(
        handle: &str,
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
    pub fn _file_tell(handle: &str) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::file_tell(handle)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn open_file(path: &str, mode: &str) -> Result<__SifrIoNativeFileHandle, IOError> {
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
    pub fn file_write(handle: &__SifrIoNativeFileHandle, data: &str) -> Result<(), IOError> {
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
        _file_read_bytes(&handle._id.clone(), size.clone())
    }
    pub fn file_write_bytes(
        handle: &__SifrIoNativeFileHandle,
        data: &[u8],
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
        _file_seek(&handle._id.clone(), offset.clone(), whence.clone())
    }
    pub fn file_tell(handle: &__SifrIoNativeFileHandle) -> Result<SifrInt, IOError> {
        _file_tell(&handle._id.clone())
    }
    pub fn getcwd() -> Result<String, IOError> {
        ::sifr_stdlib::fs::getcwd()
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn listdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::listdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn mkdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::mkdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rmdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn remove_file(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::remove_file(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rename(src: &str, dst: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rename(src, dst)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn chdir(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::chdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn stat_size(path: &str) -> Result<SifrInt, IOError> {
        ::sifr_stdlib::fs::stat_size(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn disk_usage(path: &str) -> Vec<SifrInt> {
        ::sifr_stdlib::fs::disk_usage(path)
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn is_file(path: &str) -> bool {
        ::sifr_stdlib::fs::is_file(path)
    }
    pub fn is_dir(path: &str) -> bool {
        ::sifr_stdlib::fs::is_dir(path)
    }
    pub fn copy_file(src: &str, dst: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::copy_file(src, dst)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn walk_dir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::walk_dir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rmdir_all(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::rmdir_all(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn gettempdir() -> String {
        ::sifr_stdlib::fs::gettempdir()
    }
    pub fn makedirs(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::makedirs(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn touch(path: &str) -> Result<(), IOError> {
        ::sifr_stdlib::fs::touch(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn resolve_path(path: &str) -> Result<String, IOError> {
        ::sifr_stdlib::fs::resolve_path(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn iterdir(path: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::iterdir(path)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn glob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::glob_pattern(dir, pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn rglob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
        ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
    }
    pub fn set_global_level(level: SifrInt) {
        ::sifr_stdlib::logging::set_global_level(
            ::sifr_runtime::interop::SifrIntBridge::from(level),
        );
    }
    pub fn get_global_level() -> SifrInt {
        ::sifr_stdlib::logging::get_global_level().into_sifr_int()
    }
    pub fn __const_ENCODING_UTF8() -> String {
        "utf-8".to_string().to_string()
    }
    pub fn __const_ENCODING_UTF8_SIG() -> String {
        "utf-8-sig".to_string().to_string()
    }
    pub fn __const_ENCODING_ASCII() -> String {
        "ascii".to_string().to_string()
    }
    pub fn __const_ENCODING_LATIN1() -> String {
        "latin-1".to_string().to_string()
    }
    pub fn __const_ENCODING_UTF16_LE() -> String {
        "utf-16-le".to_string().to_string()
    }
    pub fn __const_ENCODING_UTF16_BE() -> String {
        "utf-16-be".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1250() -> String {
        "windows-1250".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1251() -> String {
        "windows-1251".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1252() -> String {
        "windows-1252".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1253() -> String {
        "windows-1253".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1254() -> String {
        "windows-1254".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1255() -> String {
        "windows-1255".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1256() -> String {
        "windows-1256".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1257() -> String {
        "windows-1257".to_string().to_string()
    }
    pub fn __const_ENCODING_WINDOWS_1258() -> String {
        "windows-1258".to_string().to_string()
    }
    pub fn __const_DECODE_ERRORS_STRICT() -> String {
        "strict".to_string().to_string()
    }
    pub fn __const_DECODE_ERRORS_REPLACE() -> String {
        "replace".to_string().to_string()
    }
    pub fn __const_DECODE_ERRORS_IGNORE() -> String {
        "ignore".to_string().to_string()
    }
    pub fn __const_DECODE_ERRORS_BACKSLASH_REPLACE() -> String {
        "backslashreplace".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_STRICT() -> String {
        "strict".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_REPLACE() -> String {
        "replace".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_IGNORE() -> String {
        "ignore".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_BACKSLASH_REPLACE() -> String {
        "backslashreplace".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_XMLCHARREF_REPLACE() -> String {
        "xmlcharrefreplace".to_string().to_string()
    }
    pub fn __const_ENCODE_ERRORS_NAME_REPLACE() -> String {
        "namereplace".to_string().to_string()
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
        pub message: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeError {
        pub fn new(message: String) -> Self {
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
        pub message: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeError {
        pub fn new(message: String) -> Self {
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        pub label: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        pub fn new(label: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(label.len() + 0usize);
                __sifr_concat.push_str(label.as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            Self { label: __sifr_field_init_0 }
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        pub fn canonical_label(
            &self,
        ) -> Result<String, __SifrStdlib_sifr_x2eencoding_x2eDecodeError> {
            _encoding_canonical_label(&self.label)
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        pub fn is_supported(&self) -> bool {
            _encoding_is_supported(&self.label)
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Encoding(label={})", self.label)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        pub name: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        pub fn new(name: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
                __sifr_concat.push_str(name.as_str());
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        pub name: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        pub fn new(name: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
                __sifr_concat.push_str(name.as_str());
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
        pub text: String,
        pub recoveries: Vec<String>,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
        pub fn new(text: String, recoveries: Vec<String>) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
                __sifr_concat.push_str(text.as_str());
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
        pub fn get_text(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str(self.text.clone().as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            }
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
        pub fn get_recoveries(&self) -> Vec<String> {
            self.recoveries.clone()
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
        pub data: Vec<u8>,
        pub recoveries: Vec<String>,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
        pub fn new(data: Vec<u8>, recoveries: Vec<String>) -> Self {
            let __sifr_field_init_0: Vec<u8> = data;
            let __sifr_field_init_1: Vec<String> = recoveries;
            Self {
                data: __sifr_field_init_0,
                recoveries: __sifr_field_init_1,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
        pub fn get_data(&self) -> Vec<u8> {
            self.data.clone()
        }
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome {
        pub fn get_recoveries(&self) -> Vec<String> {
            self.recoveries.clone()
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecoder {
        pub _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        pub _errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
        pub _exhausted: bool,
        pub _pending: Vec<u8>,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecoder {
        pub fn new(
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
        pub fn decode(
            &mut self,
            data: &[u8],
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncoder {
        pub _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        pub _errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
        pub _exhausted: bool,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncoder {
        pub fn new(
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
        pub fn encode(
            &mut self,
            text: &str,
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
                    &Some(self._errors.clone()),
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
    pub fn _encoding_is_supported(label: &str) -> bool {
        _encoding_is_supported_impl(label)
    }
    pub fn _encoding_canonical_label(
        label: &str,
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
    pub fn _encoding_decode_text(
        data: &[u8],
        encoding: &str,
        errors: &str,
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
    pub fn _encoding_decode_recoveries(
        data: &[u8],
        encoding: &str,
        errors: &str,
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
    pub fn _encoding_decode_outcome(
        data: &[u8],
        encoding: &str,
        errors: &str,
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
    pub fn _encoding_decode_incremental_outcome(
        data: &[u8],
        pending: &[u8],
        encoding: &str,
        errors: &str,
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
    pub fn _encoding_decode_incremental_pending(
        data: &[u8],
        pending: &[u8],
        encoding: &str,
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
    pub fn _encoding_encode_bytes(
        text: &str,
        encoding: &str,
        errors: &str,
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
    pub fn _encoding_encode_recoveries(
        text: &str,
        encoding: &str,
        errors: &str,
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
    pub fn _encoding_encode_outcome(
        text: &str,
        encoding: &str,
        errors: &str,
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
    pub fn encoding(label: &str) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(label.to_owned())
    }
    pub fn utf8() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8())
    }
    pub fn utf8_sig() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF8_SIG())
    }
    pub fn ascii() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_ASCII())
    }
    pub fn latin1() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_LATIN1())
    }
    pub fn utf16_le() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_LE())
    }
    pub fn utf16_be() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_UTF16_BE())
    }
    pub fn windows1252() -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(__const_ENCODING_WINDOWS_1252())
    }
    pub fn strict_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            __const_DECODE_ERRORS_STRICT(),
        )
    }
    pub fn replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            __const_DECODE_ERRORS_REPLACE(),
        )
    }
    pub fn ignore_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            __const_DECODE_ERRORS_IGNORE(),
        )
    }
    pub fn backslash_replace_decode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            __const_DECODE_ERRORS_BACKSLASH_REPLACE(),
        )
    }
    pub fn strict_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_STRICT(),
        )
    }
    pub fn replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_REPLACE(),
        )
    }
    pub fn ignore_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_IGNORE(),
        )
    }
    pub fn backslash_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_BACKSLASH_REPLACE(),
        )
    }
    pub fn xmlcharref_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_XMLCHARREF_REPLACE(),
        )
    }
    pub fn name_replace_encode_handler() -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            __const_ENCODE_ERRORS_NAME_REPLACE(),
        )
    }
    pub fn _decode_handler_name(
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> String {
        if let Some(errors) = errors.as_ref() {
            return {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str(errors.name.clone().as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        __const_DECODE_ERRORS_STRICT()
    }
    pub fn _encode_handler_name(
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
    ) -> String {
        if let Some(errors) = errors.as_ref() {
            return {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str(errors.name.clone().as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        __const_ENCODE_ERRORS_STRICT()
    }
    pub fn _decode_handler_or_strict(
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        if let Some(errors) = errors.as_ref() {
            return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
                format!("{}{}", errors.name.clone(), ""),
            );
        }
        strict_decode_handler()
    }
    pub fn _encode_handler_or_strict(
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler>,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        if let Some(errors) = errors.as_ref() {
            return __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
                format!("{}{}", errors.name.clone(), ""),
            );
        }
        strict_encode_handler()
    }
    pub fn decode_outcome(
        data: &[u8],
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
    pub fn decode(
        data: &[u8],
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
    pub fn encode_outcome(
        text: &str,
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
    pub fn encode(
        text: &str,
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
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        __SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(IOError),
        __SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
            __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
        ),
    }
    impl From<IOError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        fn from(value: IOError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        __SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(IOError),
        __SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
            __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
        ),
    }
    impl From<IOError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        fn from(value: IOError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn new() -> Self {
            let __sifr_field_init_0: bool = false;
            Self {
                _closed: __sifr_field_init_0,
            }
        }
    }
    impl ::std::default::Default for __SifrStdlib_sifr_x2eio_x2eIOBase {
        fn default() -> Self {
            Self::new()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            let _ = offset.clone();
            let _ = whence.clone();
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn readable(&self) -> bool {
            false
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn writable(&self) -> bool {
            false
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn seekable(&self) -> bool {
            false
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eIOBase {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "IOBase(_closed={})", self._closed)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eTextIOBase {
        pub iobase: __SifrStdlib_sifr_x2eio_x2eIOBase,
    }
    impl ::std::ops::Deref for __SifrStdlib_sifr_x2eio_x2eTextIOBase {
        type Target = __SifrStdlib_sifr_x2eio_x2eIOBase;
        fn deref(&self) -> &Self::Target {
            &self.iobase
        }
    }
    impl ::std::ops::DerefMut for __SifrStdlib_sifr_x2eio_x2eTextIOBase {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.iobase
        }
    }
    impl ::std::convert::From<__SifrStdlib_sifr_x2eio_x2eTextIOBase>
    for __SifrStdlib_sifr_x2eio_x2eIOBase {
        fn from(value: __SifrStdlib_sifr_x2eio_x2eTextIOBase) -> Self {
            value.iobase
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextIOBase {}
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextIOBase {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "TextIOBase(iobase={})", self.iobase)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
        pub iobase: __SifrStdlib_sifr_x2eio_x2eIOBase,
    }
    impl ::std::ops::Deref for __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
        type Target = __SifrStdlib_sifr_x2eio_x2eIOBase;
        fn deref(&self) -> &Self::Target {
            &self.iobase
        }
    }
    impl ::std::ops::DerefMut for __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.iobase
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {}
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "BinaryIOBase(iobase={})", self.iobase)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrIoFileHandle {
        pub _handle: __SifrIoNativeFileHandle,
        pub _mode: String,
        pub _closed: bool,
    }
    impl __SifrIoFileHandle {
        pub fn new(handle: __SifrIoNativeFileHandle, mode: String) -> Self {
            let __sifr_field_init_0: __SifrIoNativeFileHandle = handle;
            let __sifr_field_init_1: String = mode;
            let __sifr_field_init_2: bool = false;
            Self {
                _handle: __sifr_field_init_0,
                _mode: __sifr_field_init_1,
                _closed: __sifr_field_init_2,
            }
        }
    }
    impl __SifrIoFileHandle {
        pub fn close(&mut self) {
            if self._closed {
                return;
            }
            file_close(&self._handle);
            self._closed = true;
        }
    }
    impl __SifrIoFileHandle {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrIoFileHandle {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_flush(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn read(&self) -> Result<String, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn write(&self, data: &str) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write(&self._handle, data)
        }
    }
    impl __SifrIoFileHandle {
        pub fn readline(&self) -> Result<Option<String>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_readline(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn readlines(&self) -> Result<Vec<String>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_readlines(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read_bytes(&self._handle, size.clone())
        }
    }
    impl __SifrIoFileHandle {
        pub fn write_bytes(&self, data: &[u8]) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self._handle, data)
        }
    }
    impl __SifrIoFileHandle {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_seek(&self._handle, offset.clone(), whence.clone())
        }
    }
    impl __SifrIoFileHandle {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_tell(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn readable(&self) -> bool {
            _mode_is_readable(&self._mode)
        }
    }
    impl __SifrIoFileHandle {
        pub fn writable(&self) -> bool {
            _mode_is_writable(&self._mode)
        }
    }
    impl __SifrIoFileHandle {
        pub fn seekable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrIoFileHandle {
        pub fn __enter__(&self) -> __SifrIoFileHandle {
            self.clone()
        }
    }
    impl __SifrIoFileHandle {
        pub fn __exit__(&mut self) {
            self.close();
        }
    }
    impl ::std::fmt::Display for __SifrIoFileHandle {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "FileHandle(_handle={:?}, _mode={}, _closed={})", self._handle, self
                ._mode, self._closed
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrIoBinaryFileHandle {
        pub _handle: __SifrIoNativeFileHandle,
        pub _mode: String,
        pub _closed: bool,
    }
    impl __SifrIoBinaryFileHandle {
        pub fn new(handle: __SifrIoNativeFileHandle, mode: String) -> Self {
            let __sifr_field_init_0: __SifrIoNativeFileHandle = handle;
            let __sifr_field_init_1: String = mode;
            let __sifr_field_init_2: bool = false;
            Self {
                _handle: __sifr_field_init_0,
                _mode: __sifr_field_init_1,
                _closed: __sifr_field_init_2,
            }
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn close(&mut self) {
            if self._closed {
                return;
            }
            file_close(&self._handle);
            self._closed = true;
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_flush(&self._handle)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read_bytes(&self._handle, size.clone())
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn write_bytes(&self, data: &[u8]) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self._handle, data)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_seek(&self._handle, offset.clone(), whence.clone())
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            file_tell(&self._handle)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn readable(&self) -> bool {
            _mode_is_readable(&self._mode)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn writable(&self) -> bool {
            _mode_is_writable(&self._mode)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn seekable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn __enter__(&self) -> __SifrIoBinaryFileHandle {
            self.clone()
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn __exit__(&mut self) {
            self.close();
        }
    }
    impl ::std::fmt::Display for __SifrIoBinaryFileHandle {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "BinaryFileHandle(_handle={:?}, _mode={}, _closed={})", self._handle, self
                ._mode, self._closed
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrIoTextFileHandle {
        pub _binary: __SifrIoBinaryFileHandle,
        pub _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
        pub _decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
        pub _encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
    }
    impl __SifrIoTextFileHandle {
        pub fn new(
            binary: __SifrIoBinaryFileHandle,
            enc: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
            decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
            encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
        ) -> Self {
            let __sifr_field_init_0: __SifrIoBinaryFileHandle = binary;
            let __sifr_field_init_1: __SifrStdlib_sifr_x2eencoding_x2eEncoding = enc;
            let __sifr_field_init_2: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = decode_errors;
            let __sifr_field_init_3: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler = encode_errors;
            Self {
                _binary: __sifr_field_init_0,
                _encoding: __sifr_field_init_1,
                _decode_errors: __sifr_field_init_2,
                _encode_errors: __sifr_field_init_3,
            }
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn close(&mut self) {
            self._binary.close();
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn closed(&self) -> bool {
            self._binary.closed()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn flush(&self) -> Result<(), IOError> {
            self._binary.flush()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn read(&self) -> Result<String, IOError> {
            let __sifr_try_res: Result<
                Result<String, IOError>,
                __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0,
            > = (|| {
                let data: Vec<u8> = (self._binary.read_bytes(&None))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                        __e,
                    ))?;
                let text: String = (decode(
                    &data,
                    &self._encoding,
                    &Some(self._decode_errors.clone()),
                ))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
                        __e,
                    ))?;
                Ok(Ok(text))
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    match __sifr_try_err {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                            __sifr_try_variant_error,
                        ) => {
                            let e = __sifr_try_variant_error.clone();
                            return Err(e);
                        }
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
                            __sifr_try_variant_error,
                        ) => {
                            let e = __sifr_try_variant_error.clone();
                            return Err(
                                IOError::new({
                                    let mut __sifr_concat: String = String::with_capacity(
                                        20usize + 0usize,
                                    );
                                    __sifr_concat.push_str("text decode failed: ");
                                    __sifr_concat.push_str(e.message.clone().as_str());
                                    __sifr_concat
                                }),
                            );
                        }
                    }
                }
            }
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn write(&self, text: &str) -> Result<(), IOError> {
            let __sifr_try_res: Result<
                Result<(), IOError>,
                __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0,
            > = (|| {
                let data: Vec<u8> = (encode(
                    text,
                    &self._encoding,
                    &Some(self._encode_errors.clone()),
                ))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
                        __e,
                    ))?;
                let result: () = (self._binary.write_bytes(&data))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                        __e,
                    ))?;
                Ok(Ok(()))
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    match __sifr_try_err {
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                            __sifr_try_variant_error,
                        ) => {
                            let e = __sifr_try_variant_error.clone();
                            return Err(e);
                        }
                        __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
                            __sifr_try_variant_error,
                        ) => {
                            let e = __sifr_try_variant_error.clone();
                            return Err(
                                IOError::new({
                                    let mut __sifr_concat: String = String::with_capacity(
                                        20usize + 0usize,
                                    );
                                    __sifr_concat.push_str("text encode failed: ");
                                    __sifr_concat.push_str(e.message.clone().as_str());
                                    __sifr_concat
                                }),
                            );
                        }
                    }
                }
            }
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn readline(&self) -> Result<Option<String>, IOError> {
            Err(
                IOError::new(
                    "TextFileHandle.readline is deferred; use read().split(\"\\n\")"
                        .to_string(),
                ),
            )
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn readlines(&self) -> Result<Vec<String>, IOError> {
            Err(
                IOError::new(
                    "TextFileHandle.readlines is deferred; use read().split(\"\\n\")"
                        .to_string(),
                ),
            )
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            self._binary.seek(offset, whence)
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            self._binary.tell()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn readable(&self) -> bool {
            self._binary.readable()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn writable(&self) -> bool {
            self._binary.writable()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn seekable(&self) -> bool {
            self._binary.seekable()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn __enter__(&self) -> __SifrIoTextFileHandle {
            self.clone()
        }
    }
    impl __SifrIoTextFileHandle {
        pub fn __exit__(&mut self) {
            self.close();
        }
    }
    impl ::std::fmt::Display for __SifrIoTextFileHandle {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "TextFileHandle(_binary={}, _encoding={:?}, _decode_errors={:?}, _encode_errors={:?})",
                self._binary, self._encoding, self._decode_errors, self._encode_errors
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn new() -> Self {
            let __sifr_field_init_0: bool = false;
            Self {
                _closed: __sifr_field_init_0,
            }
        }
    }
    impl ::std::default::Default for __SifrStdlib_sifr_x2eio_x2eTextReader {
        fn default() -> Self {
            Self::new()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn read(&self) -> Result<String, IOError> {
            Err(
                IOError::new(
                    "TextReader direct construction is unsupported; use open_text"
                        .to_string(),
                ),
            )
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn readline(&self) -> Result<Option<String>, IOError> {
            Err(
                IOError::new(
                    "TextReader.readline is deferred; use read().split(\"\\n\")".to_string(),
                ),
            )
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn readlines(&self) -> Result<Vec<String>, IOError> {
            Err(
                IOError::new(
                    "TextReader.readlines is deferred; use read().split(\"\\n\")".to_string(),
                ),
            )
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextReader {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextReader {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "TextReader(_closed={})", self._closed)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eTextWriter {
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
        pub fn new() -> Self {
            let __sifr_field_init_0: bool = false;
            Self {
                _closed: __sifr_field_init_0,
            }
        }
    }
    impl ::std::default::Default for __SifrStdlib_sifr_x2eio_x2eTextWriter {
        fn default() -> Self {
            Self::new()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
        pub fn write(&self, text: &str) -> Result<(), IOError> {
            let _ = text.to_owned();
            Err(
                IOError::new(
                    "TextWriter direct construction is unsupported; use open_text"
                        .to_string(),
                ),
            )
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextWriter {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "TextWriter(_closed={})", self._closed)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub _buffer: String,
        pub _cursor: SifrInt,
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn new(initial: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    initial.len() + 0usize,
                );
                __sifr_concat.push_str(initial.as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: SifrInt = SifrInt::from_i64(0);
            let __sifr_field_init_2: bool = false;
            Self {
                _buffer: __sifr_field_init_0,
                _cursor: __sifr_field_init_1,
                _closed: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn read(&mut self, size: &Option<SifrInt>) -> Result<String, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let start: SifrInt = self._cursor.clone();
            let mut end: SifrInt = SifrInt::from(self._buffer.chars().count());
            if let Some(size) = size.as_ref() {
                let maybe_size: SifrInt = size.clone();
                if (&maybe_size >= &SifrInt::from_i64(0)) {
                    let requested: SifrInt = &start + &maybe_size;
                    if (&requested < &end) {
                        end = requested;
                    }
                }
            }
            let piece: String = {
                let _slice_src = &self._buffer.clone();
                let _slice_len = _slice_src.chars().count();
                let _slice_start = start.clamp_slice_bound(_slice_len);
                let _slice_stop = end.clamp_slice_bound(_slice_len);
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start)),
                )
            };
            self._cursor = end.clone();
            Ok(piece)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn write(&mut self, data: &str) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let left: String = {
                let _slice_src = &self._buffer.clone();
                let _slice_len = _slice_src.chars().count();
                let _slice_start = 0;
                let _slice_stop = self._cursor.clone().clamp_slice_bound(_slice_len);
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start)),
                )
            };
            let tail_start: SifrInt = &self._cursor.clone()
                + &SifrInt::from(data.chars().count());
            let mut right: String = "".to_string();
            if (&tail_start < &SifrInt::from(self._buffer.chars().count())) {
                right = {
                    let _slice_src = &self._buffer.clone();
                    let _slice_len = _slice_src.chars().count();
                    let _slice_start = tail_start.clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    String::from_iter(
                        _slice_src
                            .chars()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start)),
                    )
                };
            }
            self._buffer = {
                let mut __sifr_concat: String = String::with_capacity(
                    (left.len() + data.len()) + right.len(),
                );
                __sifr_concat.push_str(left.as_str());
                __sifr_concat.push_str(data);
                __sifr_concat.push_str(right.as_str());
                __sifr_concat
            };
            self._cursor = &self._cursor.clone() + &SifrInt::from(data.chars().count());
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn getvalue(&self) -> String {
            self._buffer.clone()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn seek(
            &mut self,
            offset: &SifrInt,
            whence: &SifrInt,
        ) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let mut origin: SifrInt = SifrInt::from_i64(0);
            if (whence == &SifrInt::from_i64(0)) {
                origin = SifrInt::from_i64(0);
            } else {
                if (whence == &SifrInt::from_i64(1)) {
                    origin = self._cursor.clone();
                } else {
                    if (whence == &SifrInt::from_i64(2)) {
                        origin = SifrInt::from(self._buffer.chars().count());
                    } else {
                        return Err(IOError::new(_invalid_whence_error(whence.clone())));
                    }
                }
            }
            let mut next_pos: SifrInt = &origin + offset;
            if (&next_pos < &SifrInt::from_i64(0)) {
                return Err(IOError::new(_negative_seek_error(next_pos.clone())));
            }
            let end: SifrInt = SifrInt::from(self._buffer.chars().count());
            if &next_pos > &end {
                next_pos = end.clone();
            }
            self._cursor = next_pos.clone();
            Ok(self._cursor.clone())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(self._cursor.clone())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn readable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn writable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn seekable(&self) -> bool {
            !(self._closed)
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eStringIO {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "StringIO(_buffer={}, _cursor={}, _closed={})", self._buffer, self
                ._cursor, self._closed
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub _buffer: Vec<u8>,
        pub _cursor: SifrInt,
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn new(initial: Vec<u8>) -> Self {
            let __sifr_field_init_0: Vec<u8> = initial;
            let __sifr_field_init_1: SifrInt = SifrInt::from_i64(0);
            let __sifr_field_init_2: bool = false;
            Self {
                _buffer: __sifr_field_init_0,
                _cursor: __sifr_field_init_1,
                _closed: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn close(&mut self) {
            self._closed = true;
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn closed(&self) -> bool {
            self._closed
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn flush(&self) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn read_bytes(&mut self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let start: SifrInt = self._cursor.clone();
            let mut end: SifrInt = SifrInt::from(self._buffer.len());
            if let Some(size) = size.as_ref() {
                let maybe_size: SifrInt = size.clone();
                if (&maybe_size >= &SifrInt::from_i64(0)) {
                    let requested: SifrInt = &start + &maybe_size;
                    if (&requested < &end) {
                        end = requested;
                    }
                }
            }
            let chunk: Vec<u8> = {
                let _slice_src = &self._buffer.clone();
                let _slice_len = _slice_src.len();
                let _slice_start = start.clamp_slice_bound(_slice_len);
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
            Ok(chunk)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if (&self._cursor.clone() == &SifrInt::from(self._buffer.len())) {
                self._buffer = {
                    let mut __v = (self._buffer.clone()).to_vec();
                    __v.extend((data).iter().cloned());
                    __v
                };
                self._cursor = &self._cursor.clone() + &SifrInt::from(data.len());
                return Ok(());
            }
            let left: Vec<u8> = {
                let _slice_src = &self._buffer.clone();
                let _slice_len = _slice_src.len();
                let _slice_start = 0;
                let _slice_stop = self._cursor.clone().clamp_slice_bound(_slice_len);
                Vec::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start)
                        .take(_slice_stop.saturating_sub(_slice_start))
                        .cloned(),
                )
            };
            let tail_start: SifrInt = &self._cursor.clone() + &SifrInt::from(data.len());
            let mut right: Vec<u8> = {
                let __sifr_empty_bytes_literal: Vec<u8> = vec![];
                __sifr_empty_bytes_literal
            };
            if (&tail_start < &SifrInt::from(self._buffer.len())) {
                right = {
                    let _slice_src = &self._buffer.clone();
                    let _slice_len = _slice_src.len();
                    let _slice_start = tail_start.clamp_slice_bound(_slice_len);
                    let _slice_stop = _slice_len;
                    Vec::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start)
                            .take(_slice_stop.saturating_sub(_slice_start))
                            .cloned(),
                    )
                };
            }
            self._buffer = {
                let mut __v = ({
                    let mut __v = (left).to_vec();
                    __v.extend((data).iter().cloned());
                    __v
                })
                    .to_vec();
                __v.extend((right).iter().cloned());
                __v
            };
            self._cursor = &self._cursor.clone() + &SifrInt::from(data.len());
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn getvalue(&self) -> Vec<u8> {
            self._buffer.clone()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn seek(
            &mut self,
            offset: &SifrInt,
            whence: &SifrInt,
        ) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let mut origin: SifrInt = SifrInt::from_i64(0);
            if (whence == &SifrInt::from_i64(0)) {
                origin = SifrInt::from_i64(0);
            } else {
                if (whence == &SifrInt::from_i64(1)) {
                    origin = self._cursor.clone();
                } else {
                    if (whence == &SifrInt::from_i64(2)) {
                        origin = SifrInt::from(self._buffer.len());
                    } else {
                        return Err(IOError::new(_invalid_whence_error(whence.clone())));
                    }
                }
            }
            let mut next_pos: SifrInt = &origin + offset;
            if (&next_pos < &SifrInt::from_i64(0)) {
                return Err(IOError::new(_negative_seek_error(next_pos.clone())));
            }
            let end: SifrInt = SifrInt::from(self._buffer.len());
            if &next_pos > &end {
                next_pos = end.clone();
            }
            self._cursor = next_pos.clone();
            Ok(self._cursor.clone())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(self._cursor.clone())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn readable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn writable(&self) -> bool {
            !(self._closed)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn seekable(&self) -> bool {
            !(self._closed)
        }
    }
    pub fn _closed_stream_error() -> String {
        "I/O operation on closed stream".to_string()
    }
    pub fn _invalid_whence_error(whence: SifrInt) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
            __sifr_concat.push_str("invalid whence: ");
            __sifr_concat.push_str(format!("{}", whence).as_str());
            __sifr_concat
        }
    }
    pub fn _negative_seek_error(offset: SifrInt) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
            __sifr_concat.push_str("negative seek position: ");
            __sifr_concat.push_str(format!("{}", offset).as_str());
            __sifr_concat
        }
    }
    pub fn _unsupported_seek_tell_error() -> String {
        "seek/tell is unsupported for this stream".to_string()
    }
    pub fn _mode_is_readable(mode: &str) -> bool {
        mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
    }
    pub fn _mode_is_writable(mode: &str) -> bool {
        (mode.contains(&"w".to_string()) || mode.contains(&"a".to_string()))
            || mode.contains(&"+".to_string())
    }
    pub fn _text_binary_mode(mode: &str) -> Result<String, IOError> {
        if mode.contains(&"b".to_string()) {
            return Err(
                IOError::new("open_text requires a text mode without \'b\'".to_string()),
            );
        }
        if (mode == "r") || (mode == "rt") {
            return Ok("rb".to_string());
        }
        if (mode == "w") || (mode == "wt") {
            return Ok("wb".to_string());
        }
        if (mode == "a") || (mode == "at") {
            return Ok("ab".to_string());
        }
        Err(
            IOError::new({
                let mut __sifr_concat: String = String::with_capacity(19usize + mode.len());
                __sifr_concat.push_str("invalid text mode: ");
                __sifr_concat.push_str(mode);
                __sifr_concat
            }),
        )
    }
    pub fn _text_encoding_or_default(
        enc: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncoding>,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        if let Some(enc) = enc.as_ref() {
            return __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(
                format!("{}{}", enc.label.clone(), ""),
            );
        }
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new("utf-8".to_string())
    }
    pub fn _decode_errors_or_default(
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
        if let Some(errors) = errors.as_ref() {
            return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
                format!("{}{}", errors.name.clone(), ""),
            );
        }
        strict_decode_handler()
    }
    pub fn _encode_errors_from_decode_errors(
        errors: &__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
    ) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        )
    }
    pub fn open(path: &str, mode: &str) -> Result<__SifrIoFileHandle, IOError> {
        let __sifr_try_res: Result<Result<__SifrIoFileHandle, IOError>, IOError> = (|| {
            let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
            Ok(Ok(__SifrIoFileHandle::new(handle, mode.to_owned())))
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
    pub fn open_binary(path: &str, mode: &str) -> Result<__SifrIoBinaryFileHandle, IOError> {
        if !mode.contains(&"b".to_string()) {
            return Err(IOError::new("open_binary requires binary mode".to_string()));
        }
        let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
            let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
            Ok(Ok(__SifrIoBinaryFileHandle::new(handle, mode.to_owned())))
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
    pub fn open_text(
        path: &str,
        mode: &str,
        encoding: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncoding>,
        errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
    ) -> Result<__SifrIoTextFileHandle, IOError> {
        let __sifr_try_res: Result<Result<__SifrIoTextFileHandle, IOError>, IOError> = (|| {
            let binary_mode: String = _text_binary_mode(mode)?;
            let text_encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding = _text_encoding_or_default(
                encoding,
            );
            let decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = _decode_errors_or_default(
                errors,
            );
            let encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler = _encode_errors_from_decode_errors(
                &decode_errors,
            );
            let binary: __SifrIoBinaryFileHandle = open_binary(path, &binary_mode)?;
            Ok(
                Ok(
                    __SifrIoTextFileHandle::new(
                        binary,
                        text_encoding,
                        decode_errors,
                        encode_errors,
                    ),
                ),
            )
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
    pub fn __const_DEBUG() -> SifrInt {
        SifrInt::from_i64(10)
    }
    pub fn __const_INFO() -> SifrInt {
        SifrInt::from_i64(20)
    }
    pub fn __const_WARNING() -> SifrInt {
        SifrInt::from_i64(30)
    }
    pub fn __const_ERROR() -> SifrInt {
        SifrInt::from_i64(40)
    }
    pub fn __const_CRITICAL() -> SifrInt {
        SifrInt::from_i64(50)
    }
    pub fn __const_NOTSET() -> SifrInt {
        SifrInt::from_i64(0)
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2elogging_x2eFormatter {
        pub _fmt: String,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
        pub fn new(fmt: String) -> Self {
            let __sifr_field_init_0: String = fmt;
            Self { _fmt: __sifr_field_init_0 }
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
        pub fn template(&self) -> String {
            self._fmt.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFormatter {
        pub fn format(&self, level: &str, name: &str, msg: &str) -> String {
            let mut result: String = self._fmt.clone();
            result = result.replace("%(levelname)s", &level);
            result = result.replace("%(name)s", &name);
            result = result.replace("%(message)s", &msg);
            result
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eFormatter {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "Formatter(_fmt={})", self._fmt)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub _level: SifrInt,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn new(level: SifrInt) -> Self {
            let __sifr_field_init_0: SifrInt = level.clone();
            let __sifr_field_init_1: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                "%(levelname)s:%(name)s:%(message)s".to_string(),
            );
            Self {
                _level: __sifr_field_init_0,
                _formatter: __sifr_field_init_1,
            }
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn set_level(&mut self, level: &SifrInt) {
            self._level = level.clone();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn level(&self) -> SifrInt {
            self._level.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
            self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                format!("{}{}", fmt._fmt.clone(), ""),
            );
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn format_template(&self) -> String {
            self._formatter.template()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn _allows(&self, level_num: &SifrInt) -> bool {
            if (&self._level.clone() == &__const_NOTSET()) {
                return true;
            }
            (level_num >= &self._level)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn emit(&self, level: &str, name: &str, msg: &str) {
            let level_num: SifrInt = _level_name_to_num(level);
            if !self._allows(&level_num) {
                return;
            }
            let line: String = self._formatter.format(level, name, msg);
            println!("{}", line);
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "StreamHandler(_level={}, _formatter={})", self._level, self._formatter
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub _path: String,
        pub _level: SifrInt,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn new(path: String, level: SifrInt) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
                __sifr_concat.push_str(path.as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: SifrInt = level.clone();
            let __sifr_field_init_2: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                "%(levelname)s:%(name)s:%(message)s".to_string(),
            );
            Self {
                _path: __sifr_field_init_0,
                _level: __sifr_field_init_1,
                _formatter: __sifr_field_init_2,
            }
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn path(&self) -> String {
            self._path.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn set_level(&mut self, level: &SifrInt) {
            self._level = level.clone();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn level(&self) -> SifrInt {
            self._level.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
            self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                format!("{}{}", fmt._fmt.clone(), ""),
            );
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn format_template(&self) -> String {
            self._formatter.template()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn _allows(&self, level_num: &SifrInt) -> bool {
            if (&self._level.clone() == &__const_NOTSET()) {
                return true;
            }
            (level_num >= &self._level)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn emit(&self, level: &str, name: &str, msg: &str) {
            let level_num: SifrInt = _level_name_to_num(level);
            if !self._allows(&level_num) {
                return;
            }
            let line: String = {
                let mut __sifr_concat: String = String::with_capacity(0usize + 1usize);
                __sifr_concat.push_str(self._formatter.format(level, name, msg).as_str());
                __sifr_concat.push('\n');
                __sifr_concat
            };
            let __sifr_try_res: Result<(), IOError> = (|| {
                let mut fh: __SifrIoTextFileHandle = open_text(
                    &self._path,
                    &"a".to_string(),
                    &Some(utf8().clone()),
                    &None,
                )?;
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let _ = fh.write(&line)?;
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e2 = __sifr_try_err.clone();
                    let _ = e2.message.clone();
                }
                fh.close();
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _ = e.message.clone();
            }
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "FileHandler(_path={}, _level={}, _formatter={})", self._path, self
                ._level, self._formatter
            )
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub _level: SifrInt,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn new(level: SifrInt) -> Self {
            let __sifr_field_init_0: SifrInt = level.clone();
            let __sifr_field_init_1: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                "%(levelname)s:%(name)s:%(message)s".to_string(),
            );
            Self {
                _level: __sifr_field_init_0,
                _formatter: __sifr_field_init_1,
            }
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn set_level(&mut self, level: &SifrInt) {
            self._level = level.clone();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn level(&self) -> SifrInt {
            self._level.clone()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn set_formatter(&mut self, fmt: &__SifrStdlib_sifr_x2elogging_x2eFormatter) {
            self._formatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                format!("{}{}", fmt._fmt.clone(), ""),
            );
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn format_template(&self) -> String {
            self._formatter.template()
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn emit(&self, level: &str, name: &str, msg: &str) {
            let _ = level.to_owned();
            let _ = name.to_owned();
            let _ = msg.to_owned();
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "NullHandler(_level={}, _formatter={})", self._level, self._formatter)
        }
    }
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub _name: String,
        pub _level: SifrInt,
        pub _log_path: String,
        pub _handler_kind: String,
        pub _handler_path: String,
        pub _handler_level: SifrInt,
        pub _handler_fmt: String,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn new(name: String, level: SifrInt) -> Self {
            let __sifr_field_init_0: String = name;
            let __sifr_field_init_1: SifrInt = level.clone();
            let __sifr_field_init_2: String = "".to_string();
            let __sifr_field_init_3: String = "".to_string();
            let __sifr_field_init_4: String = "".to_string();
            let __sifr_field_init_5: SifrInt = __const_NOTSET().clone();
            let __sifr_field_init_6: String = "%(levelname)s:%(name)s:%(message)s"
                .to_string();
            Self {
                _name: __sifr_field_init_0,
                _level: __sifr_field_init_1,
                _log_path: __sifr_field_init_2,
                _handler_kind: __sifr_field_init_3,
                _handler_path: __sifr_field_init_4,
                _handler_level: __sifr_field_init_5,
                _handler_fmt: __sifr_field_init_6,
            }
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn set_level(&mut self, level: &SifrInt) {
            self._level = level.clone();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn set_file(&mut self, path: &str) {
            self._log_path = {
                let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
                __sifr_concat.push_str(path);
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn add_handler(
            &mut self,
            handler: &__SifrStdlib_sifr_x2elogging_x2eFileHandler,
        ) {
            self._handler_kind = "file".to_string();
            self._handler_path = handler.path();
            self._handler_level = handler.level();
            self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn set_stream_handler(
            &mut self,
            handler: &__SifrStdlib_sifr_x2elogging_x2eStreamHandler,
        ) {
            self._handler_kind = "stream".to_string();
            self._handler_path = "".to_string();
            self._handler_level = handler.level();
            self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn set_null_handler(
            &mut self,
            handler: &__SifrStdlib_sifr_x2elogging_x2eNullHandler,
        ) {
            self._handler_kind = "null".to_string();
            self._handler_path = "".to_string();
            self._handler_level = handler.level();
            self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn clear_handler(&mut self) {
            self._handler_kind = "".to_string();
            self._handler_path = "".to_string();
            self._handler_level = __const_NOTSET().clone();
            self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _handler_allows(&self, level_num: &SifrInt) -> bool {
            if (&self._handler_level.clone() == &__const_NOTSET()) {
                return true;
            }
            (level_num >= &self._handler_level)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _handler_line(&self, level: &str, msg: &str) -> String {
            let formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                self._handler_fmt.clone(),
            );
            formatter.format(level, &self._name.clone(), msg)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _emit(&self, level: &str, level_num: &SifrInt, msg: &str) {
            if (&self._level > level_num) {
                return;
            }
            if (self._handler_kind.clone() == "null") {
                return;
            }
            if (self._handler_kind.clone() == "stream") {
                if self._handler_allows(level_num) {
                    println!("{}", self._handler_line(level, msg));
                }
                return;
            }
            if (self._handler_kind.clone() == "file") {
                if self._handler_allows(level_num) && (self._handler_path.clone() != "") {
                    let line: String = {
                        let mut __sifr_concat: String = String::with_capacity(
                            0usize + 1usize,
                        );
                        __sifr_concat.push_str(self._handler_line(level, msg).as_str());
                        __sifr_concat.push('\n');
                        __sifr_concat
                    };
                    let __sifr_try_res: Result<(), IOError> = (|| {
                        let mut fh: __SifrIoTextFileHandle = open_text(
                            &self._handler_path,
                            &"a".to_string(),
                            &Some(utf8().clone()),
                            &None,
                        )?;
                        let __sifr_try_res: Result<(), IOError> = (|| {
                            let _ = fh.write(&line)?;
                            Ok(())
                        })();
                        if let Err(__sifr_try_err) = __sifr_try_res {
                            let e2 = __sifr_try_err.clone();
                            let _ = e2.message.clone();
                        }
                        fh.close();
                        Ok(())
                    })();
                    if let Err(__sifr_try_err) = __sifr_try_res {
                        let e = __sifr_try_err.clone();
                        let _ = e.message.clone();
                    }
                }
                return;
            }
            let line: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    ((((1usize + level.len()) + 2usize) + 0usize) + 2usize) + msg.len(),
                );
                __sifr_concat.push('[');
                __sifr_concat.push_str(level);
                __sifr_concat.push_str("] ");
                __sifr_concat.push_str(self._name.clone().as_str());
                __sifr_concat.push_str(": ");
                __sifr_concat.push_str(msg);
                __sifr_concat
            };
            println!("{}", line);
            if (self._log_path.clone() != "") {
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let mut fh: __SifrIoTextFileHandle = open_text(
                        &self._log_path,
                        &"a".to_string(),
                        &Some(utf8().clone()),
                        &None,
                    )?;
                    let __sifr_try_res: Result<(), IOError> = (|| {
                        let _ = fh
                            .write(
                                &({
                                    let mut __sifr_concat: String = String::with_capacity(
                                        line.len() + 1usize,
                                    );
                                    __sifr_concat.push_str(line.as_str());
                                    __sifr_concat.push('\n');
                                    __sifr_concat
                                }),
                            )?;
                        Ok(())
                    })();
                    if let Err(__sifr_try_err) = __sifr_try_res {
                        let e2 = __sifr_try_err.clone();
                        let _ = e2.message.clone();
                    }
                    fh.close();
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    let _ = e.message.clone();
                }
            }
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn debug(&self, msg: &str) {
            self._emit(&"DEBUG".to_string(), &__const_DEBUG(), msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn info(&self, msg: &str) {
            self._emit(&"INFO".to_string(), &__const_INFO(), msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn warning(&self, msg: &str) {
            self._emit(&"WARNING".to_string(), &__const_WARNING(), msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn error(&self, msg: &str) {
            self._emit(&"ERROR".to_string(), &__const_ERROR(), msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn critical(&self, msg: &str) {
            self._emit(&"CRITICAL".to_string(), &__const_CRITICAL(), msg);
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2elogging_x2eLogger {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f,
                "Logger(_name={}, _level={}, _log_path={}, _handler_kind={}, _handler_path={}, _handler_level={}, _handler_fmt={})",
                self._name, self._level, self._log_path, self._handler_kind, self
                ._handler_path, self._handler_level, self._handler_fmt
            )
        }
    }
    pub fn _level_name_to_num(level: &str) -> SifrInt {
        if level == "DEBUG" {
            return __const_DEBUG();
        }
        if level == "INFO" {
            return __const_INFO();
        }
        if level == "WARNING" {
            return __const_WARNING();
        }
        if level == "ERROR" {
            return __const_ERROR();
        }
        if level == "CRITICAL" {
            return __const_CRITICAL();
        }
        __const_NOTSET()
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
    pub struct Error {
        pub message: String,
    }
    impl Error {
        pub fn new(message: String) -> Self {
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
    pub struct ParseError {
        pub message: String,
    }
    impl ParseError {
        pub fn new(message: String) -> Self {
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
}
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::__SifrIoBinaryFileHandle;
pub use __sifr_project_nominals::__SifrIoFileHandle;
pub use __sifr_project_nominals::__SifrIoTextFileHandle;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eDecodeError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eDecoder;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncodeError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncodeOutcome;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncoder;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eencoding_x2eEncoding;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eBinaryIOBase;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eBytesIO;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eIOBase;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eStringIO;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eTextIOBase;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eTextReader;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2eio_x2eTextWriter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eFileHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eFormatter;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eLogger;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eNullHandler;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2elogging_x2eStreamHandler;
use ::sifr_runtime::SifrInt;
fn _encoding_is_supported_impl(label: &str) -> bool {
    ::sifr_stdlib::encoding::encoding_is_supported(label)
}
fn _encoding_canonical_label_impl(label: &str) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_canonical_label(label)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_text_impl(
    data: &[u8],
    encoding: &str,
    errors: &str,
) -> Result<String, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_text(data, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_recoveries_impl(
    data: &[u8],
    encoding: &str,
    errors: &str,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_decode_recoveries(data, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_decode_incremental_text_impl(
    data: &[u8],
    pending: &[u8],
    encoding: &str,
    errors: &str,
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
    data: &[u8],
    pending: &[u8],
    encoding: &str,
    errors: &str,
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
    data: &[u8],
    pending: &[u8],
    encoding: &str,
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
    text: &str,
    encoding: &str,
    errors: &str,
) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::encoding::encoding_encode_bytes(text, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _encoding_encode_recoveries_impl(
    text: &str,
    encoding: &str,
    errors: &str,
) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::encoding::encoding_encode_recoveries(text, encoding, errors)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn read_text(path: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::read_text(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn write_text(path: &str, content: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::write_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn exists(path: &str) -> bool {
    ::sifr_stdlib::fs::exists(path)
}
fn read_lines(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::read_lines(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn append_text(path: &str, content: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::append_text(path, content)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _open_file(path: &str, mode: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::open_file(path, mode)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_read(handle: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::file_read(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write(handle: &str, data: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readline(handle: &str) -> Result<Option<String>, IOError> {
    ::sifr_stdlib::fs::file_readline(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_readlines(handle: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::file_readlines(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_close(handle: &str) {
    ::sifr_stdlib::fs::file_close(handle);
}
fn _file_read_bytes(handle: &str, size: Option<SifrInt>) -> Result<Vec<u8>, IOError> {
    ::sifr_stdlib::fs::file_read_bytes(
            handle,
            size.map(::sifr_runtime::interop::SifrIntBridge::from),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_write_bytes(handle: &str, data: &[u8]) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_write_bytes(handle, data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_flush(handle: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::file_flush(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn _file_seek(
    handle: &str,
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
fn _file_tell(handle: &str) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::file_tell(handle)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn open_file(path: &str, mode: &str) -> Result<__SifrIoNativeFileHandle, IOError> {
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
fn file_write(handle: &__SifrIoNativeFileHandle, data: &str) -> Result<(), IOError> {
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
    _file_read_bytes(&handle._id.clone(), size.clone())
}
fn file_write_bytes(
    handle: &__SifrIoNativeFileHandle,
    data: &[u8],
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
    _file_seek(&handle._id.clone(), offset.clone(), whence.clone())
}
fn file_tell(handle: &__SifrIoNativeFileHandle) -> Result<SifrInt, IOError> {
    _file_tell(&handle._id.clone())
}
fn getcwd() -> Result<String, IOError> {
    ::sifr_stdlib::fs::getcwd()
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn listdir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::listdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn mkdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::mkdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn remove_file(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::remove_file(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rename(src: &str, dst: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rename(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn chdir(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::chdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn stat_size(path: &str) -> Result<SifrInt, IOError> {
    ::sifr_stdlib::fs::stat_size(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.into_sifr_int())
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn disk_usage(path: &str) -> Vec<SifrInt> {
    ::sifr_stdlib::fs::disk_usage(path)
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn is_file(path: &str) -> bool {
    ::sifr_stdlib::fs::is_file(path)
}
fn is_dir(path: &str) -> bool {
    ::sifr_stdlib::fs::is_dir(path)
}
fn copy_file(src: &str, dst: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::copy_file(src, dst)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn walk_dir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::walk_dir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rmdir_all(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::rmdir_all(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn gettempdir() -> String {
    ::sifr_stdlib::fs::gettempdir()
}
fn makedirs(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::makedirs(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn touch(path: &str) -> Result<(), IOError> {
    ::sifr_stdlib::fs::touch(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn resolve_path(path: &str) -> Result<String, IOError> {
    ::sifr_stdlib::fs::resolve_path(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn iterdir(path: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::iterdir(path)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn glob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::glob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn rglob_pattern(dir: &str, pattern: &str) -> Result<Vec<String>, IOError> {
    ::sifr_stdlib::fs::rglob_pattern(dir, pattern)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
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
fn _encoding_is_supported(label: &str) -> bool {
    _encoding_is_supported_impl(label)
}
fn _encoding_canonical_label(
    label: &str,
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
    data: &[u8],
    encoding: &str,
    errors: &str,
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
    data: &[u8],
    encoding: &str,
    errors: &str,
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
    data: &[u8],
    encoding: &str,
    errors: &str,
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
    data: &[u8],
    pending: &[u8],
    encoding: &str,
    errors: &str,
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
    data: &[u8],
    pending: &[u8],
    encoding: &str,
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
    text: &str,
    encoding: &str,
    errors: &str,
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
    text: &str,
    encoding: &str,
    errors: &str,
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
    text: &str,
    encoding: &str,
    errors: &str,
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
fn encoding(label: &str) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(label.to_owned())
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
            __sifr_concat.push_str(errors.name.clone().as_str());
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
            __sifr_concat.push_str(errors.name.clone().as_str());
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
    data: &[u8],
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
    data: &[u8],
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
    text: &str,
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
    text: &str,
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
#[derive(Debug, Clone)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    __SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(IOError),
    __SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
        __SifrStdlib_sifr_x2eencoding_x2eDecodeError,
    ),
}
impl From<IOError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    fn from(value: IOError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Debug, Clone)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    __SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(IOError),
    __SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
        __SifrStdlib_sifr_x2eencoding_x2eEncodeError,
    ),
}
impl From<IOError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    fn from(value: IOError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
fn _closed_stream_error() -> String {
    "I/O operation on closed stream".to_string()
}
fn _invalid_whence_error(whence: SifrInt) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("invalid whence: ");
        __sifr_concat.push_str(format!("{}", whence).as_str());
        __sifr_concat
    }
}
fn _negative_seek_error(offset: SifrInt) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
        __sifr_concat.push_str("negative seek position: ");
        __sifr_concat.push_str(format!("{}", offset).as_str());
        __sifr_concat
    }
}
fn _unsupported_seek_tell_error() -> String {
    "seek/tell is unsupported for this stream".to_string()
}
fn _mode_is_readable(mode: &str) -> bool {
    mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
}
fn _mode_is_writable(mode: &str) -> bool {
    (mode.contains(&"w".to_string()) || mode.contains(&"a".to_string()))
        || mode.contains(&"+".to_string())
}
fn _text_binary_mode(mode: &str) -> Result<String, IOError> {
    if mode.contains(&"b".to_string()) {
        return Err(
            IOError::new("open_text requires a text mode without \'b\'".to_string()),
        );
    }
    if (mode == "r") || (mode == "rt") {
        return Ok("rb".to_string());
    }
    if (mode == "w") || (mode == "wt") {
        return Ok("wb".to_string());
    }
    if (mode == "a") || (mode == "at") {
        return Ok("ab".to_string());
    }
    Err(
        IOError::new({
            let mut __sifr_concat: String = String::with_capacity(19usize + mode.len());
            __sifr_concat.push_str("invalid text mode: ");
            __sifr_concat.push_str(mode);
            __sifr_concat
        }),
    )
}
fn _text_encoding_or_default(
    enc: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncoding>,
) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    if let Some(enc) = enc.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eEncoding::new(
            format!("{}{}", enc.label.clone(), ""),
        );
    }
    __SifrStdlib_sifr_x2eencoding_x2eEncoding::new("utf-8".to_string())
}
fn _decode_errors_or_default(
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler::new(
            format!("{}{}", errors.name.clone(), ""),
        );
    }
    strict_decode_handler()
}
fn _encode_errors_from_decode_errors(
    errors: &__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
) -> __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler::new(
        format!("{}{}", errors.name.clone(), ""),
    )
}
fn open(path: &str, mode: &str) -> Result<__SifrIoFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoFileHandle, IOError>, IOError> = (|| {
        let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
        Ok(Ok(__SifrIoFileHandle::new(handle, mode.to_owned())))
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
fn open_binary(path: &str, mode: &str) -> Result<__SifrIoBinaryFileHandle, IOError> {
    if !mode.contains(&"b".to_string()) {
        return Err(IOError::new("open_binary requires binary mode".to_string()));
    }
    let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
        let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
        Ok(Ok(__SifrIoBinaryFileHandle::new(handle, mode.to_owned())))
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
fn open_text(
    path: &str,
    mode: &str,
    encoding: &Option<__SifrStdlib_sifr_x2eencoding_x2eEncoding>,
    errors: &Option<__SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler>,
) -> Result<__SifrIoTextFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoTextFileHandle, IOError>, IOError> = (|| {
        let binary_mode: String = _text_binary_mode(mode)?;
        let text_encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding = _text_encoding_or_default(
            encoding,
        );
        let decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler = _decode_errors_or_default(
            errors,
        );
        let encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler = _encode_errors_from_decode_errors(
            &decode_errors,
        );
        let binary: __SifrIoBinaryFileHandle = open_binary(path, &binary_mode)?;
        Ok(
            Ok(
                __SifrIoTextFileHandle::new(
                    binary,
                    text_encoding,
                    decode_errors,
                    encode_errors,
                ),
            ),
        )
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
fn set_global_level(level: SifrInt) {
    ::sifr_stdlib::logging::set_global_level(
        ::sifr_runtime::interop::SifrIntBridge::from(level),
    );
}
fn get_global_level() -> SifrInt {
    ::sifr_stdlib::logging::get_global_level().into_sifr_int()
}
fn __const_DEBUG() -> SifrInt {
    SifrInt::from_i64(10)
}
fn __const_INFO() -> SifrInt {
    SifrInt::from_i64(20)
}
fn __const_WARNING() -> SifrInt {
    SifrInt::from_i64(30)
}
fn __const_ERROR() -> SifrInt {
    SifrInt::from_i64(40)
}
fn __const_CRITICAL() -> SifrInt {
    SifrInt::from_i64(50)
}
fn __const_NOTSET() -> SifrInt {
    SifrInt::from_i64(0)
}
fn _level_name_to_num(level: &str) -> SifrInt {
    if level == "DEBUG" {
        return __const_DEBUG();
    }
    if level == "INFO" {
        return __const_INFO();
    }
    if level == "WARNING" {
        return __const_WARNING();
    }
    if level == "ERROR" {
        return __const_ERROR();
    }
    if level == "CRITICAL" {
        return __const_CRITICAL();
    }
    __const_NOTSET()
}
fn basicConfig(level: SifrInt) -> __SifrStdlib_sifr_x2elogging_x2eLogger {
    set_global_level(level.clone());
    __SifrStdlib_sifr_x2elogging_x2eLogger::new("root".to_string(), level.clone())
}
fn getLogger(name: &str) -> __SifrStdlib_sifr_x2elogging_x2eLogger {
    let level: SifrInt = get_global_level();
    __SifrStdlib_sifr_x2elogging_x2eLogger::new(name.to_owned(), level.clone())
}
fn run_command(cmd: &str) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn env_get(key: &str) -> Option<String> {
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
fn which(name: &str) -> Option<String> {
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
    words: &[SifrInt],
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
fn base64_encode(s: &str) -> String {
    ::sifr_stdlib::base64::base64_encode(s)
}
fn base64_encode_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::base64::base64_encode_bytes(data)
}
fn base64_decode(s: &str) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_decode_bytes(data: &[u8]) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::base64_decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn base64_encode_opts(
    s: &str,
    altchars: &str,
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
    s: &str,
    altchars: &str,
    validate: bool,
    ignorechars: &str,
) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::base64_decode_opts(s, altchars, validate, ignorechars)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64encode(s: &str) -> String {
    ::sifr_stdlib::base64::urlsafe_b64encode(s)
}
fn urlsafe_b64encode_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::base64::urlsafe_b64encode_bytes(data)
}
fn urlsafe_b64decode(s: &str) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn urlsafe_b64decode_bytes(data: &[u8]) -> Result<Vec<u8>, ParseError> {
    ::sifr_stdlib::base64::urlsafe_b64decode_bytes(data)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32encode(s: &str) -> String {
    ::sifr_stdlib::base64::b32encode(s)
}
fn b32decode(s: &str) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32decode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn b32hexencode(s: &str) -> String {
    ::sifr_stdlib::base64::b32hexencode(s)
}
fn b32hexdecode(s: &str) -> Result<String, ParseError> {
    ::sifr_stdlib::base64::b32hexdecode(s)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn sha256_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha256_bytes(data)
}
fn md5_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::md5_bytes(data)
}
fn sha1_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha1_bytes(data)
}
fn sha224_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha224_bytes(data)
}
fn sha384_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha384_bytes(data)
}
fn sha512_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::sha512_bytes(data)
}
fn blake2b_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2b_bytes(data)
}
fn blake2s_bytes(data: &[u8]) -> Vec<u8> {
    ::sifr_stdlib::hash::blake2s_bytes(data)
}
fn _random_suffix() -> String {
    let n: SifrInt = random_int(SifrInt::from_i64(100000), SifrInt::from_i64(999999));
    format!("{}", n)
}
fn mktemp_path(prefix: &str) -> String {
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
                    __sifr_concat.push_str(root.as_str());
                    __sifr_concat.push_str(prefix);
                    __sifr_concat.push_str(suffix.as_str());
                    __sifr_concat
                };
            }
        }
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            ((root.len() + 1usize) + prefix.len()) + suffix.len(),
        );
        __sifr_concat.push_str(root.as_str());
        __sifr_concat.push('/');
        __sifr_concat.push_str(prefix);
        __sifr_concat.push_str(suffix.as_str());
        __sifr_concat
    }
}
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() })
        );
        i = &i + &SifrInt::from_i64(1);
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
fn collect_logger_actual(base: &str) -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let app_log: String = {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 8usize);
        __sifr_concat.push_str(base);
        __sifr_concat.push_str("/app.log");
        __sifr_concat
    };
    let mut app_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _w: () = write_text(&app_log, &"".to_string())?;
        let mut app: __SifrStdlib_sifr_x2elogging_x2eLogger = getLogger(
            &"demo".to_string(),
        );
        app.set_file(&app_log);
        app.set_level(&__const_INFO());
        app.info(&"start".to_string());
        app.warning(&"warn".to_string());
        app.debug(&"hidden".to_string());
        let app_content: String = read_text(&app_log)?;
        app_ok = (app_content == "[INFO] demo: start\n[WARNING] demo: warn\n");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    actual.push(app_ok);
    actual
}
fn collect_root_and_handler_actual(base: &str) -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let root_log: String = {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 9usize);
        __sifr_concat.push_str(base);
        __sifr_concat.push_str("/root.log");
        __sifr_concat
    };
    let handler_log: String = {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 12usize);
        __sifr_concat.push_str(base);
        __sifr_concat.push_str("/handler.log");
        __sifr_concat
    };
    let mut root_ok: bool = false;
    let mut handler_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _w1: () = write_text(&root_log, &"".to_string())?;
        let _w2: () = write_text(&handler_log, &"".to_string())?;
        let mut root: __SifrStdlib_sifr_x2elogging_x2eLogger = basicConfig(
            __const_WARNING(),
        );
        root.set_file(&root_log);
        root.info(&"skip".to_string());
        root.error(&"boom".to_string());
        let root_content: String = read_text(&root_log)?;
        root_ok = (root_content == "[ERROR] root: boom\n");
        let mut handler: __SifrStdlib_sifr_x2elogging_x2eFileHandler = __SifrStdlib_sifr_x2elogging_x2eFileHandler::new(
            format!("{}{}", handler_log, ""),
            SifrInt::from_i64(0),
        );
        handler
            .set_formatter(
                &__SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                    "%(levelname)s:%(message)s".to_string(),
                ),
            );
        handler.emit(&"INFO".to_string(), &"demo".to_string(), &"formatted".to_string());
        let handler_content: String = read_text(&handler_log)?;
        handler_ok = (handler_content == "INFO:formatted\n");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    actual.push(root_ok);
    actual.push(handler_ok);
    actual
}
fn collect_safety_actual(base: &str) -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let missing_log: String = {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 20usize);
        __sifr_concat.push_str(base);
        __sifr_concat.push_str("/missing/blocked.log");
        __sifr_concat
    };
    let mut missing_safe: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let mut bad: __SifrStdlib_sifr_x2elogging_x2eLogger = getLogger(
            &"bad".to_string(),
        );
        bad.set_file(&missing_log);
        bad.error(&"should fail".to_string());
        missing_safe = !(exists(&missing_log));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        missing_safe = false;
    }
    actual.push(missing_safe);
    actual
        .push(
            (&__const_INFO() == &SifrInt::from_i64(20))
                && (&__const_WARNING() == &SifrInt::from_i64(30)),
        );
    actual
}
fn collect_cleanup_actual(base: &str) -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut cleanup_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _cleanup: String = run_command(&format!("{}{}", "rm -rf ", base))?;
        cleanup_ok = !(exists(base));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
        cleanup_ok = !(exists(base));
    }
    actual.push(cleanup_ok);
    actual
}
fn append_all(target: &mut Vec<bool>, values: &[bool]) {
    for value in values.iter().copied() {
        target.push(value);
    }
}
fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    let base: String = mktemp_path(&"sifr_logging_logging_demo_".to_string());
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _mk: String = run_command(&format!("{}{}", "mkdir -p ", base))?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    append_all(&mut actual, &collect_logger_actual(&base));
    append_all(&mut actual, &collect_root_and_handler_actual(&base));
    append_all(&mut actual, &collect_safety_actual(&base));
    append_all(&mut actual, &collect_cleanup_actual(&base));
    assert_bool_vector_eq(&actual, &expected);
    println!("logging logging parity demo: pass");
}
