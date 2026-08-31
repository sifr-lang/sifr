// src/main.rs
mod __sifr_project_nominals {
    pub use ::sifr_runtime::SifrInt;
    pub fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    pub fn time_format(epoch: f64, fmt: &str) -> String {
        ::sifr_stdlib::time::time_format(epoch, fmt)
    }
    pub fn perf_counter() -> f64 {
        ::sifr_stdlib::time::perf_counter()
    }
    pub fn sleep(seconds: f64) {
        ::sifr_stdlib::time::sleep(seconds);
    }
    pub fn monotonic() -> f64 {
        ::sifr_stdlib::time::monotonic()
    }
    pub fn strptime(s: &str, fmt: &str) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _strptime_intrinsic(s: &str, fmt: &str) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn gmtime(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn _gmtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::gmtime(epoch)
    }
    pub fn localtime(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn _localtime_intrinsic(epoch: f64) -> String {
        ::sifr_stdlib::time::localtime(epoch)
    }
    pub fn time_strptime(s: &str, fmt: &str) -> Result<Vec<SifrInt>, ValueError> {
        ::sifr_stdlib::time::time_strptime(s, fmt)
            .map(|__sifr_bridge_ok| {
                __sifr_bridge_ok
                    .into_iter()
                    .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
                    .collect()
            })
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn time_gmtime() -> Vec<SifrInt> {
        ::sifr_stdlib::time::time_gmtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    pub fn time_localtime() -> Vec<SifrInt> {
        ::sifr_stdlib::time::time_localtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
            .collect()
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub tm_year: SifrInt,
        pub tm_mon: SifrInt,
        pub tm_mday: SifrInt,
        pub tm_hour: SifrInt,
        pub tm_min: SifrInt,
        pub tm_sec: SifrInt,
        pub tm_wday: SifrInt,
        pub tm_yday: SifrInt,
        pub tm_isdst: SifrInt,
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn new(
            tm_year: SifrInt,
            tm_mon: SifrInt,
            tm_mday: SifrInt,
            tm_hour: SifrInt,
            tm_min: SifrInt,
            tm_sec: SifrInt,
            tm_wday: SifrInt,
            tm_yday: SifrInt,
            tm_isdst: SifrInt,
        ) -> Self {
            let __sifr_field_init_0: SifrInt = tm_year.clone();
            let __sifr_field_init_1: SifrInt = tm_mon.clone();
            let __sifr_field_init_2: SifrInt = tm_mday.clone();
            let __sifr_field_init_3: SifrInt = tm_hour.clone();
            let __sifr_field_init_4: SifrInt = tm_min.clone();
            let __sifr_field_init_5: SifrInt = tm_sec.clone();
            let __sifr_field_init_6: SifrInt = tm_wday.clone();
            let __sifr_field_init_7: SifrInt = tm_yday.clone();
            let __sifr_field_init_8: SifrInt = tm_isdst.clone();
            Self {
                tm_year: __sifr_field_init_0,
                tm_mon: __sifr_field_init_1,
                tm_mday: __sifr_field_init_2,
                tm_hour: __sifr_field_init_3,
                tm_min: __sifr_field_init_4,
                tm_sec: __sifr_field_init_5,
                tm_wday: __sifr_field_init_6,
                tm_yday: __sifr_field_init_7,
                tm_isdst: __sifr_field_init_8,
            }
        }
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn as_tuple(
            &self,
        ) -> (
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
            SifrInt,
        ) {
            (
                self.tm_year.clone(),
                self.tm_mon.clone(),
                self.tm_mday.clone(),
                self.tm_hour.clone(),
                self.tm_min.clone(),
                self.tm_sec.clone(),
                self.tm_wday.clone(),
                self.tm_yday.clone(),
                self.tm_isdst.clone(),
            )
        }
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn isoformat(&self) -> String {
            let y: String = format!("{}", self.tm_year.clone());
            let mut mo: String = format!("{}", self.tm_mon.clone());
            if (&SifrInt::from(mo.chars().count()) < &SifrInt::from_i64(2)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str(mo.as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.tm_mday.clone());
            if (&SifrInt::from(d.chars().count()) < &SifrInt::from_i64(2)) {
                d = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str(d.as_str());
                    __sifr_concat
                };
            }
            let mut h: String = format!("{}", self.tm_hour.clone());
            if (&SifrInt::from(h.chars().count()) < &SifrInt::from_i64(2)) {
                h = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str(h.as_str());
                    __sifr_concat
                };
            }
            let mut mi: String = format!("{}", self.tm_min.clone());
            if (&SifrInt::from(mi.chars().count()) < &SifrInt::from_i64(2)) {
                mi = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str(mi.as_str());
                    __sifr_concat
                };
            }
            let mut s: String = format!("{}", self.tm_sec.clone());
            if (&SifrInt::from(s.chars().count()) < &SifrInt::from_i64(2)) {
                s = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + s.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str(s.as_str());
                    __sifr_concat
                };
            }
            {
                let mut __sifr_concat: String = String::with_capacity(
                    (((((((((y.len() + 1usize) + mo.len()) + 1usize) + d.len()) + 1usize)
                        + h.len()) + 1usize) + mi.len()) + 1usize) + s.len(),
                );
                __sifr_concat.push_str(y.as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str(mo.as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str(d.as_str());
                __sifr_concat.push('T');
                __sifr_concat.push_str(h.as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str(mi.as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str(s.as_str());
                __sifr_concat
            }
        }
    }
    impl PartialEq for __SifrStdlib_sifr_x2etime_x2estruct__time {
        fn eq(&self, other: &__SifrStdlib_sifr_x2etime_x2estruct__time) -> bool {
            self.as_tuple() == other.as_tuple()
        }
    }
    impl ::std::fmt::Display for __SifrStdlib_sifr_x2etime_x2estruct__time {
        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
            write!(f, "{}", self.isoformat())
        }
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
}
pub use __sifr_project_nominals::ParseError;
pub use __sifr_project_nominals::ValueError;
pub use __sifr_project_nominals::__SifrStdlib_sifr_x2etime_x2estruct__time;
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
fn sha224(data: &[u8]) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _build_hash(&"sha224".to_string(), data)
}
fn sha384(data: &[u8]) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _build_hash(&"sha384".to_string(), data)
}
fn blake2b(data: &[u8]) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _build_hash(&"blake2b".to_string(), data)
}
fn blake2s(data: &[u8]) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _build_hash(&"blake2s".to_string(), data)
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
fn platform_system() -> String {
    ::sifr_stdlib::platform::platform_system()
}
fn platform_arch() -> String {
    ::sifr_stdlib::platform::platform_arch()
}
fn platform_node() -> String {
    ::sifr_stdlib::platform::platform_node()
}
fn platform_release() -> String {
    ::sifr_stdlib::platform::platform_release()
}
fn platform_version() -> String {
    ::sifr_stdlib::platform::platform_version()
}
fn platform_processor() -> String {
    ::sifr_stdlib::platform::platform_processor()
}
fn system() -> String {
    platform_system()
}
fn machine() -> String {
    platform_arch()
}
fn processor() -> String {
    platform_processor()
}
fn time_now() -> f64 {
    ::sifr_stdlib::time::time_now()
}
fn time_format(epoch: f64, fmt: &str) -> String {
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
fn strptime(s: &str, fmt: &str) -> Result<String, ValueError> {
    ::sifr_stdlib::time::strptime(s, fmt)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn _strptime_intrinsic(s: &str, fmt: &str) -> Result<String, ValueError> {
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
fn time_strptime(s: &str, fmt: &str) -> Result<Vec<SifrInt>, ValueError> {
    ::sifr_stdlib::time::time_strptime(s, fmt)
        .map(|__sifr_bridge_ok| {
            __sifr_bridge_ok
                .into_iter()
                .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
                .collect()
        })
        .map_err(|__sifr_bridge_error| ValueError {
            message: __sifr_bridge_error.to_string(),
        })
}
fn time_gmtime() -> Vec<SifrInt> {
    ::sifr_stdlib::time::time_gmtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn time_localtime() -> Vec<SifrInt> {
    ::sifr_stdlib::time::time_localtime()
        .into_iter()
        .map(|__sifr_bridge_value| __sifr_bridge_value.into_sifr_int())
        .collect()
}
fn _is_leap_year(year: SifrInt) -> bool {
    (((&year.floor_mod_known_nonzero(&SifrInt::from_i64(4)) == &SifrInt::from_i64(0))
        && (&year.floor_mod_known_nonzero(&SifrInt::from_i64(100))
            != &SifrInt::from_i64(0)))
        || ((&year.floor_mod_known_nonzero(&SifrInt::from_i64(400))
            == &SifrInt::from_i64(0))))
}
fn _days_in_year(year: SifrInt) -> SifrInt {
    if _is_leap_year(year.clone()) {
        return SifrInt::from_i64(366);
    }
    SifrInt::from_i64(365)
}
fn _days_in_month(year: SifrInt, month: SifrInt) -> SifrInt {
    let month_days: Vec<SifrInt> = vec![
        SifrInt::from_i64(31), SifrInt::from_i64(28), SifrInt::from_i64(31),
        SifrInt::from_i64(30), SifrInt::from_i64(31), SifrInt::from_i64(30),
        SifrInt::from_i64(31), SifrInt::from_i64(31), SifrInt::from_i64(30),
        SifrInt::from_i64(31), SifrInt::from_i64(30), SifrInt::from_i64(31)
    ];
    let idx: SifrInt = &month - &SifrInt::from_i64(1);
    let d: Option<SifrInt> = {
        let __sifr_checked_read_collection = &month_days;
        let __sifr_checked_read_index = idx.clone();
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if (&month == &SifrInt::from_i64(2)) && _is_leap_year(year.clone()) {
        return SifrInt::from_i64(29);
    }
    if let Some(d) = d.clone() {
        return d;
    }
    SifrInt::from_i64(0)
}
fn _substring(value: &str, start: SifrInt, end: SifrInt) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: SifrInt = start.clone();
    while (&i < &end) {
        let ch: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_value.len());
            __sifr_chars_value.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str(ch.as_str());
        }
        i = &i + &SifrInt::from_i64(1);
    }
    result
}
fn _digit_value(ch: &str) -> Option<SifrInt> {
    if ch == "0" {
        return Some(SifrInt::from_i64(0));
    }
    if ch == "1" {
        return Some(SifrInt::from_i64(1));
    }
    if ch == "2" {
        return Some(SifrInt::from_i64(2));
    }
    if ch == "3" {
        return Some(SifrInt::from_i64(3));
    }
    if ch == "4" {
        return Some(SifrInt::from_i64(4));
    }
    if ch == "5" {
        return Some(SifrInt::from_i64(5));
    }
    if ch == "6" {
        return Some(SifrInt::from_i64(6));
    }
    if ch == "7" {
        return Some(SifrInt::from_i64(7));
    }
    if ch == "8" {
        return Some(SifrInt::from_i64(8));
    }
    if ch == "9" {
        return Some(SifrInt::from_i64(9));
    }
    None
}
fn _parse_decimal(text: &str) -> Option<SifrInt> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (&SifrInt::from(__sifr_chars_text.len()) == &SifrInt::from_i64(0)) {
        return None;
    }
    let mut out: SifrInt = SifrInt::from_i64(0);
    let mut i: SifrInt = SifrInt::from_i64(0);
    while (&i < &SifrInt::from(__sifr_chars_text.len())) {
        let ch_opt: Option<String> = ({
            let __sifr_string_index = i.clone();
            let __sifr_string_index_normalized = __sifr_string_index
                .normalize_index_or_len(__sifr_chars_text.len());
            __sifr_chars_text.get(__sifr_string_index_normalized)
        })
            .map(|c| c.to_string());
        let Some(ch_opt) = ch_opt else {
            return None;
        };
        let ch: String = ch_opt;
        let digit_opt: Option<SifrInt> = _digit_value(&ch);
        let Some(digit_opt) = digit_opt.clone() else {
            return None;
        };
        let digit: SifrInt = digit_opt.clone();
        out = &(&out * &SifrInt::from_i64(10)) + &digit;
        i = &i + &SifrInt::from_i64(1);
    }
    Some(out)
}
fn _int_or_negative_one(value: Option<SifrInt>) -> SifrInt {
    let Some(value) = value.clone() else {
        return -&SifrInt::from_i64(1);
    };
    value.clone()
}
fn _day_of_year(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
    let mut yday: SifrInt = SifrInt::from_i64(0);
    let mut m: SifrInt = SifrInt::from_i64(1);
    while (&m < &month) {
        yday = &yday + &_days_in_month(year.clone(), m.clone());
        m = &m + &SifrInt::from_i64(1);
    }
    &yday + &day
}
fn _weekday(year: SifrInt, month: SifrInt, day: SifrInt) -> SifrInt {
    let mut days_since_epoch: SifrInt = SifrInt::from_i64(0);
    if (&year >= &SifrInt::from_i64(1970)) {
        let mut y: SifrInt = SifrInt::from_i64(1970);
        while (&y < &year) {
            days_since_epoch = &days_since_epoch + &_days_in_year(y.clone());
            y = &y + &SifrInt::from_i64(1);
        }
    } else {
        let mut y: SifrInt = SifrInt::from_i64(1969);
        while (&y >= &year) {
            days_since_epoch = &days_since_epoch - &_days_in_year(y.clone());
            y = &y - &SifrInt::from_i64(1);
        }
    }
    let mut m: SifrInt = SifrInt::from_i64(1);
    while (&m < &month) {
        days_since_epoch = &days_since_epoch + &_days_in_month(year.clone(), m.clone());
        m = &m + &SifrInt::from_i64(1);
    }
    days_since_epoch = &(&days_since_epoch + &day) - &SifrInt::from_i64(1);
    let mut wd: SifrInt = (&SifrInt::from_i64(3) + &days_since_epoch)
        .floor_mod_known_nonzero(&SifrInt::from_i64(7));
    if &wd < &SifrInt::from_i64(0) {
        wd = &wd + &SifrInt::from_i64(7);
    }
    wd.clone()
}
fn _valid_date(year: SifrInt, month: SifrInt, day: SifrInt) -> bool {
    if &year <= &SifrInt::from_i64(0) {
        return false;
    }
    if (&month < &SifrInt::from_i64(1)) || (&month > &SifrInt::from_i64(12)) {
        return false;
    }
    let max_day: SifrInt = _days_in_month(year.clone(), month.clone());
    (&day >= &SifrInt::from_i64(1)) && (&day <= &max_day)
}
fn _invalid_struct_time() -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    __SifrStdlib_sifr_x2etime_x2estruct__time::new(
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
        SifrInt::from_i64(0),
    )
}
fn _to_struct_time(rendered: &str) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
    let Some(__sifr_checked_value_3) = ({
        let __sifr_string_index = SifrInt::from_i64(4);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_rendered.len());
        __sifr_chars_rendered.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return _invalid_struct_time();
    };
    let Some(__sifr_checked_value_4) = ({
        let __sifr_string_index = SifrInt::from_i64(7);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_rendered.len());
        __sifr_chars_rendered.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return _invalid_struct_time();
    };
    let Some(__sifr_checked_value_5) = ({
        let __sifr_string_index = SifrInt::from_i64(10);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_rendered.len());
        __sifr_chars_rendered.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return _invalid_struct_time();
    };
    let Some(__sifr_checked_value_6) = ({
        let __sifr_string_index = SifrInt::from_i64(13);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_rendered.len());
        __sifr_chars_rendered.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return _invalid_struct_time();
    };
    let Some(__sifr_checked_value_7) = ({
        let __sifr_string_index = SifrInt::from_i64(16);
        let __sifr_string_index_normalized = __sifr_string_index
            .normalize_index_or_len(__sifr_chars_rendered.len());
        __sifr_chars_rendered.get(__sifr_string_index_normalized)
    })
        .map(|c| c.to_string()) else {
        return _invalid_struct_time();
    };
    if ((((__sifr_checked_value_3.clone() != "-")
        || (__sifr_checked_value_4.clone() != "-"))
        || (__sifr_checked_value_5.clone() != "T"))
        || (__sifr_checked_value_6.clone() != ":"))
        || (__sifr_checked_value_7.clone() != ":")
    {
        return _invalid_struct_time();
    }
    let year: SifrInt = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, SifrInt::from_i64(0), SifrInt::from_i64(4))),
    );
    let month: SifrInt = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, SifrInt::from_i64(5), SifrInt::from_i64(7))),
    );
    let day: SifrInt = _int_or_negative_one(
        _parse_decimal(
            &_substring(rendered, SifrInt::from_i64(8), SifrInt::from_i64(10)),
        ),
    );
    let hour: SifrInt = _int_or_negative_one(
        _parse_decimal(
            &_substring(rendered, SifrInt::from_i64(11), SifrInt::from_i64(13)),
        ),
    );
    let minute: SifrInt = _int_or_negative_one(
        _parse_decimal(
            &_substring(rendered, SifrInt::from_i64(14), SifrInt::from_i64(16)),
        ),
    );
    let second: SifrInt = _int_or_negative_one(
        _parse_decimal(
            &_substring(rendered, SifrInt::from_i64(17), SifrInt::from_i64(19)),
        ),
    );
    if (((((&year < &SifrInt::from_i64(0)) || (&month < &SifrInt::from_i64(0)))
        || (&day < &SifrInt::from_i64(0))) || (&hour < &SifrInt::from_i64(0)))
        || (&minute < &SifrInt::from_i64(0))) || (&second < &SifrInt::from_i64(0))
    {
        return _invalid_struct_time();
    }
    if !_valid_date(year.clone(), month.clone(), day.clone()) {
        return _invalid_struct_time();
    }
    let wday: SifrInt = _weekday(year.clone(), month.clone(), day.clone());
    let yday: SifrInt = _day_of_year(year.clone(), month.clone(), day.clone());
    __SifrStdlib_sifr_x2etime_x2estruct__time::new(
        year.clone(),
        month.clone(),
        day.clone(),
        hour.clone(),
        minute.clone(),
        second.clone(),
        wday.clone(),
        yday.clone(),
        SifrInt::from_i64(0),
    )
}
fn gmtime_struct(epoch: f64) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let rendered: String = _gmtime_intrinsic(epoch);
    _to_struct_time(&rendered)
}
fn localtime_struct(epoch: f64) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let rendered: String = _localtime_intrinsic(epoch);
    _to_struct_time(&rendered)
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
fn demo_math() {
    println!("=== math new intrinsics ===");
    let e0: f64 = erf(0.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("erf near 0 = "); __sifr_concat.push_str(format!("{}", (e0
        < (0.001_f64)) && (e0 > - (0.001_f64))) .as_str()); __sifr_concat }
    );
    let ec0: f64 = erfc(0.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("erfc near 1 = "); __sifr_concat.push_str(format!("{}",
        (ec0 > (0.999_f64)) && (ec0 < (1.001_f64))) .as_str()); __sifr_concat }
    );
    let g: f64 = gamma(5.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("gamma(5) > 23 = "); __sifr_concat.push_str(format!("{}",
        g > (23.0_f64)) .as_str()); __sifr_concat }
    );
    let lg: f64 = lgamma(5.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("lgamma(5) > 3 = "); __sifr_concat.push_str(format!("{}",
        lg > (3.0_f64)) .as_str()); __sifr_concat }
    );
    let fp: Vec<f64> = frexp(8.0_f64);
    let mantissa: Option<f64> = {
        let __sifr_checked_read_collection = &fp;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(mantissa) = mantissa {
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(22usize +
            0usize); __sifr_concat.push_str("frexp(8.0) mantissa = "); __sifr_concat
            .push_str(format!("{}", mantissa) .as_str()); __sifr_concat }
        );
    }
    let ld: f64 = ldexp(0.5_f64, SifrInt::from_i64(4));
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("ldexp(0.5, 4) = "); __sifr_concat.push_str(format!("{}",
        ld) .as_str()); __sifr_concat }
    );
    let md: Vec<f64> = modf(3.7_f64);
    let frac: Option<f64> = {
        let __sifr_checked_read_collection = &md;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(frac) = frac {
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(21usize +
            0usize); __sifr_concat.push_str("modf(3.7) frac > 0 = "); __sifr_concat
            .push_str(format!("{}", frac > (0.0_f64)) .as_str()); __sifr_concat }
        );
    }
    let na: f64 = nextafter(1.0_f64, 2.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(28usize + 0usize);
        __sifr_concat.push_str("nextafter(1.0, 2.0) > 1.0 = "); __sifr_concat
        .push_str(format!("{}", na > (1.0_f64)) .as_str()); __sifr_concat }
    );
    let u: f64 = ulp(1.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(15usize + 0usize);
        __sifr_concat.push_str("ulp(1.0) > 0 = "); __sifr_concat.push_str(format!("{}", u
        > (0.0_f64)) .as_str()); __sifr_concat }
    );
}
fn demo_os() {
    println!("=== os new intrinsics ===");
    let pid: SifrInt = getpid();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(10usize + 0usize);
        __sifr_concat.push_str("pid > 0 = "); __sifr_concat.push_str(format!("{}", & pid
        > & SifrInt::from_i64(0)) .as_str()); __sifr_concat }
    );
    let cpus: SifrInt = cpu_count();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(17usize + 0usize);
        __sifr_concat.push_str("cpu_count >= 1 = "); __sifr_concat.push_str(format!("{}",
        & cpus >= & SifrInt::from_i64(1)) .as_str()); __sifr_concat }
    );
}
fn demo_hashlib() {
    println!("=== hashlib new intrinsics ===");
    let data: Vec<u8> = vec![104u8, 101u8, 108u8, 108u8, 111u8];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("sha224 len = "); __sifr_concat.push_str(format!("{}",
        SifrInt::from(sha224(& data).hexdigest().chars().count())) .as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("sha384 len = "); __sifr_concat.push_str(format!("{}",
        SifrInt::from(sha384(& data).hexdigest().chars().count())) .as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("blake2b len = "); __sifr_concat.push_str(format!("{}",
        SifrInt::from(blake2b(& data).hexdigest().chars().count())) .as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("blake2s len = "); __sifr_concat.push_str(format!("{}",
        SifrInt::from(blake2s(& data).hexdigest().chars().count())) .as_str());
        __sifr_concat }
    );
}
fn demo_platform() {
    println!("=== platform new intrinsics ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(17usize + 0usize);
        __sifr_concat.push_str("system len > 0 = "); __sifr_concat.push_str(format!("{}",
        & SifrInt::from(system().chars().count()) > & SifrInt::from_i64(0)) .as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(18usize + 0usize);
        __sifr_concat.push_str("machine len > 0 = "); __sifr_concat
        .push_str(format!("{}", & SifrInt::from(machine().chars().count()) > &
        SifrInt::from_i64(0)) .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("processor len > 0 = "); __sifr_concat
        .push_str(format!("{}", & SifrInt::from(processor().chars().count()) > &
        SifrInt::from_i64(0)) .as_str()); __sifr_concat }
    );
}
fn demo_time() {
    println!("=== time new intrinsics ===");
    let gmt: __SifrStdlib_sifr_x2etime_x2estruct__time = gmtime_struct(0.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("gmtime year = "); __sifr_concat.push_str(format!("{}", &
        gmt.tm_year.clone() == & SifrInt::from_i64(1970)) .as_str()); __sifr_concat }
    );
    let lt: __SifrStdlib_sifr_x2etime_x2estruct__time = localtime_struct(0.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(22usize + 0usize);
        __sifr_concat.push_str("localtime yday >= 1 = "); __sifr_concat
        .push_str(format!("{}", & lt.tm_yday.clone() >= & SifrInt::from_i64(1))
        .as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let parsed: String = strptime(
            &"2024-01-15 10:30:00".to_string(),
            &"%Y-%m-%d %H:%M:%S".to_string(),
        )?;
        let __sifr_chars_parsed: Vec<char> = parsed.chars().collect::<Vec<char>>();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(14usize +
            0usize); __sifr_concat.push_str("strptime ok = "); __sifr_concat
            .push_str(format!("{}", SifrInt::from(parsed.chars().count()) >
            SifrInt::from_i64(0)) .as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(16usize +
            0usize); __sifr_concat.push_str("strptime error: "); __sifr_concat.push_str(e
            .message.clone().as_str()); __sifr_concat }
        );
    }
}
fn demo_base64() {
    println!("=== base64 new intrinsics ===");
    let encoded: String = b32encode(&"hello world".to_string());
    let __sifr_chars_encoded: Vec<char> = encoded.chars().collect::<Vec<char>>();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("b32encode len > 0 = "); __sifr_concat
        .push_str(format!("{}", & SifrInt::from(encoded.chars().count()) > &
        SifrInt::from_i64(0)) .as_str()); __sifr_concat }
    );
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b32decode(&encoded)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(12usize +
            decoded.len()); __sifr_concat.push_str("b32decode = "); __sifr_concat
            .push_str(decoded.as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
            0usize); __sifr_concat.push_str("b32decode error: "); __sifr_concat
            .push_str(e.message.clone().as_str()); __sifr_concat }
        );
    }
}
fn demo_shutil() {
    println!("=== shutil new intrinsics ===");
    let usage: Vec<SifrInt> = disk_usage(&"/".to_string());
    let total: Option<SifrInt> = {
        let __sifr_checked_read_collection = &usage;
        let __sifr_checked_read_index = SifrInt::from_i64(0);
        let __sifr_checked_read_normalized = __sifr_checked_read_index
            .normalize_index_or_len(__sifr_checked_read_collection.len());
        __sifr_checked_read_collection.get(__sifr_checked_read_normalized).cloned()
    };
    if let Some(total) = total.clone() {
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
            0usize); __sifr_concat.push_str("disk_total > 0 = "); __sifr_concat
            .push_str(format!("{}", & total > & SifrInt::from_i64(0)) .as_str());
            __sifr_concat }
        );
    }
}
fn main() {
    demo_math();
    demo_os();
    demo_hashlib();
    demo_platform();
    demo_time();
    demo_base64();
    demo_shutil();
}
