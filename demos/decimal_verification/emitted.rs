// src/main.rs
mod __sifr_project_nominals {
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct DecimalConversionError {
        pub message: String,
    }
    impl DecimalConversionError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for DecimalConversionError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for DecimalConversionError {}
}
pub use __sifr_project_nominals::DecimalConversionError;
pub use __sifr_project_nominals::DivisionError;

mod __sifr_project_unions {
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0 {
        __SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
            crate::__sifr_project_nominals::DecimalConversionError,
        ),
        __SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
            crate::__sifr_project_nominals::DivisionError,
        ),
    }
    impl From<crate::__sifr_project_nominals::DecimalConversionError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::DecimalConversionError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::__sifr_project_nominals::DivisionError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0 {
        fn from(value: crate::__sifr_project_nominals::DivisionError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0;
use ::rust_decimal::Decimal;
use ::bigdecimal::BigDecimal;
use ::sifr_runtime::SifrInt;
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrIoNativeFileHandle {
    _id: String,
}
impl __SifrIoNativeFileHandle {
    fn new(id: String) -> Self {
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
fn json_load_tokens(text: &String) -> Result<Vec<String>, JSONDecodeError> {
    ::sifr_stdlib::json::json_load_tokens(text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| JSONDecodeError {
            message: __sifr_bridge_error.message().to_string(),
            line: SifrInt::from(__sifr_bridge_error.line()),
            column: SifrInt::from(__sifr_bridge_error.column()),
        })
}
fn json_validate_integer_digit_limits(text: &String) -> Result<(), JsonLimitError> {
    ::sifr_stdlib::json::json_validate_integer_digit_limits(text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| JsonLimitError {
            message: __sifr_bridge_error.message().to_string(),
            limit: SifrInt::from(__sifr_bridge_error.limit()),
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eIOBase {
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn new() -> Self {
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
    fn close(&mut self) {
        self._closed = true;
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
        let _ = offset.clone();
        let _ = whence.clone();
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn tell(&self) -> Result<SifrInt, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn readable(&self) -> bool {
        false
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn writable(&self) -> bool {
        false
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn seekable(&self) -> bool {
        false
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "IOBase(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eTextIOBase {
    iobase: __SifrStdlib_sifr_x2eio_x2eIOBase,
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
struct __SifrStdlib_sifr_x2eio_x2eBinaryIOBase {
    iobase: __SifrStdlib_sifr_x2eio_x2eIOBase,
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
struct __SifrIoFileHandle {
    _handle: __SifrIoNativeFileHandle,
    _mode: String,
    _closed: bool,
}
impl __SifrIoFileHandle {
    fn new(handle: __SifrIoNativeFileHandle, mode: String) -> Self {
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
    fn close(&mut self) {
        if self._closed {
            return;
        }
        file_close(&self._handle);
        self._closed = true;
    }
}
impl __SifrIoFileHandle {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrIoFileHandle {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        file_flush(&self._handle)
    }
}
impl __SifrIoFileHandle {
    fn read(&self) -> Result<String, IOError> {
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
    fn write(&self, data: &String) -> Result<(), IOError> {
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
    fn readline(&self) -> Result<Option<String>, IOError> {
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
    fn readlines(&self) -> Result<Vec<String>, IOError> {
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
    fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !self.readable() {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        file_read_bytes(&self._handle, (size.clone()).clone())
    }
}
impl __SifrIoFileHandle {
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
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
    fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        file_seek(&self._handle, (offset.clone()).clone(), (whence.clone()).clone())
    }
}
impl __SifrIoFileHandle {
    fn tell(&self) -> Result<SifrInt, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        file_tell(&self._handle)
    }
}
impl __SifrIoFileHandle {
    fn readable(&self) -> bool {
        _mode_is_readable(&self._mode)
    }
}
impl __SifrIoFileHandle {
    fn writable(&self) -> bool {
        _mode_is_writable(&self._mode)
    }
}
impl __SifrIoFileHandle {
    fn seekable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrIoFileHandle {
    fn __enter__(&self) -> __SifrIoFileHandle {
        self.clone()
    }
}
impl __SifrIoFileHandle {
    fn __exit__(&mut self) {
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
struct __SifrIoBinaryFileHandle {
    _handle: __SifrIoNativeFileHandle,
    _mode: String,
    _closed: bool,
}
impl __SifrIoBinaryFileHandle {
    fn new(handle: __SifrIoNativeFileHandle, mode: String) -> Self {
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
    fn close(&mut self) {
        if self._closed {
            return;
        }
        file_close(&self._handle);
        self._closed = true;
    }
}
impl __SifrIoBinaryFileHandle {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrIoBinaryFileHandle {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        file_flush(&self._handle)
    }
}
impl __SifrIoBinaryFileHandle {
    fn read_bytes(&self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !self.readable() {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        file_read_bytes(&self._handle, (size.clone()).clone())
    }
}
impl __SifrIoBinaryFileHandle {
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
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
    fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        file_seek(&self._handle, (offset.clone()).clone(), (whence.clone()).clone())
    }
}
impl __SifrIoBinaryFileHandle {
    fn tell(&self) -> Result<SifrInt, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        file_tell(&self._handle)
    }
}
impl __SifrIoBinaryFileHandle {
    fn readable(&self) -> bool {
        _mode_is_readable(&self._mode)
    }
}
impl __SifrIoBinaryFileHandle {
    fn writable(&self) -> bool {
        _mode_is_writable(&self._mode)
    }
}
impl __SifrIoBinaryFileHandle {
    fn seekable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrIoBinaryFileHandle {
    fn __enter__(&self) -> __SifrIoBinaryFileHandle {
        self.clone()
    }
}
impl __SifrIoBinaryFileHandle {
    fn __exit__(&mut self) {
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
struct __SifrIoTextFileHandle {
    _binary: __SifrIoBinaryFileHandle,
    _encoding: __SifrStdlib_sifr_x2eencoding_x2eEncoding,
    _decode_errors: __SifrStdlib_sifr_x2eencoding_x2eDecodeErrorHandler,
    _encode_errors: __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler,
}
impl __SifrIoTextFileHandle {
    fn new(
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
    fn close(&mut self) {
        self._binary.close();
    }
}
impl __SifrIoTextFileHandle {
    fn closed(&self) -> bool {
        self._binary.closed()
    }
}
impl __SifrIoTextFileHandle {
    fn flush(&self) -> Result<(), IOError> {
        self._binary.flush()
    }
}
impl __SifrIoTextFileHandle {
    fn read(&self) -> Result<String, IOError> {
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
    fn write(&self, text: &String) -> Result<(), IOError> {
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
    fn readline(&self) -> Result<Option<String>, IOError> {
        Err(
            IOError::new(
                "TextFileHandle.readline is deferred; use read().split(\"\\n\")"
                    .to_string(),
            ),
        )
    }
}
impl __SifrIoTextFileHandle {
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        Err(
            IOError::new(
                "TextFileHandle.readlines is deferred; use read().split(\"\\n\")"
                    .to_string(),
            ),
        )
    }
}
impl __SifrIoTextFileHandle {
    fn seek(&self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
        self._binary.seek(offset, whence)
    }
}
impl __SifrIoTextFileHandle {
    fn tell(&self) -> Result<SifrInt, IOError> {
        self._binary.tell()
    }
}
impl __SifrIoTextFileHandle {
    fn readable(&self) -> bool {
        self._binary.readable()
    }
}
impl __SifrIoTextFileHandle {
    fn writable(&self) -> bool {
        self._binary.writable()
    }
}
impl __SifrIoTextFileHandle {
    fn seekable(&self) -> bool {
        self._binary.seekable()
    }
}
impl __SifrIoTextFileHandle {
    fn __enter__(&self) -> __SifrIoTextFileHandle {
        self.clone()
    }
}
impl __SifrIoTextFileHandle {
    fn __exit__(&mut self) {
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
struct __SifrStdlib_sifr_x2eio_x2eTextReader {
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn new() -> Self {
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
    fn read(&self) -> Result<String, IOError> {
        Err(
            IOError::new(
                "TextReader direct construction is unsupported; use open_text"
                    .to_string(),
            ),
        )
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn readline(&self) -> Result<Option<String>, IOError> {
        Err(
            IOError::new(
                "TextReader.readline is deferred; use read().split(\"\\n\")".to_string(),
            ),
        )
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        Err(
            IOError::new(
                "TextReader.readlines is deferred; use read().split(\"\\n\")".to_string(),
            ),
        )
    }
}
impl __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn close(&mut self) {
        self._closed = true;
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextReader {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "TextReader(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eTextWriter {
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eTextWriter {
    fn new() -> Self {
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
    fn write(&self, text: &String) -> Result<(), IOError> {
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
    fn close(&mut self) {
        self._closed = true;
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2eio_x2eTextWriter {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "TextWriter(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2eio_x2eStringIO {
    _buffer: String,
    _cursor: SifrInt,
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn new(initial: String) -> Self {
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
    fn close(&mut self) {
        self._closed = true;
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn read(&mut self, size: &Option<SifrInt>) -> Result<String, IOError> {
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
    fn write(&mut self, data: &String) -> Result<(), IOError> {
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
    fn getvalue(&self) -> String {
        self._buffer.clone()
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn seek(&mut self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
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
    fn tell(&self) -> Result<SifrInt, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(self._cursor.clone())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn readable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn writable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn seekable(&self) -> bool {
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
struct __SifrStdlib_sifr_x2eio_x2eBytesIO {
    _buffer: Vec<u8>,
    _cursor: SifrInt,
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn new(initial: Vec<u8>) -> Self {
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
    fn close(&mut self) {
        self._closed = true;
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn closed(&self) -> bool {
        self._closed
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn read_bytes(&mut self, size: &Option<SifrInt>) -> Result<Vec<u8>, IOError> {
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
    fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
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
    fn getvalue(&self) -> Vec<u8> {
        self._buffer.clone()
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn seek(&mut self, offset: &SifrInt, whence: &SifrInt) -> Result<SifrInt, IOError> {
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
    fn tell(&self) -> Result<SifrInt, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(self._cursor.clone())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn readable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn writable(&self) -> bool {
        !(self._closed)
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn seekable(&self) -> bool {
        !(self._closed)
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
            return Err(e);
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
            return Err(e);
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
            return Err(e);
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0 {
    __SifrUnionVariant_4_x3aatom4_x3abool(bool),
    __SifrUnionVariant_4_x3aatom3_x3aint(SifrInt),
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
struct __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<SifrInt>,
    float_value: Option<f64>,
    str_value: Option<String>,
    array_items: Box<Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue>>,
    object_items: Box<Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)>>,
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn new(
        kind: String,
        bool_value: Option<bool>,
        int_value: Option<SifrInt>,
        float_value: Option<f64>,
        str_value: Option<String>,
    ) -> Self {
        let __sifr_field_init_0: String = kind;
        let __sifr_field_init_1: Option<bool> = bool_value;
        let __sifr_field_init_2: Option<SifrInt> = int_value.clone();
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
    fn is_null(&self) -> bool {
        (self.kind.clone() == "null")
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn is_bool(&self) -> bool {
        (self.kind.clone() == "bool")
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn is_int(&self) -> bool {
        (self.kind.clone() == "int")
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn is_float(&self) -> bool {
        (self.kind.clone() == "float")
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn is_str(&self) -> bool {
        (self.kind.clone() == "str")
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn is_array(&self) -> bool {
        (self.kind.clone() == "array")
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn is_object(&self) -> bool {
        (self.kind.clone() == "object")
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn as_bool(&self) -> Option<bool> {
        self.bool_value
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn as_int(&self) -> Option<SifrInt> {
        self.int_value.clone()
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn as_float(&self) -> Option<f64> {
        self.float_value
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn as_str(&self) -> Option<String> {
        self.str_value.clone()
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn as_array(&self) -> Option<Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue>> {
        if !self.is_array() {
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
    fn as_object(
        &self,
    ) -> Option<Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)>> {
        if !self.is_object() {
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
    fn at(&self, index: &SifrInt) -> Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
        if !self.is_array() {
            return None;
        }
        if (&index.clone() < &SifrInt::from_i64(0))
            || (&index.clone() >= &SifrInt::from(self.array_items.len()))
        {
            return None;
        }
        let value: Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> = {
            let __sifr_checked_read_collection = &self.array_items;
            let __sifr_checked_read_index = index.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        value
    }
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn get(&self, key: &String) -> Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
        if !self.is_object() {
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
    fn keys(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        if !self.is_object() {
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
    fn values(&self) -> Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
        let mut result: Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue> = vec![];
        if !self.is_object() {
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
    fn items(&self) -> Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)> {
        if !self.is_object() {
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
fn from_int(value: SifrInt) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    let int_value: Option<SifrInt> = Some(value.clone());
    __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
        "int".to_string(),
        None,
        (int_value).clone(),
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
fn _json_append_tokens(
    mut tokens: Vec<String>,
    value: &__SifrStdlib_sifr_x2ejson_x2eJsonValue,
) -> Vec<String> {
    tokens.push(format!("{}{}", value.kind.clone(), ""));
    if (value.kind.clone() == "bool") {
        let bool_value: Option<bool> = value.bool_value;
        if (bool_value.is_none()) {
            tokens.push("false".to_string());
        } else {
            if let Some(bool_value) = bool_value {
                tokens.push(format!("{}", bool_value).to_lowercase());
            }
        }
    } else {
        if (value.kind.clone() == "int") {
            let int_value: Option<SifrInt> = value.int_value.clone();
            if (int_value.is_none()) {
                tokens.push("0".to_string());
            } else {
                if let Some(int_value) = int_value.clone() {
                    tokens.push(format!("{}", int_value));
                }
            }
        } else {
            if (value.kind.clone() == "float") {
                let float_value: Option<f64> = value.float_value;
                if (float_value.is_none()) {
                    tokens.push("0.0".to_string());
                } else {
                    if let Some(float_value) = float_value {
                        tokens.push(format!("{}", float_value));
                    }
                }
            } else {
                if (value.kind.clone() == "str") {
                    let str_value: Option<String> = value.as_str();
                    if (str_value.is_none()) {
                        tokens.push("".to_string());
                    } else {
                        if let Some(str_value) = str_value {
                            tokens.push(str_value.clone());
                        }
                    }
                } else {
                    if (value.kind.clone() == "array") {
                        tokens
                            .push(format!("{}", SifrInt::from(value.array_items.len())));
                        for item in (value.array_items).as_ref().clone().iter().cloned()
                        {
                            tokens = _json_append_tokens(tokens, &item);
                        }
                    } else {
                        if (value.kind.clone() == "object") {
                            tokens
                                .push(
                                    format!("{}", SifrInt::from(value.object_items.len())),
                                );
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
fn dumps(
    value: &__SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0,
) -> String {
    match value {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0(
            value,
        ) => {
            return json_dump_tokens(&_json_bridge_tokens(value));
        }
        __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom4_x3abool(
            value,
        ) => {
            return json_dump_tokens(&_json_bridge_tokens(&from_bool((value).clone())));
        }
        __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom3_x3aint(
            value,
        ) => {
            return json_dump_tokens(
                &_json_bridge_tokens(&from_int((value.clone()).clone())),
            );
        }
        __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom5_x3afloat(
            value,
        ) => {
            return json_dump_tokens(&_json_bridge_tokens(&from_float((value).clone())));
        }
        value => {
            return json_dump_tokens(
                &_json_bridge_tokens(&from_str(&format!("{}", value))),
            );
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IOError {
    message: String,
    kind: String,
}
impl IOError {
    fn new(message: String) -> Self {
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
struct JSONDecodeError {
    message: String,
    line: SifrInt,
    column: SifrInt,
}
impl JSONDecodeError {
    fn new(message: String) -> Self {
        Self {
            message,
            line: SifrInt::from_i64(0),
            column: SifrInt::from_i64(0),
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
struct JsonIntegerRangeError {
    message: String,
    path: String,
    profile: String,
}
impl JsonIntegerRangeError {
    fn new(message: String) -> Self {
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
struct JsonLimitError {
    message: String,
    limit: SifrInt,
}
impl JsonLimitError {
    fn new(message: String) -> Self {
        Self {
            message,
            limit: SifrInt::from_i64(0),
        }
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
impl From<DivisionError> for Error {
    fn from(err: DivisionError) -> Self {
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
impl From<DecimalConversionError> for Error {
    fn from(err: DecimalConversionError) -> Self {
        Self::new(err.message)
    }
}
fn main() {
    println!("decimal_verification verification corpus and determinism gates demo");
    let d: Decimal = Decimal::from_i128_with_scale(-75_i128, 1);
    let bd: BigDecimal = BigDecimal::new(
        ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![181]),
        1,
    );
    println!(
        "{}", dumps(&
        __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom7_x3adecimal(Decimal::from_i128_with_scale(12300_i128,
        4)))
    );
    println!(
        "{}", dumps(&
        __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_4_x3aatom10_x3abigdecimal((BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
        vec![48, 12]), 4)).clone()))
    );
    let __sifr_try_res: Result<
        (),
        __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0,
    > = (|| {
        let d_floor: Decimal = ({
            let __sifr_decimal_left_result = Ok(d);
            let __sifr_decimal_right_result = Ok(
                Decimal::from_i128_with_scale(2_i128, 0),
            );
            __sifr_decimal_left_result
                .and_then(move |__sifr_decimal_left| {
                    __sifr_decimal_right_result
                        .and_then(move |__sifr_decimal_right| {
                            Decimal::checked_div(
                                    __sifr_decimal_left,
                                    __sifr_decimal_right,
                                )
                                .map(|__sifr_decimal_quotient| {
                                    __sifr_decimal_quotient.floor()
                                })
                                .map_or_else(
                                    || Err(
                                        if __sifr_decimal_right.is_zero() {
                                            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                                                DivisionError::new("division by zero".to_string()),
                                            )
                                        } else {
                                            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                                                DecimalConversionError::new(
                                                    "decimal // operation overflowed its exact representation"
                                                        .to_string(),
                                                ),
                                            )
                                        },
                                    ),
                                    |__sifr_decimal_value| Ok(__sifr_decimal_value),
                                )
                        })
                })
        })?;
        let d_remainder: Decimal = ({
            let __sifr_decimal_left_result = Ok(d);
            let __sifr_decimal_right_result = Ok(
                Decimal::from_i128_with_scale(2_i128, 0),
            );
            __sifr_decimal_left_result
                .and_then(move |__sifr_decimal_left| {
                    __sifr_decimal_right_result
                        .and_then(move |__sifr_decimal_right| {
                            Decimal::checked_div(
                                    __sifr_decimal_left,
                                    __sifr_decimal_right,
                                )
                                .and_then(|__sifr_decimal_quotient| {
                                    Decimal::checked_mul(
                                            __sifr_decimal_quotient.floor(),
                                            __sifr_decimal_right,
                                        )
                                        .and_then(|__sifr_decimal_product| Decimal::checked_sub(
                                            __sifr_decimal_left,
                                            __sifr_decimal_product,
                                        ))
                                })
                                .map_or_else(
                                    || Err(
                                        if __sifr_decimal_right.is_zero() {
                                            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                                                DivisionError::new("division by zero".to_string()),
                                            )
                                        } else {
                                            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                                                DecimalConversionError::new(
                                                    "decimal % operation overflowed its exact representation"
                                                        .to_string(),
                                                ),
                                            )
                                        },
                                    ),
                                    |__sifr_decimal_value| Ok(__sifr_decimal_value),
                                )
                        })
                })
        })?;
        let bd_floor: BigDecimal = ({
            let __sifr_bigdecimal_left = bd.clone();
            let __sifr_bigdecimal_right = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![2]),
                    0,
                )
                .clone();
            if ::bigdecimal::Zero::is_zero(&__sifr_bigdecimal_right) {
                Err(DivisionError::new("division by zero".to_string()))
            } else {
                Ok(
                    ::bigdecimal::Context::new(
                            ::std::num::NonZeroU64::MIN.saturating_add(27),
                            ::bigdecimal::RoundingMode::HalfEven,
                        )
                        .round_decimal_ref(
                            &((&__sifr_bigdecimal_left / &__sifr_bigdecimal_right)
                                .with_scale_round(0, ::bigdecimal::RoundingMode::Floor)),
                        ),
                )
            }
        })
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                __e,
            ))?;
        let bd_remainder: BigDecimal = ({
            let __sifr_bigdecimal_left = bd.clone();
            let __sifr_bigdecimal_right = BigDecimal::new(
                    ::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&vec![2]),
                    0,
                )
                .clone();
            if ::bigdecimal::Zero::is_zero(&__sifr_bigdecimal_right) {
                Err(DivisionError::new("division by zero".to_string()))
            } else {
                Ok(
                    ::bigdecimal::Context::new(
                            ::std::num::NonZeroU64::MIN.saturating_add(27),
                            ::bigdecimal::RoundingMode::HalfEven,
                        )
                        .round_decimal_ref(
                            &(&__sifr_bigdecimal_left
                                - ((&__sifr_bigdecimal_left / &__sifr_bigdecimal_right)
                                    .with_scale_round(0, ::bigdecimal::RoundingMode::Floor)
                                    * &__sifr_bigdecimal_right)),
                        ),
                )
            }
        })
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                __e,
            ))?;
        println!("{}", d_floor);
        println!("{}", d_remainder);
        println!("{}", bd_floor);
        println!("{}", bd_remainder);
        let baseline_tmp_d: Decimal = ({
            let __sifr_decimal_left_result = Ok(
                Decimal::from_i128_with_scale(12345_i128, 4),
            );
            let __sifr_decimal_right_result = Ok(
                Decimal::from_i128_with_scale(30_i128, 1),
            );
            __sifr_decimal_left_result
                .and_then(move |__sifr_decimal_left| {
                    __sifr_decimal_right_result
                        .and_then(move |__sifr_decimal_right| {
                            Decimal::checked_mul(
                                    __sifr_decimal_left,
                                    __sifr_decimal_right,
                                )
                                .map_or_else(
                                    || Err(
                                        DecimalConversionError::new(
                                            "decimal * operation overflowed its exact representation"
                                                .to_string(),
                                        ),
                                    ),
                                    |__sifr_decimal_value| Ok(__sifr_decimal_value),
                                )
                        })
                })
        })
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                __e,
            ))?;
        let baseline_d: String = format!(
            "{}", baseline_tmp_d.round_dp_with_strategy({ let __scale = 3; (if __scale <
            0 { 0 } else { __scale }) as u32 },
            ::rust_decimal::RoundingStrategy::MidpointNearestEven)
        );
        let baseline_bd: String = format!(
            "{}", ::bigdecimal::Context::new(::std::num::NonZeroU64::MIN
            .saturating_add(27), ::bigdecimal::RoundingMode::HalfEven)
            .round_decimal_ref(&
            (BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
            vec![39, 228, 27, 50, 70, 190, 201, 177, 110, 57, 129, 21]), 28).clone() +
            BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
            vec![0]), 0).clone())).round(6)
        );
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&i < &SifrInt::from_i64(20)) {
            let loop_tmp_d: Decimal = ({
                let __sifr_decimal_left_result = Ok(
                    Decimal::from_i128_with_scale(12345_i128, 4),
                );
                let __sifr_decimal_right_result = Ok(
                    Decimal::from_i128_with_scale(30_i128, 1),
                );
                __sifr_decimal_left_result
                    .and_then(move |__sifr_decimal_left| {
                        __sifr_decimal_right_result
                            .and_then(move |__sifr_decimal_right| {
                                Decimal::checked_mul(
                                        __sifr_decimal_left,
                                        __sifr_decimal_right,
                                    )
                                    .map_or_else(
                                        || Err(
                                            DecimalConversionError::new(
                                                "decimal * operation overflowed its exact representation"
                                                    .to_string(),
                                            ),
                                        ),
                                        |__sifr_decimal_value| Ok(__sifr_decimal_value),
                                    )
                            })
                    })
            })
                .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                    __e,
                ))?;
            assert!(
                (format!("{}", loop_tmp_d.round_dp_with_strategy({ let __scale = 3; (if
                __scale < 0 { 0 } else { __scale }) as u32 },
                ::rust_decimal::RoundingStrategy::MidpointNearestEven)) == baseline_d)
            );
            assert!(
                (format!("{}", ::bigdecimal::Context::new(::std::num::NonZeroU64::MIN
                .saturating_add(27), ::bigdecimal::RoundingMode::HalfEven)
                .round_decimal_ref(&
                (BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
                vec![39, 228, 27, 50, 70, 190, 201, 177, 110, 57, 129, 21]), 28).clone()
                +
                BigDecimal::new(::bigdecimal::num_bigint::BigInt::from_signed_bytes_be(&
                vec![0]), 0).clone())).round(6)) == baseline_bd)
            );
            i = &i + &SifrInt::from_i64(1);
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        match __sifr_try_err {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass22_x3aDecimalConversionError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let error = __sifr_try_variant_error.clone();
                assert!(false, "{}", format!("{}", error));
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a226_x3a5_x3aclass13_x3aDivisionError1_x3a035_x3a5_x3aclass22_x3aDecimalConversionError1_x3a0::__SifrUnionVariant_5_x3aclass13_x3aDivisionError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let error = __sifr_try_variant_error.clone();
                assert!(false, "{}", format!("{}", error));
            }
        }
    }
    println!("deterministic decimal and bigdecimal corpus checks passed");
}
