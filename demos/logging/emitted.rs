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
    pub fn set_global_level(level: i64) {
        ::sifr_stdlib::logging::set_global_level(
            ::sifr_runtime::interop::SifrIntBridge::from(level),
        );
    }
    pub fn get_global_level() -> i64 {
        ::sifr_stdlib::logging::get_global_level().to_i64_saturating()
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
    pub const DEBUG: i64 = 10_i64;
    pub const INFO: i64 = 20_i64;
    pub const WARNING: i64 = 30_i64;
    pub const ERROR: i64 = 40_i64;
    pub const CRITICAL: i64 = 50_i64;
    pub const NOTSET: i64 = 0_i64;
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
        pub fn format(&self, level: &String, name: &String, msg: &String) -> String {
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
        pub _level: i64,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn new(level: i64) -> Self {
            let __sifr_field_init_0: i64 = level;
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
        pub fn set_level(&mut self, level: i64) {
            self._level = level;
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn level(&self) -> i64 {
            self._level
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
        pub fn _allows(&self, level_num: i64) -> bool {
            if (self._level == NOTSET) {
                return true;
            }
            (level_num >= self._level)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eStreamHandler {
        pub fn emit(&self, level: &String, name: &String, msg: &String) {
            let level_num: i64 = _level_name_to_num(level);
            if !(self._allows(level_num)) {
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
        pub _level: i64,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn new(path: String, level: i64) -> Self {
            let __sifr_field_init_0: String = {
                let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
                __sifr_concat.push_str((path).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
            let __sifr_field_init_1: i64 = level;
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
        pub fn set_level(&mut self, level: i64) {
            self._level = level;
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn level(&self) -> i64 {
            self._level
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
        pub fn _allows(&self, level_num: i64) -> bool {
            if (self._level == NOTSET) {
                return true;
            }
            (level_num >= self._level)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eFileHandler {
        pub fn emit(&self, level: &String, name: &String, msg: &String) {
            let level_num: i64 = _level_name_to_num(level);
            if !(self._allows(level_num)) {
                return;
            }
            let line: String = {
                let mut __sifr_concat: String = String::with_capacity(0usize + 1usize);
                __sifr_concat.push_str((self._formatter.format(level, name, msg)).as_str());
                __sifr_concat.push('\n');
                __sifr_concat
            };
            let __sifr_try_res: Result<(), IOError> = (|| {
                let mut fh: __SifrIoTextFileHandle = open_text(
                    &self._path,
                    &"a".to_string(),
                    &Some((utf8()).clone()),
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
        pub _level: i64,
        pub _formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn new(level: i64) -> Self {
            let __sifr_field_init_0: i64 = level;
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
        pub fn set_level(&mut self, level: i64) {
            self._level = level;
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eNullHandler {
        pub fn level(&self) -> i64 {
            self._level
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
        pub fn emit(&self, level: &String, name: &String, msg: &String) {
            let _ = (level).clone();
            let _ = (name).clone();
            let _ = (msg).clone();
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
        pub _level: i64,
        pub _log_path: String,
        pub _handler_kind: String,
        pub _handler_path: String,
        pub _handler_level: i64,
        pub _handler_fmt: String,
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn new(name: String, level: i64) -> Self {
            let __sifr_field_init_0: String = name;
            let __sifr_field_init_1: i64 = level;
            let __sifr_field_init_2: String = "".to_string();
            let __sifr_field_init_3: String = "".to_string();
            let __sifr_field_init_4: String = "".to_string();
            let __sifr_field_init_5: i64 = NOTSET;
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
        pub fn set_level(&mut self, level: i64) {
            self._level = level;
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn set_file(&mut self, path: &String) {
            self._log_path = {
                let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
                __sifr_concat.push_str((path).as_str());
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
            self._handler_level = NOTSET;
            self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _handler_allows(&self, level_num: i64) -> bool {
            if (self._handler_level == NOTSET) {
                return true;
            }
            (level_num >= self._handler_level)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _handler_line(&self, level: &String, msg: &String) -> String {
            let formatter: __SifrStdlib_sifr_x2elogging_x2eFormatter = __SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                self._handler_fmt.clone(),
            );
            formatter.format(level, &self._name.clone(), msg)
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn _emit(&self, level: &String, level_num: i64, msg: &String) {
            if (self._level > level_num) {
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
                        __sifr_concat.push_str((self._handler_line(level, msg)).as_str());
                        __sifr_concat.push('\n');
                        __sifr_concat
                    };
                    let __sifr_try_res: Result<(), IOError> = (|| {
                        let mut fh: __SifrIoTextFileHandle = open_text(
                            &self._handler_path,
                            &"a".to_string(),
                            &Some((utf8()).clone()),
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
                __sifr_concat.push_str((level).as_str());
                __sifr_concat.push_str("] ");
                __sifr_concat.push_str((self._name.clone()).as_str());
                __sifr_concat.push_str(": ");
                __sifr_concat.push_str((msg).as_str());
                __sifr_concat
            };
            println!("{}", line);
            if (self._log_path.clone() != "") {
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let mut fh: __SifrIoTextFileHandle = open_text(
                        &self._log_path,
                        &"a".to_string(),
                        &Some((utf8()).clone()),
                        &None,
                    )?;
                    let __sifr_try_res: Result<(), IOError> = (|| {
                        let _ = fh
                            .write(
                                &({
                                    let mut __sifr_concat: String = String::with_capacity(
                                        line.len() + 1usize,
                                    );
                                    __sifr_concat.push_str((line).as_str());
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
        pub fn debug(&self, msg: &String) {
            self._emit(&"DEBUG".to_string(), DEBUG, msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn info(&self, msg: &String) {
            self._emit(&"INFO".to_string(), INFO, msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn warning(&self, msg: &String) {
            self._emit(&"WARNING".to_string(), WARNING, msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn error(&self, msg: &String) {
            self._emit(&"ERROR".to_string(), ERROR, msg);
        }
    }
    impl __SifrStdlib_sifr_x2elogging_x2eLogger {
        pub fn critical(&self, msg: &String) {
            self._emit(&"CRITICAL".to_string(), CRITICAL, msg);
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
    pub fn _level_name_to_num(level: &String) -> i64 {
        if (level).as_str() == "DEBUG" {
            return DEBUG;
        }
        if (level).as_str() == "INFO" {
            return INFO;
        }
        if (level).as_str() == "WARNING" {
            return WARNING;
        }
        if (level).as_str() == "ERROR" {
            return ERROR;
        }
        if (level).as_str() == "CRITICAL" {
            return CRITICAL;
        }
        NOTSET
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
pub use __sifr_project_nominals::IOError;
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
fn set_global_level(level: i64) {
    ::sifr_stdlib::logging::set_global_level(
        ::sifr_runtime::interop::SifrIntBridge::from(level),
    );
}
fn get_global_level() -> i64 {
    ::sifr_stdlib::logging::get_global_level().to_i64_saturating()
}
const DEBUG: i64 = 10_i64;
const INFO: i64 = 20_i64;
const WARNING: i64 = 30_i64;
const ERROR: i64 = 40_i64;
const CRITICAL: i64 = 50_i64;
const NOTSET: i64 = 0_i64;
fn _level_name_to_num(level: &String) -> i64 {
    if (level).as_str() == "DEBUG" {
        return DEBUG;
    }
    if (level).as_str() == "INFO" {
        return INFO;
    }
    if (level).as_str() == "WARNING" {
        return WARNING;
    }
    if (level).as_str() == "ERROR" {
        return ERROR;
    }
    if (level).as_str() == "CRITICAL" {
        return CRITICAL;
    }
    NOTSET
}
fn basicConfig(level: i64) -> __SifrStdlib_sifr_x2elogging_x2eLogger {
    set_global_level(level);
    __SifrStdlib_sifr_x2elogging_x2eLogger::new("root".to_string(), level)
}
fn getLogger(name: &String) -> __SifrStdlib_sifr_x2elogging_x2eLogger {
    let level: i64 = get_global_level();
    __SifrStdlib_sifr_x2elogging_x2eLogger::new((name).clone(), level)
}
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
fn random_int(min: i64, max: i64) -> i64 {
    ::sifr_stdlib::random::random_int(
            ::sifr_runtime::interop::SifrIntBridge::from(min),
            ::sifr_runtime::interop::SifrIntBridge::from(max),
        )
        .to_i64_saturating()
}
fn random_float() -> f64 {
    ::sifr_stdlib::random::random_float()
}
fn random_uniform(min: f64, max: f64) -> f64 {
    ::sifr_stdlib::random::random_uniform(min, max)
}
fn random_randrange(start: i64, stop: i64, step: i64) -> Result<i64, ValueError> {
    ::sifr_stdlib::random::random_randrange(
            ::sifr_runtime::interop::SifrIntBridge::from(start),
            ::sifr_runtime::interop::SifrIntBridge::from(stop),
            ::sifr_runtime::interop::SifrIntBridge::from(step),
        )
        .map(|__sifr_bridge_ok| __sifr_bridge_ok.to_i64_saturating())
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn random_gauss(mu: f64, sigma: f64) -> f64 {
    ::sifr_stdlib::random::random_gauss(mu, sigma)
}
fn random_module_state_words() -> Vec<i64> {
    ::sifr_stdlib::random::random_module_state_words()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn random_module_state_index() -> i64 {
    ::sifr_stdlib::random::random_module_state_index().to_i64_saturating()
}
fn random_module_state_gauss_next() -> Option<f64> {
    ::sifr_stdlib::random::random_module_state_gauss_next()
}
fn random_module_set_state(
    words: &Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
) -> Result<(), ValueError> {
    ::sifr_stdlib::random::random_module_set_state(
            &words
                .iter()
                .copied()
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
    wrapcol: i64,
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
    let n: i64 = random_int(100000_i64, 999999_i64);
    format!("{}", n)
}
fn mktemp_path(prefix: &String) -> String {
    let suffix: String = _random_suffix();
    let mut root: String = gettempdir();
    let mut __sifr_chars_root: Vec<char> = root.chars().collect::<Vec<char>>();
    if ((__sifr_chars_root.len() as i64) == (0_i64)) {
        root = "/tmp".to_string();
        __sifr_chars_root = root.chars().collect::<Vec<char>>();
    } else {
        let last: Option<String> = __sifr_chars_root
            .get(((root.chars().count() as i64) - (1_i64)) as usize)
            .map(|c| c.to_string());
        if let Some(last) = last {
            if last == "/" {
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
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
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
fn collect_logger_actual(base: &String) -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let app_log: String = {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 8usize);
        __sifr_concat.push_str((base).as_str());
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
        app.set_level(INFO);
        app.info(&"start".to_string());
        app.warning(&"warn".to_string());
        app.debug(&"hidden".to_string());
        let app_content: String = read_text(&app_log)?;
        app_ok = app_content == "[INFO] demo: start\n[WARNING] demo: warn\n";
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = format!("{}", e.message.clone());
    }
    actual.push(app_ok);
    actual
}
fn collect_root_and_handler_actual(base: &String) -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let root_log: String = {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 9usize);
        __sifr_concat.push_str((base).as_str());
        __sifr_concat.push_str("/root.log");
        __sifr_concat
    };
    let handler_log: String = {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 12usize);
        __sifr_concat.push_str((base).as_str());
        __sifr_concat.push_str("/handler.log");
        __sifr_concat
    };
    let mut root_ok: bool = false;
    let mut handler_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _w1: () = write_text(&root_log, &"".to_string())?;
        let _w2: () = write_text(&handler_log, &"".to_string())?;
        let mut root: __SifrStdlib_sifr_x2elogging_x2eLogger = basicConfig(WARNING);
        root.set_file(&root_log);
        root.info(&"skip".to_string());
        root.error(&"boom".to_string());
        let root_content: String = read_text(&root_log)?;
        root_ok = root_content == "[ERROR] root: boom\n";
        let mut handler: __SifrStdlib_sifr_x2elogging_x2eFileHandler = __SifrStdlib_sifr_x2elogging_x2eFileHandler::new(
            format!("{}{}", handler_log, ""),
            0_i64,
        );
        handler
            .set_formatter(
                &__SifrStdlib_sifr_x2elogging_x2eFormatter::new(
                    "%(levelname)s:%(message)s".to_string(),
                ),
            );
        handler.emit(&"INFO".to_string(), &"demo".to_string(), &"formatted".to_string());
        let handler_content: String = read_text(&handler_log)?;
        handler_ok = handler_content == "INFO:formatted\n";
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
fn collect_safety_actual(base: &String) -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let missing_log: String = {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 20usize);
        __sifr_concat.push_str((base).as_str());
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
    actual.push((INFO == (20_i64)) && (WARNING == (30_i64)));
    actual
}
fn collect_cleanup_actual(base: &String) -> Vec<bool> {
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
fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
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
