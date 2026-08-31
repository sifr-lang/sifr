// src/main.rs
mod __sifr_project_nominals {
    pub use ::sifr_runtime::SifrInt;
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
    pub struct ValueError {
        pub message: String,
    }
    impl ValueError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ValueError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ValueError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct JSONDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl JSONDecodeError {
        pub fn new(message: String) -> Self {
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
        pub limit: SifrInt,
    }
    impl JsonLimitError {
        pub fn new(message: String) -> Self {
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TOMLDecodeError {
        pub message: String,
        pub line: SifrInt,
        pub column: SifrInt,
    }
    impl TOMLDecodeError {
        pub fn new(message: String) -> Self {
            Self {
                message,
                line: SifrInt::from_i64(0),
                column: SifrInt::from_i64(0),
            }
        }
    }
    impl ::std::fmt::Display for TOMLDecodeError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for TOMLDecodeError {}
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
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct TimeoutError {
        pub message: String,
    }
    impl TimeoutError {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for TimeoutError {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for TimeoutError {}
    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub struct ScopeFailure {
        pub message: String,
    }
    impl ScopeFailure {
        pub fn new(message: String) -> Self {
            Self { message }
        }
    }
    impl ::std::fmt::Display for ScopeFailure {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            ::std::fmt::Display::fmt(&self.message, f)
        }
    }
    impl ::std::error::Error for ScopeFailure {}
}
pub use __sifr_project_nominals::IOError;
pub use __sifr_project_nominals::JSONDecodeError;
pub use __sifr_project_nominals::JsonIntegerRangeError;
pub use __sifr_project_nominals::JsonLimitError;
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::RegexError;
pub use __sifr_project_nominals::ScopeFailure;
pub use __sifr_project_nominals::TOMLDecodeError;
pub use __sifr_project_nominals::TimeoutError;
pub use __sifr_project_nominals::ValueError;
use ::sifr_runtime::SifrInt;
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
fn b64encode(s: &str) -> String {
    base64_encode(s)
}
fn b64decode(s: &str) -> Result<String, ParseError> {
    base64_decode(s)
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _algorithm: String,
    _data: Vec<u8>,
    name: String,
    digest_size: SifrInt,
    block_size: SifrInt,
}
impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    fn new(
        algorithm: String,
        data: Vec<u8>,
        name: String,
        digest_size: SifrInt,
        block_size: SifrInt,
    ) -> Self {
        let __sifr_field_init_0: String = algorithm;
        let __sifr_field_init_1: Vec<u8> = data;
        let __sifr_field_init_2: String = name;
        let __sifr_field_init_3: SifrInt = digest_size.clone();
        let __sifr_field_init_4: SifrInt = block_size.clone();
        Self {
            _algorithm: __sifr_field_init_0,
            _data: __sifr_field_init_1,
            name: __sifr_field_init_2,
            digest_size: __sifr_field_init_3,
            block_size: __sifr_field_init_4,
        }
    }
}
impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    fn update(&mut self, data: &[u8]) {
        self._data = {
            let mut __v = (self._data.clone()).to_vec();
            __v.extend((data).iter().cloned());
            __v
        };
    }
}
impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    fn hexdigest(&self) -> String {
        _hash_hex(&self._algorithm, &self._data)
    }
}
impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    fn digest(&self) -> Vec<u8> {
        _hash_bytes(&self._algorithm, &self._data)
    }
}
fn _build_hash(
    algorithm: &str,
    data: &[u8],
) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    let alg: String = algorithm.to_lowercase();
    if (alg == "md5") {
        return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
            alg,
            data.to_vec(),
            "md5".to_string(),
            SifrInt::from_i64(16),
            SifrInt::from_i64(64),
        );
    } else {
        if (alg == "sha1") {
            return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                alg,
                data.to_vec(),
                "sha1".to_string(),
                SifrInt::from_i64(20),
                SifrInt::from_i64(64),
            );
        } else {
            if (alg == "sha224") {
                return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                    alg,
                    data.to_vec(),
                    "sha224".to_string(),
                    SifrInt::from_i64(28),
                    SifrInt::from_i64(64),
                );
            } else {
                if (alg == "sha256") {
                    return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                        alg,
                        data.to_vec(),
                        "sha256".to_string(),
                        SifrInt::from_i64(32),
                        SifrInt::from_i64(64),
                    );
                } else {
                    if (alg == "sha384") {
                        return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                            alg,
                            data.to_vec(),
                            "sha384".to_string(),
                            SifrInt::from_i64(48),
                            SifrInt::from_i64(128),
                        );
                    } else {
                        if (alg == "sha512") {
                            return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                alg,
                                data.to_vec(),
                                "sha512".to_string(),
                                SifrInt::from_i64(64),
                                SifrInt::from_i64(128),
                            );
                        } else {
                            if (alg == "blake2b") {
                                return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                    alg,
                                    data.to_vec(),
                                    "blake2b".to_string(),
                                    SifrInt::from_i64(64),
                                    SifrInt::from_i64(128),
                                );
                            } else {
                                if (alg == "blake2s") {
                                    return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                        alg,
                                        data.to_vec(),
                                        "blake2s".to_string(),
                                        SifrInt::from_i64(32),
                                        SifrInt::from_i64(64),
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
        alg,
        data.to_vec(),
        "unknown".to_string(),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
    )
}
fn _hash_bytes(algorithm: &str, data: &[u8]) -> Vec<u8> {
    if algorithm == "md5" {
        return md5_bytes(data);
    } else {
        if algorithm == "sha1" {
            return sha1_bytes(data);
        } else {
            if algorithm == "sha224" {
                return sha224_bytes(data);
            } else {
                if algorithm == "sha256" {
                    return sha256_bytes(data);
                } else {
                    if algorithm == "sha384" {
                        return sha384_bytes(data);
                    } else {
                        if algorithm == "sha512" {
                            return sha512_bytes(data);
                        } else {
                            if algorithm == "blake2b" {
                                return blake2b_bytes(data);
                            } else {
                                if algorithm == "blake2s" {
                                    return blake2s_bytes(data);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    {
        let __sifr_empty_bytes_literal: Vec<u8> = vec![];
        __sifr_empty_bytes_literal
    }
}
fn _hash_hex(algorithm: &str, data: &[u8]) -> String {
    {
        let __bytes_receiver = &_hash_bytes(algorithm, data);
        let mut __hex = String::with_capacity(
            __bytes_receiver.len().saturating_mul(2_usize),
        );
        for __byte in __bytes_receiver.iter() {
            __hex.push_str(&format!("{:02x}", * __byte));
        }
        __hex
    }
}
fn sha256(data: &[u8]) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _build_hash(&"sha256".to_string(), data)
}
const PI: f64 = 3.141592653589793_f64;
const E: f64 = 2.718281828459045_f64;
const TAU: f64 = 6.283185307179586_f64;
const INF: f64 = f64::INFINITY;
const NAN: f64 = f64::NAN;
fn sqrt(x: f64) -> f64 {
    ::sifr_stdlib::math::sqrt(x)
}
fn floor(x: f64) -> SifrInt {
    ::sifr_stdlib::math::floor(x).into_sifr_int()
}
fn ceil(x: f64) -> SifrInt {
    ::sifr_stdlib::math::ceil(x).into_sifr_int()
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
fn round_val(x: f64) -> SifrInt {
    ::sifr_stdlib::math::round_val(x).into_sifr_int()
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
fn trunc(x: f64) -> SifrInt {
    ::sifr_stdlib::math::trunc(x).into_sifr_int()
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
fn isqrt(n: SifrInt) -> SifrInt {
    ::sifr_stdlib::math::isqrt(::sifr_runtime::interop::SifrIntBridge::from(n))
        .into_sifr_int()
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
fn ldexp(m: f64, e: SifrInt) -> f64 {
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
    assert_eq!(& SifrInt::from_i64(1) + & SifrInt::from_i64(1), SifrInt::from_i64(2));
    assert!(true);
    let result: f64 = sqrt(9.0_f64);
    assert!(result == (3.0_f64));
    assert!(PI > (3.14_f64));
    let h: String = sha256(&vec![104u8, 101u8, 108u8, 108u8, 111u8]).hexdigest();
    let __sifr_chars_h: Vec<char> = h.chars().collect::<Vec<char>>();
    assert!(& SifrInt::from(h.chars().count()) == & SifrInt::from_i64(64));
    let encoded: String = b64encode(&"Hello!".to_string());
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b64decode(&encoded)?;
        assert_eq!(decoded, "Hello!");
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("base64 error: {}", err.message.clone());
        assert!(
            (format!("{}", format!("base64 error: {}", err.message.clone())) ==
            "stdlib_migration demo: all checks passed!")
        );
    }
    println!("stdlib_migration demo: all checks passed!");
}
