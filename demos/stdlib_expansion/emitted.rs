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
fn parse_flag(args: &[String], flag: &str) -> bool {
    for arg in args.iter().cloned() {
        if arg == *flag {
            return true;
        }
    }
    false
}
fn _split_inline_option(token: &str) -> (bool, String, String) {
    let __sifr_chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
    let mut key: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_token.len())) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_token.len());
            __sifr_chars_token.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if (ch.is_some()) && (ch == Some("=".to_string())) {
            let mut value: String = "".to_string();
            let mut j: SifrInt = &i + &SifrInt::from_i64(1);
            while (&j < &SifrInt::from(__sifr_chars_token.len())) {
                let part: Option<String> = ({
                    let __sifr_string_index = j.clone();
                    let __sifr_string_index_normalized = __sifr_string_index
                        .normalize_index_or_len(__sifr_chars_token.len());
                    __sifr_chars_token.get(__sifr_string_index_normalized)
                })
                    .map(|c| c.to_string());
                if let Some(part) = part {
                    value.push_str(part.as_str());
                }
                j = &j + &SifrInt::from_i64(1);
            }
            return (true, key, value);
        }
        if let Some(ch) = ch {
            key.push_str(ch.as_str());
        }
        i = &i + &SifrInt::from_i64(1);
    }
    (
        false,
        {
            let mut __sifr_concat: String = String::with_capacity(token.len() + 0usize);
            __sifr_concat.push_str(token);
            __sifr_concat.push_str("");
            __sifr_concat
        },
        "".to_string(),
    )
}
fn parse_option(args: &[String], name: &str, default: &str) -> String {
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(args.len())) {
        let arg: Option<String> = {
            let __sifr_checked_read_collection = &args;
            let __sifr_checked_read_index = i.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(arg) = arg {
            if (arg == *name)
                && (&(&i + &SifrInt::from_i64(1)) < &SifrInt::from(args.len()))
            {
                let next_val: Option<String> = {
                    let __sifr_checked_read_collection = &args;
                    let __sifr_checked_read_index = &i + &SifrInt::from_i64(1);
                    let __sifr_checked_read_normalized = __sifr_checked_read_index
                        .normalize_index_or_len(__sifr_checked_read_collection.len());
                    __sifr_checked_read_collection
                        .get(__sifr_checked_read_normalized)
                        .cloned()
                };
                if let Some(next_val) = next_val {
                    if (next_val == "--") || next_val.starts_with("--") {} else {
                        return {
                            let mut __sifr_concat: String = String::with_capacity(
                                next_val.len() + 0usize,
                            );
                            __sifr_concat.push_str(next_val.as_str());
                            __sifr_concat.push_str("");
                            __sifr_concat
                        };
                    }
                }
            }
            let (inline_has_value, inline_name, inline_value) = _split_inline_option(
                &arg,
            );
            let __sifr_chars_inline_name: Vec<char> = inline_name
                .chars()
                .collect::<Vec<char>>();
            let __sifr_chars_inline_value: Vec<char> = inline_value
                .chars()
                .collect::<Vec<char>>();
            if inline_has_value && (inline_name == *name) {
                return {
                    let mut __sifr_concat: String = String::with_capacity(
                        inline_value.len() + 0usize,
                    );
                    __sifr_concat.push_str(inline_value.as_str());
                    __sifr_concat.push_str("");
                    __sifr_concat
                };
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(default.len() + 0usize);
        __sifr_concat.push_str(default);
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn bisect_left<T: Clone + 'static + PartialOrd>(
    a: &[T],
    x: &T,
    lo: SifrInt,
    hi: Option<SifrInt>,
) -> SifrInt {
    let mut left: SifrInt = lo.clone();
    if &left < &SifrInt::from_i64(0) {
        left = SifrInt::from_i64(0);
    }
    let mut right: SifrInt = SifrInt::from(a.len());
    if (hi.is_none()) {
        right = SifrInt::from(a.len());
    } else {
        if let Some(hi) = hi.clone() {
            if (&hi < &SifrInt::from_i64(0)) {
                right = SifrInt::from_i64(0);
            } else {
                if (&hi > &SifrInt::from(a.len())) {
                    right = SifrInt::from(a.len());
                } else {
                    right = hi;
                }
            }
        }
    }
    while (&left < &right) {
        let mid: SifrInt = (&left + &right)
            .floor_div_known_nonzero(&SifrInt::from_i64(2));
        let val: Option<T> = {
            let __sifr_checked_read_collection = &a;
            let __sifr_checked_read_index = mid.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(val) = val {
            if val < *x {
                left = &mid + &SifrInt::from_i64(1);
            } else {
                right = mid;
            }
        } else {
            left = &mid + &SifrInt::from_i64(1);
        }
    }
    left.clone()
}
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
            __sifr_concat.push_str(label.as_str());
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
struct __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    name: String,
}
impl __SifrStdlib_sifr_x2eencoding_x2eEncodeErrorHandler {
    fn new(name: String) -> Self {
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
struct __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    text: String,
    recoveries: Vec<String>,
}
impl __SifrStdlib_sifr_x2eencoding_x2eDecodeOutcome {
    fn new(text: String, recoveries: Vec<String>) -> Self {
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
    fn get_text(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str(self.text.clone().as_str());
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
    fn write(&self, data: &str) -> Result<(), IOError> {
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
        file_read_bytes(&self._handle, size.clone())
    }
}
impl __SifrIoFileHandle {
    fn write_bytes(&self, data: &[u8]) -> Result<(), IOError> {
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
        file_seek(&self._handle, offset.clone(), whence.clone())
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
        file_read_bytes(&self._handle, size.clone())
    }
}
impl __SifrIoBinaryFileHandle {
    fn write_bytes(&self, data: &[u8]) -> Result<(), IOError> {
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
        file_seek(&self._handle, offset.clone(), whence.clone())
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
    fn write(&self, text: &str) -> Result<(), IOError> {
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
    fn write(&self, text: &str) -> Result<(), IOError> {
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
    fn write(&mut self, data: &str) -> Result<(), IOError> {
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
    fn write_bytes(&mut self, data: &[u8]) -> Result<(), IOError> {
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
fn __const_QUOTE_ALL() -> SifrInt {
    SifrInt::from_i64(1)
}
fn __const_QUOTE_NONNUMERIC() -> SifrInt {
    SifrInt::from_i64(2)
}
fn __const_QUOTE_NONE() -> SifrInt {
    SifrInt::from_i64(3)
}
fn __const_QUOTE_STRINGS() -> SifrInt {
    SifrInt::from_i64(4)
}
fn __const_QUOTE_NOTNULL() -> SifrInt {
    SifrInt::from_i64(5)
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2ecsv_x2eDialect {
    delimiter: String,
    quotechar: String,
    escapechar: String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: String,
    quoting: SifrInt,
}
impl __SifrStdlib_sifr_x2ecsv_x2eDialect {
    fn new(
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        lineterminator: String,
        quoting: SifrInt,
    ) -> Self {
        let mut resolved_quoting: SifrInt = quoting.clone();
        _validate_char(&"delimiter".to_string(), &delimiter);
        if (quotechar != "") {
            _validate_char(&"quotechar".to_string(), &quotechar);
        }
        if (escapechar != "") {
            _validate_char(&"escapechar".to_string(), &escapechar);
        }
        if (quotechar == "") && (&resolved_quoting != &__const_QUOTE_NONE()) {
            resolved_quoting = __const_QUOTE_NONE().clone();
        }
        let __sifr_field_init_0: String = delimiter;
        let __sifr_field_init_1: String = quotechar;
        let __sifr_field_init_2: String = escapechar;
        let __sifr_field_init_3: bool = doublequote;
        let __sifr_field_init_4: bool = skipinitialspace;
        let __sifr_field_init_5: String = lineterminator;
        let __sifr_field_init_6: SifrInt = resolved_quoting.clone();
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
        dialect.quoting.clone(),
    )
}
fn _validate_char(name: &str, value: &str) {
    let _ = name.to_owned();
    let _ = value.to_owned();
}
fn _resolve_dialect(
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &str,
    quotechar: &str,
    escapechar: &str,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &str,
    quoting: SifrInt,
) -> __SifrStdlib_sifr_x2ecsv_x2eDialect {
    if let Some(dialect) = dialect.as_ref() {
        return _copy_dialect(dialect);
    }
    __SifrStdlib_sifr_x2ecsv_x2eDialect::new(
        delimiter.to_owned(),
        quotechar.to_owned(),
        escapechar.to_owned(),
        doublequote,
        skipinitialspace,
        lineterminator.to_owned(),
        quoting.clone(),
    )
}
fn _quotechar_value(dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> String {
    let quotechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str(dialect.quotechar.clone().as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if (quotechar).as_str() == ("".to_string()).as_str() {
        return "\"".to_string();
    }
    quotechar
}
fn _append_field(row: &mut Vec<String>, field: String) {
    row.push(format!("{}{}", field, ""));
}
fn _append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
    rows.push(row.to_vec());
}
fn _char_at(text: &str, index: SifrInt) -> String {
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
fn _first_char(text: &str) -> String {
    _char_at(text, SifrInt::from_i64(0))
}
fn _last_char(text: &str) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    _char_at(text, SifrInt::from(text.chars().count()) - SifrInt::from_i64(1))
}
fn parse_row(
    line: &str,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &str,
    quotechar: &str,
    escapechar: &str,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: SifrInt,
) -> Vec<String> {
    let rows: Vec<Vec<String>> = parse_csv(
        line,
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        quoting.clone(),
    );
    if &SifrInt::from(rows.len()) == &SifrInt::from_i64(0) {
        return vec![];
    }
    for (index, row) in Box::new(
        (rows)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| (SifrInt::from(__pair.0) + SifrInt::from_i64(0), __pair.1)),
    ) {
        if (&index == &SifrInt::from_i64(0)) {
            let mut copied: Vec<String> = vec![];
            for field in row.iter().cloned() {
                copied.push(format!("{}{}", field, ""));
            }
            return copied;
        }
    }
    vec![]
}
fn parse_csv(
    text: &str,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &str,
    quotechar: &str,
    escapechar: &str,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: SifrInt,
) -> Vec<Vec<String>> {
    let quotechar = quotechar.to_owned();
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        &quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        quoting.clone(),
    );
    let mut rows: Vec<Vec<String>> = vec![];
    let mut row: Vec<String> = vec![];
    let mut field: String = "".to_string();
    let mut in_quotes: bool = false;
    let mut field_started: bool = false;
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch_value: String = _char_at(text, i.clone());
        if in_quotes {
            if (resolved.escapechar.clone() != "")
                && (ch_value == resolved.escapechar.clone())
            {
                if (&(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(__sifr_chars_text.len()))
                {
                    let escaped_value: String = _char_at(
                        text,
                        &i + &SifrInt::from_i64(1),
                    );
                    field.push_str(escaped_value.as_str());
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                field.push_str(ch_value.as_str());
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            if (resolved.quotechar.clone() != "")
                && (ch_value == resolved.quotechar.clone())
            {
                let quotechar: String = _quotechar_value(&resolved);
                if (resolved.doublequote
                    && (&(&i + &SifrInt::from_i64(1))
                        < &SifrInt::from(__sifr_chars_text.len())))
                    && (_char_at(text, &i + &SifrInt::from_i64(1)) == quotechar)
                {
                    field.push_str(quotechar.as_str());
                    i = &i + &SifrInt::from_i64(2);
                    continue;
                }
                in_quotes = false;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
            field.push_str(ch_value.as_str());
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (!field_started && resolved.skipinitialspace) && (ch_value == " ") {
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (resolved.escapechar.clone() != "")
            && (ch_value == resolved.escapechar.clone())
        {
            if (&(&i + &SifrInt::from_i64(1)) < &SifrInt::from(__sifr_chars_text.len()))
            {
                let escaped_plain_value: String = _char_at(
                    text,
                    &i + &SifrInt::from_i64(1),
                );
                field.push_str(escaped_plain_value.as_str());
                field_started = true;
                i = &i + &SifrInt::from_i64(2);
                continue;
            }
            field.push_str(ch_value.as_str());
            field_started = true;
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (&resolved.quoting.clone() != &__const_QUOTE_NONE())
            && (resolved.quotechar.clone() != "")
        {
            let quotechar2: String = _quotechar_value(&resolved);
            if (ch_value == quotechar2) {
                in_quotes = true;
                field_started = true;
                i = &i + &SifrInt::from_i64(1);
                continue;
            }
        }
        if (ch_value == resolved.delimiter.clone()) {
            _append_field(&mut row, field);
            field = "".to_string();
            field_started = false;
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        if (ch_value == "\n") || (ch_value == "\r") {
            if ((ch_value == "\r")
                && (&(&i + &SifrInt::from_i64(1))
                    < &SifrInt::from(__sifr_chars_text.len())))
                && (_char_at(text, &i + &SifrInt::from_i64(1)) == "\n")
            {
                i = &i + &SifrInt::from_i64(1);
            }
            if (&SifrInt::from(row.len()) == &SifrInt::from_i64(0)) && (field == "") {
                _append_row(&mut rows, vec![]);
            } else {
                _append_field(&mut row, field);
                _append_row(&mut rows, row);
            }
            row = vec![];
            field = "".to_string();
            field_started = false;
            i = &i + &SifrInt::from_i64(1);
            continue;
        }
        field.push_str(ch_value.as_str());
        field_started = true;
        i = &i + &SifrInt::from_i64(1);
    }
    if in_quotes {
        in_quotes = false;
    }
    if (&SifrInt::from(row.len()) > &SifrInt::from_i64(0)) || (field != "") {
        _append_field(&mut row, field);
        _append_row(&mut rows, row);
    }
    rows
}
fn _needs_quote(field: &str, dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> bool {
    let __sifr_chars_field: Vec<char> = field.chars().collect::<Vec<char>>();
    if (&dialect.quoting.clone() == &__const_QUOTE_ALL()) {
        return true;
    }
    if (&dialect.quoting.clone() == &__const_QUOTE_NONNUMERIC()) {
        return true;
    }
    if (&dialect.quoting.clone() == &__const_QUOTE_STRINGS()) {
        return true;
    }
    if (&dialect.quoting.clone() == &__const_QUOTE_NOTNULL()) {
        return true;
    }
    if (&dialect.quoting.clone() == &__const_QUOTE_NONE()) {
        return false;
    }
    if (field).contains(dialect.delimiter.clone().as_str()) {
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
    if (&SifrInt::from(__sifr_chars_field.len()) > &SifrInt::from_i64(0)) {
        let first: String = _first_char(field);
        let last: String = _last_char(field);
        if (first == " ") {
            return true;
        }
        if (last == " ") {
            return true;
        }
    }
    false
}
fn _quote_field(field: &str, dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect) -> String {
    let quotechar: String = _quotechar_value(dialect);
    let mut escaped: String = {
        let mut __sifr_concat: String = String::with_capacity(field.len() + 0usize);
        __sifr_concat.push_str(field);
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
                    __sifr_concat.push_str(dialect.escapechar.clone().as_str());
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
        __sifr_concat.push_str(quotechar.as_str());
        __sifr_concat.push_str(escaped.as_str());
        __sifr_concat.push_str(quotechar.as_str());
        __sifr_concat
    }
}
fn _escape_unquoted_field(
    field: &str,
    dialect: &__SifrStdlib_sifr_x2ecsv_x2eDialect,
) -> String {
    let mut result: String = {
        let mut __sifr_concat: String = String::with_capacity(field.len() + 0usize);
        __sifr_concat.push_str(field);
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if (result).contains(dialect.delimiter.clone().as_str()) {
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
    fields: &[String],
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &str,
    quotechar: &str,
    escapechar: &str,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: SifrInt,
) -> String {
    let resolved: __SifrStdlib_sifr_x2ecsv_x2eDialect = _resolve_dialect(
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        &"\n".to_string(),
        quoting.clone(),
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
fn fnmatch(name: &str, pattern: &str) -> bool {
    _match(name, SifrInt::from_i64(0), pattern, SifrInt::from_i64(0))
}
fn _match(name: &str, mut ni: SifrInt, pattern: &str, mut pi: SifrInt) -> bool {
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
                    if _match(name, j.clone(), pattern, pi.clone()) {
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
fn filter(names: &[String], pattern: &str) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name.to_owned());
        }
    }
    result
}
fn reduce<T: Clone + 'static, U: Clone + 'static>(
    func: impl Fn(&U, &T) -> U,
    data: &[T],
    initial: &U,
) -> U {
    let mut result: U = initial.clone();
    for val in data.iter().cloned() {
        result = func(&result, &val);
    }
    result
}
fn _sift_down<T: Clone + 'static + PartialOrd>(
    data: &mut Vec<T>,
    mut pos: SifrInt,
    n: SifrInt,
) {
    let mut done: bool = false;
    while !done {
        let mut smallest: SifrInt = pos.clone();
        let left: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(1);
        let right: SifrInt = &(&SifrInt::from_i64(2) * &pos) + &SifrInt::from_i64(2);
        if (&left < &n) {
            let s_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let l_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = left.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(s_val) = s_val {
                if let Some(l_val) = l_val {
                    if (l_val < s_val) {
                        smallest = left;
                    }
                }
            }
        }
        if (&right < &n) {
            let s_val2: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let r_val: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = right.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(s_val2) = s_val2 {
                if let Some(r_val) = r_val {
                    if (r_val < s_val2) {
                        smallest = right;
                    }
                }
            }
        }
        if (&smallest == &pos) {
            done = true;
        } else {
            let tmp_pos: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = pos.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let tmp_sm: Option<T> = {
                let __sifr_checked_read_collection = &data;
                let __sifr_checked_read_index = smallest.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_sm) = tmp_sm {
                    if (&SifrInt::from_i64(0) <= &pos)
                        && (&pos < &SifrInt::from(data.len()))
                    {
                        {
                            let __assign_value = tmp_sm.clone();
                            {
                                let __index_raw = pos.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(data.len());
                                if let Some(__elem) = data.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                    if (&SifrInt::from_i64(0) <= &smallest)
                        && (&smallest < &SifrInt::from(data.len()))
                    {
                        {
                            let __assign_value = tmp_pos.clone();
                            {
                                let __index_raw = smallest.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(data.len());
                                if let Some(__elem) = data.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                }
            }
            pos = smallest;
        }
    }
}
fn _sift_up<T: Clone + 'static + PartialOrd>(heap: &mut Vec<T>, mut pos: SifrInt) {
    let mut done: bool = false;
    while !done {
        if (&pos <= &SifrInt::from_i64(0)) {
            done = true;
        } else {
            let parent: SifrInt = (&pos - &SifrInt::from_i64(1))
                .floor_div_known_nonzero(&SifrInt::from_i64(2));
            let p_val: Option<T> = {
                let __sifr_checked_read_collection = &heap;
                let __sifr_checked_read_index = parent.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let c_val: Option<T> = {
                let __sifr_checked_read_collection = &heap;
                let __sifr_checked_read_index = pos.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(p_val) = p_val {
                if let Some(c_val) = c_val {
                    if (c_val < p_val) {
                        if (&SifrInt::from_i64(0) <= &parent)
                            && (&parent < &SifrInt::from(heap.len()))
                        {
                            {
                                let __assign_value = c_val.clone();
                                {
                                    let __index_raw = parent.clone();
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(heap.len());
                                    if let Some(__elem) = heap.get_mut(__index_normalized) {
                                        *__elem = __assign_value;
                                    }
                                }
                            }
                        }
                        if (&SifrInt::from_i64(0) <= &pos)
                            && (&pos < &SifrInt::from(heap.len()))
                        {
                            {
                                let __assign_value = p_val.clone();
                                {
                                    let __index_raw = pos.clone();
                                    let __index_normalized = __index_raw
                                        .normalize_index_or_len(heap.len());
                                    if let Some(__elem) = heap.get_mut(__index_normalized) {
                                        *__elem = __assign_value;
                                    }
                                }
                            }
                        }
                        pos = parent;
                    } else {
                        done = true;
                    }
                } else {
                    done = true;
                }
            } else {
                done = true;
            }
        }
    }
}
fn heapify<T: Clone + 'static + PartialOrd>(data: &mut Vec<T>) {
    "Convert list to a min-heap in-place. O(n) time.".to_string();
    let n: SifrInt = SifrInt::from(data.len());
    let mut i: SifrInt = &n.floor_div_known_nonzero(&SifrInt::from_i64(2))
        - &SifrInt::from_i64(1);
    while (&i >= &SifrInt::from_i64(0)) {
        _sift_down(data, i.clone(), n.clone());
        i = &i - &SifrInt::from_i64(1);
    }
}
fn heappush<T: Clone + 'static + PartialOrd>(heap: &mut Vec<T>, item: &T) {
    "Push item onto the heap in-place. O(log n) time.".to_string();
    heap.push(item.clone());
    let pos: SifrInt = &SifrInt::from(heap.len()) - &SifrInt::from_i64(1);
    _sift_up(heap, pos.clone());
}
fn heappop<T: Clone + 'static + PartialOrd>(heap: &mut Vec<T>) -> Option<T> {
    "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
        .to_string();
    let n: SifrInt = SifrInt::from(heap.len());
    if &n == &SifrInt::from_i64(0) {
        return None;
    }
    let top: Option<T> = {
        let __sifr_checked_read_collection = &heap;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let last: Option<T> = {
        let __sifr_checked_read_collection = &heap;
        let __sifr_checked_read_index = &n - &SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    heap.remove(heap.len() - (1_usize));
    let n2: SifrInt = SifrInt::from(heap.len());
    if (&n2 > &SifrInt::from_i64(0)) {
        if let Some(last) = last {
            {
                let __assign_value = last.clone();
                {
                    let __index_raw = SifrInt::from_i64(0);
                    let __index_normalized = __index_raw
                        .normalize_index_or_len(heap.len());
                    if let Some(__elem) = heap.get_mut(__index_normalized) {
                        *__elem = __assign_value;
                    }
                }
            }
        }
        _sift_down(heap, SifrInt::from_i64(0), n2.clone());
    }
    top
}
fn nsmallest<T: Clone + 'static + PartialOrd>(n: SifrInt, data: &[T]) -> Vec<T> {
    let mut heap: Vec<T> = data.to_vec();
    heapify(&mut heap);
    let mut result: Vec<T> = vec![];
    let mut count: SifrInt = SifrInt::from_i64(0);
    while (&count < &n) {
        if (&SifrInt::from(heap.len()) == &SifrInt::from_i64(0)) {
            return result;
        }
        let val: Option<T> = heappop(&mut heap);
        if let Some(val) = val {
            result.push(val.clone());
        }
        count = &count + &SifrInt::from_i64(1);
    }
    result
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
pub trait __SifrAdd: Sized {
    fn __sifr_add(self, rhs: Self) -> Self;
}
impl __SifrAdd for ::sifr_runtime::SifrInt {
    fn __sifr_add(self, rhs: Self) -> Self {
        self + rhs
    }
}
impl __SifrAdd for f64 {
    fn __sifr_add(self, rhs: Self) -> Self {
        self + rhs
    }
}
impl __SifrAdd for String {
    fn __sifr_add(mut self, rhs: Self) -> Self {
        self.push_str(&rhs);
        self
    }
}
fn chain<T: Clone + 'static>(iterables: &[Vec<T>]) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.to_vec();
    Box::new(
        __SifrGenerator::new(async move |__sifr_yielder: __SifrYielder<T>| {
            for iterable in iterables.iter().cloned() {
                for item in iterable.iter().cloned() {
                    __sifr_yielder.suspend(item.clone()).await;
                }
            }
        }),
    )
}
fn take<T: Clone + 'static>(n: SifrInt, data: &[T]) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut count: SifrInt = SifrInt::from_i64(0);
    for item in data.iter().cloned() {
        if (&count >= &n) {
            return result;
        }
        result.push(item.clone());
        count = &count + &SifrInt::from_i64(1);
    }
    result
}
fn flatten<T: Clone + 'static>(lists: &[Vec<T>]) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    for inner in lists.iter().cloned() {
        for val in inner.iter().cloned() {
            result.push(val.clone());
        }
    }
    result
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
fn factorial(n: SifrInt) -> SifrInt {
    if &n < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(2);
    while &i <= &n {
        result = &result * &i;
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn gcd(a: SifrInt, b: SifrInt) -> SifrInt {
    let mut x: SifrInt = a.clone();
    let mut y: SifrInt = b.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    while (&y != &SifrInt::from_i64(0)) {
        let temp: SifrInt = y.clone();
        y = x.floor_mod_known_nonzero(&y);
        x = temp;
    }
    x.clone()
}
fn lcm(a: SifrInt, b: SifrInt) -> SifrInt {
    if &a == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &b == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let g: SifrInt = gcd(a.clone(), b.clone());
    if &g == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    let mut x: SifrInt = a.clone();
    if &x < &SifrInt::from_i64(0) {
        x = &SifrInt::from_i64(0) - &x;
    }
    let mut y: SifrInt = b.clone();
    if &y < &SifrInt::from_i64(0) {
        y = &SifrInt::from_i64(0) - &y;
    }
    &x.floor_div_known_nonzero(&g) * &y
}
fn comb(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    if &k == &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    if &k == &n {
        return SifrInt::from_i64(1);
    }
    let mut r: SifrInt = k.clone();
    if &r > &(&n - &k) {
        r = &n - &k;
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &r) {
        result = &result * &(&n - &i);
        let divisor: SifrInt = &i + &SifrInt::from_i64(1);
        if (&divisor == &SifrInt::from_i64(0)) {
            return SifrInt::from_i64(0);
        }
        result = result.floor_div_known_nonzero(&divisor);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn perm(n: SifrInt, k: SifrInt) -> SifrInt {
    if &k < &SifrInt::from_i64(0) {
        return SifrInt::from_i64(0);
    }
    if &k > &n {
        return SifrInt::from_i64(0);
    }
    let mut result: SifrInt = SifrInt::from_i64(1);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &k {
        result = &result * &(&n - &i);
        i = &i + &SifrInt::from_i64(1);
    }
    result.clone()
}
fn log_base(x: f64, base: f64) -> f64 {
    log(x) / log(base)
}
fn isclose(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if rel_tol < (0.0_f64) {
        return false;
    }
    if abs_tol < (0.0_f64) {
        return false;
    }
    if a == b {
        return true;
    }
    if isnan(a) || isnan(b) {
        return false;
    }
    if isinf(a) || isinf(b) {
        return false;
    }
    let mut diff: f64 = a - b;
    if diff < (0.0_f64) {
        diff = (0.0_f64) - diff;
    }
    let mut a_abs: f64 = a;
    if a_abs < (0.0_f64) {
        a_abs = (0.0_f64) - a_abs;
    }
    let mut b_abs: f64 = b;
    if b_abs < (0.0_f64) {
        b_abs = (0.0_f64) - b_abs;
    }
    let mut larger_abs: f64 = a_abs;
    if b_abs > larger_abs {
        larger_abs = b_abs;
    }
    let mut rel_bound: f64 = rel_tol * larger_abs;
    if abs_tol > rel_bound {
        rel_bound = abs_tol;
    }
    diff <= rel_bound
}
fn prod(data: &[SifrInt]) -> SifrInt {
    let mut result: SifrInt = SifrInt::from_i64(1);
    for val in data.iter().cloned() {
        result = &result * &val;
    }
    result.clone()
}
fn _copy_float_list(data: &[f64]) -> Vec<f64> {
    let mut out: Vec<f64> = vec![];
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &[f64], q: &[f64]) -> f64 {
    dist_impl(_copy_float_list(p), _copy_float_list(q))
}
fn fsum(data: &[f64]) -> f64 {
    fsum_impl(_copy_float_list(data))
}
fn sumprod(p: &[f64], q: &[f64]) -> f64 {
    sumprod_impl(_copy_float_list(p), _copy_float_list(q))
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = frexp(x);
    let m: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> SifrInt {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(exp_val) = exp_val else {
        return SifrInt::from_i64(0);
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_checked_read_collection = &parts;
        let __sifr_checked_read_index = SifrInt::from_i64(1);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
fn __const__MT_N() -> SifrInt {
    SifrInt::from_i64(624)
}
fn __const__MT_M() -> SifrInt {
    SifrInt::from_i64(397)
}
fn __const__MT_MATRIX_A() -> SifrInt {
    SifrInt::from_i64(2567483615)
}
fn __const__MT_UPPER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483648)
}
fn __const__MT_LOWER_MASK() -> SifrInt {
    SifrInt::from_i64(2147483647)
}
fn __const__MT_F() -> SifrInt {
    SifrInt::from_i64(1812433253)
}
fn __const__MT_WORD_MASK() -> SifrInt {
    SifrInt::from_i64(4294967295)
}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandomState {
    version: SifrInt,
    state_words: Vec<SifrInt>,
    index: SifrInt,
    gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {
    fn new(
        version: SifrInt,
        state_words: Vec<SifrInt>,
        index: SifrInt,
        gauss_next: Option<f64>,
    ) -> Self {
        let __sifr_field_init_0: SifrInt = version.clone();
        let __sifr_field_init_1: Vec<SifrInt> = state_words;
        let __sifr_field_init_2: SifrInt = index.clone();
        let __sifr_field_init_3: Option<f64> = gauss_next;
        Self {
            version: __sifr_field_init_0,
            state_words: __sifr_field_init_1,
            index: __sifr_field_init_2,
            gauss_next: __sifr_field_init_3,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {}
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandom {
    _state_words: Vec<SifrInt>,
    _index: SifrInt,
    _gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn new(seed_value: Option<SifrInt>) -> Self {
        let normalized_seed: SifrInt = _normalize_seed_input(seed_value.clone());
        let __sifr_field_init_0: Vec<SifrInt> = _seed_words_from_seed(
            normalized_seed.clone(),
        );
        let __sifr_field_init_1: SifrInt = __const__MT_N().clone();
        let __sifr_field_init_2: Option<f64> = None;
        Self {
            _state_words: __sifr_field_init_0,
            _index: __sifr_field_init_1,
            _gauss_next: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn seed(&mut self, seed_value: &Option<SifrInt>) {
        let normalized_seed: SifrInt = _normalize_seed_input(seed_value.clone());
        self._state_words = _seed_words_from_seed(normalized_seed.clone());
        self._index = __const__MT_N().clone();
        self._gauss_next = None;
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _twist(&mut self) {
        let mut i: SifrInt = SifrInt::from_i64(0);
        while (&SifrInt::from_i64(0) <= &i)
            && (&i < &SifrInt::from(self._state_words.len()))
        {
            let y: SifrInt = &(&_state_word_at(&self._state_words, i.clone())
                & &__const__MT_UPPER_MASK())
                + &(&_state_word_at(
                    &self._state_words,
                    (&i + &SifrInt::from_i64(1))
                        .floor_mod_known_nonzero(&__const__MT_N()),
                ) & &__const__MT_LOWER_MASK());
            let mut x_a: SifrInt = y.floor_div_known_nonzero(&SifrInt::from_i64(2));
            if (&y.floor_mod_known_nonzero(&SifrInt::from_i64(2))
                != &SifrInt::from_i64(0))
            {
                x_a = &x_a ^ &__const__MT_MATRIX_A();
            }
            let new_word: SifrInt = &_state_word_at(
                &self._state_words,
                (&i + &__const__MT_M()).floor_mod_known_nonzero(&__const__MT_N()),
            ) ^ &x_a;
            {
                let __assign_value = &new_word & &__const__MT_WORD_MASK();
                {
                    let __index_raw = i.clone();
                    let __index_normalized = __index_raw
                        .normalize_index_or_len(self._state_words.len());
                    if let Some(__elem) = self._state_words.get_mut(__index_normalized) {
                        *__elem = __assign_value;
                    }
                }
            }
            i = &i + &SifrInt::from_i64(1);
        }
        self._index = SifrInt::from_i64(0);
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _next_u32(&mut self) -> SifrInt {
        if (&self._index.clone() >= &__const__MT_N()) {
            self._twist();
        }
        let mut y: SifrInt = _state_word_at(&self._state_words, self._index.clone());
        self._index = &self._index.clone() + &SifrInt::from_i64(1);
        y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(2048));
        y = &y ^ &(&(&y * &SifrInt::from_i64(128)) & &SifrInt::from_i64(2636928640));
        y = &y ^ &(&(&y * &SifrInt::from_i64(32768)) & &SifrInt::from_i64(4022730752));
        y = &y ^ &y.floor_div_known_nonzero(&SifrInt::from_i64(262144));
        &y & &__const__MT_WORD_MASK()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn random(&mut self) -> f64 {
        random_word_to_unit_float(self._next_u32())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn uniform(&mut self, minimum: f64, maximum: f64) -> f64 {
        minimum + ((maximum - minimum) * self.random())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randrange(
        &mut self,
        start: &SifrInt,
        stop: &Option<SifrInt>,
        step: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if (step == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let mut actual_start: SifrInt = start.clone();
        let mut actual_stop: SifrInt = start.clone();
        if (stop.is_none()) {
            actual_start = SifrInt::from_i64(0);
        } else {
            if let Some(stop) = stop.as_ref() {
                actual_stop = stop.clone();
            }
        }
        let width: SifrInt = &actual_stop - &actual_start;
        if (step > &SifrInt::from_i64(0)) {
            if (&width <= &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if (&width >= &SifrInt::from_i64(0)) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        }
        let mut abs_width: SifrInt = width.clone();
        if &abs_width < &SifrInt::from_i64(0) {
            abs_width = &SifrInt::from_i64(0) - &abs_width;
        }
        let mut abs_step: SifrInt = step.clone();
        if &abs_step < &SifrInt::from_i64(0) {
            abs_step = &SifrInt::from_i64(0) - &abs_step;
        }
        if (&abs_step == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let count: SifrInt = (&(&abs_width + &abs_step) - &SifrInt::from_i64(1))
            .floor_div_known_nonzero(&abs_step);
        if (&count <= &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        if (&count == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        let pick: SifrInt = self._next_u32().floor_mod_known_nonzero(&count);
        Ok(&actual_start + &(&pick * step))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randint(
        &mut self,
        minimum: &SifrInt,
        maximum: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if *minimum > *maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        self.randrange(
            minimum,
            &Some((maximum + &SifrInt::from_i64(1)).clone()),
            &SifrInt::from_i64(1),
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getrandbits(&mut self, k: &SifrInt) -> Result<SifrInt, ValueError> {
        if (k < &SifrInt::from_i64(0)) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut bits_left: SifrInt = k.clone();
        while (&bits_left > &SifrInt::from_i64(0)) {
            let word: SifrInt = self._next_u32();
            let mut take: SifrInt = SifrInt::from_i64(32);
            if (&bits_left < &SifrInt::from_i64(32)) {
                take = bits_left.clone();
            }
            let mut mask: SifrInt = SifrInt::from_i64(0);
            let mut shifted_result: SifrInt = result;
            let mut shift_index: SifrInt = SifrInt::from_i64(0);
            while (&shift_index < &take) {
                mask = &(&mask * &SifrInt::from_i64(2)) + &SifrInt::from_i64(1);
                shifted_result = &shifted_result * &SifrInt::from_i64(2);
                shift_index = &shift_index + &SifrInt::from_i64(1);
            }
            result = &shifted_result | &(&word & &mask);
            bits_left = &bits_left - &take;
        }
        Ok(result.clone())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randbytes(&mut self, n: &SifrInt) -> Result<Vec<u8>, ValueError> {
        if (n < &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *n {
            let byte_value: SifrInt = &self._next_u32() & &SifrInt::from_i64(255);
            values.push(byte_value.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                __out
                    .push(
                        __pair
                            .1
                            .try_to_u8()
                            .map_err(|_error| ValueError {
                                message: format!(
                                    "byte out of range at index {}: {}", __pair.0, * __pair.1
                                ),
                            })?,
                    );
            }
            Ok::<Vec<u8>, ValueError>(__out)
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let cached: Option<f64> = self._gauss_next;
        if let Some(cached) = cached {
            self._gauss_next = None;
            return mu + (sigma * cached);
        }
        let mut u1: f64 = self.random();
        if u1 <= (0.0_f64) {
            u1 = 0.000000000001_f64;
        }
        let u2: f64 = self.random();
        let radius: f64 = sqrt(-(2.0_f64) * log(u1));
        let theta: f64 = ((2.0_f64) * PI) * u2;
        let z0: f64 = radius * cos(theta);
        let z1: f64 = radius * sin(theta);
        let next_cached: Option<f64> = Some(z1);
        self._gauss_next = next_cached;
        mu + (sigma * z0)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getstate(&self) -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
        __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
            SifrInt::from_i64(3),
            _clone_words(&self._state_words),
            self._index.clone(),
            self._gauss_next,
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn setstate(
        &mut self,
        state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        if (&state.version.clone() != &SifrInt::from_i64(3)) {
            return Err(ValueError::new("setstate: unsupported version".to_string()));
        }
        if (&SifrInt::from(state.state_words.len()) != &__const__MT_N()) {
            return Err(
                ValueError::new("setstate: state_words must have length 624".to_string()),
            );
        }
        if (&state.index.clone() < &SifrInt::from_i64(0))
            || (&state.index.clone() > &__const__MT_N())
        {
            return Err(
                ValueError::new("setstate: index must be in range [0, 624]".to_string()),
            );
        }
        let mut normalized: Vec<SifrInt> = vec![];
        for word in state.state_words.clone().iter().cloned() {
            if (&word < &SifrInt::from_i64(0)) || (&word > &__const__MT_WORD_MASK()) {
                return Err(ValueError::new("setstate: word out of range".to_string()));
            }
            normalized.push(&word & &__const__MT_WORD_MASK());
        }
        self._state_words = normalized;
        self._index = state.index.clone();
        self._gauss_next = state.gauss_next;
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2erandom_x2eSystemRandom {}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn new() -> Self {
        Self {}
    }
}
impl ::std::default::Default for __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn default() -> Self {
        Self::new()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn seed(&self, _seed_value: &Option<SifrInt>) {}
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn getstate(
        &self,
    ) -> Result<__SifrStdlib_sifr_x2erandom_x2eRandomState, ValueError> {
        Err(ValueError::new("SystemRandom does not support getstate".to_string()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn setstate(
        &self,
        _state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        Err(ValueError::new("SystemRandom does not support setstate".to_string()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn random(&self) -> f64 {
        random_float()
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn uniform(&self, minimum: f64, maximum: f64) -> f64 {
        random_uniform(minimum, maximum)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randrange(
        &self,
        start: &SifrInt,
        stop: &Option<SifrInt>,
        step: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        let mut actual_start: SifrInt = start.clone();
        let mut actual_stop: SifrInt = start.clone();
        if (stop.is_none()) {
            actual_start = SifrInt::from_i64(0);
        } else {
            if let Some(stop) = stop.as_ref() {
                actual_stop = stop.clone();
            }
        }
        random_randrange(actual_start.clone(), actual_stop.clone(), step.clone())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randint(
        &self,
        minimum: &SifrInt,
        maximum: &SifrInt,
    ) -> Result<SifrInt, ValueError> {
        if *minimum > *maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        Ok(random_int(minimum.clone(), maximum.clone()))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn getrandbits(&self, k: &SifrInt) -> Result<SifrInt, ValueError> {
        if (k < &SifrInt::from_i64(0)) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: SifrInt = SifrInt::from_i64(0);
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *k {
            let mut bit: SifrInt = SifrInt::from_i64(0);
            if (random_float() >= (0.5_f64)) {
                bit = SifrInt::from_i64(1);
            }
            result = &(&result * &SifrInt::from_i64(2)) + &bit;
            i = &i + &SifrInt::from_i64(1);
        }
        Ok(result.clone())
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn gauss(&self, mu: f64, sigma: f64) -> f64 {
        random_gauss(mu, sigma)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randbytes(&self, n: &SifrInt) -> Result<Vec<u8>, ValueError> {
        if (n < &SifrInt::from_i64(0)) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<SifrInt> = vec![];
        let mut i: SifrInt = SifrInt::from_i64(0);
        while i < *n {
            let value: SifrInt = random_int(
                SifrInt::from_i64(0),
                SifrInt::from_i64(255),
            );
            values.push(value.clone());
            i = &i + &SifrInt::from_i64(1);
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                __out
                    .push(
                        __pair
                            .1
                            .try_to_u8()
                            .map_err(|_error| ValueError {
                                message: format!(
                                    "byte out of range at index {}: {}", __pair.0, * __pair.1
                                ),
                            })?,
                    );
            }
            Ok::<Vec<u8>, ValueError>(__out)
        }
    }
}
fn _state_word_at(words: &[SifrInt], index: SifrInt) -> SifrInt {
    let value: Option<SifrInt> = {
        let __sifr_checked_read_collection = &words;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(value) = value.clone() {
        return value;
    }
    SifrInt::from_i64(0)
}
fn _clone_words(words: &[SifrInt]) -> Vec<SifrInt> {
    let mut copied: Vec<SifrInt> = vec![];
    for word in words.iter().cloned() {
        copied.push(word.clone());
    }
    copied
}
fn _normalize_seed_input(seed_value: Option<SifrInt>) -> SifrInt {
    if let Some(seed_value) = seed_value.clone() {
        return seed_value.clone();
    }
    random_seed()
}
fn _seed_words_from_seed(seed_value: SifrInt) -> Vec<SifrInt> {
    let mut words: Vec<SifrInt> = vec![];
    words.push(&seed_value & &__const__MT_WORD_MASK());
    let mut i: SifrInt = SifrInt::from_i64(1);
    while (&i < &__const__MT_N()) {
        let prev: SifrInt = _state_word_at(&words, &i - &SifrInt::from_i64(1));
        let next_word: SifrInt = &(&(&__const__MT_F()
            * &(&prev ^ &prev.floor_div_known_nonzero(&SifrInt::from_i64(1073741824))))
            + &i) & &__const__MT_WORD_MASK();
        words.push(next_word.clone());
        i = &i + &SifrInt::from_i64(1);
    }
    words
}
fn _build_state_from_module_storage() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
        SifrInt::from_i64(3),
        random_module_state_words(),
        random_module_state_index(),
        random_module_state_gauss_next(),
    )
}
fn _store_state_into_module_storage(state: &__SifrStdlib_sifr_x2erandom_x2eRandomState) {
    let _set_result: Result<(), ValueError> = random_module_set_state(
        &_clone_words(&state.state_words.clone()),
        state.index.clone(),
        state.gauss_next,
    );
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<SifrInt> = random_module_state_words();
    if &SifrInt::from(words.len()) == &__const__MT_N() {
        return;
    }
    let bootstrap: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(5489)),
    );
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> __SifrStdlib_sifr_x2erandom_x2eRandom {
    _ensure_module_state_initialized();
    let mut r: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r
            .setstate(&_build_state_from_module_storage());
        let _ = _set_result;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message.clone();
    }
    r
}
fn _sync_module_random(generator: &mut __SifrStdlib_sifr_x2erandom_x2eRandom) {
    _store_state_into_module_storage(&generator.getstate());
}
fn seed(seed_value: Option<SifrInt>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        seed_value.clone(),
    );
    _sync_module_random(&mut generator);
}
fn getstate() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    _ensure_module_state_initialized();
    _build_state_from_module_storage()
}
fn setstate(
    state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
) -> Result<(), ValueError> {
    let mut probe: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(SifrInt::from_i64(0)),
    );
    let result: Result<(), ValueError> = probe.setstate(state);
    _sync_module_random(&mut probe);
    result
}
fn randint(minimum: SifrInt, maximum: SifrInt) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.randint(&minimum, &maximum);
    _sync_module_random(&mut generator);
    value
}
fn random() -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.random();
    _sync_module_random(&mut generator);
    value
}
fn uniform(minimum: f64, maximum: f64) -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.uniform(minimum, maximum);
    _sync_module_random(&mut generator);
    value
}
fn randrange(
    start: SifrInt,
    stop: Option<SifrInt>,
    step: SifrInt,
) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.randrange(&start, &stop, &step);
    _sync_module_random(&mut generator);
    value
}
fn getrandbits(k: SifrInt) -> Result<SifrInt, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<SifrInt, ValueError> = generator.getrandbits(&k);
    _sync_module_random(&mut generator);
    value
}
fn gauss(mu: f64, sigma: f64) -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.gauss(mu, sigma);
    _sync_module_random(&mut generator);
    value
}
fn choice<T: Clone + 'static>(items: &[T]) -> Result<T, ValueError> {
    let item_count: SifrInt = SifrInt::from(items.len());
    if (&item_count == &SifrInt::from_i64(0)) {
        return Err(ValueError::new("choice: items must not be empty".to_string()));
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let index: SifrInt = generator._next_u32().floor_mod_known_nonzero(&item_count);
    let picked: Option<T> = {
        let __sifr_checked_read_collection = &items;
        let __sifr_checked_read_index = index.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    _sync_module_random(&mut generator);
    if let Some(picked) = picked {
        return Ok(picked);
    }
    Err(ValueError::new("choice: index out of range".to_string()))
}
fn choices<T: Clone + 'static>(items: &[T], k: SifrInt) -> Result<Vec<T>, ValueError> {
    if &k <= &SifrInt::from_i64(0) {
        return Ok(vec![]);
    }
    let item_count: SifrInt = SifrInt::from(items.len());
    if (&item_count == &SifrInt::from_i64(0)) {
        return Err(ValueError::new("choices: items must not be empty".to_string()));
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &k) {
        let index: SifrInt = generator._next_u32().floor_mod_known_nonzero(&item_count);
        let picked: Option<T> = {
            let __sifr_checked_read_collection = &items;
            let __sifr_checked_read_index = index.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone());
        } else {
            return Err(ValueError::new("choices: index out of range".to_string()));
        }
        i = &i + &SifrInt::from_i64(1);
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn sample<T: Clone + 'static>(items: &[T], k: SifrInt) -> Result<Vec<T>, ValueError> {
    if (&k < &SifrInt::from_i64(0)) {
        return Err(ValueError::new("sample: k must be >= 0".to_string()));
    }
    if (&k > &SifrInt::from(items.len())) {
        return Err(ValueError::new("sample larger than population".to_string()));
    }
    let mut pool: Vec<T> = vec![];
    for item in items.iter().cloned() {
        pool.push(item.clone());
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut remaining: SifrInt = SifrInt::from(pool.len());
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &k) {
        if (&remaining == &SifrInt::from_i64(0)) {
            return Err(ValueError::new("sample larger than population".to_string()));
        }
        let pick_index: SifrInt = generator
            ._next_u32()
            .floor_mod_known_nonzero(&remaining);
        let picked: Option<T> = {
            let __sifr_checked_read_collection = &pool;
            let __sifr_checked_read_index = pick_index.clone();
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone());
        }
        let last: Option<T> = {
            let __sifr_checked_read_collection = &pool;
            let __sifr_checked_read_index = &remaining - &SifrInt::from_i64(1);
            let __sifr_checked_read_normalized = __sifr_checked_read_index
                .normalize_index_or_len(__sifr_checked_read_collection.len());
            __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
        };
        if let Some(last) = last {
            if (&SifrInt::from_i64(0) <= &pick_index)
                && (&pick_index < &SifrInt::from(pool.len()))
            {
                {
                    let __assign_value = last.clone();
                    {
                        let __index_raw = pick_index.clone();
                        let __index_normalized = __index_raw
                            .normalize_index_or_len(pool.len());
                        if let Some(__elem) = pool.get_mut(__index_normalized) {
                            *__elem = __assign_value;
                        }
                    }
                }
            }
        }
        remaining = &remaining - &SifrInt::from_i64(1);
        i = &i + &SifrInt::from_i64(1);
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn shuffle<T: Clone + 'static>(items: &mut Vec<T>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let n: SifrInt = SifrInt::from(items.len());
    if (&n > &SifrInt::from_i64(1)) {
        let mut i: SifrInt = &n - &SifrInt::from_i64(1);
        while (&i > &SifrInt::from_i64(0)) {
            let divisor: SifrInt = &i + &SifrInt::from_i64(1);
            if (&divisor == &SifrInt::from_i64(0)) {
                return;
            }
            let j: SifrInt = generator._next_u32().floor_mod_known_nonzero(&divisor);
            let left: Option<T> = {
                let __sifr_checked_read_collection = &items;
                let __sifr_checked_read_index = i.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            let right: Option<T> = {
                let __sifr_checked_read_collection = &items;
                let __sifr_checked_read_index = j.clone();
                let __sifr_checked_read_normalized = __sifr_checked_read_index
                    .normalize_index_or_len(__sifr_checked_read_collection.len());
                __sifr_checked_read_collection
                    .get(__sifr_checked_read_normalized)
                    .cloned()
            };
            if let Some(left) = left {
                if let Some(right) = right {
                    if (&SifrInt::from_i64(0) <= &i)
                        && (&i < &SifrInt::from(items.len()))
                    {
                        {
                            let __assign_value = right.clone();
                            {
                                let __index_raw = i.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(items.len());
                                if let Some(__elem) = items.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                    if (&SifrInt::from_i64(0) <= &j)
                        && (&j < &SifrInt::from(items.len()))
                    {
                        {
                            let __assign_value = left.clone();
                            {
                                let __index_raw = j.clone();
                                let __index_normalized = __index_raw
                                    .normalize_index_or_len(items.len());
                                if let Some(__elem) = items.get_mut(__index_normalized) {
                                    *__elem = __assign_value;
                                }
                            }
                        }
                    }
                }
            }
            i = &i - &SifrInt::from_i64(1);
        }
    }
    _sync_module_random(&mut generator);
}
fn randbytes(n: SifrInt) -> Result<Vec<u8>, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<Vec<u8>, ValueError> = generator.randbytes(&n);
    _sync_module_random(&mut generator);
    value
}
fn token_hex(nbytes: SifrInt) -> String {
    let hex_chars: String = "0123456789abcdef".to_string();
    let mut result: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &(&nbytes * &SifrInt::from_i64(2))) {
        let idx: SifrInt = random_int(SifrInt::from_i64(0), SifrInt::from_i64(15));
        let ch: Option<String> = ({
            let __sifr_string_source = &hex_chars;
            let __sifr_string_index = idx.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_string_source.chars().count());
            __sifr_string_source.chars().nth(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str(ch.as_str());
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    __SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(FloatOverflowError),
    __SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
        FloatPrecisionLossError,
    ),
}
impl From<FloatOverflowError>
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn from(value: FloatOverflowError) -> Self {
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
            value,
        )
    }
}
impl ::std::fmt::Display
for __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0 {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
            __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                v,
            ) => {
                return write!(f, "{}", v);
            }
        }
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    message: String,
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
impl ::std::fmt::Debug for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        f.debug_struct("StatisticsError").field("message", &self.message).finish()
    }
}
impl ::std::fmt::Display for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl ::std::error::Error for __SifrStdlib_sifr_x2estatistics_x2eStatisticsError {}
fn _sum(data: &[f64]) -> f64 {
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    total
}
fn _float_int(
    value: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0,
    > = (|| {
        let converted: f64 = value
            .clone()
            .checked_to_f64()
            .map_err(|__sifr_float_error| match __sifr_float_error {
                ::sifr_runtime::IntegerFloatConversionError::Overflow => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                        FloatOverflowError::new(
                            "exact integer is outside the finite float range".to_string(),
                        ),
                    )
                }
                ::sifr_runtime::IntegerFloatConversionError::PrecisionLoss => {
                    __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                        FloatPrecisionLossError::new(
                            "exact integer cannot be represented without float precision loss"
                                .to_string(),
                        ),
                    )
                }
            })?;
        Ok(Ok(converted))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            match __sifr_try_err {
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass18_x3aFloatOverflowError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
                __SifrUnion_8_x3asequence5_x3aunion1_x3a231_x3a5_x3aclass18_x3aFloatOverflowError1_x3a036_x3a5_x3aclass23_x3aFloatPrecisionLossError1_x3a0::__SifrUnionVariant_5_x3aclass23_x3aFloatPrecisionLossError1_x3a0(
                    __sifr_try_variant_error,
                ) => {
                    let error = __sifr_try_variant_error.clone();
                    return Err(
                        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                            error.message.clone(),
                        ),
                    );
                }
            }
        }
    }
}
fn _divide_by_int(
    numerator: f64,
    denominator: SifrInt,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let __sifr_try_res: Result<
        Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError>,
        __SifrStdlib_sifr_x2estatistics_x2eStatisticsError,
    > = (|| {
        let divisor: f64 = _float_int(denominator.clone())?;
        Ok(Ok(numerator / divisor))
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let error = __sifr_try_err.clone();
            return Err(
                __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                    error.message.clone(),
                ),
            );
        }
    }
}
fn mean(
    data: &[f64],
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let count: SifrInt = SifrInt::from(data.len());
    if (&count == &SifrInt::from_i64(0)) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ),
        );
    }
    let total: f64 = _sum(data);
    _divide_by_int(total, count.clone())
}
fn __const_ascii_lowercase() -> String {
    "abcdefghijklmnopqrstuvwxyz".to_string().to_string()
}
fn __const_digits() -> String {
    "0123456789".to_string().to_string()
}
fn _replace_whitespace_chars(text: &str, replace_tabs: bool) -> String {
    let normalized: String = text
        .replace('\n', " ")
        .replace('\r', " ")
        .replace('\u{b}', " ")
        .replace('\u{c}', " ");
    if replace_tabs {
        return normalized.replace('\t', " ");
    }
    normalized
}
fn _expand_tabs_impl(text: &str, tabsize: SifrInt) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut effective_tabsize: SifrInt = tabsize.clone();
    if &effective_tabsize <= &SifrInt::from_i64(0) {
        effective_tabsize = SifrInt::from_i64(1);
    }
    if (&effective_tabsize == &SifrInt::from_i64(0)) {
        return text.to_owned();
    }
    let mut result: String = "".to_string();
    let mut column: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch_opt: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if (ch == "\t") {
                let mut spaces: SifrInt = &effective_tabsize
                    - &column.floor_mod_known_nonzero(&effective_tabsize);
                if (&spaces <= &SifrInt::from_i64(0)) {
                    spaces = effective_tabsize.clone();
                }
                let mut j: SifrInt = SifrInt::from_i64(0);
                while (&j < &spaces) {
                    result.push(' ');
                    j = &j + &SifrInt::from_i64(1);
                }
                column = &column + &spaces;
            } else {
                if (ch == "\n") || (ch == "\r") {
                    result.push_str(ch.as_str());
                    column = SifrInt::from_i64(0);
                } else {
                    result.push_str(ch.as_str());
                    column = &column + &SifrInt::from_i64(1);
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _prepare_text(
    text: &str,
    expand_tabs: bool,
    tabsize: SifrInt,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = {
        let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
        __sifr_concat.push_str(text);
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, tabsize.clone());
    }
    if replace_whitespace {
        prepared = _replace_whitespace_chars(&prepared, true);
    }
    prepared
}
fn _normalize_whitespace(text: &str) -> String {
    _prepare_text(text, true, SifrInt::from_i64(8), true)
}
fn _has_non_whitespace(text: &str) -> bool {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            if (ch != " ") {
                if (ch != "\t") {
                    if (ch != "\n") {
                        if (ch != "\r") {
                            if (ch != "\u{b}") {
                                if (ch != "\u{c}") {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        i = &i + &SifrInt::from_i64(1);
    }
    false
}
fn _split_word_units(word: &str, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str(word); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let parts: Vec<String> = word
        .split('-')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if (&SifrInt::from(parts.len()) <= &SifrInt::from_i64(1)) {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str(word); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let mut units: Vec<String> = vec![];
    let mut index: SifrInt = SifrInt::from_i64(0);
    for part in parts.iter().cloned() {
        let __sifr_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
        let is_last: bool = (&index
            == &(&SifrInt::from(parts.len()) - &SifrInt::from_i64(1)));
        if is_last {
            if (&SifrInt::from(__sifr_chars_part.len()) > &SifrInt::from_i64(0)) {
                units.push(part.to_owned());
            }
        } else {
            if (&SifrInt::from(__sifr_chars_part.len()) == &SifrInt::from_i64(0)) {
                units.push("-".to_string());
            } else {
                units.push(format!("{}{}", part, "-"));
            }
        }
        index = &index + &SifrInt::from_i64(1);
    }
    if (&SifrInt::from(units.len()) == &SifrInt::from_i64(0)) {
        units.push(format!("{}{}", word, ""));
    }
    units
}
fn _trim_line(line: &str) -> String {
    let __sifr_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
    let mut start: SifrInt = SifrInt::from_i64(0);
    while (&start < &SifrInt::from(__sifr_chars_line.len()))
        && ({
            let __sifr_string_index = start.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_line.len());
            __sifr_chars_line.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string())
            .is_some_and(|__sifr_checked_value_2| {
                (__sifr_checked_value_2.clone() == " ")
            })
    {
        start = &start + &SifrInt::from_i64(1);
    }
    let mut end: SifrInt = SifrInt::from(__sifr_chars_line.len());
    while (&end > &start)
        && (({
            let __sifr_string_index = &end - &SifrInt::from_i64(1);
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_line.len());
            __sifr_chars_line.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string()) == Some(" ".to_string()))
    {
        end = &end - &SifrInt::from_i64(1);
    }
    {
        let _slice_src = &__sifr_chars_line;
        let _slice_len = _slice_src.len();
        let _slice_start = start.clamp_slice_bound(_slice_len);
        let _slice_stop = end.clamp_slice_bound(_slice_len);
        String::from_iter(
            _slice_src
                .iter()
                .skip(_slice_start)
                .take(_slice_stop.saturating_sub(_slice_start))
                .copied(),
        )
    }
}
fn _finalize_line(line: &str, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return _trim_line(line);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(line.len() + 0usize);
        __sifr_concat.push_str(line);
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _wrap_impl(text: &str, width: SifrInt) -> Vec<String> {
    let normalized: String = _normalize_whitespace(text);
    _wrap_with_indents(
        &normalized,
        width.clone(),
        &"".to_string(),
        &"".to_string(),
        true,
        true,
    )
}
fn _effective_content_width(total_width: SifrInt, indent: &str) -> SifrInt {
    let __sifr_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
    let available: SifrInt = &total_width - &SifrInt::from(__sifr_chars_indent.len());
    if &available <= &SifrInt::from_i64(0) {
        return SifrInt::from_i64(1);
    }
    available.clone()
}
fn _push_current_line(
    result: &mut Vec<String>,
    line: &str,
    indent: &str,
    drop_whitespace: bool,
) {
    let candidate: String = _finalize_line(
        &format!("{}{}", indent, line),
        drop_whitespace,
    );
    let __sifr_chars_candidate: Vec<char> = candidate.chars().collect::<Vec<char>>();
    if drop_whitespace {
        if (&SifrInt::from(__sifr_chars_candidate.len()) > &SifrInt::from_i64(0)) {
            result.push(candidate.to_owned());
        }
    } else {
        result.push(candidate.to_owned());
    }
}
fn _wrap_with_indents(
    text: &str,
    total_width: SifrInt,
    initial_indent: &str,
    subsequent_indent: &str,
    break_on_hyphens: bool,
    drop_whitespace: bool,
) -> Vec<String> {
    let words: Vec<String> = text
        .split(' ')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: Vec<String> = vec![];
    let mut current: String = "".to_string();
    let mut first_line: bool = true;
    let mut current_limit: SifrInt = _effective_content_width(
        total_width.clone(),
        initial_indent,
    );
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = _split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
            if (&SifrInt::from(__sifr_chars_word.len()) == &SifrInt::from_i64(0)) {
                if drop_whitespace {
                    continue;
                }
                if (&SifrInt::from(current.chars().count()) > &SifrInt::from_i64(0)) {
                    if (&(&SifrInt::from(current.chars().count())
                        + &SifrInt::from_i64(1)) <= &current_limit)
                    {
                        current.push(' ');
                    }
                }
                continue;
            }
            if (&SifrInt::from(current.chars().count()) == &SifrInt::from_i64(0)) {
                current = word;
            } else {
                if (&(&(&SifrInt::from(current.chars().count()) + &SifrInt::from_i64(1))
                    + &SifrInt::from(__sifr_chars_word.len())) <= &current_limit)
                {
                    current.push(' ');
                    current.push_str(word.as_str());
                } else {
                    if first_line {
                        _push_current_line(
                            &mut result,
                            &current,
                            initial_indent,
                            drop_whitespace,
                        );
                        first_line = false;
                        current_limit = _effective_content_width(
                            total_width.clone(),
                            subsequent_indent,
                        );
                    } else {
                        _push_current_line(
                            &mut result,
                            &current,
                            subsequent_indent,
                            drop_whitespace,
                        );
                    }
                    current = word;
                }
            }
        }
    }
    if (&SifrInt::from(current.chars().count()) > &SifrInt::from_i64(0)) {
        if first_line {
            _push_current_line(&mut result, &current, initial_indent, drop_whitespace);
        } else {
            _push_current_line(
                &mut result,
                &current,
                subsequent_indent,
                drop_whitespace,
            );
        }
    }
    result
}
fn fill(text: &str, width: SifrInt) -> Result<String, ValueError> {
    if (&width <= &SifrInt::from_i64(0)) {
        return Err(ValueError::new("fill: width must be > 0".to_string()));
    }
    let lines: Vec<String> = _wrap_impl(text, width.clone());
    let mut result: String = "".to_string();
    let mut i: SifrInt = SifrInt::from_i64(0);
    for line in lines.iter().cloned() {
        if (&i > &SifrInt::from_i64(0)) {
            result.push('\n');
        }
        result.push_str(line.as_str());
        i = &i + &SifrInt::from_i64(1);
    }
    Ok(result)
}
fn indent(text: &str, prefix: &str) -> String {
    let lines: Vec<String> = text
        .split('\n')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    let mut result: String = "".to_string();
    let mut first: bool = true;
    for line in lines.iter().cloned() {
        if !first {
            result.push('\n');
        }
        first = false;
        if _has_non_whitespace(&line) {
            result.push_str(prefix);
            result.push_str(line.as_str());
        } else {
            result.push_str(line.as_str());
        }
    }
    result
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
struct FloatOverflowError {
    message: String,
}
impl FloatOverflowError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for FloatOverflowError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for FloatOverflowError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FloatPrecisionLossError {
    message: String,
}
impl FloatPrecisionLossError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl ::std::fmt::Display for FloatPrecisionLossError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}
impl ::std::error::Error for FloatPrecisionLossError {}
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
impl From<TOMLDecodeError> for Error {
    fn from(err: TOMLDecodeError) -> Self {
        Self::new(err.message)
    }
}
impl From<RegexError> for Error {
    fn from(err: RegexError) -> Self {
        Self::new(err.message)
    }
}
impl From<FloatOverflowError> for Error {
    fn from(err: FloatOverflowError) -> Self {
        Self::new(err.message)
    }
}
impl From<FloatPrecisionLossError> for Error {
    fn from(err: FloatPrecisionLossError) -> Self {
        Self::new(err.message)
    }
}
impl From<TimeoutError> for Error {
    fn from(err: TimeoutError) -> Self {
        Self::new(err.message)
    }
}
impl From<ScopeFailure> for Error {
    fn from(err: ScopeFailure) -> Self {
        Self::new(err.message)
    }
}
fn add_ints(a: SifrInt, b: SifrInt) -> SifrInt {
    &a + &b
}
fn main() {
    assert!(
        & SifrInt::from(__const_ascii_lowercase().chars().count()) == &
        SifrInt::from_i64(26)
    );
    assert!(
        & SifrInt::from(__const_digits().chars().count()) == & SifrInt::from_i64(10)
    );
    let data: Vec<SifrInt> = vec![
        SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30),
        SifrInt::from_i64(40), SifrInt::from_i64(50)
    ];
    let pos: SifrInt = bisect_left(
        &data,
        &SifrInt::from_i64(30),
        SifrInt::from_i64(0),
        None,
    );
    assert_eq!(pos, SifrInt::from_i64(2));
    let nums_r: Vec<SifrInt> = vec![
        SifrInt::from_i64(1), SifrInt::from_i64(2), SifrInt::from_i64(3),
        SifrInt::from_i64(4), SifrInt::from_i64(5)
    ];
    let total: SifrInt = reduce(
        |__arg0, __arg1| add_ints((__arg0).clone(), (__arg1).clone()),
        &nums_r,
        &SifrInt::from_i64(0),
    );
    assert_eq!(total, SifrInt::from_i64(15));
    let token: String = token_hex(SifrInt::from_i64(8));
    let __sifr_chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
    assert!(& SifrInt::from(token.chars().count()) == & SifrInt::from_i64(16));
    let data2: Vec<f64> = vec![2.0_f64, 4.0_f64, 6.0_f64];
    let __sifr_try_res: Result<(), __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> = (||
    {
        let avg: f64 = mean(&data2)?;
        assert!(avg == (4.0_f64));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let se = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(18usize +
            0usize); __sifr_concat.push_str("statistics error: "); __sifr_concat
            .push_str(se.message.clone().as_str()); __sifr_concat }
        );
        assert!(
            (format!("{}", format!("{}{}", "statistics error: ", se.message.clone())) ==
            "stdlib_expansion demo: all checks passed!")
        );
    }
    let mut heap: Vec<SifrInt> = vec![];
    heappush(&mut heap, &SifrInt::from_i64(5));
    heappush(&mut heap, &SifrInt::from_i64(1));
    heappush(&mut heap, &SifrInt::from_i64(3));
    let top: Option<SifrInt> = heappop(&mut heap);
    if let Some(top) = top.clone() {
        assert_eq!(top, SifrInt::from_i64(1));
    }
    let small: Vec<SifrInt> = nsmallest(
        SifrInt::from_i64(2),
        &vec![
            SifrInt::from_i64(9), SifrInt::from_i64(3), SifrInt::from_i64(7),
            SifrInt::from_i64(1)
        ],
    );
    assert_eq!(SifrInt::from(small.len()), SifrInt::from_i64(2));
    let merged: Vec<SifrInt> = chain(
            &vec![
                vec![SifrInt::from_i64(1), SifrInt::from_i64(2)],
                vec![SifrInt::from_i64(3), SifrInt::from_i64(4)]
            ],
        )
        .collect::<Vec<_>>();
    assert_eq!(SifrInt::from(merged.len()), SifrInt::from_i64(4));
    let first3: Vec<SifrInt> = take(
        SifrInt::from_i64(3),
        &(vec![
            SifrInt::from_i64(10), SifrInt::from_i64(20), SifrInt::from_i64(30),
            SifrInt::from_i64(40)
        ])
            .into_iter()
            .collect::<Vec<_>>(),
    );
    assert_eq!(SifrInt::from(first3.len()), SifrInt::from_i64(3));
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let filled: String = fill(
            &"hello world foo bar".to_string(),
            SifrInt::from_i64(12),
        )?;
        let __sifr_chars_filled: Vec<char> = filled.chars().collect::<Vec<char>>();
        assert!(SifrInt::from(filled.chars().count()) > SifrInt::from_i64(0));
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("error: {}", e.message.clone());
    }
    let row: Vec<String> = parse_row(
        &"a,b,c".to_string(),
        &None,
        &",".to_string(),
        &"\"".to_string(),
        &"".to_string(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    assert_eq!(SifrInt::from(row.len()), SifrInt::from_i64(3));
    let line: String = format_row(
        &vec!["x".to_string(), "y".to_string()],
        &None,
        &",".to_string(),
        &"\"".to_string(),
        &"".to_string(),
        true,
        false,
        SifrInt::from_i64(0),
    );
    assert_eq!(line, "x,y");
    let args: Vec<String> = vec!["--output".to_string(), "file.txt".to_string()];
    let has_output: bool = parse_flag(&args, &"--output".to_string());
    assert!(has_output);
    let val: String = parse_option(
        &args,
        &"--output".to_string(),
        &"default".to_string(),
    );
    assert_eq!(val, "file.txt");
    assert!(fnmatch(& "test.py".to_string(), & "*.py".to_string()));
    assert!(! (fnmatch(& "test.py".to_string(), & "*.txt".to_string())));
    println!("stdlib_expansion demo: all checks passed!");
}
