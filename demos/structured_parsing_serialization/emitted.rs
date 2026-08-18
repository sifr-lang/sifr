// src/main.rs
mod __sifr_project_nominals {
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
    pub struct TOMLDecodeError {
        pub message: String,
        pub line: i64,
        pub column: i64,
    }
    impl TOMLDecodeError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                line: 0,
                column: 0,
            }
        }
    }
    impl ::std::fmt::Display for TOMLDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for TOMLDecodeError {}
    impl From<IOError> for Error {
        fn from(err: IOError) -> Self {
            Self::new(err.message)
        }
    }
    impl From<TOMLDecodeError> for Error {
        fn from(err: TOMLDecodeError) -> Self {
            Self::new(err.message)
        }
    }
}
pub use __sifr_project_nominals::Error;
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::TOMLDecodeError;

mod __sifr_project_unions {
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        __SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(crate::Error),
        __SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(crate::IOError),
    }
    impl From<crate::Error>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        fn from(value: crate::Error) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::IOError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        fn from(value: crate::IOError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
    #[derive(Debug, Clone)]
    pub enum __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        __SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(crate::Error),
        __SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(crate::TOMLDecodeError),
    }
    impl From<crate::Error>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::Error) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                value,
            )
        }
    }
    impl From<crate::TOMLDecodeError>
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn from(value: crate::TOMLDecodeError) -> Self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                value,
            )
        }
    }
    impl ::std::fmt::Display
    for __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0 {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            match self {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                    v,
                ) => {
                    return write!(f, "{}", v);
                }
            }
        }
    }
}
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0;
pub use __sifr_project_unions::__SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0;
use ::std::collections::HashMap;
use ::rust_decimal::Decimal;
use ::bigdecimal::BigDecimal;
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
            return Err(IOError::new(e.message));
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
        encoding_canonical_label(&self.label)
    }
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncoding {
    fn is_supported(&self) -> bool {
        encoding_is_supported(&self.label)
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
            let outcome: __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome = encoding_decode_incremental_outcome(
                data,
                &self._pending,
                &self._encoding.clone().label,
                &self._errors.clone().name,
                r#final,
            )?;
            let next_pending: Vec<u8> = encoding_decode_incremental_pending(
                data,
                &self._pending,
                &self._encoding.clone().label,
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
                return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
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
                return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
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
fn encoding_is_supported(label: &String) -> bool {
    _encoding_is_supported_impl(label)
}
fn encoding_canonical_label(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_text(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_recoveries(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_outcome(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_incremental_outcome(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_decode_incremental_pending(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
        }
    }
}
fn encoding_encode_bytes(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
        }
    }
}
fn encoding_encode_recoveries(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
        }
    }
}
fn encoding_encode_outcome(
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
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
        return Ok(encoding_decode_outcome(data, &enc.label.clone(), &handler_name));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eDecodeError::new(e.message));
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
        return Ok(encoding_encode_outcome(text, &enc.label.clone(), &handler_name));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
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
            return Err(__SifrStdlib_sifr_x2eencoding_x2eEncodeError::new(e.message));
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
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrStdlib_sifr_x2eio_x2eIOBase {
    fn tell(&self) -> Result<i64, IOError> {
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
        Ok(())
    }
}
impl __SifrIoFileHandle {
    fn read(&self) -> Result<String, IOError> {
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
    fn write(&self, data: &String) -> Result<(), IOError> {
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
    fn readline(&self) -> Result<Option<String>, IOError> {
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
    fn readlines(&self) -> Result<Vec<String>, IOError> {
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
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
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
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
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
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrIoFileHandle {
    fn tell(&self) -> Result<i64, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
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
        false
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
        Ok(())
    }
}
impl __SifrIoBinaryFileHandle {
    fn read_bytes(&self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
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
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
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
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
}
impl __SifrIoBinaryFileHandle {
    fn tell(&self) -> Result<i64, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
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
        false
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
                        return Err(IOError::new(e.message));
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
                                __sifr_concat.push_str((e.message).as_str());
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
                        return Err(IOError::new(e.message));
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
                                __sifr_concat.push_str((e.message).as_str());
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
    _cursor: i64,
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
    fn read(&mut self, size: Option<i64>) -> Result<String, IOError> {
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
    fn write(&mut self, data: &String) -> Result<(), IOError> {
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
    fn getvalue(&self) -> String {
        self._buffer.clone()
    }
}
impl __SifrStdlib_sifr_x2eio_x2eStringIO {
    fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
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
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(self._cursor)
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
    _cursor: i64,
    _closed: bool,
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn new(initial: Vec<u8>) -> Self {
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
    fn read_bytes(&mut self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
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
    fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
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
    fn getvalue(&self) -> Result<Vec<u8>, IOError> {
        Ok(self._buffer.clone())
    }
}
impl __SifrStdlib_sifr_x2eio_x2eBytesIO {
    fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
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
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(self._cursor)
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
            return Err(IOError::new(e.message));
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
            return Err(IOError::new(e.message));
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
            return Err(IOError::new(e.message));
        }
    }
}
fn __const_DEFAULTSECT() -> String {
    "DEFAULT".to_string().to_string()
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2econfigparser_x2eParsingError {
    line: i64,
    message: String,
}
impl __SifrStdlib_sifr_x2econfigparser_x2eParsingError {
    fn new(line: i64, message: String) -> Self {
        let __sifr_field_init_0: i64 = line;
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
struct __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
    name: String,
    _values: HashMap<String, Option<String>>,
}
impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
    fn new(name: String, values: HashMap<String, Option<String>>) -> Self {
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
    fn has_option(&self, option: &String) -> bool {
        _has_option_key(&self._values, &_normalize_option(option))
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
    fn get(
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
            return Some(_resolve_interpolation(&value, &self._values, 0_i64));
        }
        _copy_optional_str(fallback)
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
    fn options(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![];
        for key in self._values.clone().keys().cloned() {
            names.push(key.clone());
        }
        names
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy {
    fn items(&self) -> Vec<(String, Option<String>)> {
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
struct __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    _defaults: HashMap<String, Option<String>>,
    _sections: HashMap<String, HashMap<String, Option<String>>>,
    strict: bool,
    allow_no_value: bool,
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn new(
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
                    let __assign_key = normalized.clone();
                    let __assign_value = _copy_optional_str(&value);
                    defaults_map.insert(__assign_key, __assign_value);
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
    fn defaults(&self) -> HashMap<String, Option<String>> {
        _copy_values(&self._defaults)
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn read_string(
        &mut self,
        text: &String,
    ) -> Result<(), __SifrStdlib_sifr_x2econfigparser_x2eParsingError> {
        let mut current_section: String = "".to_string();
        let default_section: String = _default_section();
        for (line_no, raw_line) in Box::new(
            (text.split('\n').map(|s| s.to_string()).collect::<Vec<String>>())
                .into_iter()
                .enumerate()
                .map(|__pair| ((__pair.0 as i64) + (1_i64), __pair.1)),
        ) {
            let line: String = raw_line.trim().to_string();
            if ((line == "") || line.starts_with("#")) || line.starts_with(";") {
                continue;
            }
            if line.starts_with("[") && line.ends_with("]") {
                let section_name: String = line
                    .chars()
                    .skip((1_i64) as usize)
                    .take(
                        (((line.chars().count() as i64) - (1_i64)) as usize)
                            - ((1_i64) as usize),
                    )
                    .collect::<String>()
                    .trim()
                    .to_string();
                if section_name == "" {
                    return Err(
                        __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                            line_no,
                            "section name is empty".to_string(),
                        ),
                    );
                }
                if section_name == default_section {
                    current_section = _default_section();
                    continue;
                }
                if self.strict && (self._sections).contains_key(&(section_name)) {
                    return Err(
                        __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                            line_no,
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
                if !((self._sections).contains_key(&(section_name))) {
                    self._sections.insert(section_name.clone(), HashMap::from([]));
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
                    line_no,
                )?;
                let (option_name, option_value) = parsed_option_pair;
                let __sifr_chars_option_name: Vec<char> = option_name
                    .chars()
                    .collect::<Vec<char>>();
                if (current_section == "") || (current_section == default_section) {
                    self._defaults
                        .insert(option_name.clone(), _copy_optional_str(&option_value));
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
                        if section_name != section_key {
                            continue;
                        }
                        if self.strict && _has_option_key(&section_values, &option_name)
                        {
                            return Err(
                                __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                                    line_no,
                                    format!("{}{}", "duplicate option: ", option_name),
                                ),
                            );
                        }
                        let mut updated_section: HashMap<String, Option<String>> = _copy_values(
                            &section_values,
                        );
                        {
                            let __assign_key = option_name.clone();
                            let __assign_value = _copy_optional_str(&option_value);
                            updated_section.insert(__assign_key, __assign_value);
                        }
                        self._sections.insert(section_name.clone(), updated_section);
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
    fn read(&mut self, path: &String) -> Result<Vec<String>, IOError> {
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
                        __sifr_concat.push_str((format!("{}", e.line)).as_str());
                        __sifr_concat.push_str(": ");
                        __sifr_concat.push_str((e.message).as_str());
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
            return Ok(Ok(vec![loaded_path.clone()]));
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
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn sections(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![];
        for section in self._sections.clone().keys().cloned() {
            names.push(section.clone());
        }
        names
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn has_section(&self, section: &String) -> bool {
        (self._sections).contains_key((section).as_str())
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn options(&self, section: &String) -> Vec<String> {
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        let mut names: Vec<String> = vec![];
        for option in merged.keys().cloned() {
            names.push(option.clone());
        }
        names
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn items(&self, section: &String) -> Vec<(String, Option<String>)> {
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
    fn _merged_section(&self, section: &String) -> HashMap<String, Option<String>> {
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
                    let __assign_key = option.clone();
                    let __assign_value = _copy_optional_str(&value);
                    merged.insert(__assign_key, __assign_value);
                }
            }
            return merged;
        }
        merged
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn has_option(&self, section: &String, option: &String) -> bool {
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
    fn get(
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
            if !(_has_option_key(&merged, &normalized)) {
                return _copy_optional_str(fallback);
            }
            let raw_value: Option<String> = _lookup_option(&merged, &normalized);
            let Some(raw_value) = raw_value else {
                return None;
            };
            if raw {
                return Some(raw_value);
            }
            return Some(_resolve_interpolation(&raw_value, &merged, 0_i64));
        }
        if !(self.has_section(section)) {
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
                return Some(_resolve_interpolation(&default_value, &merged, 0_i64));
            }
            return _copy_optional_str(fallback);
        }
        if !(_has_option_key(&merged, &normalized)) {
            return _copy_optional_str(fallback);
        }
        let raw_value2: Option<String> = _lookup_option(&merged, &normalized);
        let Some(raw_value2) = raw_value2 else {
            return None;
        };
        if raw {
            return Some(raw_value2);
        }
        Some(_resolve_interpolation(&raw_value2, &merged, 0_i64))
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn getint(
        &self,
        section: &String,
        option: &String,
        fallback: Option<i64>,
    ) -> Option<i64> {
        let raw: Option<String> = self.get(section, option, &None, false);
        let Some(raw) = raw else {
            return fallback;
        };
        let __sifr_try_res: Result<Option<i64>, ParseError> = (|| {
            let parsed: i64 = (raw)
                .parse::<i64>()
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            return Ok(Some(parsed));
            unreachable!("sifr try/except return capture fell through");
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
    fn getfloat(
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
            return Ok(Some(parsed));
            unreachable!("sifr try/except return capture fell through");
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
    fn getboolean(
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
    fn set(&mut self, section: &String, option: &String, value: &Option<String>) {
        let normalized: String = _normalize_option(option);
        let default_section: String = _default_section();
        if *section == default_section {
            self._defaults.insert(normalized.clone(), _copy_optional_str(value));
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
                let __assign_key = normalized.clone();
                let __assign_value = _copy_optional_str(value);
                updated_section.insert(__assign_key, __assign_value);
            }
            self._sections.insert(section_name.clone(), updated_section);
            return;
        }
        if !((self._sections).contains_key((section).as_str())) {
            self._sections.insert(section.clone(), HashMap::from([]));
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
            let __assign_key = normalized.clone();
            let __assign_value = _copy_optional_str(value);
            created_section.insert(__assign_key, __assign_value);
        }
        self._sections.insert(section.clone(), created_section);
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn add_section(&mut self, section: &String) {
        let default_section: String = _default_section();
        if *section == default_section {
            return;
        }
        if (self._sections).contains_key((section).as_str()) {
            return;
        }
        self._sections.insert(section.clone(), HashMap::from([]));
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn remove_option(&mut self, section: &String, option: &String) -> bool {
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
                self._sections
                    .insert(
                        section_name.clone(),
                        _without_option(&section_values, &normalized),
                    );
                return true;
            }
            return false;
        }
        false
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn remove_section(&mut self, section: &String) -> bool {
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
    fn proxy(
        &self,
        section: &String,
    ) -> Option<__SifrStdlib_sifr_x2econfigparser_x2eSectionProxy> {
        let default_section: String = _default_section();
        if (*section != default_section) && !(self.has_section(section)) {
            return None;
        }
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        Some(
            __SifrStdlib_sifr_x2econfigparser_x2eSectionProxy::new(
                (section).clone(),
                merged,
            ),
        )
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn to_ini_string(&self) -> String {
        let mut lines: Vec<String> = vec![];
        if ((self._defaults.len() as i64) > (0_i64)) {
            lines.push(format!("{}{}", format!("{}{}", "[", _default_section()), "]"));
            for (key, value) in self
                ._defaults
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if value.is_none() {
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
                if value.is_none() {
                    lines.push(key.clone());
                } else {
                    if let Some(value) = value {
                        lines.push(format!("{}{}", format!("{}{}", key, " = "), value));
                    }
                }
            }
            lines.push("".to_string());
        }
        if ((lines.len() as i64) > (0_i64)) {
            let maybe_last: Option<String> = {
                let __sifr_index_list = &lines;
                let __sifr_index_i = (lines.len() as i64) - (1_i64);
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if maybe_last.is_some() && (maybe_last == Some("".to_string())) {
                let _ = {
                    let Some(__sifr_nonempty_pop_value) = lines.pop() else {
                        unreachable!(
                            "compiler-verified non-empty pop should return Some"
                        );
                    };
                    __sifr_nonempty_pop_value
                };
            }
        }
        lines.join("\n")
    }
}
impl __SifrStdlib_sifr_x2econfigparser_x2eConfigParser {
    fn write(&self, path: &String) -> Result<(), IOError> {
        let payload: String = self.to_ini_string();
        write_text(path, &payload)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2econfigparser_x2eRawConfigParser {
    configparser: __SifrStdlib_sifr_x2econfigparser_x2eConfigParser,
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
            let __assign_key = key.clone();
            let __assign_value = _copy_optional_str(&value);
            copied.insert(__assign_key, __assign_value);
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
            let __assign_key = key.clone();
            let __assign_value = _copy_optional_str(&value);
            copied.insert(__assign_key, __assign_value);
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
            let __assign_key = key.clone();
            let __assign_value = _copy_values(&section);
            copied.insert(__assign_key, __assign_value);
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
    line_no: i64,
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
                line_no,
                "expected key=value or key:value entry".to_string(),
            ),
        );
    };
    let parts: Vec<String> = if (1_i64) < 0 {
        line.split(&delimiter).map(|s| s.to_string()).collect::<Vec<String>>()
    } else {
        line.splitn(((1_i64) + 1) as usize, &delimiter)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
    if ((parts.len() as i64) != (2_i64)) {
        return Err(
            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                line_no,
                "invalid option line".to_string(),
            ),
        );
    }
    let raw_key: Option<String> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let raw_value: Option<String> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let Some(raw_key) = raw_key else {
        return Err(
            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                line_no,
                "option name is missing".to_string(),
            ),
        );
    };
    let key: String = _normalize_option(&raw_key);
    if key == "" {
        return Err(
            __SifrStdlib_sifr_x2econfigparser_x2eParsingError::new(
                line_no,
                "option name is empty".to_string(),
            ),
        );
    }
    let Some(raw_value) = raw_value else {
        return Ok((key, None));
    };
    let stripped_value: Option<String> = _some_str(&raw_value.trim().to_string());
    Ok((key.clone(), stripped_value.clone()))
}
fn _char_at(text: &String, index: i64) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (index < (0_i64)) || (index >= (__sifr_chars_text.len() as i64)) {
        return "".to_string();
    }
    let ch: Option<String> = Some({
        let Some(__indexed_char) = __sifr_chars_text
            .get(index as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    });
    let Some(ch) = ch else {
        return "".to_string();
    };
    ch
}
fn _resolve_interpolation(
    value: &String,
    merged: &HashMap<String, Option<String>>,
    depth: i64,
) -> String {
    if depth >= (8_i64) {
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
    let mut i: i64 = 0_i64;
    while (i < (value.chars().count() as i64)) {
        let ch: String = _char_at(value, i);
        if ((ch == "%") && ((i + (1_i64)) < (value.chars().count() as i64)))
            && (_char_at(value, i + (1_i64)) == "(")
        {
            let mut j: i64 = i + (2_i64);
            let mut key: String = "".to_string();
            let mut matched: bool = false;
            while (j < (value.chars().count() as i64)) {
                let part: String = _char_at(value, j);
                if ((part == ")") && ((j + (1_i64)) < (value.chars().count() as i64)))
                    && (_char_at(value, j + (1_i64)) == "s")
                {
                    matched = true;
                    let normalized_key: String = _normalize_option(&key);
                    let replacement: Option<String> = _lookup_option(
                        merged,
                        &normalized_key,
                    );
                    if replacement.is_none() {
                        result.push_str("%(");
                        result.push_str((key).as_str());
                        result.push_str(")s");
                    } else {
                        if let Some(replacement) = replacement {
                            replaced = true;
                            result.push_str((replacement).as_str());
                        }
                    }
                    i = j + (2_i64);
                    break;
                }
                key.push_str((part).as_str());
                j += 1_i64;
            }
            if matched {
                continue;
            }
        }
        result.push_str((ch).as_str());
        i += 1_i64;
    }
    if replaced {
        return _resolve_interpolation(&result, merged, depth + (1_i64));
    }
    result
}
const QUOTE_ALL: i64 = 1_i64;
const QUOTE_NONNUMERIC: i64 = 2_i64;
const QUOTE_NONE: i64 = 3_i64;
const QUOTE_STRINGS: i64 = 4_i64;
const QUOTE_NOTNULL: i64 = 5_i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2ecsv_x2eDialect {
    delimiter: String,
    quotechar: String,
    escapechar: String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: String,
    quoting: i64,
}
impl __SifrStdlib_sifr_x2ecsv_x2eDialect {
    fn new(
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let mut resolved_quoting: i64 = quoting;
        _validate_char(&"delimiter".to_string(), &delimiter);
        if quotechar != "" {
            _validate_char(&"quotechar".to_string(), &quotechar);
        }
        if escapechar != "" {
            _validate_char(&"escapechar".to_string(), &escapechar);
        }
        if (quotechar == "") && (resolved_quoting != QUOTE_NONE) {
            resolved_quoting = QUOTE_NONE;
        }
        let __sifr_field_init_0: String = delimiter;
        let __sifr_field_init_1: String = quotechar;
        let __sifr_field_init_2: String = escapechar;
        let __sifr_field_init_3: bool = doublequote;
        let __sifr_field_init_4: bool = skipinitialspace;
        let __sifr_field_init_5: String = lineterminator;
        let __sifr_field_init_6: i64 = resolved_quoting;
        Self {
            delimiter: __sifr_field_init_0,
            quotechar: __sifr_field_init_1,
            escapechar: __sifr_field_init_2,
            doublequote: __sifr_field_init_3,
            skipinitialspace: __sifr_field_init_4,
            lineterminator: __sifr_field_init_5,
            quoting: __sifr_field_init_6,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDialect {}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2ecsv_x2eDialect {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(
            f,
            "Dialect(delimiter={}, quotechar={}, escapechar={}, doublequote={}, skipinitialspace={}, lineterminator={}, quoting={})",
            self.delimiter, self.quotechar, self.escapechar, self.doublequote, self
            .skipinitialspace, self.lineterminator, self.quoting
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecsv_x2ewriter {
    _rows: Vec<Vec<String>>,
    dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect,
}
impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
    fn new(
        dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let resolved_dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
            &dialect,
            &delimiter,
            &quotechar,
            &escapechar,
            doublequote,
            skipinitialspace,
            &lineterminator,
            quoting,
        );
        let __sifr_field_init_0: __SifrStdlib_sifr_x2ecsv_x2eDialect = resolved_dialect;
        let __sifr_field_init_1: Vec<Vec<String>> = vec![];
        Self {
            dialect: __sifr_field_init_0,
            _rows: __sifr_field_init_1,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
    fn writerow(&mut self, row: &Vec<String>) {
        let mut copied: Vec<String> = vec![];
        for value in row.iter().cloned() {
            copied.push(value.clone());
        }
        self._rows.push(copied.clone());
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
    fn writerows(&mut self, rows: &Vec<Vec<String>>) {
        for row in rows.iter().cloned() {
            let mut copied: Vec<String> = vec![];
            for value in row.iter().cloned() {
                copied.push(format!("{}{}", value, ""));
            }
            self._rows.push(copied.clone());
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2ewriter {
    fn getvalue(&self) -> String {
        format_csv(
            &self._rows,
            &Some((self.dialect.clone()).clone()),
            &",".to_string(),
            &"\"".to_string(),
            &"".to_string(),
            true,
            false,
            &"\n".to_string(),
            0_i64,
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    _fieldnames: Vec<String>,
    _rows: Vec<Vec<String>>,
    _pos: i64,
    restkey: String,
    restval: String,
    dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect,
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    fn new(
        text: String,
        fieldnames: Option<Vec<String>>,
        restkey: String,
        restval: String,
        dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: i64,
    ) -> Self {
        let resolved_dialect: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
            &dialect,
            &delimiter,
            &quotechar,
            &escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            quoting,
        );
        let all_rows: Vec<Vec<String>> = parse_csv(
            &text,
            &None,
            &format!("{}{}", resolved_dialect.delimiter, ""),
            &format!("{}{}", resolved_dialect.quotechar, ""),
            &format!("{}{}", resolved_dialect.escapechar, ""),
            resolved_dialect.doublequote,
            resolved_dialect.skipinitialspace,
            resolved_dialect.quoting,
        );
        let mut fieldnames_data: Vec<String> = vec![];
        let mut rows_data: Vec<Vec<String>> = vec![];
        if let Some(fieldnames) = fieldnames {
            for field in fieldnames.iter().cloned() {
                fieldnames_data.push(format!("{}{}", field, ""));
            }
            for row in all_rows.iter().cloned() {
                let mut copied_row: Vec<String> = vec![];
                for value in row.iter().cloned() {
                    copied_row.push(format!("{}{}", value, ""));
                }
                rows_data.push(copied_row.clone());
            }
        } else {
            for (index, row) in Box::new(
                (all_rows)
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
            ) {
                if index == (0_i64) {
                    for field in row.iter().cloned() {
                        fieldnames_data.push(format!("{}{}", field, ""));
                    }
                } else {
                    let mut copied_row2: Vec<String> = vec![];
                    for value in row.iter().cloned() {
                        copied_row2.push(format!("{}{}", value, ""));
                    }
                    rows_data.push(copied_row2.clone());
                }
            }
        }
        let __sifr_field_init_0: __SifrStdlib_sifr_x2ecsv_x2eDialect = resolved_dialect;
        let __sifr_field_init_1: String = restkey;
        let __sifr_field_init_2: String = restval;
        let __sifr_field_init_3: i64 = 0_i64;
        let __sifr_field_init_4: Vec<String> = fieldnames_data;
        let __sifr_field_init_5: Vec<Vec<String>> = rows_data;
        Self {
            dialect: __sifr_field_init_0,
            restkey: __sifr_field_init_1,
            restval: __sifr_field_init_2,
            _pos: __sifr_field_init_3,
            _fieldnames: __sifr_field_init_4,
            _rows: __sifr_field_init_5,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    fn fieldnames(&self) -> Vec<String> {
        let mut copied: Vec<String> = vec![];
        for field in self._fieldnames.clone().iter().cloned() {
            copied.push(format!("{}{}", field, ""));
        }
        copied
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    fn __next__(&mut self) -> Option<HashMap<String, String>> {
        while (self._pos < (self._rows.len() as i64)) {
            let row: Option<Vec<String>> = {
                let __sifr_index_list = &self._rows;
                let __sifr_index_i = self._pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            self._pos += 1_i64;
            let Some(row) = row else {
                return None;
            };
            if ((row.len() as i64) == (0_i64)) {
                continue;
            }
            return Some(
                _dict_reader_row(&self._fieldnames, &row, &self.restkey, &self.restval),
            );
        }
        None
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictReader {
    fn rows(&self) -> Vec<HashMap<String, String>> {
        let mut result: Vec<HashMap<String, String>> = vec![];
        for row in self._rows.clone().iter().cloned() {
            if ((row.len() as i64) == (0_i64)) {
                continue;
            }
            result
                .push(
                    _dict_reader_row(
                        &self._fieldnames,
                        &row,
                        &self.restkey,
                        &self.restval,
                    ),
                );
        }
        result
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fieldnames: Vec<String>,
    restval: String,
    extrasaction: String,
    _writer: __SifrStdlib_sifr_x2ecsv_x2ewriter,
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn new(
        fieldnames: Vec<String>,
        restval: String,
        extrasaction: String,
        dialect: Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: i64,
    ) -> Self {
        let mut fieldnames_data: Vec<String> = vec![];
        for field in fieldnames.iter().cloned() {
            fieldnames_data.push(format!("{}{}", field, ""));
        }
        let mut action: String = extrasaction.to_lowercase();
        if (action != "raise") && (action != "ignore") {
            action = "raise".to_string();
        }
        let writer_value: __SifrStdlib_sifr_x2ecsv_x2ewriter = __SifrStdlib_sifr_x2ecsv_x2ewriter::new(
            dialect,
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            lineterminator,
            quoting,
        );
        let __sifr_field_init_0: Vec<String> = fieldnames_data;
        let __sifr_field_init_1: String = restval;
        let __sifr_field_init_2: String = action;
        let __sifr_field_init_3: __SifrStdlib_sifr_x2ecsv_x2ewriter = writer_value;
        Self {
            fieldnames: __sifr_field_init_0,
            restval: __sifr_field_init_1,
            extrasaction: __sifr_field_init_2,
            _writer: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn writeheader(&mut self) {
        let mut current_writer: __SifrStdlib_sifr_x2ecsv_x2ewriter = self
            ._writer
            .clone();
        current_writer.writerow(&self.fieldnames.clone());
        self._writer = current_writer;
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn writerow(&mut self, row: &HashMap<String, String>) {
        let mut ordered: Vec<String> = vec![];
        for fieldname in self.fieldnames.clone().iter().cloned() {
            if row.contains_key(&fieldname) {
                ordered.push(_dict_value_at(row, &fieldname));
            } else {
                ordered.push(self.restval.clone());
            }
        }
        let mut current_writer: __SifrStdlib_sifr_x2ecsv_x2ewriter = self
            ._writer
            .clone();
        current_writer.writerow(&ordered);
        self._writer = current_writer;
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn writerows(&mut self, rows: &Vec<HashMap<String, String>>) {
        let mut current_writer: __SifrStdlib_sifr_x2ecsv_x2ewriter = self
            ._writer
            .clone();
        for row in rows.iter().cloned() {
            let mut ordered: Vec<String> = vec![];
            for fieldname in self.fieldnames.clone().iter().cloned() {
                if row.contains_key(&fieldname) {
                    ordered.push(_dict_value_at(&row, &fieldname));
                } else {
                    ordered.push(self.restval.clone());
                }
            }
            current_writer.writerow(&ordered);
        }
        self._writer = current_writer;
    }
}
impl __SifrStdlib_sifr_x2ecsv_x2eDictWriter {
    fn getvalue(&self) -> String {
        self._writer.getvalue()
    }
}
fn _copy_dialect(
    dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
) -> __SifrStdlib_sifr_x2ecsv_x2eDialect {
    __SifrStdlib_sifr_x2ecsv_x2eDialect::new(
        format!("{}{}", dialect.delimiter.clone(), ""),
        format!("{}{}", dialect.quotechar.clone(), ""),
        format!("{}{}", dialect.escapechar.clone(), ""),
        dialect.doublequote,
        dialect.skipinitialspace,
        format!("{}{}", dialect.lineterminator.clone(), ""),
        dialect.quoting,
    )
}
fn _validate_char(name: &String, value: &String) {
    let _ = (name).clone();
    let _ = (value).clone();
}
fn _resolve_dialect(
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> __SifrStdlib_sifr_x2ecsv_x2eDialect {
    if let Some(dialect) = dialect.as_ref() {
        return _copy_dialect(dialect);
    }
    __SifrStdlib_sifr_x2ecsv_x2eDialect::new(
        (delimiter).clone(),
        (quotechar).clone(),
        (escapechar).clone(),
        doublequote,
        skipinitialspace,
        (lineterminator).clone(),
        quoting,
    )
}
fn _quotechar_value(dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> String {
    let quotechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((dialect.quotechar.clone()).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if quotechar == "" {
        return "\"".to_string();
    }
    quotechar
}
fn _append_field(row: &mut Vec<String>, field: String) {
    row.push(format!("{}{}", field, ""));
}
fn _append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
    rows.push(row.clone());
}
fn _list_value_at(values: &Vec<String>, index: i64) -> String {
    if (index < (0_i64)) || (index >= (values.len() as i64)) {
        return "".to_string();
    }
    for (current_index, value) in Box::new(
        (values)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
    ) {
        if current_index == index {
            return {
                let mut __sifr_concat: String = String::with_capacity(
                    value.len() + 0usize,
                );
                __sifr_concat.push_str((value).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            };
        }
    }
    "".to_string()
}
fn _dict_value_at(values: &HashMap<String, String>, key: &String) -> String {
    for item_key in values.keys().cloned() {
        if item_key != *key {
            continue;
        }
        let value: Option<String> = values.get(&item_key).cloned();
        let Some(value) = value else {
            return "".to_string();
        };
        return {
            let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
            __sifr_concat.push_str((value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    "".to_string()
}
fn _first_char(text: &String) -> String {
    _char_at(text, 0_i64)
}
fn _last_char(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    _char_at(text, (text.chars().count() as i64) - (1_i64))
}
fn parse_csv(
    text: &String,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> Vec<Vec<String>> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        quoting,
    );
    let mut rows: Vec<Vec<String>> = vec![];
    let mut row: Vec<String> = vec![];
    let mut field: String = "".to_string();
    let mut in_quotes: bool = false;
    let mut field_started: bool = false;
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_text.len() as i64)) {
        let ch_value: String = _char_at(text, i);
        if in_quotes {
            if (resolved.escapechar != "") && (ch_value == resolved.escapechar) {
                if ((i + (1_i64)) < (__sifr_chars_text.len() as i64)) {
                    let escaped_value: String = _char_at(text, i + (1_i64));
                    field.push_str((escaped_value).as_str());
                    i += 2_i64;
                    continue;
                }
                field.push_str((ch_value).as_str());
                i += 1_i64;
                continue;
            }
            if (resolved.quotechar != "") && (ch_value == resolved.quotechar) {
                let quotechar: String = _quotechar_value(&resolved);
                if (resolved.doublequote
                    && ((i + (1_i64)) < (__sifr_chars_text.len() as i64)))
                    && (_char_at(text, i + (1_i64)) == quotechar.clone())
                {
                    field.push_str((quotechar).as_str());
                    i += 2_i64;
                    continue;
                }
                in_quotes = false;
                i += 1_i64;
                continue;
            }
            field.push_str((ch_value).as_str());
            i += 1_i64;
            continue;
        }
        if (!field_started && resolved.skipinitialspace) && (ch_value == " ") {
            i += 1_i64;
            continue;
        }
        if (resolved.escapechar != "") && (ch_value == resolved.escapechar) {
            if ((i + (1_i64)) < (__sifr_chars_text.len() as i64)) {
                let escaped_plain_value: String = _char_at(text, i + (1_i64));
                field.push_str((escaped_plain_value).as_str());
                field_started = true;
                i += 2_i64;
                continue;
            }
            field.push_str((ch_value).as_str());
            field_started = true;
            i += 1_i64;
            continue;
        }
        if (resolved.quoting != QUOTE_NONE) && (resolved.quotechar != "") {
            let quotechar2: String = _quotechar_value(&resolved);
            if ch_value == quotechar2 {
                in_quotes = true;
                field_started = true;
                i += 1_i64;
                continue;
            }
        }
        if (ch_value == resolved.delimiter) {
            _append_field(&mut row, field);
            field = "".to_string();
            field_started = false;
            i += 1_i64;
            continue;
        }
        if (ch_value == "\n") || (ch_value == "\r") {
            if ((ch_value == "\r") && ((i + (1_i64)) < (__sifr_chars_text.len() as i64)))
                && (_char_at(text, i + (1_i64)) == "\n")
            {
                i += 1_i64;
            }
            if ((row.len() as i64) == (0_i64)) && (field == "") {
                _append_row(&mut rows, vec![]);
            } else {
                _append_field(&mut row, field);
                _append_row(&mut rows, row);
            }
            row = vec![];
            field = "".to_string();
            field_started = false;
            i += 1_i64;
            continue;
        }
        field.push_str((ch_value).as_str());
        field_started = true;
        i += 1_i64;
    }
    if in_quotes {
        in_quotes = false;
    }
    if ((row.len() as i64) > (0_i64)) || (field != "") {
        _append_field(&mut row, field);
        _append_row(&mut rows, row);
    }
    rows
}
fn _needs_quote(field: &String, dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> bool {
    let __sifr_chars_field: Vec<char> = field.chars().collect::<Vec<char>>();
    if (dialect.quoting == QUOTE_ALL) {
        return true;
    }
    if (dialect.quoting == QUOTE_NONNUMERIC) {
        return true;
    }
    if (dialect.quoting == QUOTE_STRINGS) {
        return true;
    }
    if (dialect.quoting == QUOTE_NOTNULL) {
        return true;
    }
    if (dialect.quoting == QUOTE_NONE) {
        return false;
    }
    if (field).contains((dialect.delimiter.clone()).as_str()) {
        return true;
    }
    if field.contains(&"\n".to_string()) || field.contains(&"\r".to_string()) {
        return true;
    }
    if (dialect.quotechar.clone() != "") {
        let quotechar: String = _quotechar_value(dialect);
        if field.contains(&quotechar) {
            return true;
        }
    }
    if ((__sifr_chars_field.len() as i64) > (0_i64)) {
        let first: String = _first_char(field);
        let last: String = _last_char(field);
        if first == " " {
            return true;
        }
        if last == " " {
            return true;
        }
    }
    false
}
fn _quote_field(
    field: &String,
    dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
) -> String {
    let quotechar: String = _quotechar_value(dialect);
    let mut escaped: String = {
        let mut __sifr_concat: String = String::with_capacity(field.len() + 0usize);
        __sifr_concat.push_str((field).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if escaped.contains(&quotechar) {
        if dialect.doublequote {
            escaped = escaped
                .replace(&quotechar, &format!("{}{}", quotechar, quotechar));
        } else {
            if (dialect.escapechar.clone() != "") {
                let escapechar_value: String = {
                    let mut __sifr_concat: String = String::with_capacity(
                        0usize + 0usize,
                    );
                    __sifr_concat.push_str((dialect.escapechar.clone()).as_str());
                    __sifr_concat.push_str("");
                    __sifr_concat
                };
                escaped = escaped
                    .replace(&quotechar, &format!("{}{}", escapechar_value, quotechar));
            } else {
                escaped = escaped
                    .replace(&quotechar, &format!("{}{}", quotechar, quotechar));
            }
        }
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            (quotechar.len() + escaped.len()) + quotechar.len(),
        );
        __sifr_concat.push_str((quotechar).as_str());
        __sifr_concat.push_str((escaped).as_str());
        __sifr_concat.push_str((quotechar).as_str());
        __sifr_concat
    }
}
fn _escape_unquoted_field(
    field: &String,
    dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
) -> String {
    let mut result: String = {
        let mut __sifr_concat: String = String::with_capacity(field.len() + 0usize);
        __sifr_concat.push_str((field).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if (result).contains((dialect.delimiter.clone()).as_str()) {
        if (dialect.escapechar.clone() != "") {
            result = result
                .replace(
                    &dialect.delimiter.clone(),
                    &format!(
                        "{}{}", dialect.escapechar.clone(), dialect.delimiter.clone()
                    ),
                );
        }
    }
    if result.contains(&"\n".to_string()) {
        if (dialect.escapechar.clone() != "") {
            result = result
                .replace('\n', &format!("{}{}", dialect.escapechar.clone(), "\n"));
        }
    }
    if result.contains(&"\r".to_string()) {
        if (dialect.escapechar.clone() != "") {
            result = result
                .replace('\r', &format!("{}{}", dialect.escapechar.clone(), "\r"));
        }
    }
    if (dialect.quotechar.clone() != "") {
        let quotechar2: String = _quotechar_value(dialect);
        if result.contains(&quotechar2) {
            if (dialect.escapechar.clone() != "") {
                result = result
                    .replace(
                        &quotechar2,
                        &format!("{}{}", dialect.escapechar.clone(), quotechar2),
                    );
            } else {
                result = result
                    .replace(&quotechar2, &format!("{}{}", quotechar2, quotechar2));
            }
        }
    }
    result
}
fn format_row(
    fields: &Vec<String>,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> String {
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        quoting,
    );
    let mut parts: Vec<String> = vec![];
    for field in fields.iter().cloned() {
        if _needs_quote(&field, &resolved) {
            parts.push(_quote_field(&field, &resolved));
        } else {
            parts.push(_escape_unquoted_field(&field, &resolved));
        }
    }
    parts.join(&resolved.delimiter)
}
fn format_csv(
    rows: &Vec<Vec<String>>,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> String {
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        lineterminator,
        quoting,
    );
    let mut rendered: Vec<String> = vec![];
    let resolved_delimiter: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.delimiter).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_quotechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.quotechar).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_escapechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.escapechar).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    let resolved_lineterminator: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((resolved.lineterminator).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    for row in rows.iter().cloned() {
        rendered
            .push(
                format_row(
                    &row,
                    &None,
                    &resolved_delimiter,
                    &resolved_quotechar,
                    &resolved_escapechar,
                    resolved.doublequote,
                    resolved.skipinitialspace,
                    resolved.quoting,
                ),
            );
    }
    rendered.join(&resolved_lineterminator)
}
fn _dict_reader_row(
    fieldnames: &Vec<String>,
    row: &Vec<String>,
    restkey: &String,
    restval: &String,
) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::from([]);
    for (i, key) in Box::new(
        (fieldnames)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
    ) {
        if (i < (row.len() as i64)) {
            {
                let __assign_key = key.clone();
                let __assign_value = _list_value_at(row, i);
                result.insert(__assign_key, __assign_value);
            }
        } else {
            result
                .insert(
                    key.clone(),
                    {
                        let mut __sifr_concat: String = String::with_capacity(
                            restval.len() + 0usize,
                        );
                        __sifr_concat.push_str((restval).as_str());
                        __sifr_concat.push_str("");
                        __sifr_concat
                    },
                );
        }
    }
    if ((restkey).as_str() != "") && ((row.len() as i64) > (fieldnames.len() as i64)) {
        let mut extras: Vec<String> = vec![];
        let mut j: i64 = fieldnames.len() as i64;
        while (j < (row.len() as i64)) {
            extras.push(_list_value_at(row, j));
            j += 1_i64;
        }
        {
            let __assign_key = restkey.clone();
            let __assign_value = format!("{:?}", extras);
            result.insert(__assign_key, __assign_value);
        }
    }
    result
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
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    str_value: Option<String>,
    array_items: Box<Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue>>,
    object_items: Box<Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)>>,
}
impl __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    fn new(
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
    fn as_int(&self) -> Option<i64> {
        self.int_value
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
    fn as_object(
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
    fn at(&self, index: i64) -> Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
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
    fn get(&self, key: &String) -> Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
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
    fn keys(&self) -> Vec<String> {
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
    fn values(&self) -> Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue> {
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
    fn items(&self) -> Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)> {
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
fn _append_array_item(
    mut value: __SifrStdlib_sifr_x2ejson_x2eJsonValue,
    item: __SifrStdlib_sifr_x2ejson_x2eJsonValue,
) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    value.array_items.push(item.clone());
    value
}
fn _append_object_item(
    mut value: __SifrStdlib_sifr_x2ejson_x2eJsonValue,
    key: String,
    item_value: __SifrStdlib_sifr_x2ejson_x2eJsonValue,
) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    value.object_items.push(((key).clone(), (item_value).clone()));
    value
}
fn from_array(
    items: &Vec<__SifrStdlib_sifr_x2ejson_x2eJsonValue>,
) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    let mut value: __SifrStdlib_sifr_x2ejson_x2eJsonValue = __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
        "array".to_string(),
        None,
        None,
        None,
        None,
    );
    for item in items.iter().cloned() {
        value = _append_array_item(value, item);
    }
    value
}
fn from_object(
    items: &Vec<(String, __SifrStdlib_sifr_x2ejson_x2eJsonValue)>,
) -> __SifrStdlib_sifr_x2ejson_x2eJsonValue {
    let mut value: __SifrStdlib_sifr_x2ejson_x2eJsonValue = __SifrStdlib_sifr_x2ejson_x2eJsonValue::new(
        "object".to_string(),
        None,
        None,
        None,
        None,
    );
    for (key, item_value) in items.iter().cloned() {
        value = _append_object_item(value, key, item_value);
    }
    value
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
                    return Err(JSONDecodeError::new(e.message));
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
                    return Err(JSONDecodeError::new(e.message));
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
            return Err(JSONDecodeError::new(e.message));
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
            return Err(JSONDecodeError::new(e.message));
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
fn _loads_impl(
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
fn _decode_loaded_json(
    content: &String,
) -> Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, Error> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, Error>,
        JSONDecodeError,
    > = (|| {
        let value: __SifrStdlib_sifr_x2ejson_x2eJsonValue = _loads_impl(content)?;
        return Ok(Ok(value));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(Error::new(e.message));
        }
    }
}
fn load(path: &String) -> Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, Error> {
    let content_result: Result<String, IOError> = read_text(path);
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2ejson_x2eJsonValue, Error>,
        IOError,
    > = (|| {
        let content: String = content_result?;
        return Ok(_decode_loaded_json(&content));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(Error::new(e.message));
        }
    }
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
fn toml_parse_tokens(text: &String) -> Result<Vec<String>, ParseError> {
    ::sifr_stdlib::toml::toml_parse_tokens(text)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ParseError {
            message: __sifr_bridge_error.to_string(),
        })
}
#[derive(Debug, Clone)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0 {
    __SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(ParseError),
    __SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(TOMLDecodeError),
}
impl From<ParseError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0 {
    fn from(value: ParseError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    str_value: Option<String>,
    datetime_value: Option<String>,
    array_items: Box<Vec<__SifrStdlib_sifr_x2etomllib_x2eTomlValue>>,
    table_items: Box<Vec<(String, __SifrStdlib_sifr_x2etomllib_x2eTomlValue)>>,
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn new(
        kind: String,
        bool_value: Option<bool>,
        int_value: Option<i64>,
        float_value: Option<f64>,
        str_value: Option<String>,
        datetime_value: Option<String>,
    ) -> Self {
        let __sifr_field_init_0: String = kind;
        let __sifr_field_init_1: Option<bool> = bool_value;
        let __sifr_field_init_2: Option<i64> = int_value;
        let __sifr_field_init_3: Option<f64> = float_value;
        let __sifr_field_init_4: Option<String> = str_value;
        let __sifr_field_init_5: Option<String> = datetime_value;
        let __sifr_field_init_6: Box<Vec<__SifrStdlib_sifr_x2etomllib_x2eTomlValue>> = Box::default();
        let __sifr_field_init_7: Box<
            Vec<(String, __SifrStdlib_sifr_x2etomllib_x2eTomlValue)>,
        > = Box::default();
        Self {
            kind: __sifr_field_init_0,
            bool_value: __sifr_field_init_1,
            int_value: __sifr_field_init_2,
            float_value: __sifr_field_init_3,
            str_value: __sifr_field_init_4,
            datetime_value: __sifr_field_init_5,
            array_items: __sifr_field_init_6,
            table_items: __sifr_field_init_7,
        }
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn is_bool(&self) -> bool {
        (self.kind.clone() == "bool")
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn is_int(&self) -> bool {
        (self.kind.clone() == "int")
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn is_float(&self) -> bool {
        (self.kind.clone() == "float")
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn is_str(&self) -> bool {
        (self.kind.clone() == "str")
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn is_datetime(&self) -> bool {
        (self.kind.clone() == "datetime")
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn is_array(&self) -> bool {
        (self.kind.clone() == "array")
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn is_table(&self) -> bool {
        (self.kind.clone() == "table")
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn as_bool(&self) -> Option<bool> {
        self.bool_value
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn as_int(&self) -> Option<i64> {
        self.int_value
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn as_float(&self) -> Option<f64> {
        self.float_value
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn as_str(&self) -> Option<String> {
        self.str_value.clone()
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn as_datetime(&self) -> Option<String> {
        self.datetime_value.clone()
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn as_array(&self) -> Option<Vec<__SifrStdlib_sifr_x2etomllib_x2eTomlValue>> {
        if !(self.is_array()) {
            return None;
        }
        let mut result: Vec<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> = vec![];
        for item in (self.array_items).as_ref().clone().iter().cloned() {
            result.push(item.clone());
        }
        Some(result)
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn as_table(
        &self,
    ) -> Option<Vec<(String, __SifrStdlib_sifr_x2etomllib_x2eTomlValue)>> {
        if !(self.is_table()) {
            return None;
        }
        let mut result: Vec<(String, __SifrStdlib_sifr_x2etomllib_x2eTomlValue)> = vec![];
        for (key, value) in (self.table_items).as_ref().clone().iter().cloned() {
            result.push(((key).clone(), (value).clone()));
        }
        Some(result)
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn at(&self, index: i64) -> Option<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> {
        if !(self.is_array()) {
            return None;
        }
        if (index < (0_i64)) || (index >= (self.array_items.len() as i64)) {
            return None;
        }
        let value: Option<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> = Some(
            (self.array_items).as_ref().clone()[index as usize].clone(),
        );
        value
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn get(&self, key: &String) -> Option<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> {
        if !(self.is_table()) {
            return None;
        }
        for (item_key, item_value) in (self.table_items).as_ref().clone().iter().cloned()
        {
            if item_key == *key {
                return Some(item_value);
            }
        }
        None
    }
}
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn keys(&self) -> Vec<String> {
        let mut result: Vec<String> = vec![];
        if !(self.is_table()) {
            return result;
        }
        for (item_key, _item_value) in (self.table_items)
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
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn values(&self) -> Vec<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> {
        let mut result: Vec<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> = vec![];
        if !(self.is_table()) {
            return result;
        }
        for (_item_key, item_value) in (self.table_items)
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
impl __SifrStdlib_sifr_x2etomllib_x2eTomlValue {
    fn items(&self) -> Vec<(String, __SifrStdlib_sifr_x2etomllib_x2eTomlValue)> {
        if !(self.is_table()) {
            return vec![];
        }
        let mut result: Vec<(String, __SifrStdlib_sifr_x2etomllib_x2eTomlValue)> = vec![];
        for (key, value) in (self.table_items).as_ref().clone().iter().cloned() {
            result.push(((key).clone(), (value).clone()));
        }
        result
    }
}
fn _token_at(tokens: &Vec<String>, index: i64) -> Result<String, TOMLDecodeError> {
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
            TOMLDecodeError::new("TOML bridge payload ended unexpectedly".to_string()),
        );
    };
    Ok({
        let mut __sifr_concat: String = String::with_capacity(value.len() + 0usize);
        __sifr_concat.push_str((value).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    })
}
fn _token_int(tokens: &Vec<String>, index: i64) -> Result<i64, TOMLDecodeError> {
    let __sifr_try_res: Result<
        Result<i64, TOMLDecodeError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0,
    > = (|| {
        let token: String = (_token_at(tokens, index))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                __e,
            ))?;
        let parsed: i64 = ((token)
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
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
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let _e = __sifr_try_variant_error.clone();
                    return Err(
                        TOMLDecodeError::new(
                            "TOML bridge payload has invalid integer metadata"
                                .to_string(),
                        ),
                    );
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(TOMLDecodeError::new(e.message));
                }
            }
        }
    }
}
fn _token_float(tokens: &Vec<String>, index: i64) -> Result<f64, TOMLDecodeError> {
    let __sifr_try_res: Result<
        Result<f64, TOMLDecodeError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0,
    > = (|| {
        let token: String = (_token_at(tokens, index))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                __e,
            ))?;
        let parsed: f64 = ((token)
            .parse::<f64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            }))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
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
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass10_x3aParseError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let _e = __sifr_try_variant_error.clone();
                    return Err(
                        TOMLDecodeError::new(
                            "TOML bridge payload has invalid float metadata".to_string(),
                        ),
                    );
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a223_x3a5_x3aclass10_x3aParseError1_x3a028_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let e = __sifr_try_variant_error.clone();
                    return Err(TOMLDecodeError::new(e.message));
                }
            }
        }
    }
}
fn _decode_bool_token(value: &String) -> Result<bool, TOMLDecodeError> {
    if (value).as_str() == "true" {
        return Ok(true);
    }
    if (value).as_str() == "false" {
        return Ok(false);
    }
    Err(
        TOMLDecodeError::new("TOML bridge payload has invalid bool metadata".to_string()),
    )
}
fn _decode_toml_value_at(
    tokens: &Vec<String>,
    index: i64,
) -> Result<(__SifrStdlib_sifr_x2etomllib_x2eTomlValue, i64), TOMLDecodeError> {
    let __sifr_try_res: Result<
        Result<(__SifrStdlib_sifr_x2etomllib_x2eTomlValue, i64), TOMLDecodeError>,
        TOMLDecodeError,
    > = (|| {
        let tag: String = _token_at(tokens, index)?;
        let payload_index: i64 = index + (1_i64);
        if tag == "bool" {
            let bool_token: String = _token_at(tokens, payload_index)?;
            let bool_value: bool = _decode_bool_token(&bool_token)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2etomllib_x2eTomlValue::new(
                        "bool".to_string(),
                        Some(bool_value),
                        None,
                        None,
                        None,
                        None,
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "int" {
            let int_value: i64 = _token_int(tokens, payload_index)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2etomllib_x2eTomlValue::new(
                        "int".to_string(),
                        None,
                        Some(int_value),
                        None,
                        None,
                        None,
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "float" {
            let float_value: f64 = _token_float(tokens, payload_index)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2etomllib_x2eTomlValue::new(
                        "float".to_string(),
                        None,
                        None,
                        Some(float_value),
                        None,
                        None,
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "str" {
            let str_value: String = _token_at(tokens, payload_index)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2etomllib_x2eTomlValue::new(
                        "str".to_string(),
                        None,
                        None,
                        None,
                        Some(str_value),
                        None,
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "datetime" {
            let datetime_value: String = _token_at(tokens, payload_index)?;
            return Ok(
                Ok((
                    __SifrStdlib_sifr_x2etomllib_x2eTomlValue::new(
                        "datetime".to_string(),
                        None,
                        None,
                        None,
                        None,
                        Some(datetime_value),
                    ),
                    payload_index + (1_i64),
                )),
            );
        }
        if tag == "array" {
            let array_count: i64 = _token_int(tokens, payload_index)?;
            if array_count < (0_i64) {
                return Err(
                    TOMLDecodeError::new(
                        "TOML bridge payload has invalid array length".to_string(),
                    ),
                );
            }
            let mut array_value: __SifrStdlib_sifr_x2etomllib_x2eTomlValue = __SifrStdlib_sifr_x2etomllib_x2eTomlValue::new(
                "array".to_string(),
                None,
                None,
                None,
                None,
                None,
            );
            let mut next_index: i64 = payload_index + (1_i64);
            let mut consumed: i64 = 0_i64;
            while consumed < array_count {
                let item_result: (__SifrStdlib_sifr_x2etomllib_x2eTomlValue, i64) = _decode_toml_value_at(
                    tokens,
                    next_index,
                )?;
                array_value.array_items.push(item_result.0);
                next_index = (item_result).1;
                consumed += 1_i64;
            }
            return Ok(Ok((array_value.clone(), next_index)));
        }
        if tag == "table" {
            let table_count: i64 = _token_int(tokens, payload_index)?;
            if table_count < (0_i64) {
                return Err(
                    TOMLDecodeError::new(
                        "TOML bridge payload has invalid table length".to_string(),
                    ),
                );
            }
            let mut table_value: __SifrStdlib_sifr_x2etomllib_x2eTomlValue = __SifrStdlib_sifr_x2etomllib_x2eTomlValue::new(
                "table".to_string(),
                None,
                None,
                None,
                None,
                None,
            );
            let mut next_index: i64 = payload_index + (1_i64);
            let mut consumed: i64 = 0_i64;
            while consumed < table_count {
                let key: String = _token_at(tokens, next_index)?;
                let item_result: (__SifrStdlib_sifr_x2etomllib_x2eTomlValue, i64) = _decode_toml_value_at(
                    tokens,
                    next_index + (1_i64),
                )?;
                table_value.table_items.push(((key).clone(), item_result.0));
                next_index = (item_result).1;
                consumed += 1_i64;
            }
            return Ok(Ok((table_value.clone(), next_index)));
        }
        return Err(
            TOMLDecodeError::new({
                let mut __sifr_concat: String = String::with_capacity(
                    43usize + tag.len(),
                );
                __sifr_concat.push_str("TOML bridge payload has unknown value tag: ");
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
            return Err(TOMLDecodeError::new(e.message));
        }
    }
}
fn _decode_toml_tokens(
    tokens: &Vec<String>,
) -> Result<__SifrStdlib_sifr_x2etomllib_x2eTomlValue, TOMLDecodeError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2etomllib_x2eTomlValue, TOMLDecodeError>,
        TOMLDecodeError,
    > = (|| {
        let decoded: (__SifrStdlib_sifr_x2etomllib_x2eTomlValue, i64) = _decode_toml_value_at(
            tokens,
            0_i64,
        )?;
        if ((decoded).1 != (tokens.len() as i64)) {
            return Err(
                TOMLDecodeError::new("TOML bridge payload has trailing data".to_string()),
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
            return Err(TOMLDecodeError::new(e.message));
        }
    }
}
fn loads(
    text: &String,
) -> Result<__SifrStdlib_sifr_x2etomllib_x2eTomlValue, TOMLDecodeError> {
    let __sifr_try_res: Result<
        Result<__SifrStdlib_sifr_x2etomllib_x2eTomlValue, TOMLDecodeError>,
        ParseError,
    > = (|| {
        let tokens: Vec<String> = toml_parse_tokens(text)?;
        return Ok(_decode_toml_tokens(&tokens));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(TOMLDecodeError::new(e.message));
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
    line: i64,
    column: i64,
}
impl JSONDecodeError {
    fn new(message: String) -> Self {
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
    limit: i64,
}
impl JsonLimitError {
    fn new(message: String) -> Self {
        Self { message, limit: 0 }
    }
}
impl ::std::fmt::Display for JsonLimitError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for JsonLimitError {}
fn main() {
    println!("structured-parsing-sample structured parsing and serialization demo");
    let __sifr_try_res: Result<
        (),
        __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0,
    > = (|| {
        let json_path: String = "/tmp/sifr_structured_parsing_serialization.json"
            .to_string();
        let _ = (write_text(
            &json_path,
            &"{\"name\":\"sifr\",\"items\":[1,true]}".to_string(),
        ))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                __e,
            ))?;
        let json_value: __SifrStdlib_sifr_x2ejson_x2eJsonValue = (load(&json_path))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                __e,
            ))?;
        let json_items: Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> = json_value
            .get(&"items".to_string());
        let json_name: Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> = json_value
            .get(&"name".to_string());
        let mut json_second: Option<__SifrStdlib_sifr_x2ejson_x2eJsonValue> = None;
        if let Some(json_items) = json_items {
            json_second = json_items.at(1_i64);
        }
        if let Some(json_name) = json_name {
            println!(
                "{}", (json_name.as_str()).map_or("None".to_string().to_string(), | __v |
                format!("{}", __v))
            );
        }
        if let Some(json_second) = json_second {
            println!(
                "{}", (json_second.as_bool()).map_or("None".to_string().to_string(), |
                __v | format!("{}", __v))
            );
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        match __sifr_try_err {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let e = __sifr_try_variant_error.clone();
                println!("{}", e.message);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a217_x3a5_x3aclass5_x3aError1_x3a019_x3a5_x3aclass7_x3aIOError1_x3a0::__SifrUnionVariant_5_x3aclass7_x3aIOError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let e = __sifr_try_variant_error.clone();
                println!("{}", e.message);
            }
        }
    }
    println!(
        "{}", dumps(&
        __SifrUnion_8_x3asequence5_x3aunion1_x3a719_x3a4_x3aatom10_x3abigdecimal11_x3a4_x3aatom3_x3aint11_x3a4_x3aatom3_x3astr12_x3a4_x3aatom4_x3abool13_x3a4_x3aatom5_x3afloat15_x3a4_x3aatom7_x3adecimal32_x3a5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0::__SifrUnionVariant_5_x3aclass19_x3asifr_x2ejson_x2eJsonValue1_x3a0((from_object(&
        vec![("name".to_string(), from_str(& "sifr".to_string())), ("items".to_string(),
        from_array(& vec![from_int(1_i64), from_bool(true)]))])).clone()))
    );
    let __sifr_try_res: Result<
        (),
        __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0,
    > = (|| {
        let toml_value: __SifrStdlib_sifr_x2etomllib_x2eTomlValue = (loads(
            &"title = \"sifr\"\n[owner]\nactive = true\n".to_string(),
        ))
            .map_err(|__e| __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                __e,
            ))?;
        let owner: Option<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> = toml_value
            .get(&"owner".to_string());
        let title: Option<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> = toml_value
            .get(&"title".to_string());
        if let Some(owner) = owner {
            let active: Option<__SifrStdlib_sifr_x2etomllib_x2eTomlValue> = owner
                .get(&"active".to_string());
            if let Some(title) = title {
                println!(
                    "{}", (title.as_str()).map_or("None".to_string().to_string(), | __v |
                    format!("{}", __v))
                );
            }
            if let Some(active) = active {
                println!(
                    "{}", (active.as_bool()).map_or("None".to_string().to_string(), | __v
                    | format!("{}", __v))
                );
            }
        }
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        match __sifr_try_err {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass5_x3aError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let e = __sifr_try_variant_error.clone();
                println!("{}", e.message);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a228_x3a5_x3aclass15_x3aTOMLDecodeError1_x3a017_x3a5_x3aclass5_x3aError1_x3a0::__SifrUnionVariant_5_x3aclass15_x3aTOMLDecodeError1_x3a0(
                __sifr_try_variant_error,
            ) => {
                let e = __sifr_try_variant_error.clone();
                println!("{}", e.message);
            }
        }
    }
    let quoted: String = format_row(
        &vec!["alpha".to_string(), "beta".to_string()],
        &Some(
            (__SifrStdlib_sifr_x2ecsv_x2eDialect::new(
                ",".to_string(),
                "\"".to_string(),
                "".to_string(),
                true,
                false,
                "\n".to_string(),
                QUOTE_ALL,
            ))
                .clone(),
        ),
        &",".to_string(),
        &"\"".to_string(),
        &"".to_string(),
        true,
        false,
        0_i64,
    );
    println!("{}", quoted);
    let dict_reader: __SifrStdlib_sifr_x2ecsv_x2eDictReader = __SifrStdlib_sifr_x2ecsv_x2eDictReader::new(
        "name,age\nalice,30\n".to_string(),
        None,
        "".to_string(),
        "".to_string(),
        None,
        ",".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        0_i64,
    );
    println!("{}", format!("{:?}", dict_reader.rows()));
    let mut dict_writer: __SifrStdlib_sifr_x2ecsv_x2eDictWriter = __SifrStdlib_sifr_x2ecsv_x2eDictWriter::new(
        vec!["name".to_string(), "age".to_string()],
        "".to_string(),
        "raise".to_string(),
        None,
        ",".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        "\n".to_string(),
        0_i64,
    );
    dict_writer.writeheader();
    dict_writer
        .writerow(
            &HashMap::from([
                ("name".to_string(), "alice".to_string()),
                ("age".to_string(), "30".to_string()),
            ]),
        );
    println!("{}", dict_writer.getvalue());
    let mut defaults: HashMap<String, Option<String>> = HashMap::from([]);
    let encoding_value: Option<String> = Some("utf-8".to_string());
    defaults.insert("encoding".to_string(), encoding_value.clone());
    let mut parser: __SifrStdlib_sifr_x2econfigparser_x2eConfigParser = __SifrStdlib_sifr_x2econfigparser_x2eConfigParser::new(
        Some(defaults),
        false,
        true,
    );
    let __sifr_try_res: Result<
        Option<()>,
        __SifrStdlib_sifr_x2econfigparser_x2eParsingError,
    > = (|| {
        let _ = parser
            .read_string(
                &"[server]\nport = 8080\nenabled = true\nfeature\n".to_string(),
            )?;
        Ok(None)
    })();
    match __sifr_try_res {
        Ok(Some(__sifr_ret_val)) => {
            return __sifr_ret_val;
        }
        Ok(None) => {}
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            println!("{}", e.message);
            return;
        }
    }
    println!(
        "{}", (parser.getint(& "server".to_string(), & "port".to_string(), None))
        .map_or("None".to_string().to_string(), | __v | format!("{}", __v))
    );
    println!(
        "{}", (parser.getboolean(& "server".to_string(), & "enabled".to_string(), None))
        .map_or("None".to_string().to_string(), | __v | format!("{}", __v))
    );
    let fallback_value: Option<String> = Some("missing".to_string());
    println!(
        "{}", (parser.get(& "server".to_string(), & "feature".to_string(), &
        fallback_value, false)).map_or("None".to_string().to_string(), | __v |
        format!("{}", __v))
    );
}
