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
fn parse_flag(args: &Vec<String>, flag: &String) -> bool {
    for arg in args.iter().cloned() {
        if arg == *flag {
            return true;
        }
    }
    false
}
fn _split_inline_option(token: &String) -> (bool, String, String) {
    let __sifr_chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
    let mut key: String = "".to_string();
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_token.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_token
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if ch.is_some() && (ch == Some("=".to_string())) {
            let mut value: String = "".to_string();
            let mut j: i64 = i + (1_i64);
            while (j < (__sifr_chars_token.len() as i64)) {
                let part: Option<String> = Some({
                    let Some(__indexed_char) = __sifr_chars_token
                        .get(j as usize)
                        .map(|c| c.to_string()) else {
                        unreachable!(
                            "compiler-verified string index should be in range"
                        );
                    };
                    __indexed_char
                });
                if let Some(part) = part {
                    value.push_str((part).as_str());
                }
                j += 1_i64;
            }
            return (true, key, value);
        }
        if let Some(ch) = ch {
            key.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    (
        false,
        {
            let mut __sifr_concat: String = String::with_capacity(token.len() + 0usize);
            __sifr_concat.push_str((token).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        },
        "".to_string(),
    )
}
fn parse_option(args: &Vec<String>, name: &String, default: &String) -> String {
    let mut i: i64 = 0_i64;
    while (i < (args.len() as i64)) {
        let arg: Option<String> = Some(args[i as usize].clone());
        if let Some(arg) = arg {
            if (arg == *name) && ((i + (1_i64)) < (args.len() as i64)) {
                let next_val: Option<String> = Some(
                    args[(i + (1_i64)) as usize].clone(),
                );
                if let Some(next_val) = next_val {
                    if (next_val == "--") || next_val.starts_with("--") {} else {
                        return {
                            let mut __sifr_concat: String = String::with_capacity(
                                next_val.len() + 0usize,
                            );
                            __sifr_concat.push_str((next_val).as_str());
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
                    __sifr_concat.push_str((inline_value).as_str());
                    __sifr_concat.push_str("");
                    __sifr_concat
                };
            }
        }
        i += 1_i64;
    }
    {
        let mut __sifr_concat: String = String::with_capacity(default.len() + 0usize);
        __sifr_concat.push_str((default).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn bisect_left<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    a: &Vec<T>,
    x: &T,
    lo: i64,
    hi: Option<i64>,
) -> i64 {
    let mut left: i64 = lo;
    if left < (0_i64) {
        left = 0_i64;
    }
    let mut right: i64 = a.len() as i64;
    if hi.is_none() {
        right = a.len() as i64;
    } else {
        if let Some(hi) = hi {
            if hi < (0_i64) {
                right = 0_i64;
            } else {
                if (hi > (a.len() as i64)) {
                    right = a.len() as i64;
                } else {
                    right = hi;
                }
            }
        }
    }
    while left < right {
        let mid: i64 = (left + right) / (2_i64);
        let val: Option<T> = {
            let __sifr_index_list = &a;
            let __sifr_index_i = mid;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(val) = val {
            if val < *x {
                left = mid + (1_i64);
            } else {
                right = mid;
            }
        } else {
            left = mid + (1_i64);
        }
    }
    left
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
    fn getvalue(&self) -> Vec<u8> {
        self._buffer.clone()
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
fn _first_char(text: &String) -> String {
    _char_at(text, 0_i64)
}
fn _last_char(text: &String) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    _char_at(text, (text.chars().count() as i64) - (1_i64))
}
fn parse_row(
    line: &String,
    dialect: &Option<__SifrStdlib_sifr_x2ecsv_x2eDialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> Vec<String> {
    let rows: Vec<Vec<String>> = parse_csv(
        line,
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        quoting,
    );
    if (rows.len() as i64) == (0_i64) {
        return vec![];
    }
    for (index, row) in Box::new(
        (rows)
            .iter()
            .cloned()
            .enumerate()
            .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
    ) {
        if index == (0_i64) {
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
            if (resolved.escapechar.clone() != "")
                && (ch_value == resolved.escapechar.clone())
            {
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
            if (resolved.quotechar.clone() != "")
                && (ch_value == resolved.quotechar.clone())
            {
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
        if (resolved.escapechar.clone() != "")
            && (ch_value == resolved.escapechar.clone())
        {
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
        if (resolved.quoting != QUOTE_NONE) && (resolved.quotechar.clone() != "") {
            let quotechar2: String = _quotechar_value(&resolved);
            if ch_value == quotechar2 {
                in_quotes = true;
                field_started = true;
                i += 1_i64;
                continue;
            }
        }
        if (ch_value == resolved.delimiter.clone()) {
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
fn filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name.clone());
        }
    }
    result
}
fn reduce<
    T: Clone + ::std::fmt::Display + PartialOrd + 'static,
    U: Clone + ::std::fmt::Display + PartialOrd + 'static,
>(func: impl Fn(&U, &T) -> U, data: &Vec<T>, initial: &U) -> U {
    let mut result: U = (initial).clone();
    for val in data.iter().cloned() {
        result = func(&result, &val);
    }
    result
}
fn _sift_down<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    data: &mut Vec<T>,
    mut pos: i64,
    n: i64,
) {
    let mut done: bool = false;
    while !done {
        let mut smallest: i64 = pos;
        let left: i64 = ((2_i64) * pos) + (1_i64);
        let right: i64 = ((2_i64) * pos) + (2_i64);
        if left < n {
            let s_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let l_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = left;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(s_val) = s_val {
                if let Some(l_val) = l_val {
                    if l_val < s_val {
                        smallest = left;
                    }
                }
            }
        }
        if right < n {
            let s_val2: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let r_val: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = right;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(s_val2) = s_val2 {
                if let Some(r_val) = r_val {
                    if r_val < s_val2 {
                        smallest = right;
                    }
                }
            }
        }
        if smallest == pos {
            done = true;
        } else {
            let tmp_pos: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let tmp_sm: Option<T> = {
                let __sifr_index_list = &data;
                let __sifr_index_i = smallest;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(tmp_pos) = tmp_pos {
                if let Some(tmp_sm) = tmp_sm {
                    {
                        let __idx_raw = pos;
                        let __idx_norm = if __idx_raw < 0 {
                            (data.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = data.get_mut(__idx_norm as usize) {
                                *__elem = tmp_sm.clone();
                            }
                        }
                    }
                    {
                        let __idx_raw = smallest;
                        let __idx_norm = if __idx_raw < 0 {
                            (data.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = data.get_mut(__idx_norm as usize) {
                                *__elem = tmp_pos.clone();
                            }
                        }
                    }
                }
            }
            pos = smallest;
        }
    }
}
fn _sift_up<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    mut pos: i64,
) {
    let mut done: bool = false;
    while !done {
        if pos <= (0_i64) {
            done = true;
        } else {
            let parent: i64 = (pos - (1_i64)) / (2_i64);
            let p_val: Option<T> = {
                let __sifr_index_list = &heap;
                let __sifr_index_i = parent;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            let c_val: Option<T> = {
                let __sifr_index_list = &heap;
                let __sifr_index_i = pos;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(p_val) = p_val {
                if let Some(c_val) = c_val {
                    if c_val < p_val {
                        {
                            let __idx_raw = parent;
                            let __idx_norm = if __idx_raw < 0 {
                                (heap.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                                    *__elem = c_val.clone();
                                }
                            }
                        }
                        {
                            let __idx_raw = pos;
                            let __idx_norm = if __idx_raw < 0 {
                                (heap.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                                    *__elem = p_val.clone();
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
fn heapify<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(data: &mut Vec<T>) {
    "Convert list to a min-heap in-place. O(n) time.".to_string();
    let n: i64 = data.len() as i64;
    let mut i: i64 = (n / (2_i64)) - (1_i64);
    while i >= (0_i64) {
        _sift_down(data, i, n);
        i -= 1_i64;
    }
}
fn heappush<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
    item: &T,
) {
    "Push item onto the heap in-place. O(log n) time.".to_string();
    heap.push(item.clone().clone());
    let pos: i64 = (heap.len() as i64) - (1_i64);
    _sift_up(heap, pos);
}
fn heappop<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    heap: &mut Vec<T>,
) -> Option<T> {
    "Pop and return the smallest item. Heap is modified in-place. O(log n) time.\n    Returns None if the heap is empty."
        .to_string();
    let n: i64 = heap.len() as i64;
    if n == (0_i64) {
        return None;
    }
    let top: Option<T> = Some(heap[(0_i64) as usize].clone());
    let last: Option<T> = {
        let __sifr_index_list = &heap;
        let __sifr_index_i = n - (1_i64);
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    {
        let Some(__sifr_nonempty_pop_value) = heap.pop() else {
            unreachable!("compiler-verified non-empty pop should return Some");
        };
        __sifr_nonempty_pop_value
    };
    let n2: i64 = heap.len() as i64;
    if n2 > (0_i64) {
        if let Some(last) = last {
            {
                let __idx_raw = 0_i64;
                let __idx_norm = if __idx_raw < 0 {
                    (heap.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = heap.get_mut(__idx_norm as usize) {
                        *__elem = last.clone();
                    }
                }
            }
        }
        _sift_down(heap, 0_i64, n2);
    }
    top
}
fn nsmallest<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    n: i64,
    data: &Vec<T>,
) -> Vec<T> {
    let mut heap: Vec<T> = data.clone();
    heapify(&mut heap);
    let mut result: Vec<T> = vec![];
    let mut count: i64 = 0_i64;
    while count < n {
        if ((heap.len() as i64) == (0_i64)) {
            return result;
        }
        let val: Option<T> = heappop(&mut heap);
        if let Some(val) = val {
            result.push(val.clone().clone());
        }
        count += 1_i64;
    }
    result
}
fn chain<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    iterables: &Vec<Vec<T>>,
) -> Box<dyn Iterator<Item = T>> {
    let iterables = iterables.clone();
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: ::std::vec::IntoIter<T> = Vec::new().into_iter();
    Box::new(
        ::std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<T> = Vec::new();
                for iterable in iterables.iter().cloned() {
                    for item in iterable.iter().cloned() {
                        _yields.push(item);
                    }
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn take<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    n: i64,
    data: &Vec<T>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    let mut count: i64 = 0_i64;
    for item in data.iter().cloned() {
        if count >= n {
            return result;
        }
        result.push(item.clone().clone());
        count += 1_i64;
    }
    result
}
fn flatten<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    lists: &Vec<Vec<T>>,
) -> Vec<T> {
    let mut result: Vec<T> = vec![];
    for inner in lists.iter().cloned() {
        for val in inner.iter().cloned() {
            result.push(val.clone().clone());
        }
    }
    result
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
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &String) -> String {
    ::sifr_stdlib::time::time_format(epoch, fmt)
}
fn perf_counter() -> f64 {
    ::sifr_stdlib::time::perf_counter()
}
fn sleep(seconds: f64) {
    ::sifr_stdlib::time::sleep(seconds);
}
fn monotonic() -> f64 {
    ::sifr_stdlib::time::monotonic()
}
fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn gmtime(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn _gmtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::gmtime(epoch)
}
fn localtime(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn _localtime_intrinsic(epoch: f64) -> String {
    ::sifr_stdlib::time::localtime(epoch)
}
fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn time_localtime() -> Vec<i64> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
        .collect()
}
fn factorial(n: i64) -> i64 {
    if n < (0_i64) {
        return 0_i64;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 2_i64;
    while i <= n {
        result *= i;
        i += 1_i64;
    }
    result
}
fn gcd(a: i64, b: i64) -> i64 {
    let mut x: i64 = a;
    let mut y: i64 = b;
    if x < (0_i64) {
        x = (0_i64) - x;
    }
    if y < (0_i64) {
        y = (0_i64) - y;
    }
    while y != (0_i64) {
        let temp: i64 = y;
        y = x % y;
        x = temp;
    }
    x
}
fn lcm(a: i64, b: i64) -> i64 {
    if a == (0_i64) {
        return 0_i64;
    }
    if b == (0_i64) {
        return 0_i64;
    }
    let g: i64 = gcd(a, b);
    let mut x: i64 = a;
    if x < (0_i64) {
        x = (0_i64) - x;
    }
    let mut y: i64 = b;
    if y < (0_i64) {
        y = (0_i64) - y;
    }
    (x / g) * y
}
fn comb(n: i64, k: i64) -> i64 {
    if k < (0_i64) {
        return 0_i64;
    }
    if k > n {
        return 0_i64;
    }
    if k == (0_i64) {
        return 1_i64;
    }
    if k == n {
        return 1_i64;
    }
    let mut r: i64 = k;
    if r > (n - k) {
        r = n - k;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < r {
        result *= n - i;
        result /= i + (1_i64);
        i += 1_i64;
    }
    result
}
fn perm(n: i64, k: i64) -> i64 {
    if k < (0_i64) {
        return 0_i64;
    }
    if k > n {
        return 0_i64;
    }
    let mut result: i64 = 1_i64;
    let mut i: i64 = 0_i64;
    while i < k {
        result *= n - i;
        i += 1_i64;
    }
    result
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
fn prod(data: &Vec<i64>) -> i64 {
    let mut result: i64 = 1_i64;
    for val in data.iter().copied() {
        result *= val;
    }
    result
}
fn _copy_float_list(data: &Vec<f64>) -> Vec<f64> {
    let mut out: Vec<f64> = vec![];
    for value in data.iter().copied() {
        out.push(value);
    }
    out
}
fn dist(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    dist_impl(_copy_float_list(p), _copy_float_list(q))
}
fn fsum(data: &Vec<f64>) -> f64 {
    fsum_impl(_copy_float_list(data))
}
fn sumprod(p: &Vec<f64>, q: &Vec<f64>) -> f64 {
    sumprod_impl(_copy_float_list(p), _copy_float_list(q))
}
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = frexp(x);
    let m: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(m) = m else {
        return NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = frexp(x);
    let exp_val: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(exp_val) = exp_val else {
        return 0_i64;
    };
    trunc(exp_val)
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let f: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(f) = f else {
        return NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = modf(x);
    let i: Option<f64> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    let Some(i) = i else {
        return NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    pow_val(x, y)
}
const _MT_N: i64 = 624_i64;
const _MT_M: i64 = 397_i64;
const _MT_MATRIX_A: i64 = 2567483615_i64;
const _MT_UPPER_MASK: i64 = 2147483648_i64;
const _MT_LOWER_MASK: i64 = 2147483647_i64;
const _MT_F: i64 = 1812433253_i64;
const _MT_WORD_MASK: i64 = 4294967295_i64;
#[derive(Debug, Clone, PartialEq)]
struct __SifrStdlib_sifr_x2erandom_x2eRandomState {
    version: i64,
    state_words: Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandomState {
    fn new(
        version: i64,
        state_words: Vec<i64>,
        index: i64,
        gauss_next: Option<f64>,
    ) -> Self {
        let __sifr_field_init_0: i64 = version;
        let __sifr_field_init_1: Vec<i64> = state_words;
        let __sifr_field_init_2: i64 = index;
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
    _state_words: Vec<i64>,
    _index: i64,
    _gauss_next: Option<f64>,
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn new(seed_value: Option<i64>) -> Self {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        let __sifr_field_init_0: Vec<i64> = _seed_words_from_seed(normalized_seed);
        let __sifr_field_init_1: i64 = _MT_N;
        let __sifr_field_init_2: Option<f64> = None;
        Self {
            _state_words: __sifr_field_init_0,
            _index: __sifr_field_init_1,
            _gauss_next: __sifr_field_init_2,
        }
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn seed(&mut self, seed_value: Option<i64>) {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        self._state_words = _seed_words_from_seed(normalized_seed);
        self._index = _MT_N;
        self._gauss_next = None;
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _twist(&mut self) {
        let mut i: i64 = 0_i64;
        while i < _MT_N {
            let y: i64 = (_state_word_at(&self._state_words, i) & _MT_UPPER_MASK)
                + (_state_word_at(&self._state_words, (i + (1_i64)) % _MT_N)
                    & _MT_LOWER_MASK);
            let mut x_a: i64 = y >> (1_i64);
            if (y % (2_i64)) != (0_i64) {
                x_a = x_a ^ _MT_MATRIX_A;
            }
            let new_word: i64 = _state_word_at(&self._state_words, (i + _MT_M) % _MT_N)
                ^ x_a;
            {
                let __idx_raw = i;
                let __idx_norm = if __idx_raw < 0 {
                    (self._state_words.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = self._state_words.get_mut(__idx_norm as usize)
                    {
                        *__elem = new_word & _MT_WORD_MASK;
                    }
                }
            }
            i += 1_i64;
        }
        self._index = 0_i64;
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn _next_u32(&mut self) -> i64 {
        if (self._index >= _MT_N) {
            self._twist();
        }
        let mut y: i64 = _state_word_at(&self._state_words, self._index);
        self._index += 1_i64;
        y = y ^ (y >> (11_i64));
        y = y ^ ((y << (7_i64)) & (2636928640_i64));
        y = y ^ ((y << (15_i64)) & (4022730752_i64));
        y = y ^ (y >> (18_i64));
        y & _MT_WORD_MASK
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn random(&mut self) -> f64 {
        (self._next_u32() as f64) / (4294967296.0_f64)
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
        start: i64,
        stop: Option<i64>,
        step: i64,
    ) -> Result<i64, ValueError> {
        if step == (0_i64) {
            return Err(ValueError::new("randrange: step must not be zero".to_string()));
        }
        let mut actual_start: i64 = start;
        let mut actual_stop: i64 = start;
        if stop.is_none() {
            actual_start = 0_i64;
        } else {
            if let Some(stop) = stop {
                actual_stop = stop;
            }
        }
        let width: i64 = actual_stop - actual_start;
        if step > (0_i64) {
            if width <= (0_i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        } else {
            if width >= (0_i64) {
                return Err(ValueError::new("randrange: empty range".to_string()));
            }
        }
        let mut abs_width: i64 = width;
        if abs_width < (0_i64) {
            abs_width = (0_i64) - abs_width;
        }
        let mut abs_step: i64 = step;
        if abs_step < (0_i64) {
            abs_step = (0_i64) - abs_step;
        }
        let count: i64 = ((abs_width + abs_step) - (1_i64)) / abs_step;
        if count <= (0_i64) {
            return Err(ValueError::new("randrange: empty range".to_string()));
        }
        let pick: i64 = self._next_u32() % count;
        Ok(actual_start + (pick * step))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randint(&mut self, minimum: i64, maximum: i64) -> Result<i64, ValueError> {
        if minimum > maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        self.randrange(minimum, Some(maximum + (1_i64)), 1_i64)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn getrandbits(&mut self, k: i64) -> Result<i64, ValueError> {
        if k < (0_i64) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: i64 = 0_i64;
        let mut bits_left: i64 = k;
        while bits_left > (0_i64) {
            let word: i64 = self._next_u32();
            let mut take: i64 = 32_i64;
            if bits_left < (32_i64) {
                take = bits_left;
            }
            let mask: i64 = ((1_i64) << take) - (1_i64);
            result = (result << take) | (word & mask);
            bits_left -= take;
        }
        Ok(result)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn randbytes(&mut self, n: i64) -> Result<Vec<u8>, ValueError> {
        if n < (0_i64) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = 0_i64;
        while i < n {
            let byte_value: i64 = self._next_u32() & (255_i64);
            values.push(byte_value);
            i += 1_i64;
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                if (*__pair.1 < 0) || (*__pair.1 > 255) {
                    return Err(ValueError {
                        message: format!(
                            "byte out of range at index {}: {}", __pair.0, * __pair.1
                        ),
                    });
                }
                __out.push(*__pair.1 as u8);
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
            3_i64,
            _clone_words(&self._state_words),
            self._index,
            self._gauss_next,
        )
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eRandom {
    fn setstate(
        &mut self,
        state: &__SifrStdlib_sifr_x2erandom_x2eRandomState,
    ) -> Result<(), ValueError> {
        if (state.version != (3_i64)) {
            return Err(ValueError::new("setstate: unsupported version".to_string()));
        }
        if ((state.state_words.len() as i64) != _MT_N) {
            return Err(
                ValueError::new("setstate: state_words must have length 624".to_string()),
            );
        }
        if (state.index < (0_i64)) || (state.index > _MT_N) {
            return Err(
                ValueError::new("setstate: index must be in range [0, 624]".to_string()),
            );
        }
        let mut normalized: Vec<i64> = vec![];
        for word in state.state_words.clone().iter().copied() {
            if (word < (0_i64)) || (word > _MT_WORD_MASK) {
                return Err(ValueError::new("setstate: word out of range".to_string()));
            }
            normalized.push(word & _MT_WORD_MASK);
        }
        self._state_words = normalized;
        self._index = state.index;
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
    fn seed(&self, _seed_value: Option<i64>) {}
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
        start: i64,
        stop: Option<i64>,
        step: i64,
    ) -> Result<i64, ValueError> {
        let mut actual_start: i64 = start;
        let mut actual_stop: i64 = start;
        if stop.is_none() {
            actual_start = 0_i64;
        } else {
            if let Some(stop) = stop {
                actual_stop = stop;
            }
        }
        random_randrange(actual_start, actual_stop, step)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randint(&self, minimum: i64, maximum: i64) -> Result<i64, ValueError> {
        if minimum > maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        Ok(random_int(minimum, maximum))
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn getrandbits(&self, k: i64) -> Result<i64, ValueError> {
        if k < (0_i64) {
            return Err(
                ValueError::new("getrandbits: number of bits must be >= 0".to_string()),
            );
        }
        let mut result: i64 = 0_i64;
        let mut i: i64 = 0_i64;
        while i < k {
            let mut bit: i64 = 0_i64;
            if (random_float() >= (0.5_f64)) {
                bit = 1_i64;
            }
            result = (result * (2_i64)) + bit;
            i += 1_i64;
        }
        Ok(result)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn gauss(&self, mu: f64, sigma: f64) -> f64 {
        random_gauss(mu, sigma)
    }
}
impl __SifrStdlib_sifr_x2erandom_x2eSystemRandom {
    fn randbytes(&self, n: i64) -> Result<Vec<u8>, ValueError> {
        if n < (0_i64) {
            return Err(ValueError::new("randbytes: n must be >= 0".to_string()));
        }
        let mut values: Vec<i64> = vec![];
        let mut i: i64 = 0_i64;
        while i < n {
            let mut value: i64 = (random_float() * (256.0_f64)) as i64;
            if value > (255_i64) {
                value = 255_i64;
            }
            values.push(value);
            i += 1_i64;
        }
        {
            let __vals = values;
            let mut __out = Vec::new();
            for __pair in __vals.iter().enumerate() {
                if (*__pair.1 < 0) || (*__pair.1 > 255) {
                    return Err(ValueError {
                        message: format!(
                            "byte out of range at index {}: {}", __pair.0, * __pair.1
                        ),
                    });
                }
                __out.push(*__pair.1 as u8);
            }
            Ok::<Vec<u8>, ValueError>(__out)
        }
    }
}
fn _state_word_at(words: &Vec<i64>, index: i64) -> i64 {
    let value: Option<i64> = {
        let __sifr_index_list = &words;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(value) = value {
        return value;
    }
    0_i64
}
fn _clone_words(words: &Vec<i64>) -> Vec<i64> {
    let mut copied: Vec<i64> = vec![];
    for word in words.iter().copied() {
        copied.push(word);
    }
    copied
}
fn _normalize_seed_input(seed_value: Option<i64>) -> i64 {
    if let Some(seed_value) = seed_value {
        return seed_value;
    }
    (time_now() * (1000000.0_f64)) as i64
}
fn _seed_words_from_seed(seed_value: i64) -> Vec<i64> {
    let mut words: Vec<i64> = vec![];
    words.push(seed_value & _MT_WORD_MASK);
    let mut i: i64 = 1_i64;
    while i < _MT_N {
        let prev: i64 = _state_word_at(&words, i - (1_i64));
        let next_word: i64 = ((_MT_F * (prev ^ (prev >> (30_i64)))) + i) & _MT_WORD_MASK;
        words.push(next_word);
        i += 1_i64;
    }
    words
}
fn _build_state_from_module_storage() -> __SifrStdlib_sifr_x2erandom_x2eRandomState {
    __SifrStdlib_sifr_x2erandom_x2eRandomState::new(
        3_i64,
        random_module_state_words(),
        random_module_state_index(),
        random_module_state_gauss_next(),
    )
}
fn _store_state_into_module_storage(state: &__SifrStdlib_sifr_x2erandom_x2eRandomState) {
    let _set_result: Result<(), ValueError> = random_module_set_state(
        &_clone_words(&state.state_words.clone()),
        state.index,
        state.gauss_next,
    );
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<i64> = random_module_state_words();
    if (words.len() as i64) == _MT_N {
        return;
    }
    let bootstrap: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(5489_i64),
    );
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> __SifrStdlib_sifr_x2erandom_x2eRandom {
    _ensure_module_state_initialized();
    let mut r: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        Some(0_i64),
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
fn seed(seed_value: Option<i64>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = __SifrStdlib_sifr_x2erandom_x2eRandom::new(
        seed_value,
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
        Some(0_i64),
    );
    let result: Result<(), ValueError> = probe.setstate(state);
    _sync_module_random(&mut probe);
    result
}
fn randint(minimum: i64, maximum: i64) -> Result<i64, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<i64, ValueError> = generator.randint(minimum, maximum);
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
fn randrange(start: i64, stop: Option<i64>, step: i64) -> Result<i64, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<i64, ValueError> = generator.randrange(start, stop, step);
    _sync_module_random(&mut generator);
    value
}
fn getrandbits(k: i64) -> Result<i64, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<i64, ValueError> = generator.getrandbits(k);
    _sync_module_random(&mut generator);
    value
}
fn gauss(mu: f64, sigma: f64) -> f64 {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: f64 = generator.gauss(mu, sigma);
    _sync_module_random(&mut generator);
    value
}
fn choice<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
) -> Result<T, ValueError> {
    if ((items.len() as i64) == (0_i64)) {
        return Err(ValueError::new("choice: items must not be empty".to_string()));
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let index: i64 = generator._next_u32() % (items.len() as i64);
    let picked: Option<T> = {
        let __sifr_index_list = &items;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    _sync_module_random(&mut generator);
    if let Some(picked) = picked {
        return Ok(picked);
    }
    Err(ValueError::new("choice: index out of range".to_string()))
}
fn choices<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    k: i64,
) -> Result<Vec<T>, ValueError> {
    if k <= (0_i64) {
        return Ok(vec![]);
    }
    if ((items.len() as i64) == (0_i64)) {
        return Err(ValueError::new("choices: items must not be empty".to_string()));
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut i: i64 = 0_i64;
    while i < k {
        let index: i64 = generator._next_u32() % (items.len() as i64);
        let picked: Option<T> = {
            let __sifr_index_list = &items;
            let __sifr_index_i = index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone().clone());
        } else {
            return Err(ValueError::new("choices: index out of range".to_string()));
        }
        i += 1_i64;
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn sample<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    k: i64,
) -> Result<Vec<T>, ValueError> {
    if k < (0_i64) {
        return Err(ValueError::new("sample: k must be >= 0".to_string()));
    }
    if (k > (items.len() as i64)) {
        return Err(ValueError::new("sample larger than population".to_string()));
    }
    let mut pool: Vec<T> = vec![];
    for item in items.iter().cloned() {
        pool.push(item.clone().clone());
    }
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let mut result: Vec<T> = vec![];
    let mut remaining: i64 = pool.len() as i64;
    let mut i: i64 = 0_i64;
    while i < k {
        let pick_index: i64 = generator._next_u32() % remaining;
        let picked: Option<T> = {
            let __sifr_index_list = &pool;
            let __sifr_index_i = pick_index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(picked) = picked {
            result.push(picked.clone().clone());
        }
        let last: Option<T> = {
            let __sifr_index_list = &pool;
            let __sifr_index_i = remaining - (1_i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        if let Some(last) = last {
            {
                let __idx_raw = pick_index;
                let __idx_norm = if __idx_raw < 0 {
                    (pool.len() as i64) + __idx_raw
                } else {
                    __idx_raw
                };
                if __idx_norm >= 0 {
                    if let Some(__elem) = pool.get_mut(__idx_norm as usize) {
                        *__elem = last.clone();
                    }
                }
            }
        }
        remaining -= 1_i64;
        i += 1_i64;
    }
    _sync_module_random(&mut generator);
    Ok(result)
}
fn shuffle<T: Clone + ::std::fmt::Display + PartialOrd + 'static>(items: &mut Vec<T>) {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let n: i64 = items.len() as i64;
    if n > (1_i64) {
        let mut i: i64 = n - (1_i64);
        while i > (0_i64) {
            let j: i64 = generator._next_u32() % (i + (1_i64));
            let left: Option<T> = Some(items[i as usize].clone());
            let right: Option<T> = {
                let __sifr_index_list = &items;
                let __sifr_index_i = j;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if let Some(left) = left {
                if let Some(right) = right {
                    {
                        let __idx_raw = i;
                        let __idx_norm = if __idx_raw < 0 {
                            (items.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                                *__elem = right.clone();
                            }
                        }
                    }
                    {
                        let __idx_raw = j;
                        let __idx_norm = if __idx_raw < 0 {
                            (items.len() as i64) + __idx_raw
                        } else {
                            __idx_raw
                        };
                        if __idx_norm >= 0 {
                            if let Some(__elem) = items.get_mut(__idx_norm as usize) {
                                *__elem = left.clone();
                            }
                        }
                    }
                }
            }
            i -= 1_i64;
        }
    }
    _sync_module_random(&mut generator);
}
fn randbytes(n: i64) -> Result<Vec<u8>, ValueError> {
    let mut generator: __SifrStdlib_sifr_x2erandom_x2eRandom = _module_random();
    let value: Result<Vec<u8>, ValueError> = generator.randbytes(n);
    _sync_module_random(&mut generator);
    value
}
fn token_hex(nbytes: i64) -> String {
    let hex_chars: String = "0123456789abcdef".to_string();
    let mut result: String = "".to_string();
    let mut i: i64 = 0_i64;
    while i < (nbytes * (2_i64)) {
        let idx: i64 = random_int(0_i64, 15_i64);
        let ch: Option<String> = {
            let __sifr_index_str = &hex_chars;
            let __sifr_index_i = idx;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
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
fn _sum(data: &Vec<f64>) -> f64 {
    let mut total: f64 = 0.0_f64;
    for val in data.iter().copied() {
        total += val;
    }
    total
}
fn mean(
    data: &Vec<f64>,
) -> Result<f64, __SifrStdlib_sifr_x2estatistics_x2eStatisticsError> {
    let count: i64 = data.len() as i64;
    if count == (0_i64) {
        return Err(
            __SifrStdlib_sifr_x2estatistics_x2eStatisticsError::new(
                "mean requires at least one data point".to_string(),
            ),
        );
    }
    let total: f64 = _sum(data);
    Ok(total / (count as f64))
}
fn __const_ascii_lowercase() -> String {
    "abcdefghijklmnopqrstuvwxyz".to_string().to_string()
}
fn __const_digits() -> String {
    "0123456789".to_string().to_string()
}
fn _replace_whitespace_chars(text: &String, replace_tabs: bool) -> String {
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
fn _expand_tabs_impl(text: &String, tabsize: i64) -> String {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut effective_tabsize: i64 = tabsize;
    if effective_tabsize <= (0_i64) {
        effective_tabsize = 1_i64;
    }
    let mut result: String = "".to_string();
    let mut column: i64 = 0_i64;
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_text.len() as i64)) {
        let ch_opt: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_text
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch_opt) = ch_opt {
            let ch: String = ch_opt;
            if ch == "\t" {
                let mut spaces: i64 = effective_tabsize - (column % effective_tabsize);
                if spaces <= (0_i64) {
                    spaces = effective_tabsize;
                }
                let mut j: i64 = 0_i64;
                while j < spaces {
                    result.push(' ');
                    j += 1_i64;
                }
                column += spaces;
            } else {
                if (ch == "\n") || (ch == "\r") {
                    result.push_str((ch).as_str());
                    column = 0_i64;
                } else {
                    result.push_str((ch).as_str());
                    column += 1_i64;
                }
            }
        }
        i += 1_i64;
    }
    result
}
fn _prepare_text(
    text: &String,
    expand_tabs: bool,
    tabsize: i64,
    replace_whitespace: bool,
) -> String {
    let mut prepared: String = {
        let mut __sifr_concat: String = String::with_capacity(text.len() + 0usize);
        __sifr_concat.push_str((text).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if expand_tabs {
        prepared = _expand_tabs_impl(&prepared, tabsize);
    }
    if replace_whitespace {
        prepared = _replace_whitespace_chars(&prepared, true);
    }
    prepared
}
fn _normalize_whitespace(text: &String) -> String {
    _prepare_text(text, true, 8_i64, true)
}
fn _has_non_whitespace(text: &String) -> bool {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut i: i64 = 0_i64;
    while (i < (__sifr_chars_text.len() as i64)) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_text
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(ch) = ch {
            if ch != " " {
                if ch != "\t" {
                    if ch != "\n" {
                        if ch != "\r" {
                            if ch != "\u{b}" {
                                if ch != "\u{c}" {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
        i += 1_i64;
    }
    false
}
fn _split_word_units(word: &String, break_on_hyphens: bool) -> Vec<String> {
    if !break_on_hyphens {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str((word).as_str()); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let parts: Vec<String> = word
        .split('-')
        .map(|s| s.to_string())
        .collect::<Vec<String>>();
    if ((parts.len() as i64) <= (1_i64)) {
        return vec![
            { let mut __sifr_concat : String = String::with_capacity(word.len() +
            0usize); __sifr_concat.push_str((word).as_str()); __sifr_concat.push_str("");
            __sifr_concat }
        ];
    }
    let mut units: Vec<String> = vec![];
    let mut index: i64 = 0_i64;
    for part in parts.iter().cloned() {
        let __sifr_chars_part: Vec<char> = part.chars().collect::<Vec<char>>();
        let is_last: bool = (index == ((parts.len() as i64) - (1_i64)));
        if is_last {
            if ((__sifr_chars_part.len() as i64) > (0_i64)) {
                units.push(part.clone());
            }
        } else {
            if ((__sifr_chars_part.len() as i64) == (0_i64)) {
                units.push("-".to_string());
            } else {
                units.push(format!("{}{}", part, "-"));
            }
        }
        index += 1_i64;
    }
    if ((units.len() as i64) == (0_i64)) {
        units.push(format!("{}{}", word, ""));
    }
    units
}
fn _trim_line(line: &String) -> String {
    let __sifr_chars_line: Vec<char> = line.chars().collect::<Vec<char>>();
    let mut start: i64 = 0_i64;
    while (start < (__sifr_chars_line.len() as i64))
        && (({
            let Some(__indexed_char) = __sifr_chars_line
                .get(start as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) == " ")
    {
        start += 1_i64;
    }
    let mut end: i64 = __sifr_chars_line.len() as i64;
    while (end > start)
        && (__sifr_chars_line.get((end - (1_i64)) as usize).map(|c| c.to_string())
            == Some(" ".to_string()))
    {
        end -= 1_i64;
    }
    {
        let _slice_src = &__sifr_chars_line;
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
        String::from_iter(
            _slice_src
                .iter()
                .skip(_slice_start_i64 as usize)
                .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                .copied(),
        )
    }
}
fn _finalize_line(line: &String, drop_whitespace: bool) -> String {
    if drop_whitespace {
        return _trim_line(line);
    }
    {
        let mut __sifr_concat: String = String::with_capacity(line.len() + 0usize);
        __sifr_concat.push_str((line).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn _wrap_impl(text: &String, width: i64) -> Vec<String> {
    let normalized: String = _normalize_whitespace(text);
    _wrap_with_indents(&normalized, width, &"".to_string(), &"".to_string(), true, true)
}
fn _effective_content_width(total_width: i64, indent: &String) -> i64 {
    let __sifr_chars_indent: Vec<char> = indent.chars().collect::<Vec<char>>();
    let available: i64 = total_width - (__sifr_chars_indent.len() as i64);
    if available <= (0_i64) {
        return 1_i64;
    }
    available
}
fn _push_current_line(
    result: &mut Vec<String>,
    line: &String,
    indent: &String,
    drop_whitespace: bool,
) {
    let candidate: String = _finalize_line(
        &format!("{}{}", indent, line),
        drop_whitespace,
    );
    let __sifr_chars_candidate: Vec<char> = candidate.chars().collect::<Vec<char>>();
    if drop_whitespace {
        if ((__sifr_chars_candidate.len() as i64) > (0_i64)) {
            result.push(candidate.clone());
        }
    } else {
        result.push(candidate.clone());
    }
}
fn _wrap_with_indents(
    text: &String,
    total_width: i64,
    initial_indent: &String,
    subsequent_indent: &String,
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
    let mut current_limit: i64 = _effective_content_width(total_width, initial_indent);
    for raw_word in words.iter().cloned() {
        let units: Vec<String> = _split_word_units(&raw_word, break_on_hyphens);
        for word in units.iter().cloned() {
            let __sifr_chars_word: Vec<char> = word.chars().collect::<Vec<char>>();
            if ((__sifr_chars_word.len() as i64) == (0_i64)) {
                if drop_whitespace {
                    continue;
                }
                if ((current.chars().count() as i64) > (0_i64)) {
                    if (((current.chars().count() as i64) + (1_i64)) <= current_limit) {
                        current.push(' ');
                    }
                }
                continue;
            }
            if ((current.chars().count() as i64) == (0_i64)) {
                current = word;
            } else {
                if ((((current.chars().count() as i64) + (1_i64))
                    + (__sifr_chars_word.len() as i64)) <= current_limit)
                {
                    current.push(' ');
                    current.push_str((word).as_str());
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
                            total_width,
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
    if ((current.chars().count() as i64) > (0_i64)) {
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
fn fill(text: &String, width: i64) -> Result<String, ValueError> {
    if width <= (0_i64) {
        return Err(ValueError::new("fill: width must be > 0".to_string()));
    }
    let lines: Vec<String> = _wrap_impl(text, width);
    let mut result: String = "".to_string();
    let mut i: i64 = 0_i64;
    for line in lines.iter().cloned() {
        if i > (0_i64) {
            result.push('\n');
        }
        result.push_str((line).as_str());
        i += 1_i64;
    }
    Ok(result)
}
fn indent(text: &String, prefix: &String) -> String {
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
            result.push_str((prefix).as_str());
            result.push_str((line).as_str());
        } else {
            result.push_str((line).as_str());
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
fn add_ints(a: i64, b: i64) -> i64 {
    a + b
}
fn main() {
    assert!((__const_ascii_lowercase().chars().count() as i64) == (26_i64));
    assert!((__const_digits().chars().count() as i64) == (10_i64));
    let data: Vec<i64> = vec![10_i64, 20_i64, 30_i64, 40_i64, 50_i64];
    let pos: i64 = bisect_left(&data, &(30_i64), 0_i64, None);
    assert_eq!(pos, 2_i64);
    let nums_r: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let total: i64 = reduce(
        |__arg0, __arg1| add_ints((__arg0).clone(), (__arg1).clone()),
        &nums_r,
        &(0_i64),
    );
    assert_eq!(total, 15_i64);
    let token: String = token_hex(8_i64);
    let __sifr_chars_token: Vec<char> = token.chars().collect::<Vec<char>>();
    assert!((token.chars().count() as i64) == (16_i64));
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
            .push_str((se.message.clone()).as_str()); __sifr_concat }
        );
        assert!(
            (format!("{}", format!("{}{}", "statistics error: ", se.message.clone())) ==
            "stdlib_expansion demo: all checks passed!")
        );
    }
    let mut heap: Vec<i64> = vec![];
    heappush(&mut heap, &(5_i64));
    heappush(&mut heap, &(1_i64));
    heappush(&mut heap, &(3_i64));
    let top: Option<i64> = heappop(&mut heap);
    if let Some(top) = top {
        assert_eq!(top, 1_i64);
    }
    let small: Vec<i64> = nsmallest(2_i64, &vec![9_i64, 3_i64, 7_i64, 1_i64]);
    assert_eq!(small.len() as i64, 2_i64);
    let merged: Vec<i64> = chain(&vec![vec![1_i64, 2_i64], vec![3_i64, 4_i64]])
        .collect::<Vec<_>>();
    assert_eq!(merged.len() as i64, 4_i64);
    let first3: Vec<i64> = take(
        3_i64,
        &(vec![10_i64, 20_i64, 30_i64, 40_i64]).into_iter().collect::<Vec<_>>(),
    );
    assert_eq!(first3.len() as i64, 3_i64);
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let filled: String = fill(&"hello world foo bar".to_string(), 12_i64)?;
        let __sifr_chars_filled: Vec<char> = filled.chars().collect::<Vec<char>>();
        assert!((filled.chars().count() as i64) > (0_i64));
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
        0_i64,
    );
    assert_eq!(row.len() as i64, 3_i64);
    let line: String = format_row(
        &vec!["x".to_string(), "y".to_string()],
        &None,
        &",".to_string(),
        &"\"".to_string(),
        &"".to_string(),
        true,
        false,
        0_i64,
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
