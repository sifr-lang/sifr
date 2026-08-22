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
    pub use ::rust_decimal::Decimal;
    pub use ::bigdecimal::BigDecimal;
    pub fn _encoding_is_supported_impl(label: &String) -> bool {
        ::sifr_stdlib::encoding::encoding_is_supported(label)
    }
    pub fn _encoding_canonical_label_impl(label: &String) -> Result<String, ParseError> {
        ::sifr_stdlib::encoding::encoding_canonical_label(label)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ParseError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _encoding_decode_text_impl(
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
    pub fn _encoding_decode_recoveries_impl(
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
    pub fn _encoding_decode_incremental_text_impl(
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
    pub fn _encoding_decode_incremental_recoveries_impl(
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
    pub fn _encoding_decode_incremental_pending_impl(
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
    pub fn _encoding_encode_bytes_impl(
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
    pub fn _encoding_encode_recoveries_impl(
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
    pub fn json_load_tokens(text: &String) -> Result<Vec<String>, JSONDecodeError> {
        ::sifr_stdlib::json::json_load_tokens(text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| JSONDecodeError {
                message: __sifr_bridge_error.message().to_string(),
                line: __sifr_bridge_error.line() as i64,
                column: __sifr_bridge_error.column() as i64,
            })
    }
    pub fn json_validate_integer_digit_limits(text: &String) -> Result<(), JsonLimitError> {
        ::sifr_stdlib::json::json_validate_integer_digit_limits(text)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| JsonLimitError {
                message: __sifr_bridge_error.message().to_string(),
                limit: __sifr_bridge_error.limit() as i64,
            })
    }
    pub fn json_dump_tokens(tokens: &Vec<String>) -> String {
        ::sifr_stdlib::json::json_dump_tokens(tokens)
    }
    pub fn json_dump_tokens_exact(tokens: &Vec<String>) -> String {
        ::sifr_stdlib::json::json_dump_tokens_exact(tokens)
    }
    pub fn json_dump_tokens_string_ints(tokens: &Vec<String>) -> String {
        ::sifr_stdlib::json::json_dump_tokens_string_ints(tokens)
    }
    pub fn json_dump_tokens_web(
        tokens: &Vec<String>,
    ) -> Result<String, JsonIntegerRangeError> {
        ::sifr_stdlib::json::json_dump_tokens_web(tokens)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| JsonIntegerRangeError {
                message: __sifr_bridge_error.message().to_string(),
                path: __sifr_bridge_error.path().to_string(),
                profile: __sifr_bridge_error.profile().to_string(),
            })
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
                __sifr_concat.push_str((label).as_str());
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        pub name: String,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
        pub fn new(name: String) -> Self {
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
    pub struct __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
        pub text: String,
        pub recoveries: Vec<String>,
    }
    impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
        pub fn new(text: String, recoveries: Vec<String>) -> Self {
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
        pub fn get_text(&self) -> String {
            {
                let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
                __sifr_concat.push_str((self.text.clone()).as_str());
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
        pub fn decode(
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
    pub fn _encoding_is_supported(label: &String) -> bool {
        _encoding_is_supported_impl(label)
    }
    pub fn _encoding_canonical_label(
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
    pub fn _encoding_decode_text(
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
    pub fn _encoding_decode_recoveries(
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
    pub fn _encoding_decode_outcome(
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
    pub fn _encoding_decode_incremental_outcome(
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
    pub fn _encoding_decode_incremental_pending(
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
    pub fn _encoding_encode_bytes(
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
    pub fn _encoding_encode_recoveries(
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
    pub fn _encoding_encode_outcome(
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
    pub fn encoding(label: &String) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new((label).clone())
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
                __sifr_concat.push_str((errors.name.clone()).as_str());
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
                __sifr_concat.push_str((errors.name.clone()).as_str());
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
    pub fn decode(
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
    pub fn encode_outcome(
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
    pub fn encode(
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
        pub fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
            let _ = offset;
            let _ = whence;
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eIOBase {
        pub fn tell(&self) -> Result<i64, IOError> {
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
            Ok(())
        }
    }
    impl __SifrIoFileHandle {
        pub fn read(&self) -> Result<String, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !(self.readable()) {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn write(&self, data: &String) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !(self.writable()) {
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
            if !(self.readable()) {
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
            if !(self.readable()) {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_readlines(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !(self.readable()) {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read_bytes(&self._handle)
        }
    }
    impl __SifrIoFileHandle {
        pub fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !(self.writable()) {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self._handle, data)
        }
    }
    impl __SifrIoFileHandle {
        pub fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
            let _ = offset;
            let _ = whence;
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrIoFileHandle {
        pub fn tell(&self) -> Result<i64, IOError> {
            Err(IOError::new(_unsupported_seek_tell_error()))
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
            false
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
            Ok(())
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn read_bytes(&self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
            let _ = size;
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !(self.readable()) {
                return Err(IOError::new("stream is not readable".to_string()));
            }
            file_read_bytes(&self._handle)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !(self.writable()) {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self._handle, data)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
            let _ = offset;
            let _ = whence;
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn tell(&self) -> Result<i64, IOError> {
            Err(IOError::new(_unsupported_seek_tell_error()))
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
            false
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
                let data: Vec<u8> = (self._binary.read_bytes(None))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                        __e,
                    ))?;
                let text: String = (decode(
                    &data,
                    &self._encoding,
                    &Some((self._decode_errors.clone()).clone()),
                ))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eDecodeError1_x3a0(
                        __e,
                    ))?;
                return Ok(Ok(text));
                unreachable!("sifr try/except return capture fell through");
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
                            return Err(IOError::new(e.message.clone()));
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
                                    __sifr_concat.push_str((e.message.clone()).as_str());
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
        pub fn write(&self, text: &String) -> Result<(), IOError> {
            let __sifr_try_res: Result<
                Result<(), IOError>,
                __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0,
            > = (|| {
                let data: Vec<u8> = (encode(
                    text,
                    &self._encoding,
                    &Some((self._encode_errors.clone()).clone()),
                ))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a0(
                        __e,
                    ))?;
                let result: () = (self._binary.write_bytes(&data))
                    .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a238_x3a5_x3aclass25_x3asifr_x2eencoding_x2eEncodeError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                        __e,
                    ))?;
                return Ok(Ok(()));
                unreachable!("sifr try/except return capture fell through");
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
                            return Err(IOError::new(e.message.clone()));
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
                                    __sifr_concat.push_str((e.message.clone()).as_str());
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
        pub fn write(&self, text: &String) -> Result<(), IOError> {
            let _ = (text).clone();
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
        pub _cursor: i64,
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn new(initial: String) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(
                    initial.len() + 0usize,
                );
                __sifr_concat.push_str((initial).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: i64 = 0_i64;
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
        pub fn read(&mut self, size: Option<i64>) -> Result<String, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let start: i64 = self._cursor;
            let mut end: i64 = self._buffer.chars().count() as i64;
            if let Some(size) = size {
                let maybe_size: i64 = size;
                if maybe_size >= (0_i64) {
                    let requested: i64 = start + maybe_size;
                    if requested < end {
                        end = requested;
                    }
                }
            }
            let piece: String = {
                let _slice_src = &self._buffer.clone();
                let _slice_len_i64 = _slice_src.chars().count() as i64;
                let _slice_start_i64 = if start < 0 {
                    (_slice_len_i64 + start).max(0)
                } else {
                    start.min(_slice_len_i64)
                };
                let _slice_stop_i64 = if end < 0 {
                    (_slice_len_i64 + end).max(0)
                } else {
                    end.min(_slice_len_i64)
                };
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize),
                )
            };
            self._cursor = end;
            Ok(piece)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn write(&mut self, data: &String) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let left: String = {
                let _slice_src = &self._buffer.clone();
                let _slice_len_i64 = _slice_src.chars().count() as i64;
                let _slice_start_i64 = 0;
                let _slice_stop_i64 = if self._cursor < 0 {
                    (_slice_len_i64 + self._cursor).max(0)
                } else {
                    self._cursor.min(_slice_len_i64)
                };
                String::from_iter(
                    _slice_src
                        .chars()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize),
                )
            };
            let tail_start: i64 = self._cursor + (data.chars().count() as i64);
            let mut right: String = "".to_string();
            if (tail_start < (self._buffer.chars().count() as i64)) {
                right = {
                    let _slice_src = &self._buffer.clone();
                    let _slice_len_i64 = _slice_src.chars().count() as i64;
                    let _slice_start_i64 = if tail_start < 0 {
                        (_slice_len_i64 + tail_start).max(0)
                    } else {
                        tail_start.min(_slice_len_i64)
                    };
                    let _slice_stop_i64 = _slice_len_i64;
                    String::from_iter(
                        _slice_src
                            .chars()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize),
                    )
                };
            }
            self._buffer = {
                let mut __sifr_concat: String = String::with_capacity(
                    (left.len() + data.len()) + right.len(),
                );
                __sifr_concat.push_str((left).as_str());
                __sifr_concat.push_str((data).as_str());
                __sifr_concat.push_str((right).as_str());
                __sifr_concat
            };
            self._cursor += data.chars().count() as i64;
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn getvalue(&self) -> String {
            self._buffer.clone()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let mut origin: i64 = 0_i64;
            if whence == (0_i64) {
                origin = 0_i64;
            } else {
                if whence == (1_i64) {
                    origin = self._cursor;
                } else {
                    if whence == (2_i64) {
                        origin = self._buffer.chars().count() as i64;
                    } else {
                        return Err(IOError::new(_invalid_whence_error(whence)));
                    }
                }
            }
            let mut next_pos: i64 = origin + offset;
            if next_pos < (0_i64) {
                return Err(IOError::new(_negative_seek_error(next_pos)));
            }
            let end: i64 = self._buffer.chars().count() as i64;
            if next_pos > end {
                next_pos = end;
            }
            self._cursor = next_pos;
            Ok(self._cursor)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eStringIO {
        pub fn tell(&self) -> Result<i64, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(self._cursor)
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
        pub _cursor: i64,
        pub _closed: bool,
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn new(initial: Vec<u8>) -> Self {
            let __sifr_field_init_0: Vec<u8> = initial;
            let __sifr_field_init_1: i64 = 0_i64;
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
        pub fn read_bytes(&mut self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let start: i64 = self._cursor;
            let mut end: i64 = self._buffer.len() as i64;
            if let Some(size) = size {
                let maybe_size: i64 = size;
                if maybe_size >= (0_i64) {
                    let requested: i64 = start + maybe_size;
                    if requested < end {
                        end = requested;
                    }
                }
            }
            let chunk: Vec<u8> = {
                let _slice_src = &self._buffer.clone();
                let _slice_len_i64 = _slice_src.len() as i64;
                let _slice_start_i64 = if start < 0 {
                    (_slice_len_i64 + start).max(0)
                } else {
                    start.min(_slice_len_i64)
                };
                let _slice_stop_i64 = if end < 0 {
                    (_slice_len_i64 + end).max(0)
                } else {
                    end.min(_slice_len_i64)
                };
                Vec::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                        .cloned(),
                )
            };
            self._cursor = end;
            Ok(chunk)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if (self._cursor == (self._buffer.len() as i64)) {
                self._buffer = {
                    let mut __v = (self._buffer.clone()).clone();
                    __v.extend((data).iter().cloned());
                    __v
                };
                self._cursor += data.len() as i64;
                return Ok(());
            }
            let left: Vec<u8> = {
                let _slice_src = &self._buffer.clone();
                let _slice_len_i64 = _slice_src.len() as i64;
                let _slice_start_i64 = 0;
                let _slice_stop_i64 = if self._cursor < 0 {
                    (_slice_len_i64 + self._cursor).max(0)
                } else {
                    self._cursor.min(_slice_len_i64)
                };
                Vec::from_iter(
                    _slice_src
                        .iter()
                        .skip(_slice_start_i64 as usize)
                        .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                        .cloned(),
                )
            };
            let tail_start: i64 = self._cursor + (data.len() as i64);
            let mut right: Vec<u8> = vec![];
            if (tail_start < (self._buffer.len() as i64)) {
                right = {
                    let _slice_src = &self._buffer.clone();
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = if tail_start < 0 {
                        (_slice_len_i64 + tail_start).max(0)
                    } else {
                        tail_start.min(_slice_len_i64)
                    };
                    let _slice_stop_i64 = _slice_len_i64;
                    Vec::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .cloned(),
                    )
                };
            }
            self._buffer = {
                let mut __v = ({
                    let mut __v = (left).clone();
                    __v.extend((data).iter().cloned());
                    __v
                })
                    .clone();
                __v.extend((right).iter().cloned());
                __v
            };
            self._cursor += data.len() as i64;
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn getvalue(&self) -> Vec<u8> {
            self._buffer.clone()
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            let mut origin: i64 = 0_i64;
            if whence == (0_i64) {
                origin = 0_i64;
            } else {
                if whence == (1_i64) {
                    origin = self._cursor;
                } else {
                    if whence == (2_i64) {
                        origin = self._buffer.len() as i64;
                    } else {
                        return Err(IOError::new(_invalid_whence_error(whence)));
                    }
                }
            }
            let mut next_pos: i64 = origin + offset;
            if next_pos < (0_i64) {
                return Err(IOError::new(_negative_seek_error(next_pos)));
            }
            let end: i64 = self._buffer.len() as i64;
            if next_pos > end {
                next_pos = end;
            }
            self._cursor = next_pos;
            Ok(self._cursor)
        }
    }
    impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
        pub fn tell(&self) -> Result<i64, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            Ok(self._cursor)
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
    pub fn _invalid_whence_error(whence: i64) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
            __sifr_concat.push_str("invalid whence: ");
            __sifr_concat.push_str((format!("{}", whence)).as_str());
            __sifr_concat
        }
    }
    pub fn _negative_seek_error(offset: i64) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
            __sifr_concat.push_str("negative seek position: ");
            __sifr_concat.push_str((format!("{}", offset)).as_str());
            __sifr_concat
        }
    }
    pub fn _unsupported_seek_tell_error() -> String {
        "seek/tell is unsupported for this stream".to_string()
    }
    pub fn _mode_is_readable(mode: &String) -> bool {
        mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
    }
    pub fn _mode_is_writable(mode: &String) -> bool {
        (mode.contains(&"w".to_string()) || mode.contains(&"a".to_string()))
            || mode.contains(&"+".to_string())
    }
    pub fn _text_binary_mode(mode: &String) -> Result<String, IOError> {
        if mode.contains(&"b".to_string()) {
            return Err(
                IOError::new("open_text requires a text mode without \'b\'".to_string()),
            );
        }
        if ((mode).as_str() == "r") || ((mode).as_str() == "rt") {
            return Ok("rb".to_string());
        }
        if ((mode).as_str() == "w") || ((mode).as_str() == "wt") {
            return Ok("wb".to_string());
        }
        if ((mode).as_str() == "a") || ((mode).as_str() == "at") {
            return Ok("ab".to_string());
        }
        Err(
            IOError::new({
                let mut __sifr_concat: String = String::with_capacity(19usize + mode.len());
                __sifr_concat.push_str("invalid text mode: ");
                __sifr_concat.push_str((mode).as_str());
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
    pub fn open(path: &String, mode: &String) -> Result<__SifrIoFileHandle, IOError> {
        let __sifr_try_res: Result<Result<__SifrIoFileHandle, IOError>, IOError> = (|| {
            let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
            return Ok(Ok(__SifrIoFileHandle::new(handle, (mode).clone())));
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
    pub fn open_binary(
        path: &String,
        mode: &String,
    ) -> Result<__SifrIoBinaryFileHandle, IOError> {
        if !(mode.contains(&"b".to_string())) {
            return Err(IOError::new("open_binary requires binary mode".to_string()));
        }
        let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
            let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
            return Ok(Ok(__SifrIoBinaryFileHandle::new(handle, (mode).clone())));
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
    pub fn open_text(
        path: &String,
        mode: &String,
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
            return Ok(
                Ok(
                    __SifrIoTextFileHandle::new(
                        binary,
                        text_encoding,
                        decode_errors,
                        encode_errors,
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
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message.clone()));
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0 {
        __SifrUnionVariant_4_x3aatom4_x3abool(bool),
        __SifrUnionVariant_4_x3aatom3_x3aint(i64),
        __SifrUnionVariant_4_x3aatom5_x3afloat(f64),
        __SifrUnionVariant_4_x3aatom3_x3astr(String),
        __SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0(
            __SifrStdlib_sifr_x2ejson_x2eJsonValue,
        ),
        __SifrUnionVariant_4_x3aatom7_x3adecimal(Decimal),
        __SifrUnionVariant_4_x3aatom10_x3abigdecimal(BigDecimal),
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom4_x3abool(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom3_x3aint(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom5_x3afloat(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom3_x3astr(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom7_x3adecimal(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom10_x3abigdecimal(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub kind: String,
        pub bool_value: Option<bool>,
        pub int_value: Option<i64>,
        pub float_value: Option<f64>,
        pub str_value: Option<String>,
        pub array_items: Box<Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue>>,
        pub object_items: Box<Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)>>,
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn new(
            kind: String,
            bool_value: Option<bool>,
            int_value: Option<i64>,
            float_value: Option<f64>,
            str_value: Option<String>,
        ) -> Self {
            let __sifr_field_init_0: String = kind;
            let __sifr_field_init_1: Option<bool> = bool_value;
            let __sifr_field_init_2: Option<i64> = int_value;
            let __sifr_field_init_3: Option<f64> = float_value;
            let __sifr_field_init_4: Option<String> = str_value;
            let __sifr_field_init_5: Box<Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue>> = Box::default();
            let __sifr_field_init_6: Box<
                Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)>,
            > = Box::default();
            Self {
                kind: __sifr_field_init_0,
                bool_value: __sifr_field_init_1,
                int_value: __sifr_field_init_2,
                float_value: __sifr_field_init_3,
                str_value: __sifr_field_init_4,
                array_items: __sifr_field_init_5,
                object_items: __sifr_field_init_6,
            }
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn is_null(&self) -> bool {
            (self.kind.clone() == "null")
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn is_bool(&self) -> bool {
            (self.kind.clone() == "bool")
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn is_int(&self) -> bool {
            (self.kind.clone() == "int")
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn is_float(&self) -> bool {
            (self.kind.clone() == "float")
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn is_str(&self) -> bool {
            (self.kind.clone() == "str")
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn is_array(&self) -> bool {
            (self.kind.clone() == "array")
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn is_object(&self) -> bool {
            (self.kind.clone() == "object")
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn as_bool(&self) -> Option<bool> {
            self.bool_value
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn as_int(&self) -> Option<i64> {
            self.int_value
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn as_float(&self) -> Option<f64> {
            self.float_value
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn as_str(&self) -> Option<String> {
            self.str_value.clone()
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn as_array(&self) -> Option<Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue>> {
            if !(self.is_array()) {
                return None;
            }
            let mut result: Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue> = vec![];
            for item in (self.array_items).as_ref().clone().iter().cloned() {
                result.push(item.clone());
            }
            Some(result)
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn as_object(
            &self,
        ) -> Option<Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)>> {
            if !(self.is_object()) {
                return None;
            }
            let mut result: Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)> = vec![];
            for (key, value) in (self.object_items).as_ref().clone().iter().cloned() {
                result.push(((key).clone(), (value).clone()));
            }
            Some(result)
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn at(&self, index: i64) -> Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
            if !(self.is_array()) {
                return None;
            }
            if (index < (0_i64)) || (index >= (self.array_items.len() as i64)) {
                return None;
            }
            let value: Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> = Some(
                (self.array_items).as_ref().clone()[index as usize].clone(),
            );
            value
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn get(&self, key: &String) -> Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
            if !(self.is_object()) {
                return None;
            }
            for (item_key, item_value) in (self.object_items)
                .as_ref()
                .clone()
                .iter()
                .cloned()
            {
                if item_key == *key {
                    return Some(item_value);
                }
            }
            None
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn keys(&self) -> Vec<String> {
            let mut result: Vec<String> = vec![];
            if !(self.is_object()) {
                return result;
            }
            for (item_key, _item_value) in (self.object_items)
                .as_ref()
                .clone()
                .iter()
                .cloned()
            {
                result.push(item_key.clone());
            }
            result
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn values(&self) -> Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
            let mut result: Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue> = vec![];
            if !(self.is_object()) {
                return result;
            }
            for (_item_key, item_value) in (self.object_items)
                .as_ref()
                .clone()
                .iter()
                .cloned()
            {
                result.push(item_value.clone());
            }
            result
        }
    }
    impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        pub fn items(&self) -> Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)> {
            if !(self.is_object()) {
                return vec![];
            }
            let mut result: Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)> = vec![];
            for (key, value) in (self.object_items).as_ref().clone().iter().cloned() {
                result.push(((key).clone(), (value).clone()));
            }
            result
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(
                f, "{}", dumps(&
                __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0((self)
                .clone()))
            )
        }
    }
    pub fn from_bool(value: bool) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        let bool_value: Option<bool> = Some(value);
        __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
            "bool".to_string(),
            bool_value,
            None,
            None,
            None,
        )
    }
    pub fn from_int(value: i64) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        let int_value: Option<i64> = Some(value);
        __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
            "int".to_string(),
            None,
            int_value,
            None,
            None,
        )
    }
    pub fn from_float(value: f64) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        let float_value: Option<f64> = Some(value);
        __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
            "float".to_string(),
            None,
            None,
            float_value,
            None,
        )
    }
    pub fn from_str(value: &String) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
        let str_value: Option<String> = Some({
            let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
            __sifr_concat.push_str((value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        });
        __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
            "str".to_string(),
            None,
            None,
            None,
            str_value,
        )
    }
    pub fn _json_append_tokens(
        mut tokens: Vec<String>,
        value: &__SifrStdlib_sifr_x2ejson_x2eJsonValue,
    ) -> Vec<String> {
        tokens.push(format!("{}{}", value.kind.clone(), ""));
        if (value.kind.clone() == "bool") {
            let bool_value: Option<bool> = value.bool_value;
            if bool_value.is_none() {
                tokens.push("false".to_string());
            } else {
                if let Some(bool_value) = bool_value {
                    tokens.push(format!("{}", bool_value).to_lowercase());
                }
            }
        } else {
            if (value.kind.clone() == "int") {
                let int_value: Option<i64> = value.int_value;
                if int_value.is_none() {
                    tokens.push("0".to_string());
                } else {
                    if let Some(int_value) = int_value {
                        tokens.push(format!("{}", int_value));
                    }
                }
            } else {
                if (value.kind.clone() == "float") {
                    let float_value: Option<f64> = value.float_value;
                    if float_value.is_none() {
                        tokens.push("0.0".to_string());
                    } else {
                        if let Some(float_value) = float_value {
                            tokens.push(format!("{}", float_value));
                        }
                    }
                } else {
                    if (value.kind.clone() == "str") {
                        let str_value: Option<String> = value.as_str();
                        if str_value.is_none() {
                            tokens.push("".to_string());
                        } else {
                            if let Some(str_value) = str_value {
                                tokens.push(str_value.clone());
                            }
                        }
                    } else {
                        if (value.kind.clone() == "array") {
                            tokens.push(format!("{}", value.array_items.len() as i64));
                            for item in (value.array_items).as_ref().clone().iter().cloned()
                            {
                                tokens = _json_append_tokens(tokens, &item);
                            }
                        } else {
                            if (value.kind.clone() == "object") {
                                tokens.push(format!("{}", value.object_items.len() as i64));
                                for (key, item_value) in (value.object_items)
                                    .as_ref()
                                    .clone()
                                    .iter()
                                    .cloned()
                                {
                                    tokens.push(key.clone());
                                    tokens = _json_append_tokens(tokens, &item_value);
                                }
                            }
                        }
                    }
                }
            }
        }
        tokens
    }
    pub fn _json_bridge_tokens(
        value: &__SifrStdlib_sifr_x2ejson_x2eJsonValue,
    ) -> Vec<String> {
        let mut tokens: Vec<String> = vec![];
        _json_append_tokens(tokens, value)
    }
    pub fn dumps(
        value: &__SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0,
    ) -> String {
        if let __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0(
            value,
        ) = value {
            return json_dump_tokens(&_json_bridge_tokens(value));
        } else {
            if let __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom4_x3abool(
                value,
            ) = value {
                return json_dump_tokens(&_json_bridge_tokens(&from_bool((value).clone())));
            } else {
                if let __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom3_x3aint(
                    value,
                ) = value {
                    return json_dump_tokens(
                        &_json_bridge_tokens(&from_int((value).clone())),
                    );
                } else {
                    if let __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom5_x3afloat(
                        value,
                    ) = value {
                        return json_dump_tokens(
                            &_json_bridge_tokens(&from_float((value).clone())),
                        );
                    } else {
                        return json_dump_tokens(
                            &_json_bridge_tokens(&from_str(&format!("{}", value))),
                        );
                    }
                }
            }
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JSONDecodeError {
        pub message: String,
        pub line: i64,
        pub column: i64,
    }
    impl JSONDecodeError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                line: 0,
                column: 0,
            }
        }
    }
    impl ::std::fmt::Display for JSONDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JSONDecodeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JsonIntegerRangeError {
        pub message: String,
        pub path: String,
        pub profile: String,
    }
    impl JsonIntegerRangeError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                path: String::new(),
                profile: String::new(),
            }
        }
    }
    impl ::std::fmt::Display for JsonIntegerRangeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JsonIntegerRangeError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JsonLimitError {
        pub message: String,
        pub limit: i64,
    }
    impl JsonLimitError {
        pub fn new(message: String) -> Self {
            Self { message, limit: 0 }
        }
    }
    impl ::std::fmt::Display for JsonLimitError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for JsonLimitError {}
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
    impl From<JSONDecodeError> for Error {
        fn from(err: JSONDecodeError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<JsonIntegerRangeError> for Error {
        fn from(err: JsonIntegerRangeError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<JsonLimitError> for Error {
        fn from(err: JsonLimitError) -> Self {
            Self::new(err.message)
        }
    }
}
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::JSONDecodeError;
pub use __sifr_project_nominals::JsonIntegerRangeError;
pub use __sifr_project_nominals::JsonLimitError;
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
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ejson_x2eJsonValue;
use ::rust_decimal::Decimal;
use ::bigdecimal::BigDecimal;
fn run_command(cmd: &String) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn env_get(key: &String) -> Option<String> {
    ::sifr_stdlib::sys::env_get(key)
}
fn env_set(key: &String, value: &String) {
    ::sifr_stdlib::sys::env_set(key, value);
}
fn env_unset(key: &String) {
    ::sifr_stdlib::sys::env_unset(key);
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
fn getenv_opt(key: &String) -> Option<String> {
    env_get(key)
}
fn setenv(key: &String, value: &String) {
    env_set(key, value);
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
fn _invalid_whence_error(whence: i64) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("invalid whence: ");
        __sifr_concat.push_str((format!("{}", whence)).as_str());
        __sifr_concat
    }
}
fn _negative_seek_error(offset: i64) -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
        __sifr_concat.push_str("negative seek position: ");
        __sifr_concat.push_str((format!("{}", offset)).as_str());
        __sifr_concat
    }
}
fn _unsupported_seek_tell_error() -> String {
    "seek/tell is unsupported for this stream".to_string()
}
fn _mode_is_readable(mode: &String) -> bool {
    mode.contains(&"r".to_string()) || mode.contains(&"+".to_string())
}
fn _mode_is_writable(mode: &String) -> bool {
    (mode.contains(&"w".to_string()) || mode.contains(&"a".to_string()))
        || mode.contains(&"+".to_string())
}
fn _text_binary_mode(mode: &String) -> Result<String, IOError> {
    if mode.contains(&"b".to_string()) {
        return Err(
            IOError::new("open_text requires a text mode without \'b\'".to_string()),
        );
    }
    if ((mode).as_str() == "r") || ((mode).as_str() == "rt") {
        return Ok("rb".to_string());
    }
    if ((mode).as_str() == "w") || ((mode).as_str() == "wt") {
        return Ok("wb".to_string());
    }
    if ((mode).as_str() == "a") || ((mode).as_str() == "at") {
        return Ok("ab".to_string());
    }
    Err(
        IOError::new({
            let mut __sifr_concat: String = String::with_capacity(19usize + mode.len());
            __sifr_concat.push_str("invalid text mode: ");
            __sifr_concat.push_str((mode).as_str());
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
fn open(path: &String, mode: &String) -> Result<__SifrIoFileHandle, IOError> {
    let __sifr_try_res: Result<Result<__SifrIoFileHandle, IOError>, IOError> = (|| {
        let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
        return Ok(Ok(__SifrIoFileHandle::new(handle, (mode).clone())));
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
fn open_binary(
    path: &String,
    mode: &String,
) -> Result<__SifrIoBinaryFileHandle, IOError> {
    if !(mode.contains(&"b".to_string())) {
        return Err(IOError::new("open_binary requires binary mode".to_string()));
    }
    let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
        let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
        return Ok(Ok(__SifrIoBinaryFileHandle::new(handle, (mode).clone())));
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
fn open_text(
    path: &String,
    mode: &String,
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
        return Ok(
            Ok(
                __SifrIoTextFileHandle::new(
                    binary,
                    text_encoding,
                    decode_errors,
                    encode_errors,
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
            let e = __sifr_try_err.clone();
            return Err(IOError::new(e.message.clone()));
        }
    }
}
fn json_load_tokens(text: &String) -> Result<Vec<String>, JSONDecodeError> {
    ::sifr_stdlib::json::json_load_tokens(text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| JSONDecodeError {
            message: __sifr_bridge_error.message().to_string(),
            line: __sifr_bridge_error.line() as i64,
            column: __sifr_bridge_error.column() as i64,
        })
}
fn json_validate_integer_digit_limits(text: &String) -> Result<(), JsonLimitError> {
    ::sifr_stdlib::json::json_validate_integer_digit_limits(text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| JsonLimitError {
            message: __sifr_bridge_error.message().to_string(),
            limit: __sifr_bridge_error.limit() as i64,
        })
}
fn json_dump_tokens(tokens: &Vec<String>) -> String {
    ::sifr_stdlib::json::json_dump_tokens(tokens)
}
fn json_dump_tokens_exact(tokens: &Vec<String>) -> String {
    ::sifr_stdlib::json::json_dump_tokens_exact(tokens)
}
fn json_dump_tokens_string_ints(tokens: &Vec<String>) -> String {
    ::sifr_stdlib::json::json_dump_tokens_string_ints(tokens)
}
fn json_dump_tokens_web(tokens: &Vec<String>) -> Result<String, JsonIntegerRangeError> {
    ::sifr_stdlib::json::json_dump_tokens_web(tokens)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| JsonIntegerRangeError {
            message: __sifr_bridge_error.message().to_string(),
            path: __sifr_bridge_error.path().to_string(),
            profile: __sifr_bridge_error.profile().to_string(),
        })
}
#[derive(Debug, Clone)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0 {
    __SifrUnionVariant_5_x3aclass15_x3aJSONDecodeError1_x3a0(JSONDecodeError),
    __SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(ParseError),
}
impl From<JSONDecodeError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0 {
    fn from(value: JSONDecodeError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aJSONDecodeError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aJSONDecodeError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0 {
    __SifrUnionVariant_4_x3aatom4_x3abool(bool),
    __SifrUnionVariant_4_x3aatom3_x3aint(i64),
    __SifrUnionVariant_4_x3aatom5_x3afloat(f64),
    __SifrUnionVariant_4_x3aatom3_x3astr(String),
    __SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0(
        __SifrStdlib_sifr_x2ejson_x2eJsonValue,
    ),
    __SifrUnionVariant_4_x3aatom7_x3adecimal(Decimal),
    __SifrUnionVariant_4_x3aatom10_x3abigdecimal(BigDecimal),
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom4_x3abool(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom3_x3aint(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom5_x3afloat(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom3_x3astr(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom7_x3adecimal(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom10_x3abigdecimal(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
fn from_bool(value: bool) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    let bool_value: Option<bool> = Some(value);
    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
        "bool".to_string(),
        bool_value,
        None,
        None,
        None,
    )
}
fn from_int(value: i64) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    let int_value: Option<i64> = Some(value);
    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
        "int".to_string(),
        None,
        int_value,
        None,
        None,
    )
}
fn from_float(value: f64) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    let float_value: Option<f64> = Some(value);
    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
        "float".to_string(),
        None,
        None,
        float_value,
        None,
    )
}
fn from_str(value: &String) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    let str_value: Option<String> = Some({
        let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
        __sifr_concat.push_str((value).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    });
    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
        "str".to_string(),
        None,
        None,
        None,
        str_value,
    )
}
fn _json_token_at(tokens: &Vec<String>, index: i64) -> Result<String, JSONDecodeError> {
    let value: Option<String> = {
        let __sifr_index_list = &tokens;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let Some(value) = value else {
        return Err(
            JSONDecodeError::new("JSON bridge payload ended unexpectedly".to_string()),
        );
    };
    Ok({
        let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
        __sifr_concat.push_str((value).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    })
}
fn _json_token_int(tokens: &Vec<String>, index: i64) -> Result<i64, JSONDecodeError> {
    let __sifr_try_res: Result<
        Result<i64, JSONDecodeError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0,
    > = (|| {
        let token: String = (_json_token_at(tokens, index))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aJSONDecodeError1_x3a0(
                __e,
            ))?;
        let parsed: i64 = ((token)
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                __e,
            ))?;
        return Ok(Ok(parsed));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aJSONDecodeError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(JSONDecodeError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let _e = __sifr_try_variant_error.clone();
                    return Err(
                        JSONDecodeError::new(
                            "JSON bridge payload has invalid integer metadata"
                                .to_string(),
                        ),
                    );
                }
            }
        }
    }
}
fn _json_token_float(tokens: &Vec<String>, index: i64) -> Result<f64, JSONDecodeError> {
    let __sifr_try_res: Result<
        Result<f64, JSONDecodeError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0,
    > = (|| {
        let token: String = (_json_token_at(tokens, index))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aJSONDecodeError1_x3a0(
                __e,
            ))?;
        let parsed: f64 = ((token)
            .parse::<f64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                __e,
            ))?;
        return Ok(Ok(parsed));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aJSONDecodeError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(JSONDecodeError::new(e.message.clone()));
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aJSONDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let _e = __sifr_try_variant_error.clone();
                    return Err(
                        JSONDecodeError::new(
                            "JSON bridge payload has invalid float metadata".to_string(),
                        ),
                    );
                }
            }
        }
    }
}
fn _json_decode_bool_token(value: &String) -> Result<bool, JSONDecodeError> {
    if (value).as_str() == "true" {
        return Ok(true);
    }
    if (value).as_str() == "false" {
        return Ok(false);
    }
    Err(
        JSONDecodeError::new("JSON bridge payload has invalid bool metadata".to_string()),
    )
}
fn _json_decode_value_at(
    tokens: &Vec<String>,
    index: i64,
) -> Result<(__SifrStdlib_sifr_x2ejson_x2eJsonValue, i64), JSONDecodeError> {
    let __sifr_try_res: Result<
        Result<(__SifrStdlib_sifr_x2ejson_x2eJsonValue, i64), JSONDecodeError>,
        JSONDecodeError,
    > = (|| {
        let tag: String = _json_token_at(tokens, index)?;
        let payload_index: i64 = index + (1_i64);
        if tag == "null" {
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
                        "null".to_string(),
                        None,
                        None,
                        None,
                        None,
                    ),
                    payload_index,
                )),
            );
        }
        if tag == "bool" {
            let bool_token: String = _json_token_at(tokens, payload_index)?;
            let bool_value: bool = _json_decode_bool_token(&bool_token)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
                        "bool".to_string(),
                        Some(bool_value),
                        None,
                        None,
                        None,
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "int" {
            let int_value: i64 = _json_token_int(tokens, payload_index)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
                        "int".to_string(),
                        None,
                        Some(int_value),
                        None,
                        None,
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "float" {
            let float_value: f64 = _json_token_float(tokens, payload_index)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
                        "float".to_string(),
                        None,
                        None,
                        Some(float_value),
                        None,
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "str" {
            let str_value: String = _json_token_at(tokens, payload_index)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
                        "str".to_string(),
                        None,
                        None,
                        None,
                        Some(str_value),
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "array" {
            let array_count: i64 = _json_token_int(tokens, payload_index)?;
            if array_count < (0_i64) {
                return Err(
                    JSONDecodeError::new(
                        "JSON bridge payload has invalid array length".to_string(),
                    ),
                );
            }
            let mut array_value: __SifrStdlib_sifr_x2ejson_x2eJsonValue = __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
                "array".to_string(),
                None,
                None,
                None,
                None,
            );
            let mut next_index: i64 = payload_index + (1_i64);
            let mut consumed: i64 = 0_i64;
            while consumed < array_count {
                let item_result: (__SifrStdlib_sifr_x2ejson_x2eJsonValue, i64) = _json_decode_value_at(
                    tokens,
                    next_index,
                )?;
                array_value.array_items.push(item_result.0);
                next_index = (item_result).1;
                consumed += 1_i64;
            }
            return Ok(Ok((array_value.clone(), next_index)));
        }
        if tag == "object" {
            let object_count: i64 = _json_token_int(tokens, payload_index)?;
            if object_count < (0_i64) {
                return Err(
                    JSONDecodeError::new(
                        "JSON bridge payload has invalid object length".to_string(),
                    ),
                );
            }
            let mut object_value: __SifrStdlib_sifr_x2ejson_x2eJsonValue = __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
                "object".to_string(),
                None,
                None,
                None,
                None,
            );
            let mut next_index: i64 = payload_index + (1_i64);
            let mut consumed: i64 = 0_i64;
            while consumed < object_count {
                let key: String = _json_token_at(tokens, next_index)?;
                let item_result: (__SifrStdlib_sifr_x2ejson_x2eJsonValue, i64) = _json_decode_value_at(
                    tokens,
                    next_index + (1_i64),
                )?;
                object_value.object_items.push(((key).clone(), item_result.0));
                next_index = (item_result).1;
                consumed += 1_i64;
            }
            return Ok(Ok((object_value.clone(), next_index)));
        }
        return Err(
            JSONDecodeError::new({
                let mut __sifr_concat: String = String::with_capacity(
                    43usize + tag.len(),
                );
                __sifr_concat.push_str("JSON bridge payload has unknown value tag: ");
                __sifr_concat.push_str((tag).as_str());
                __sifr_concat
            }),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(JSONDecodeError::new(e.message.clone()));
        }
    }
}
fn _json_decode_tokens(
    tokens: &Vec<String>,
) -> Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, JSONDecodeError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, JSONDecodeError>,
        JSONDecodeError,
    > = (|| {
        let decoded: (__SifrStdlib_sifr_x2ejson_x2eJsonValue, i64) = _json_decode_value_at(
            tokens,
            0_i64,
        )?;
        if ((decoded).1 != (tokens.len() as i64)) {
            return Err(
                JSONDecodeError::new("JSON bridge payload has trailing data".to_string()),
            );
        }
        return Ok(Ok((decoded).0.clone()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(JSONDecodeError::new(e.message.clone()));
        }
    }
}
fn _json_append_tokens(
    mut tokens: Vec<String>,
    value: &__SifrStdlib_sifr_x2ejson_x2eJsonValue,
) -> Vec<String> {
    tokens.push(format!("{}{}", value.kind.clone(), ""));
    if (value.kind.clone() == "bool") {
        let bool_value: Option<bool> = value.bool_value;
        if bool_value.is_none() {
            tokens.push("false".to_string());
        } else {
            if let Some(bool_value) = bool_value {
                tokens.push(format!("{}", bool_value).to_lowercase());
            }
        }
    } else {
        if (value.kind.clone() == "int") {
            let int_value: Option<i64> = value.int_value;
            if int_value.is_none() {
                tokens.push("0".to_string());
            } else {
                if let Some(int_value) = int_value {
                    tokens.push(format!("{}", int_value));
                }
            }
        } else {
            if (value.kind.clone() == "float") {
                let float_value: Option<f64> = value.float_value;
                if float_value.is_none() {
                    tokens.push("0.0".to_string());
                } else {
                    if let Some(float_value) = float_value {
                        tokens.push(format!("{}", float_value));
                    }
                }
            } else {
                if (value.kind.clone() == "str") {
                    let str_value: Option<String> = value.as_str();
                    if str_value.is_none() {
                        tokens.push("".to_string());
                    } else {
                        if let Some(str_value) = str_value {
                            tokens.push(str_value.clone());
                        }
                    }
                } else {
                    if (value.kind.clone() == "array") {
                        tokens.push(format!("{}", value.array_items.len() as i64));
                        for item in (value.array_items).as_ref().clone().iter().cloned()
                        {
                            tokens = _json_append_tokens(tokens, &item);
                        }
                    } else {
                        if (value.kind.clone() == "object") {
                            tokens.push(format!("{}", value.object_items.len() as i64));
                            for (key, item_value) in (value.object_items)
                                .as_ref()
                                .clone()
                                .iter()
                                .cloned()
                            {
                                tokens.push(key.clone());
                                tokens = _json_append_tokens(tokens, &item_value);
                            }
                        }
                    }
                }
            }
        }
    }
    tokens
}
fn _json_bridge_tokens(value: &__SifrStdlib_sifr_x2ejson_x2eJsonValue) -> Vec<String> {
    let mut tokens: Vec<String> = vec![];
    _json_append_tokens(tokens, value)
}
fn _decode_json(
    s: &String,
) -> Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, JSONDecodeError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, JSONDecodeError>,
        JSONDecodeError,
    > = (|| {
        let tokens: Vec<String> = json_load_tokens(s)?;
        return Ok(_json_decode_tokens(&tokens));
        unreachable!("sifr try/except return capture fell through");
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
fn loads(s: &String) -> Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, JSONDecodeError> {
    _decode_json(s)
}
fn dumps(
    value: &__SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0,
) -> String {
    if let __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0(
        value,
    ) = value {
        return json_dump_tokens(&_json_bridge_tokens(value));
    } else {
        if let __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom4_x3abool(
            value,
        ) = value {
            return json_dump_tokens(&_json_bridge_tokens(&from_bool((value).clone())));
        } else {
            if let __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom3_x3aint(
                value,
            ) = value {
                return json_dump_tokens(
                    &_json_bridge_tokens(&from_int((value).clone())),
                );
            } else {
                if let __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom5_x3afloat(
                    value,
                ) = value {
                    return json_dump_tokens(
                        &_json_bridge_tokens(&from_float((value).clone())),
                    );
                } else {
                    return json_dump_tokens(
                        &_json_bridge_tokens(&from_str(&format!("{}", value))),
                    );
                }
            }
        }
    }
}
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> i64 {
    ::sifr_stdlib::math::floor(x).to_i64_saturating()
}
fn ceil(x: f64) -> i64 {
    ::sifr_stdlib::math::ceil(x).to_i64_saturating()
}
fn log(x: f64) -> f64 {
    ::sifr_stdlib::math::log(x)
}
fn cbrt(x: f64) -> f64 {
    ::sifr_stdlib::math::cbrt(x)
}
fn sin(x: f64) -> f64 {
    ::sifr_stdlib::math::sin(x)
}
fn cos(x: f64) -> f64 {
    ::sifr_stdlib::math::cos(x)
}
fn tan(x: f64) -> f64 {
    ::sifr_stdlib::math::tan(x)
}
fn pow_val(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::pow_val(x, y)
}
fn min_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::min_val(a, b)
}
fn max_val(a: f64, b: f64) -> f64 {
    ::sifr_stdlib::math::max_val(a, b)
}
fn round_val(x: f64) -> i64 {
    ::sifr_stdlib::math::round_val(x).to_i64_saturating()
}
fn asin(x: f64) -> f64 {
    ::sifr_stdlib::math::asin(x)
}
fn acos(x: f64) -> f64 {
    ::sifr_stdlib::math::acos(x)
}
fn atan(x: f64) -> f64 {
    ::sifr_stdlib::math::atan(x)
}
fn atan2(y: f64, x: f64) -> f64 {
    ::sifr_stdlib::math::atan2(y, x)
}
fn sinh(x: f64) -> f64 {
    ::sifr_stdlib::math::sinh(x)
}
fn cosh(x: f64) -> f64 {
    ::sifr_stdlib::math::cosh(x)
}
fn tanh(x: f64) -> f64 {
    ::sifr_stdlib::math::tanh(x)
}
fn log10(x: f64) -> f64 {
    ::sifr_stdlib::math::log10(x)
}
fn log2(x: f64) -> f64 {
    ::sifr_stdlib::math::log2(x)
}
fn exp2(x: f64) -> f64 {
    ::sifr_stdlib::math::exp2(x)
}
fn degrees(x: f64) -> f64 {
    ::sifr_stdlib::math::degrees(x)
}
fn radians(x: f64) -> f64 {
    ::sifr_stdlib::math::radians(x)
}
fn isnan(x: f64) -> bool {
    ::sifr_stdlib::math::isnan(x)
}
fn isinf(x: f64) -> bool {
    ::sifr_stdlib::math::isinf(x)
}
fn trunc(x: f64) -> i64 {
    ::sifr_stdlib::math::trunc(x).to_i64_saturating()
}
fn copysign(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::copysign(x, y)
}
fn signbit(x: f64) -> bool {
    ::sifr_stdlib::math::signbit(x)
}
fn fmod(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmod(x, y)
}
fn remainder(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::remainder(x, y)
}
fn hypot(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::hypot(x, y)
}
fn fma(x: f64, y: f64, z: f64) -> f64 {
    ::sifr_stdlib::math::fma(x, y, z)
}
fn fmax(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmax(x, y)
}
fn fmin(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::fmin(x, y)
}
fn exp(x: f64) -> f64 {
    ::sifr_stdlib::math::exp(x)
}
fn expm1(x: f64) -> f64 {
    ::sifr_stdlib::math::expm1(x)
}
fn log1p(x: f64) -> f64 {
    ::sifr_stdlib::math::log1p(x)
}
fn fabs(x: f64) -> f64 {
    ::sifr_stdlib::math::fabs(x)
}
fn isfinite(x: f64) -> bool {
    ::sifr_stdlib::math::isfinite(x)
}
fn isnormal(x: f64) -> bool {
    ::sifr_stdlib::math::isnormal(x)
}
fn issubnormal(x: f64) -> bool {
    ::sifr_stdlib::math::issubnormal(x)
}
fn acosh(x: f64) -> f64 {
    ::sifr_stdlib::math::acosh(x)
}
fn asinh(x: f64) -> f64 {
    ::sifr_stdlib::math::asinh(x)
}
fn atanh(x: f64) -> f64 {
    ::sifr_stdlib::math::atanh(x)
}
fn isqrt(n: i64) -> i64 {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .to_i64_saturating()
}
fn dist_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::dist(p, q)
}
fn fsum_impl(data: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::fsum(data)
}
fn sumprod_impl(p: Vec<f64>, q: Vec<f64>) -> f64 {
    ::sifr_stdlib::math::sumprod(p, q)
}
fn erf(x: f64) -> f64 {
    ::sifr_stdlib::math::erf(x)
}
fn erfc(x: f64) -> f64 {
    ::sifr_stdlib::math::erfc(x)
}
fn gamma(x: f64) -> f64 {
    ::sifr_stdlib::math::gamma(x)
}
fn lgamma(x: f64) -> f64 {
    ::sifr_stdlib::math::lgamma(x)
}
fn frexp(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::frexp(x)
}
fn ldexp(m: f64, e: i64) -> f64 {
    ::sifr_stdlib::math::ldexp(m, ::sifr_runtime::interop::SifrIntBridge::from(e))
}
fn modf(x: f64) -> Vec<f64> {
    ::sifr_stdlib::math::modf(x)
}
fn nextafter(x: f64, y: f64) -> f64 {
    ::sifr_stdlib::math::nextafter(x, y)
}
fn ulp(x: f64) -> f64 {
    ::sifr_stdlib::math::ulp(x)
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
fn main() {
    println!("=== sifr.io ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _w: () = write_text(
            &"/tmp/sifr_demo.txt".to_string(),
            &"Hello from Sifr!\nLine 2".to_string(),
        )?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("write error: {}", err.message.clone());
    }
    let file_exists: bool = exists(&"/tmp/sifr_demo.txt".to_string());
    println!("File exists: {}", file_exists);
    let __sifr_try_res: Result<(), IOError> = (|| {
        let content: String = read_text(&"/tmp/sifr_demo.txt".to_string())?;
        println!("Content: {}", content);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("read error: {}", err.message.clone());
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let lines: Vec<String> = read_lines(&"/tmp/sifr_demo.txt".to_string())?;
        println!("Line count: {}", lines.len() as i64);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("lines error: {}", err.message.clone());
    }
    println!("=== sifr.json ===");
    let __sifr_try_res: Result<(), JSONDecodeError> = (|| {
        let data: __SifrStdlib_sifr_x2ejson_x2eJsonValue = loads(
            &"{\"language\":\"sifr\",\"version\":1}".to_string(),
        )?;
        println!("Parsed JSON: {}", data);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("json error: {}", err.message.clone());
    }
    println!("=== sifr.env ===");
    let _ = setenv(&"SIFR_DEMO".to_string(), &"active".to_string());
    let val: Option<String> = getenv_opt(&"SIFR_DEMO".to_string());
    if let Some(val) = val {
        println!("SIFR_DEMO = {}", val);
    }
    let missing: Option<String> = getenv_opt(&"SIFR_NONEXISTENT".to_string());
    if missing.is_none() {
        println!("SIFR_NONEXISTENT not set");
    }
    println!("=== sifr.os ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let output: String = run_command(&"echo Sifr OS module works".to_string())?;
        println!("{}", output);
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("os error: {}", err.message.clone());
    }
    println!("=== sifr.math ===");
    println!("sqrt(25.0) = {}", sqrt(25.0_f64));
    println!("floor(3.7) = {}", floor(3.7_f64));
    println!("ceil(3.2) = {}", ceil(3.2_f64));
    println!("pi = {}", PI);
    println!("e = {}", E);
    println!("=== Demo complete ===");
}
