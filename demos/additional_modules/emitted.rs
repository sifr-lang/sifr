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
    pub use ::std::collections::HashMap;
    pub use ::sifr_runtime::SifrInt;
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
            Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
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
    pub fn encoding(label: &String) -> __SifrStdlib_sifr_x2eencoding_x2eEncoding {
        __SifrStdlib_sifr_x2eencoding_x2eEncoding::new((label.clone()).clone())
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
            Ok(())
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
        pub fn write(&self, data: &String) -> Result<(), IOError> {
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
        pub fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
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
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self._handle, data)
        }
    }
    impl __SifrIoFileHandle {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            let _ = offset.clone();
            let _ = whence.clone();
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrIoFileHandle {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
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
        pub fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
            let _ = (size).clone();
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if !self.readable() {
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
            if !self.writable() {
                return Err(IOError::new("stream is not writable".to_string()));
            }
            file_write_bytes(&self._handle, data)
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
            let _ = offset.clone();
            let _ = whence.clone();
            Err(IOError::new(_unsupported_seek_tell_error()))
        }
    }
    impl __SifrIoBinaryFileHandle {
        pub fn tell(&self) -> Result<SifrInt, IOError> {
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
                let data: Vec<u8> = (self._binary.read_bytes(&None))
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
        pub _cursor: SifrInt,
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
        pub fn write(&mut self, data: &String) -> Result<(), IOError> {
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
                __sifr_concat.push_str((left).as_str());
                __sifr_concat.push_str((data).as_str());
                __sifr_concat.push_str((right).as_str());
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
            if (&whence.clone() == &SifrInt::from_i64(0)) {
                origin = SifrInt::from_i64(0);
            } else {
                if (&whence.clone() == &SifrInt::from_i64(1)) {
                    origin = self._cursor.clone();
                } else {
                    if (&whence.clone() == &SifrInt::from_i64(2)) {
                        origin = SifrInt::from(self._buffer.chars().count());
                    } else {
                        return Err(
                            IOError::new(_invalid_whence_error((whence.clone()).clone())),
                        );
                    }
                }
            }
            let mut next_pos: SifrInt = &origin + offset;
            if (&next_pos < &SifrInt::from_i64(0)) {
                return Err(IOError::new(_negative_seek_error((next_pos).clone())));
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
        pub fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
            if self._closed {
                return Err(IOError::new(_closed_stream_error()));
            }
            if (&self._cursor.clone() == &SifrInt::from(self._buffer.len())) {
                self._buffer = {
                    let mut __v = (self._buffer.clone()).clone();
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
                    let mut __v = (left).clone();
                    __v.extend((data).iter().cloned());
                    __v
                })
                    .clone();
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
            if (&whence.clone() == &SifrInt::from_i64(0)) {
                origin = SifrInt::from_i64(0);
            } else {
                if (&whence.clone() == &SifrInt::from_i64(1)) {
                    origin = self._cursor.clone();
                } else {
                    if (&whence.clone() == &SifrInt::from_i64(2)) {
                        origin = SifrInt::from(self._buffer.len());
                    } else {
                        return Err(
                            IOError::new(_invalid_whence_error((whence.clone()).clone())),
                        );
                    }
                }
            }
            let mut next_pos: SifrInt = &origin + offset;
            if (&next_pos < &SifrInt::from_i64(0)) {
                return Err(IOError::new(_negative_seek_error((next_pos).clone())));
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
            __sifr_concat.push_str((format!("{}", whence)).as_str());
            __sifr_concat
        }
    }
    pub fn _negative_seek_error(offset: SifrInt) -> String {
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
            Ok(Ok(__SifrIoFileHandle::new(handle, (mode.clone()).clone())))
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
        if !mode.contains(&"b".to_string()) {
            return Err(IOError::new("open_binary requires binary mode".to_string()));
        }
        let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
            let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
            Ok(Ok(__SifrIoBinaryFileHandle::new(handle, (mode.clone()).clone())))
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
                return Err(IOError::new(e.message.clone()));
            }
        }
    }
    pub fn __const_DEFAULTSECT() -> String {
        "DEFAULT".to_string().to_string()
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct __SifrStdlib_sifr_x2econfigparser_x2eParsingError {
        pub line: SifrInt,
        pub message: String,
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eParsingError {
        pub fn new(line: SifrInt, message: String) -> Self {
            let __sifr_field_init_0: SifrInt = line.clone();
            let __sifr_field_init_1: String = message;
            Self {
                line: __sifr_field_init_0,
                message: __sifr_field_init_1,
            }
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eParsingError {}
    impl ::std::fmt::Debug for __SifrStdlib_sifr_x2econfigparser_x2eParsingError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            f.debug_struct("ParsingError")
                .field("line", &self.line)
                .field("message", &self.message)
                .finish()
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2econfigparser_x2eParsingError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.message)
        }
    }
    impl ::std::error::Error for __SifrStdlib_sifr_x2econfigparser_x2eParsingError {}
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
        pub name: String,
        pub _values: HashMap<String, Option<String>>,
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
        pub fn new(name: String, values: HashMap<String, Option<String>>) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(name.len() + 0usize);
                __sifr_concat.push_str((name).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: HashMap<String, Option<String>> = _copy_values(&values);
            Self {
                name: __sifr_field_init_0,
                _values: __sifr_field_init_1,
            }
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
        pub fn has_option(&self, option: &String) -> bool {
            _has_option_key(&self._values, &_normalize_option(option))
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
        pub fn get(
            &self,
            option: &String,
            fallback: &Option<String>,
            raw: bool,
        ) -> Option<String> {
            let normalized: String = _normalize_option(option);
            if _has_option_key(&self._values, &normalized) {
                let value: Option<String> = _lookup_option(&self._values, &normalized);
                let Some(value) = value else {
                    return None;
                };
                if raw {
                    return Some(value);
                }
                return Some(
                    _resolve_interpolation(&value, &self._values, SifrInt::from_i64(0)),
                );
            }
            _copy_optional_str(fallback)
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
        pub fn options(&self) -> Vec<String> {
            let mut names: Vec<String> = vec![];
            for key in self._values.clone().keys().cloned() {
                names.push(key.clone());
            }
            names
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
        pub fn items(&self) -> Vec<(String, Option<String>)> {
            let mut pairs: Vec<(String, Option<String>)> = vec![];
            for (key, value) in self
                ._values
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                pairs.push(((key).clone(), _copy_optional_str(&value)));
            }
            pairs
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub _defaults: HashMap<String, Option<String>>,
        pub _sections: HashMap<String, HashMap<String, Option<String>>>,
        pub strict: bool,
        pub allow_no_value: bool,
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn new(
            defaults: Option<HashMap<String, Option<String>>>,
            strict: bool,
            allow_no_value: bool,
        ) -> Self {
            let mut defaults_map: HashMap<String, Option<String>> = HashMap::from([]);
            let sections_map: HashMap<String, HashMap<String, Option<String>>> = HashMap::from([]);
            if let Some(defaults) = defaults {
                for (key, value) in defaults
                    .iter()
                    .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                    .collect::<Vec<_>>()
                {
                    let normalized: String = _normalize_option(&key);
                    {
                        let __assign_value = _copy_optional_str(&value);
                        {
                            let __assign_key = normalized.clone();
                            defaults_map.insert(__assign_key, __assign_value);
                        }
                    }
                }
            }
            let __sifr_field_init_0: bool = strict;
            let __sifr_field_init_1: bool = allow_no_value;
            let __sifr_field_init_2: HashMap<String, Option<String>> = defaults_map;
            let __sifr_field_init_3: HashMap<String, HashMap<String, Option<String>>> = sections_map;
            Self {
                strict: __sifr_field_init_0,
                allow_no_value: __sifr_field_init_1,
                _defaults: __sifr_field_init_2,
                _sections: __sifr_field_init_3,
            }
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn defaults(&self) -> HashMap<String, Option<String>> {
            _copy_values(&self._defaults)
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn read_string(
            &mut self,
            text: &String,
        ) -> Result<(), __SifrStdlib_sifr_x2econfigparser_x2eParsingError> {
            let mut current_section: String = "".to_string();
            let default_section: String = _default_section();
            for (line_no, raw_line) in Box::new(
                (text.split('\n').map(|s| s.to_string()).collect::<Vec<String>>())
                    .into_iter()
                    .enumerate()
                    .map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(1), __pair.1)),
            ) {
                let line: String = raw_line.trim().to_string();
                if ((line == "") || line.starts_with("#")) || line.starts_with(";") {
                    continue;
                }
                if line.starts_with("[") && line.ends_with("]") {
                    let section_name: String = line
                        .chars()
                        .skip(::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1))))
                        .take(
                            (::sifr_runtime::to_usize_proven(
                                &(SifrInt::from(line.chars().count()) - SifrInt::from_i64(1)),
                            )) - (::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1)))),
                        )
                        .collect::<String>()
                        .trim()
                        .to_string();
                    if (section_name == "") {
                        return Err(
                            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                                (line_no).clone(),
                                "section name is empty".to_string(),
                            ),
                        );
                    }
                    if (section_name == default_section) {
                        current_section = _default_section();
                        continue;
                    }
                    if self.strict && (self._sections).contains_key(&(section_name)) {
                        return Err(
                            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                                (line_no).clone(),
                                format!("{}{}", "duplicate section: ", section_name),
                            ),
                        );
                    }
                    current_section = {
                        let mut __sifr_concat: String = String::with_capacity(
                            section_name.len() + 0usize,
                        );
                        __sifr_concat.push_str((section_name).as_str());
                        __sifr_concat.push_str("");
                        __sifr_concat
                    };
                    if !(self._sections).contains_key(&(section_name)) {
                        {
                            let __assign_value = HashMap::from([]);
                            {
                                let __assign_key = section_name.clone();
                                self._sections.insert(__assign_key, __assign_value);
                            }
                        }
                    }
                    continue;
                }
                let __sifr_try_res: Result<
                    (),
                    __SifrStdlib_sifr_x2econfigparser_x2eParsingError,
                > = (|| {
                    let parsed_option_pair: (String, Option<String>) = _split_option_line(
                        &line,
                        self.allow_no_value,
                        (line_no).clone(),
                    )?;
                    let (option_name, option_value) = parsed_option_pair;
                    let __sifr_chars_option_name: Vec<char> = option_name
                        .chars()
                        .collect::<Vec<char>>();
                    if (current_section == "") || (current_section == default_section) {
                        {
                            let __assign_value = _copy_optional_str(&option_value);
                            {
                                let __assign_key = option_name.clone();
                                self._defaults.insert(__assign_key, __assign_value);
                            }
                        }
                    } else {
                        let section_key: String = {
                            let mut __sifr_concat: String = String::with_capacity(
                                current_section.len() + 0usize,
                            );
                            __sifr_concat.push_str((current_section).as_str());
                            __sifr_concat.push_str("");
                            __sifr_concat
                        };
                        let mut section_found: bool = false;
                        for (section_name, section_values) in self
                            ._sections
                            .iter()
                            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                            .collect::<Vec<_>>()
                        {
                            if (section_name != section_key) {
                                continue;
                            }
                            if self.strict && _has_option_key(&section_values, &option_name)
                            {
                                return Err(
                                    __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                                        (line_no).clone(),
                                        format!("{}{}", "duplicate option: ", option_name),
                                    ),
                                );
                            }
                            let mut updated_section: HashMap<String, Option<String>> = _copy_values(
                                &section_values,
                            );
                            {
                                let __assign_value = _copy_optional_str(&option_value);
                                {
                                    let __assign_key = option_name.clone();
                                    updated_section.insert(__assign_key, __assign_value);
                                }
                            }
                            {
                                let __assign_value = updated_section.clone();
                                {
                                    let __assign_key = section_name.clone();
                                    self._sections.insert(__assign_key, __assign_value);
                                }
                            }
                            section_found = true;
                            break;
                        }
                    }
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    return Err(e);
                }
            }
            Ok(())
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn read(&mut self, path: &String) -> Result<Vec<String>, IOError> {
            let __sifr_try_res: Result<Result<Vec<String>, IOError>, IOError> = (|| {
                let content: String = read_text(path)?;
                let __sifr_try_res: Result<
                    (),
                    __SifrStdlib_sifr_x2econfigparser_x2eParsingError,
                > = (|| {
                    let _ = self.read_string(&content)?;
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    return Err(
                        IOError::new({
                            let mut __sifr_concat: String = String::with_capacity(
                                ((20usize + 0usize) + 2usize) + 0usize,
                            );
                            __sifr_concat.push_str("parse error on line ");
                            __sifr_concat.push_str((format!("{}", e.line.clone())).as_str());
                            __sifr_concat.push_str(": ");
                            __sifr_concat.push_str((e.message.clone()).as_str());
                            __sifr_concat
                        }),
                    );
                }
                let loaded_path: String = {
                    let mut __sifr_concat: String = String::with_capacity(
                        path.len() + 0usize,
                    );
                    __sifr_concat.push_str((path).as_str());
                    __sifr_concat.push_str("");
                    __sifr_concat
                };
                Ok(Ok(vec![loaded_path.clone()]))
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
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn sections(&self) -> Vec<String> {
            let mut names: Vec<String> = vec![];
            for section in self._sections.clone().keys().cloned() {
                names.push(section.clone());
            }
            names
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn has_section(&self, section: &String) -> bool {
            (self._sections).contains_key((section).as_str())
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn options(&self, section: &String) -> Vec<String> {
            let merged: HashMap<String, Option<String>> = self._merged_section(section);
            let mut names: Vec<String> = vec![];
            for option in merged.keys().cloned() {
                names.push(option.clone());
            }
            names
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn items(&self, section: &String) -> Vec<(String, Option<String>)> {
            let merged: HashMap<String, Option<String>> = self._merged_section(section);
            let mut items: Vec<(String, Option<String>)> = vec![];
            for (option, value) in merged
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                items.push(((option).clone(), _copy_optional_str(&value)));
            }
            items
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn _merged_section(&self, section: &String) -> HashMap<String, Option<String>> {
            let mut merged: HashMap<String, Option<String>> = _copy_values(&self._defaults);
            let default_section: String = _default_section();
            if *section == default_section {
                return merged;
            }
            for (section_name, section_values) in self
                ._sections
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if section_name != *section {
                    continue;
                }
                for (option, value) in section_values
                    .iter()
                    .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                    .collect::<Vec<_>>()
                {
                    {
                        let __assign_value = _copy_optional_str(&value);
                        {
                            let __assign_key = option.clone();
                            merged.insert(__assign_key, __assign_value);
                        }
                    }
                }
                return merged;
            }
            merged
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn has_option(&self, section: &String, option: &String) -> bool {
            let normalized: String = _normalize_option(option);
            let default_section: String = _default_section();
            if *section == default_section {
                return (self._defaults).contains_key(&(normalized));
            }
            for (section_name, section_values) in self
                ._sections
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if section_name != *section {
                    continue;
                }
                if _has_option_key(&section_values, &normalized) {
                    return true;
                }
                return (self._defaults).contains_key(&(normalized));
            }
            false
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn get(
            &self,
            section: &String,
            option: &String,
            fallback: &Option<String>,
            raw: bool,
        ) -> Option<String> {
            let normalized: String = _normalize_option(option);
            let merged: HashMap<String, Option<String>> = self._merged_section(section);
            let default_section: String = _default_section();
            if *section == default_section {
                if !_has_option_key(&merged, &normalized) {
                    return _copy_optional_str(fallback);
                }
                let raw_value: Option<String> = _lookup_option(&merged, &normalized);
                let Some(raw_value) = raw_value else {
                    return None;
                };
                if raw {
                    return Some(raw_value);
                }
                return Some(
                    _resolve_interpolation(&raw_value, &merged, SifrInt::from_i64(0)),
                );
            }
            if !self.has_section(section) {
                if _has_option_key(&self._defaults, &normalized) {
                    let default_value: Option<String> = _lookup_option(
                        &self._defaults,
                        &normalized,
                    );
                    let Some(default_value) = default_value else {
                        return None;
                    };
                    if raw {
                        return Some(default_value);
                    }
                    return Some(
                        _resolve_interpolation(&default_value, &merged, SifrInt::from_i64(0)),
                    );
                }
                return _copy_optional_str(fallback);
            }
            if !_has_option_key(&merged, &normalized) {
                return _copy_optional_str(fallback);
            }
            let raw_value2: Option<String> = _lookup_option(&merged, &normalized);
            let Some(raw_value2) = raw_value2 else {
                return None;
            };
            if raw {
                return Some(raw_value2);
            }
            Some(_resolve_interpolation(&raw_value2, &merged, SifrInt::from_i64(0)))
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn getint(
            &self,
            section: &String,
            option: &String,
            fallback: &Option<SifrInt>,
        ) -> Option<SifrInt> {
            let raw: Option<String> = self.get(section, option, &None, false);
            let Some(raw) = raw else {
                return fallback.clone();
            };
            let __sifr_try_res: Result<Option<SifrInt>, ParseError> = (|| {
                let parsed: SifrInt = SifrInt::parse_decimal(
                        &(raw),
                        ::sifr_runtime::DEFAULT_MAX_INTEGER_DIGITS,
                    )
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                Ok(Some(parsed))
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let _e = __sifr_try_err.clone();
                    return fallback.clone();
                }
            }
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn getfloat(
            &self,
            section: &String,
            option: &String,
            fallback: Option<f64>,
        ) -> Option<f64> {
            let raw: Option<String> = self.get(section, option, &None, false);
            let Some(raw) = raw else {
                return fallback;
            };
            let __sifr_try_res: Result<Option<f64>, ParseError> = (|| {
                let parsed: f64 = (raw)
                    .parse::<f64>()
                    .map_err(|e| ParseError {
                        message: e.to_string(),
                    })?;
                Ok(Some(parsed))
            })();
            match __sifr_try_res {
                Ok(__sifr_ret_val) => {
                    return __sifr_ret_val;
                }
                Err(__sifr_try_err) => {
                    let _e = __sifr_try_err.clone();
                    return fallback;
                }
            }
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn getboolean(
            &self,
            section: &String,
            option: &String,
            fallback: Option<bool>,
        ) -> Option<bool> {
            let raw: Option<String> = self.get(section, option, &None, false);
            let Some(raw) = raw else {
                return fallback;
            };
            let normalized: String = raw.to_lowercase();
            if (((normalized == "1") || (normalized == "yes")) || (normalized == "true"))
                || (normalized == "on")
            {
                return Some(true);
            }
            if (((normalized == "0") || (normalized == "no")) || (normalized == "false"))
                || (normalized == "off")
            {
                return Some(false);
            }
            fallback
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn set(&mut self, section: &String, option: &String, value: &Option<String>) {
            let normalized: String = _normalize_option(option);
            let default_section: String = _default_section();
            if *section == default_section {
                {
                    let __assign_value = _copy_optional_str(value);
                    {
                        let __assign_key = normalized.clone();
                        self._defaults.insert(__assign_key, __assign_value);
                    }
                }
                return;
            }
            for (section_name, section_values) in self
                ._sections
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if section_name != *section {
                    continue;
                }
                let mut updated_section: HashMap<String, Option<String>> = _copy_values(
                    &section_values,
                );
                {
                    let __assign_value = _copy_optional_str(value);
                    {
                        let __assign_key = normalized.clone();
                        updated_section.insert(__assign_key, __assign_value);
                    }
                }
                {
                    let __assign_value = updated_section.clone();
                    {
                        let __assign_key = section_name.clone();
                        self._sections.insert(__assign_key, __assign_value);
                    }
                }
                return;
            }
            if !(self._sections).contains_key((section).as_str()) {
                {
                    let __assign_value = HashMap::from([]);
                    {
                        let __assign_key = section.clone();
                        self._sections.insert(__assign_key, __assign_value);
                    }
                }
            }
            let mut created_section: HashMap<String, Option<String>> = HashMap::from([]);
            for (section_name, section_values) in self
                ._sections
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if section_name != *section {
                    continue;
                }
                created_section = _copy_values(&section_values);
                break;
            }
            {
                let __assign_value = _copy_optional_str(value);
                {
                    let __assign_key = normalized.clone();
                    created_section.insert(__assign_key, __assign_value);
                }
            }
            {
                let __assign_value = created_section.clone();
                {
                    let __assign_key = section.clone();
                    self._sections.insert(__assign_key, __assign_value);
                }
            }
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn add_section(&mut self, section: &String) {
            let default_section: String = _default_section();
            if *section == default_section {
                return;
            }
            if (self._sections).contains_key((section).as_str()) {
                return;
            }
            {
                let __assign_value = HashMap::from([]);
                {
                    let __assign_key = section.clone();
                    self._sections.insert(__assign_key, __assign_value);
                }
            }
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn remove_option(&mut self, section: &String, option: &String) -> bool {
            let normalized: String = _normalize_option(option);
            let default_section: String = _default_section();
            if *section == default_section {
                if (self._defaults).contains_key(&(normalized)) {
                    self._defaults = _without_option(&self._defaults, &normalized);
                    return true;
                }
                return false;
            }
            for (section_name, section_values) in self
                ._sections
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if section_name != *section {
                    continue;
                }
                if _has_option_key(&section_values, &normalized) {
                    {
                        let __assign_value = _without_option(&section_values, &normalized);
                        {
                            let __assign_key = section_name.clone();
                            self._sections.insert(__assign_key, __assign_value);
                        }
                    }
                    return true;
                }
                return false;
            }
            false
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn remove_section(&mut self, section: &String) -> bool {
            let default_section: String = _default_section();
            if *section == default_section {
                return false;
            }
            if (self._sections).contains_key((section).as_str()) {
                self._sections = _without_section(&self._sections, section);
                return true;
            }
            false
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn proxy(
            &self,
            section: &String,
        ) -> Option<__SifrStdlib_sifr_x2econfigparser_x2eSectionProxy> {
            let default_section: String = _default_section();
            if (*section != default_section) && !self.has_section(section) {
                return None;
            }
            let merged: HashMap<String, Option<String>> = self._merged_section(section);
            Some(
                __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy::new(
                    (section.clone()).clone(),
                    merged,
                ),
            )
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn to_ini_string(&self) -> String {
            let mut lines: Vec<String> = vec![];
            if (&SifrInt::from(self._defaults.len()) > &SifrInt::from_i64(0)) {
                lines.push(format!("{}{}", format!("{}{}", "[", _default_section()), "]"));
                for (key, value) in self
                    ._defaults
                    .iter()
                    .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                    .collect::<Vec<_>>()
                {
                    if (value == None) {
                        lines.push(key.clone());
                    } else {
                        if let Some(value) = value {
                            lines.push(format!("{}{}", format!("{}{}", key, " = "), value));
                        }
                    }
                }
                lines.push("".to_string());
            }
            for (section_name, section_values) in self
                ._sections
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                lines.push(format!("{}{}", format!("{}{}", "[", section_name), "]"));
                for (key, value) in section_values
                    .iter()
                    .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                    .collect::<Vec<_>>()
                {
                    if (value == None) {
                        lines.push(key.clone());
                    } else {
                        if let Some(value) = value {
                            lines.push(format!("{}{}", format!("{}{}", key, " = "), value));
                        }
                    }
                }
                lines.push("".to_string());
            }
            if (&SifrInt::from(lines.len()) > &SifrInt::from_i64(0)) {
                let maybe_last: Option<String> = {
                    let __sifr_index_list = &lines;
                    let __sifr_index_i = SifrInt::from(lines.len()) - SifrInt::from_i64(1);
                    let __sifr_index_norm = __sifr_index_i
                        .normalize_index_or_len(__sifr_index_list.len());
                    __sifr_index_list.get(__sifr_index_norm).cloned()
                };
                if (maybe_last != None) && (maybe_last == Some("".to_string())) {
                    if let Some(_) = lines.pop() {}
                }
            }
            lines.join("\n")
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        pub fn write(&self, path: &String) -> Result<(), IOError> {
            let payload: String = self.to_ini_string();
            write_text(path, &payload)
        }
    }
    #[derive(Debug, Clone, PartialEq)]
    pub struct __SifrStdlib_sifr_x2econfigparser_x2eRawConfigParser {
        pub configparser: __SifrStdlib_sifr_x2econfigparser_x2eConfigParser,
    }
    impl ::std::ops::Deref for __SifrStdlib_sifr_x2econfigparser_x2eRawConfigParser {
        type Target = __SifrStdlib_sifr_x2econfigparser_x2eConfigParser;
        fn deref(&self) -> &Self::Target {
            &self.configparser
        }
    }
    impl ::std::ops::DerefMut for __SifrStdlib_sifr_x2econfigparser_x2eRawConfigParser {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.configparser
        }
    }
    impl ::std::convert::From<__SifrStdlib_sifr_x2econfigparser_x2eRawConfigParser>
    for __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
        fn from(value: __SifrStdlib_sifr_x2econfigparser_x2eRawConfigParser) -> Self {
            value.configparser
        }
    }
    impl __SifrStdlib_sifr_x2econfigparser_x2eRawConfigParser {}
    pub fn _default_section() -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(
                __const_DEFAULTSECT().len() + 0usize,
            );
            __sifr_concat.push_str((__const_DEFAULTSECT()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    pub fn _normalize_option(option: &String) -> String {
        option.to_lowercase().trim().to_string()
    }
    pub fn _some_str(value: &String) -> Option<String> {
        Some({
            let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
            __sifr_concat.push_str((value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        })
    }
    pub fn _copy_optional_str(value: &Option<String>) -> Option<String> {
        if let Some(value) = value.as_ref() {
            return _some_str(value);
        }
        None
    }
    pub fn _has_option_key(values: &HashMap<String, Option<String>>, key: &String) -> bool {
        for current_key in values.keys().cloned() {
            if current_key == *key {
                return true;
            }
        }
        false
    }
    pub fn _lookup_option(
        values: &HashMap<String, Option<String>>,
        key: &String,
    ) -> Option<String> {
        for (current_key, current_value) in values
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if current_key == *key {
                return _copy_optional_str(&current_value);
            }
        }
        None
    }
    pub fn _copy_values(
        values: &HashMap<String, Option<String>>,
    ) -> HashMap<String, Option<String>> {
        let mut copied: HashMap<String, Option<String>> = HashMap::from([]);
        for (key, value) in values
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            {
                let __assign_value = _copy_optional_str(&value);
                {
                    let __assign_key = key.clone();
                    copied.insert(__assign_key, __assign_value);
                }
            }
        }
        copied
    }
    pub fn _without_option(
        values: &HashMap<String, Option<String>>,
        removed_key: &String,
    ) -> HashMap<String, Option<String>> {
        let mut copied: HashMap<String, Option<String>> = HashMap::from([]);
        for (key, value) in values
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if key == *removed_key {
                continue;
            }
            {
                let __assign_value = _copy_optional_str(&value);
                {
                    let __assign_key = key.clone();
                    copied.insert(__assign_key, __assign_value);
                }
            }
        }
        copied
    }
    pub fn _without_section(
        values: &HashMap<String, HashMap<String, Option<String>>>,
        removed_key: &String,
    ) -> HashMap<String, HashMap<String, Option<String>>> {
        let mut copied: HashMap<String, HashMap<String, Option<String>>> = HashMap::from([]);
        for (key, section) in values
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if key == *removed_key {
                continue;
            }
            {
                let __assign_value = _copy_values(&section);
                {
                    let __assign_key = key.clone();
                    copied.insert(__assign_key, __assign_value);
                }
            }
        }
        copied
    }
    pub fn _find_delimiter(line: &String) -> Option<String> {
        if line.contains(&"=".to_string()) {
            return Some("=".to_string());
        }
        if line.contains(&":".to_string()) {
            return Some(":".to_string());
        }
        None
    }
    pub fn _split_option_line(
        line: &String,
        allow_no_value: bool,
        line_no: SifrInt,
    ) -> Result<
        (String, Option<String>),
        __SifrStdlib_sifr_x2econfigparser_x2eParsingError,
    > {
        let delimiter: Option<String> = _find_delimiter(line);
        let Some(delimiter) = delimiter else {
            if allow_no_value {
                return Ok((line.trim().to_string(), None));
            }
            return Err(
                __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                    (line_no).clone(),
                    "expected key=value or key:value entry".to_string(),
                ),
            );
        };
        let parts: Vec<String> = if &SifrInt::from_i64(1) < &0 {
            line.split(&delimiter).map(|s| s.to_string()).collect::<Vec<String>>()
        } else {
            line.splitn(
                    ::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1) + 1)),
                    &delimiter,
                )
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
        };
        if (&SifrInt::from(parts.len()) != &SifrInt::from_i64(2)) {
            return Err(
                __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                    (line_no).clone(),
                    "invalid option line".to_string(),
                ),
            );
        }
        let raw_key: Option<String> = {
            let __sifr_checked_read_collection = &parts;
            let __sifr_checked_read_index = SifrInt::from_i64(0);
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        let raw_value: Option<String> = {
            let __sifr_checked_read_collection = &parts;
            let __sifr_checked_read_index = SifrInt::from_i64(1);
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        let Some(raw_key) = raw_key else {
            return Err(
                __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                    (line_no).clone(),
                    "option name is missing".to_string(),
                ),
            );
        };
        let key: String = _normalize_option(&raw_key);
        if (key == "") {
            return Err(
                __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                    (line_no).clone(),
                    "option name is empty".to_string(),
                ),
            );
        }
        let Some(raw_value) = raw_value else {
            return Ok((key, None));
        };
        let stripped_value: Option<String> = _some_str(&raw_value.trim().to_string());
        Ok((key, stripped_value))
    }
    pub fn _char_at(text: &String, index: SifrInt) -> String {
        let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
        if (&index < &SifrInt::from_i64(0))
            || (&index >= &SifrInt::from(__sifr_chars_text.len()))
        {
            return "".to_string();
        }
        let ch: Option<String> = ({
            let __sifr_string_index = index.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let Some(ch) = ch else {
            return "".to_string();
        };
        ch
    }
    pub fn _resolve_interpolation(
        value: &String,
        merged: &HashMap<String, Option<String>>,
        depth: SifrInt,
    ) -> String {
        if &depth >= &SifrInt::from_i64(8) {
            return {
                let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
                __sifr_concat.push_str((value).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        if !value.contains(&"%(".to_string()) {
            return {
                let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
                __sifr_concat.push_str((value).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
        let mut result: String = "".to_string();
        let mut replaced: bool = false;
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &SifrInt::from(value.chars().count())) {
            let ch: String = _char_at(value, (i).clone());
            if ((ch == "%")
                && (&(&i + &SifrInt::from_i64(1)) < &SifrInt::from(value.chars().count())))
                && (_char_at(value, &i + &SifrInt::from_i64(1)) == "(")
            {
                let mut j: SifrInt = &i + &SifrInt::from_i64(2);
                let mut key: String = "".to_string();
                let mut matched: bool = false;
                while (&j < &SifrInt::from(value.chars().count())) {
                    let part: String = _char_at(value, (j).clone());
                    if ((part == ")")
                        && (&(&j + &SifrInt::from_i64(1))
                            < &SifrInt::from(value.chars().count())))
                        && (_char_at(value, &j + &SifrInt::from_i64(1)) == "s")
                    {
                        matched = true;
                        let normalized_key: String = _normalize_option(&key);
                        let replacement: Option<String> = _lookup_option(
                            merged,
                            &normalized_key,
                        );
                        if (replacement == None) {
                            result.push_str("%(");
                            result.push_str((key).as_str());
                            result.push_str(")s");
                        } else {
                            if let Some(replacement) = replacement {
                                replaced = true;
                                result.push_str((replacement).as_str());
                            }
                        }
                        i = &j + &SifrInt::from_i64(2);
                        break;
                    }
                    key.push_str((part).as_str());
                    j = &j + &SifrInt::from_i64(1);
                }
                if matched {
                    continue;
                }
            }
            result.push_str((ch).as_str());
            i = &i + &SifrInt::from_i64(1);
        }
        if replaced {
            return _resolve_interpolation(&result, merged, &depth + &SifrInt::from_i64(1));
        }
        result
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
    pub struct DivisionError {
        pub message: String,
    }
    impl DivisionError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for DivisionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for DivisionError {}
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
    impl From<DivisionError> for Error {
        fn from(err: DivisionError) -> Self {
            Self::new(err.message)
        }
    }
}
pub use __sifr_project_nominals::DivisionError;
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::__SifrIoBinaryFileHandle;
pub use __sifr_project_nominals::__SifrIoFileHandle;
pub use __sifr_project_nominals::__SifrIoTextFileHandle;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2econfigparser_x2eConfigParser;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2econfigparser_x2eParsingError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2econfigparser_x2eRawConfigParser;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2econfigparser_x2eSectionProxy;
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
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipFile;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipInfo;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2ezipfile_x2eZipReadHandle;
use ::std::collections::HashMap;
use ::sifr_runtime::SifrInt;
fn calendar_isleap(year: SifrInt) -> bool {
    ::sifr_stdlib::calendar::calendar_isleap(
        ::sifr_runtime::interop::SifrIntBridge::from(year),
    )
}
fn calendar_weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
    ::sifr_stdlib::calendar::calendar_weekday(
            ::sifr_runtime::interop::SifrIntBridge::from(year),
            ::sifr_runtime::interop::SifrIntBridge::from(month),
            ::sifr_runtime::interop::SifrIntBridge::from(day),
        )
        .into_sifr_int()
}
fn calendar_monthrange(year: SifrInt, month: SifrInt) -> Vec<SifrInt> {
    ::sifr_stdlib::calendar::calendar_monthrange(
            ::sifr_runtime::interop::SifrIntBridge::from(year),
            ::sifr_runtime::interop::SifrIntBridge::from(month),
        )
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn isleap(year: SifrInt) -> bool {
    calendar_isleap((year).clone())
}
fn weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
    calendar_weekday((year).clone(), (month).clone(), (day).clone())
}
fn monthrange(year: SifrInt, month: SifrInt) -> Vec<SifrInt> {
    calendar_monthrange((year).clone(), (month).clone())
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
        Ok(Ok(__SifrIoNativeFileHandle::new(handle_id)))
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
        __sifr_concat.push_str((format!("{}", whence)).as_str());
        __sifr_concat
    }
}
fn _negative_seek_error(offset: SifrInt) -> String {
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
        Ok(Ok(__SifrIoFileHandle::new(handle, (mode.clone()).clone())))
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
    if !mode.contains(&"b".to_string()) {
        return Err(IOError::new("open_binary requires binary mode".to_string()));
    }
    let __sifr_try_res: Result<Result<__SifrIoBinaryFileHandle, IOError>, IOError> = (|| {
        let handle: __SifrIoNativeFileHandle = open_file(path, mode)?;
        Ok(Ok(__SifrIoBinaryFileHandle::new(handle, (mode.clone()).clone())))
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
            return Err(IOError::new(e.message.clone()));
        }
    }
}
fn __const_DEFAULTSECT() -> String {
    "DEFAULT".to_string().to_string()
}
fn _default_section() -> String {
    {
        let mut __sifr_concat: String = String::with_capacity(
            __const_DEFAULTSECT().len() + 0usize,
        );
        __sifr_concat.push_str((__const_DEFAULTSECT()).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _normalize_option(option: &String) -> String {
    option.to_lowercase().trim().to_string()
}
fn _some_str(value: &String) -> Option<String> {
    Some({
        let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
        __sifr_concat.push_str((value).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    })
}
fn _copy_optional_str(value: &Option<String>) -> Option<String> {
    if let Some(value) = value.as_ref() {
        return _some_str(value);
    }
    None
}
fn _has_option_key(values: &HashMap<String, Option<String>>, key: &String) -> bool {
    for current_key in values.keys().cloned() {
        if current_key == *key {
            return true;
        }
    }
    false
}
fn _lookup_option(
    values: &HashMap<String, Option<String>>,
    key: &String,
) -> Option<String> {
    for (current_key, current_value) in values
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        if current_key == *key {
            return _copy_optional_str(&current_value);
        }
    }
    None
}
fn _copy_values(
    values: &HashMap<String, Option<String>>,
) -> HashMap<String, Option<String>> {
    let mut copied: HashMap<String, Option<String>> = HashMap::from([]);
    for (key, value) in values
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        {
            let __assign_value = _copy_optional_str(&value);
            {
                let __assign_key = key.clone();
                copied.insert(__assign_key, __assign_value);
            }
        }
    }
    copied
}
fn _without_option(
    values: &HashMap<String, Option<String>>,
    removed_key: &String,
) -> HashMap<String, Option<String>> {
    let mut copied: HashMap<String, Option<String>> = HashMap::from([]);
    for (key, value) in values
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        if key == *removed_key {
            continue;
        }
        {
            let __assign_value = _copy_optional_str(&value);
            {
                let __assign_key = key.clone();
                copied.insert(__assign_key, __assign_value);
            }
        }
    }
    copied
}
fn _without_section(
    values: &HashMap<String, HashMap<String, Option<String>>>,
    removed_key: &String,
) -> HashMap<String, HashMap<String, Option<String>>> {
    let mut copied: HashMap<String, HashMap<String, Option<String>>> = HashMap::from([]);
    for (key, section) in values
        .iter()
        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
        .collect::<Vec<_>>()
    {
        if key == *removed_key {
            continue;
        }
        {
            let __assign_value = _copy_values(&section);
            {
                let __assign_key = key.clone();
                copied.insert(__assign_key, __assign_value);
            }
        }
    }
    copied
}
fn _find_delimiter(line: &String) -> Option<String> {
    if line.contains(&"=".to_string()) {
        return Some("=".to_string());
    }
    if line.contains(&":".to_string()) {
        return Some(":".to_string());
    }
    None
}
fn _split_option_line(
    line: &String,
    allow_no_value: bool,
    line_no: SifrInt,
) -> Result<
    (String, Option<String>),
    __SifrStdlib_sifr_x2econfigparser_x2eParsingError,
> {
    let delimiter: Option<String> = _find_delimiter(line);
    let Some(delimiter) = delimiter else {
        if allow_no_value {
            return Ok((line.trim().to_string(), None));
        }
        return Err(
            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                (line_no).clone(),
                "expected key=value or key:value entry".to_string(),
            ),
        );
    };
    let parts: Vec<String> = if &SifrInt::from_i64(1) < &0 {
        line.split(&delimiter).map(|s| s.to_string()).collect::<Vec<String>>()
    } else {
        line.splitn(
                ::sifr_runtime::to_usize_proven(&(SifrInt::from_i64(1) + 1)),
                &delimiter,
            )
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
    if (&SifrInt::from(parts.len()) != &SifrInt::from_i64(2)) {
        return Err(
            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                (line_no).clone(),
                "invalid option line".to_string(),
            ),
        );
    }
    let raw_key: Option<String> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let raw_value: Option<String> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(raw_key) = raw_key else {
        return Err(
            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                (line_no).clone(),
                "option name is missing".to_string(),
            ),
        );
    };
    let key: String = _normalize_option(&raw_key);
    if (key == "") {
        return Err(
            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                (line_no).clone(),
                "option name is empty".to_string(),
            ),
        );
    }
    let Some(raw_value) = raw_value else {
        return Ok((key, None));
    };
    let stripped_value: Option<String> = _some_str(&raw_value.trim().to_string());
    Ok((key, stripped_value))
}
fn _char_at(text: &String, index: SifrInt) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (&index < &SifrInt::from_i64(0))
        || (&index >= &SifrInt::from(__sifr_chars_text.len()))
    {
        return "".to_string();
    }
    let ch: Option<String> = ({
        let __sifr_string_index = index.clone();
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_text.len());
        __sifr_chars_text.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string());
    let Some(ch) = ch else {
        return "".to_string();
    };
    ch
}
fn _resolve_interpolation(
    value: &String,
    merged: &HashMap<String, Option<String>>,
    depth: SifrInt,
) -> String {
    if &depth >= &SifrInt::from_i64(8) {
        return {
            let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
            __sifr_concat.push_str((value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    if !value.contains(&"%(".to_string()) {
        return {
            let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
            __sifr_concat.push_str((value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    let mut result: String = "".to_string();
    let mut replaced: bool = false;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(value.chars().count())) {
        let ch: String = _char_at(value, (i).clone());
        if ((ch == "%")
            && (&(&i + &SifrInt::from_i64(1)) < &SifrInt::from(value.chars().count())))
            && (_char_at(value, &i + &SifrInt::from_i64(1)) == "(")
        {
            let mut j: SifrInt = &i + &SifrInt::from_i64(2);
            let mut key: String = "".to_string();
            let mut matched: bool = false;
            while (&j < &SifrInt::from(value.chars().count())) {
                let part: String = _char_at(value, (j).clone());
                if ((part == ")")
                    && (&(&j + &SifrInt::from_i64(1))
                        < &SifrInt::from(value.chars().count())))
                    && (_char_at(value, &j + &SifrInt::from_i64(1)) == "s")
                {
                    matched = true;
                    let normalized_key: String = _normalize_option(&key);
                    let replacement: Option<String> = _lookup_option(
                        merged,
                        &normalized_key,
                    );
                    if (replacement == None) {
                        result.push_str("%(");
                        result.push_str((key).as_str());
                        result.push_str(")s");
                    } else {
                        if let Some(replacement) = replacement {
                            replaced = true;
                            result.push_str((replacement).as_str());
                        }
                    }
                    i = &j + &SifrInt::from_i64(2);
                    break;
                }
                key.push_str((part).as_str());
                j = &j + &SifrInt::from_i64(1);
            }
            if matched {
                continue;
            }
        }
        result.push_str((ch).as_str());
        i = &i + &SifrInt::from_i64(1);
    }
    if replaced {
        return _resolve_interpolation(&result, merged, &depth + &SifrInt::from_i64(1));
    }
    result
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
fn html_escape(s: &String) -> String {
    ::sifr_stdlib::html::html_escape(s)
}
fn html_unescape(s: &String) -> String {
    ::sifr_stdlib::html::html_unescape(s)
}
fn escape(s: &String, quote: bool) -> String {
    let escaped: String = html_escape(s);
    if quote {
        return escaped;
    }
    escaped.replace("&quot;", "\"").replace("&#x27;", "\'")
}
fn unescape(s: &String) -> String {
    html_unescape(s)
}
fn add(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}
fn sub(a: SifrInt, b: SifrInt) -> SifrInt {
    &a - &b
}
fn mul(a: SifrInt, b: SifrInt) -> SifrInt {
    &a * &b
}
fn floordiv(a: SifrInt, b: SifrInt) -> Result<SifrInt, DivisionError> {
    {
        let __sifr_floor_left: SifrInt = a.clone();
        let __sifr_floor_right: SifrInt = b.clone();
        __sifr_floor_left
            .checked_floor_div(&__sifr_floor_right)
            .ok_or_else(|| DivisionError::new("division by zero".to_string()))
    }
}
fn mod_val(a: SifrInt, b: SifrInt) -> Result<SifrInt, DivisionError> {
    {
        let __sifr_floor_left: SifrInt = a.clone();
        let __sifr_floor_right: SifrInt = b.clone();
        __sifr_floor_left
            .checked_floor_mod(&__sifr_floor_right)
            .ok_or_else(|| DivisionError::new("division by zero".to_string()))
    }
}
fn neg(a: SifrInt) -> SifrInt {
    -&a
}
fn lt(a: SifrInt, b: SifrInt) -> bool {
    &a < &b
}
fn eq(a: SifrInt, b: SifrInt) -> bool {
    &a == &b
}
fn getitem<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    index: SifrInt,
) -> Option<T> {
    {
        let __sifr_checked_read_collection = &items;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    }
}
fn itemgetter<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    index: SifrInt,
) -> Option<T> {
    getitem(items, (index).clone())
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
fn version() -> String {
    sys_version()
}
fn maxsize() -> SifrInt {
    sys_maxsize()
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
fn demo_operator() {
    println!("=== operator ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("add(10, 5) = "); __sifr_concat.push_str((format!("{}",
        add(SifrInt::from_i64(10), SifrInt::from_i64(5)))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("sub(10, 5) = "); __sifr_concat.push_str((format!("{}",
        sub(SifrInt::from_i64(10), SifrInt::from_i64(5)))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(12usize + 0usize);
        __sifr_concat.push_str("mul(3, 4) = "); __sifr_concat.push_str((format!("{}",
        mul(SifrInt::from_i64(3), SifrInt::from_i64(4)))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(17usize + 0usize);
        __sifr_concat.push_str("floordiv(7, 2) = "); __sifr_concat
        .push_str((format!("{:?}", floordiv(SifrInt::from_i64(7), SifrInt::from_i64(2))))
        .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("mod_val(7, 2) = "); __sifr_concat
        .push_str((format!("{:?}", mod_val(SifrInt::from_i64(7), SifrInt::from_i64(2))))
        .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(10usize + 0usize);
        __sifr_concat.push_str("neg(42) = "); __sifr_concat.push_str((format!("{}",
        neg(SifrInt::from_i64(42)))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(11usize + 0usize);
        __sifr_concat.push_str("lt(3, 5) = "); __sifr_concat.push_str((format!("{}",
        lt(SifrInt::from_i64(3), SifrInt::from_i64(5)))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(11usize + 0usize);
        __sifr_concat.push_str("eq(5, 5) = "); __sifr_concat.push_str((format!("{}",
        eq(SifrInt::from_i64(5), SifrInt::from_i64(5)))).as_str()); __sifr_concat }
    );
    let items: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3)
    ];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(25usize + 0usize);
        __sifr_concat.push_str("itemgetter([1,2,3], 1) = "); __sifr_concat
        .push_str(((itemgetter(& items, SifrInt::from_i64(1))).map_or("None".to_string()
        .to_string(), | __v | format!("{}", __v))).as_str()); __sifr_concat }
    );
}
fn demo_calendar() {
    println!("=== calendar ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(15usize + 0usize);
        __sifr_concat.push_str("isleap(2000) = "); __sifr_concat.push_str((format!("{}",
        isleap(SifrInt::from_i64(2000)))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(15usize + 0usize);
        __sifr_concat.push_str("isleap(1900) = "); __sifr_concat.push_str((format!("{}",
        isleap(SifrInt::from_i64(1900)))).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(15usize + 0usize);
        __sifr_concat.push_str("isleap(2024) = "); __sifr_concat.push_str((format!("{}",
        isleap(SifrInt::from_i64(2024)))).as_str()); __sifr_concat }
    );
    let wd: SifrInt = weekday(
        SifrInt::from_i64(2024),
        SifrInt::from_i64(1),
        SifrInt::from_i64(1),
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("weekday(2024,1,1) = "); __sifr_concat
        .push_str((format!("{}", wd)).as_str()); __sifr_concat }
    );
    let mr: Vec<SifrInt> = monthrange(SifrInt::from_i64(2024), SifrInt::from_i64(2));
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(24usize + 0usize);
        __sifr_concat.push_str("monthrange(2024,2)[1] = "); __sifr_concat.push_str((({
        let __sifr_index_list = & mr; let __sifr_index_i = SifrInt::from_i64(1); let
        __sifr_index_norm = __sifr_index_i.normalize_index_or_len(__sifr_index_list
        .len()); __sifr_index_list.get(__sifr_index_norm).cloned() }).map_or("None"
        .to_string().to_string(), | __v | format!("{}", __v))).as_str()); __sifr_concat }
    );
}
fn demo_html() {
    println!("=== html ===");
    let s: String = "<b>Hi & Bye</b>".to_string();
    let esc: String = escape(&s, true);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(26usize + esc
        .len()); __sifr_concat.push_str("escape(<b>Hi & Bye</b>) = "); __sifr_concat
        .push_str((esc).as_str()); __sifr_concat }
    );
    let unesc: String = unescape(&esc);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(44usize + unesc
        .len()); __sifr_concat.push_str("unescape(&lt;b&gt;Hi &amp; Bye&lt;/b&gt;) = ");
        __sifr_concat.push_str((unesc).as_str()); __sifr_concat }
    );
}
fn demo_sys() {
    println!("=== sys ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(10usize + 0usize);
        __sifr_concat.push_str("version = "); __sifr_concat.push_str((version())
        .as_str()); __sifr_concat }
    );
    let ms: SifrInt = maxsize();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("maxsize > 0 = "); __sifr_concat.push_str((format!("{}", &
        ms > & SifrInt::from_i64(0))).as_str()); __sifr_concat }
    );
}
fn demo_configparser() {
    println!("=== configparser ===");
    let mut config: __SifrStdlib_sifr_x2econfigparser_x2eConfigParser = __SifrStdlib_sifr_x2econfigparser_x2eConfigParser::new(
        None,
        false,
        false,
    );
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2econfigparser_x2eParsingError> = (||
    {
        let _ = config
            .read_string(
                &"[database]\nhost = db.example.com\nport = 5432\n".to_string(),
            )?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", e.message.clone());
        return;
    }
    let host_value: Option<String> = config
        .get(&"database".to_string(), &"host".to_string(), &None, false);
    let port_value: Option<String> = config
        .get(&"database".to_string(), &"port".to_string(), &None, false);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(7usize + 0usize);
        __sifr_concat.push_str("host = "); __sifr_concat.push_str(((host_value)
        .map_or("None".to_string().to_string(), | __v | format!("{}", __v))).as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(7usize + 0usize);
        __sifr_concat.push_str("port = "); __sifr_concat.push_str(((port_value)
        .map_or("None".to_string().to_string(), | __v | format!("{}", __v))).as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(11usize + 0usize);
        __sifr_concat.push_str("has_host = "); __sifr_concat.push_str((format!("{}",
        config.has_option(& "database".to_string(), & "host".to_string()))).as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("has_missing = "); __sifr_concat.push_str((format!("{}",
        config.has_option(& "database".to_string(), & "missing".to_string()))).as_str());
        __sifr_concat }
    );
}
fn demo_gzip() {
    println!("=== gzip ===");
    let data: String = "Sifr stdlib gzip compression!".to_string();
    let compressed: Vec<u8> = compress(&data);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(21usize + 0usize);
        __sifr_concat.push_str("compressed len > 0 = "); __sifr_concat
        .push_str((format!("{}", & SifrInt::from(compressed.len()) > &
        SifrInt::from_i64(0))).as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), IOError> = (|| {
        let decompressed: String = decompress(&compressed)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(15usize +
            decompressed.len()); __sifr_concat.push_str("decompressed = "); __sifr_concat
            .push_str((decompressed).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(7usize +
            0usize); __sifr_concat.push_str("error: "); __sifr_concat.push_str((e.message
            .clone()).as_str()); __sifr_concat }
        );
    }
}
fn demo_zipfile() {
    println!("=== zipfile ===");
    let zf: __SifrStdlib_sifr_x2ezipfile_x2eZipFile = __SifrStdlib_sifr_x2ezipfile_x2eZipFile::new(
        "/tmp/sifr_demo_zipfile.zip".to_string(),
        "a".to_string(),
        SifrInt::from_i64(0),
    );
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _c: () = zf.create()?;
        println!("zip created = true");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("create error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _w: () = zf
            .write(&"demo.txt".to_string(), &"Hello from ZipFile!".to_string())?;
        let content: String = zf.read(&"demo.txt".to_string())?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            content.len()); __sifr_concat.push_str("zip content = "); __sifr_concat
            .push_str((content).as_str()); __sifr_concat }
        );
        let names: Vec<String> = zf.namelist()?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(19usize +
            0usize); __sifr_concat.push_str("zip namelist len = "); __sifr_concat
            .push_str((format!("{}", SifrInt::from(names.len()))).as_str());
            __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(11usize +
            0usize); __sifr_concat.push_str("zip error: "); __sifr_concat.push_str((e
            .message.clone()).as_str()); __sifr_concat }
        );
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _r: () = remove_file(&"/tmp/sifr_demo_zipfile.zip".to_string())?;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
    }
}
fn main() {
    demo_operator();
    demo_calendar();
    demo_html();
    demo_sys();
    demo_configparser();
    demo_gzip();
    demo_zipfile();
}
