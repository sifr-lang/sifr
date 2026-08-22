// src/main.rs
mod __sifr_project_nominals {
    pub fn time_now() -> f64 {
        ::sifr_stdlib::time::time_now()
    }
    pub fn time_format(epoch: f64, fmt: &String) -> String {
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
    pub fn strptime(s: &String, fmt: &String) -> Result<String, ValueError> {
        ::sifr_stdlib::time::strptime(s, fmt)
            .map(|__sifr_bridge_ok| __sifr_bridge_ok)
            .map_err(|__sifr_bridge_error| ValueError {
                message: __sifr_bridge_error.to_string(),
            })
    }
    pub fn _strptime_intrinsic(s: &String, fmt: &String) -> Result<String, ValueError> {
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
    pub fn time_strptime(s: &String, fmt: &String) -> Result<Vec<i64>, ValueError> {
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
    pub fn time_gmtime() -> Vec<i64> {
        ::sifr_stdlib::time::time_gmtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
    }
    pub fn time_localtime() -> Vec<i64> {
        ::sifr_stdlib::time::time_localtime()
            .into_iter()
            .map(|__sifr_bridge_value| __sifr_bridge_value.to_i64_saturating())
            .collect()
    }
    #[derive(Debug, Clone)]
    pub struct __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub tm_year: i64,
        pub tm_mon: i64,
        pub tm_mday: i64,
        pub tm_hour: i64,
        pub tm_min: i64,
        pub tm_sec: i64,
        pub tm_wday: i64,
        pub tm_yday: i64,
        pub tm_isdst: i64,
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn new(
            tm_year: i64,
            tm_mon: i64,
            tm_mday: i64,
            tm_hour: i64,
            tm_min: i64,
            tm_sec: i64,
            tm_wday: i64,
            tm_yday: i64,
            tm_isdst: i64,
        ) -> Self {
            let __sifr_field_init_0: i64 = tm_year;
            let __sifr_field_init_1: i64 = tm_mon;
            let __sifr_field_init_2: i64 = tm_mday;
            let __sifr_field_init_3: i64 = tm_hour;
            let __sifr_field_init_4: i64 = tm_min;
            let __sifr_field_init_5: i64 = tm_sec;
            let __sifr_field_init_6: i64 = tm_wday;
            let __sifr_field_init_7: i64 = tm_yday;
            let __sifr_field_init_8: i64 = tm_isdst;
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
        pub fn as_tuple(&self) -> (i64, i64, i64, i64, i64, i64, i64, i64, i64) {
            (
                self.tm_year,
                self.tm_mon,
                self.tm_mday,
                self.tm_hour,
                self.tm_min,
                self.tm_sec,
                self.tm_wday,
                self.tm_yday,
                self.tm_isdst,
            )
        }
    }
    impl __SifrStdlib_sifr_x2etime_x2estruct__time {
        pub fn isoformat(&self) -> String {
            let y: String = format!("{}", self.tm_year);
            let mut mo: String = format!("{}", self.tm_mon);
            if ((mo.chars().count() as i64) < (2_i64)) {
                mo = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mo).as_str());
                    __sifr_concat
                };
            }
            let mut d: String = format!("{}", self.tm_mday);
            if ((d.chars().count() as i64) < (2_i64)) {
                d = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((d).as_str());
                    __sifr_concat
                };
            }
            let mut h: String = format!("{}", self.tm_hour);
            if ((h.chars().count() as i64) < (2_i64)) {
                h = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((h).as_str());
                    __sifr_concat
                };
            }
            let mut mi: String = format!("{}", self.tm_min);
            if ((mi.chars().count() as i64) < (2_i64)) {
                mi = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((mi).as_str());
                    __sifr_concat
                };
            }
            let mut s: String = format!("{}", self.tm_sec);
            if ((s.chars().count() as i64) < (2_i64)) {
                s = {
                    let mut __sifr_concat: String = String::with_capacity(1usize + s.len());
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((s).as_str());
                    __sifr_concat
                };
            }
            {
                let mut __sifr_concat: String = String::with_capacity(
                    (((((((((y.len() + 1usize) + mo.len()) + 1usize) + d.len()) + 1usize)
                        + h.len()) + 1usize) + mi.len()) + 1usize) + s.len(),
                );
                __sifr_concat.push_str((y).as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str((mo).as_str());
                __sifr_concat.push('-');
                __sifr_concat.push_str((d).as_str());
                __sifr_concat.push('T');
                __sifr_concat.push_str((h).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((mi).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((s).as_str());
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _algorithm: String,
    _data: Vec<u8>,
    name: String,
    digest_size: i64,
    block_size: i64,
}
impl __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    fn new(
        algorithm: String,
        data: Vec<u8>,
        name: String,
        digest_size: i64,
        block_size: i64,
    ) -> Self {
        let __sifr_field_init_0: String = algorithm;
        let __sifr_field_init_1: Vec<u8> = data;
        let __sifr_field_init_2: String = name;
        let __sifr_field_init_3: i64 = digest_size;
        let __sifr_field_init_4: i64 = block_size;
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
    fn update(&mut self, data: &Vec<u8>) {
        self._data = {
            let mut __v = (self._data.clone()).clone();
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
    algorithm: &String,
    data: &Vec<u8>,
) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    let alg: String = algorithm.to_lowercase();
    if alg == "md5" {
        return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
            alg,
            (data).clone(),
            "md5".to_string(),
            16_i64,
            64_i64,
        );
    } else {
        if alg == "sha1" {
            return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                alg,
                (data).clone(),
                "sha1".to_string(),
                20_i64,
                64_i64,
            );
        } else {
            if alg == "sha224" {
                return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                    alg,
                    (data).clone(),
                    "sha224".to_string(),
                    28_i64,
                    64_i64,
                );
            } else {
                if alg == "sha256" {
                    return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                        alg,
                        (data).clone(),
                        "sha256".to_string(),
                        32_i64,
                        64_i64,
                    );
                } else {
                    if alg == "sha384" {
                        return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                            alg,
                            (data).clone(),
                            "sha384".to_string(),
                            48_i64,
                            128_i64,
                        );
                    } else {
                        if alg == "sha512" {
                            return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                alg,
                                (data).clone(),
                                "sha512".to_string(),
                                64_i64,
                                128_i64,
                            );
                        } else {
                            if alg == "blake2b" {
                                return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                    alg,
                                    (data).clone(),
                                    "blake2b".to_string(),
                                    64_i64,
                                    128_i64,
                                );
                            } else {
                                if alg == "blake2s" {
                                    return __SifrStdlib_sifr_x2ehashlib_x2eHashObject::new(
                                        alg,
                                        (data).clone(),
                                        "blake2s".to_string(),
                                        32_i64,
                                        64_i64,
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
        (data).clone(),
        "unknown".to_string(),
        0_i64,
        0_i64,
    )
}
fn _hash_bytes(algorithm: &String, data: &Vec<u8>) -> Vec<u8> {
    if (algorithm).as_str() == "md5" {
        return md5_bytes(data);
    } else {
        if (algorithm).as_str() == "sha1" {
            return sha1_bytes(data);
        } else {
            if (algorithm).as_str() == "sha224" {
                return sha224_bytes(data);
            } else {
                if (algorithm).as_str() == "sha256" {
                    return sha256_bytes(data);
                } else {
                    if (algorithm).as_str() == "sha384" {
                        return sha384_bytes(data);
                    } else {
                        if (algorithm).as_str() == "sha512" {
                            return sha512_bytes(data);
                        } else {
                            if (algorithm).as_str() == "blake2b" {
                                return blake2b_bytes(data);
                            } else {
                                if (algorithm).as_str() == "blake2s" {
                                    return blake2s_bytes(data);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    vec![]
}
fn _hash_hex(algorithm: &String, data: &Vec<u8>) -> String {
    {
        let __bytes_receiver = &_hash_bytes(algorithm, data);
        let mut __hex = String::with_capacity(__bytes_receiver.len().saturating_mul(2));
        for __byte in __bytes_receiver.iter() {
            __hex.push_str(&format!("{:02x}", * __byte));
        }
        __hex
    }
}
fn sha224(data: &Vec<u8>) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _build_hash(&"sha224".to_string(), data)
}
fn sha384(data: &Vec<u8>) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _build_hash(&"sha384".to_string(), data)
}
fn blake2b(data: &Vec<u8>) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
    _build_hash(&"blake2b".to_string(), data)
}
fn blake2s(data: &Vec<u8>) -> __SifrStdlib_sifr_x2ehashlib_x2eHashObject {
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
fn _is_leap_year(year: i64) -> bool {
    (((year % (4_i64)) == (0_i64)) && ((year % (100_i64)) != (0_i64)))
        || ((year % (400_i64)) == (0_i64))
}
fn _days_in_year(year: i64) -> i64 {
    if _is_leap_year(year) {
        return 366_i64;
    }
    365_i64
}
fn _days_in_month(year: i64, month: i64) -> i64 {
    let month_days: Vec<i64> = vec![
        31_i64, 28_i64, 31_i64, 30_i64, 31_i64, 30_i64, 31_i64, 31_i64, 30_i64, 31_i64,
        30_i64, 31_i64
    ];
    let idx: i64 = month - (1_i64);
    let d: Option<i64> = {
        let __sifr_index_list = &month_days;
        let __sifr_index_i = idx;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if (month == (2_i64)) && _is_leap_year(year) {
        return 29_i64;
    }
    if let Some(d) = d {
        return d;
    }
    0_i64
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        if let Some(ch) = ch {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
}
fn _digit_value(ch: &String) -> Option<i64> {
    if (ch).as_str() == "0" {
        return Some(0_i64);
    }
    if (ch).as_str() == "1" {
        return Some(1_i64);
    }
    if (ch).as_str() == "2" {
        return Some(2_i64);
    }
    if (ch).as_str() == "3" {
        return Some(3_i64);
    }
    if (ch).as_str() == "4" {
        return Some(4_i64);
    }
    if (ch).as_str() == "5" {
        return Some(5_i64);
    }
    if (ch).as_str() == "6" {
        return Some(6_i64);
    }
    if (ch).as_str() == "7" {
        return Some(7_i64);
    }
    if (ch).as_str() == "8" {
        return Some(8_i64);
    }
    if (ch).as_str() == "9" {
        return Some(9_i64);
    }
    None
}
fn _parse_decimal(text: &String) -> Option<i64> {
    let __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if ((__sifr_chars_text.len() as i64) == (0_i64)) {
        return None;
    }
    let mut out: i64 = 0_i64;
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
        let Some(ch_opt) = ch_opt else {
            return None;
        };
        let ch: String = ch_opt;
        let digit_opt: Option<i64> = _digit_value(&ch);
        let Some(digit_opt) = digit_opt else {
            return None;
        };
        let digit: i64 = digit_opt;
        out = (out * (10_i64)) + digit;
        i += 1_i64;
    }
    Some(out)
}
fn _int_or_negative_one(value: Option<i64>) -> i64 {
    let Some(value) = value else {
        return -(1_i64);
    };
    value
}
fn _day_of_year(year: i64, month: i64, day: i64) -> i64 {
    let mut yday: i64 = 0_i64;
    let mut m: i64 = 1_i64;
    while m < month {
        yday += _days_in_month(year, m);
        m += 1_i64;
    }
    yday + day
}
fn _weekday(year: i64, month: i64, day: i64) -> i64 {
    let mut days_since_epoch: i64 = 0_i64;
    if year >= (1970_i64) {
        let mut y: i64 = 1970_i64;
        while y < year {
            days_since_epoch += _days_in_year(y);
            y += 1_i64;
        }
    } else {
        let mut y: i64 = 1969_i64;
        while y >= year {
            days_since_epoch -= _days_in_year(y);
            y -= 1_i64;
        }
    }
    let mut m: i64 = 1_i64;
    while m < month {
        days_since_epoch += _days_in_month(year, m);
        m += 1_i64;
    }
    days_since_epoch = (days_since_epoch + day) - (1_i64);
    let mut wd: i64 = ((3_i64) + days_since_epoch) % (7_i64);
    if wd < (0_i64) {
        wd += 7_i64;
    }
    wd
}
fn _valid_date(year: i64, month: i64, day: i64) -> bool {
    if year <= (0_i64) {
        return false;
    }
    if (month < (1_i64)) || (month > (12_i64)) {
        return false;
    }
    let max_day: i64 = _days_in_month(year, month);
    (day >= (1_i64)) && (day <= max_day)
}
fn _invalid_struct_time() -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    __SifrStdlib_sifr_x2etime_x2estruct__time::new(
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
        0_i64,
    )
}
fn _to_struct_time(rendered: &String) -> __SifrStdlib_sifr_x2etime_x2estruct__time {
    let __sifr_chars_rendered: Vec<char> = rendered.chars().collect::<Vec<char>>();
    if ((__sifr_chars_rendered.len() as i64) < (19_i64)) {
        return _invalid_struct_time();
    }
    if ((((({
        let Some(__indexed_char) = __sifr_chars_rendered
            .get((4_i64) as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    }) != "-")
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get((7_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "-"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get((10_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "T"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get((13_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":"))
        || (({
            let Some(__indexed_char) = __sifr_chars_rendered
                .get((16_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":")
    {
        return _invalid_struct_time();
    }
    let year: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 0_i64, 4_i64)),
    );
    let month: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 5_i64, 7_i64)),
    );
    let day: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 8_i64, 10_i64)),
    );
    let hour: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 11_i64, 13_i64)),
    );
    let minute: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 14_i64, 16_i64)),
    );
    let second: i64 = _int_or_negative_one(
        _parse_decimal(&_substring(rendered, 17_i64, 19_i64)),
    );
    if (((((year < (0_i64)) || (month < (0_i64))) || (day < (0_i64)))
        || (hour < (0_i64))) || (minute < (0_i64))) || (second < (0_i64))
    {
        return _invalid_struct_time();
    }
    if !(_valid_date(year, month, day)) {
        return _invalid_struct_time();
    }
    let wday: i64 = _weekday(year, month, day);
    let yday: i64 = _day_of_year(year, month, day);
    __SifrStdlib_sifr_x2etime_x2estruct__time::new(
        year,
        month,
        day,
        hour,
        minute,
        second,
        wday,
        yday,
        0_i64,
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
        __sifr_concat.push_str("erf near 0 = "); __sifr_concat.push_str((format!("{}",
        (e0 < (0.001_f64)) && (e0 > - (0.001_f64)))).as_str()); __sifr_concat }
    );
    let ec0: f64 = erfc(0.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("erfc near 1 = "); __sifr_concat.push_str((format!("{}",
        (ec0 > (0.999_f64)) && (ec0 < (1.001_f64)))).as_str()); __sifr_concat }
    );
    let g: f64 = gamma(5.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("gamma(5) > 23 = "); __sifr_concat.push_str((format!("{}",
        g > (23.0_f64))).as_str()); __sifr_concat }
    );
    let lg: f64 = lgamma(5.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("lgamma(5) > 3 = "); __sifr_concat.push_str((format!("{}",
        lg > (3.0_f64))).as_str()); __sifr_concat }
    );
    let fp: Vec<f64> = frexp(8.0_f64);
    let mantissa: Option<f64> = {
        let __sifr_index_list = &fp;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(mantissa) = mantissa {
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(22usize +
            0usize); __sifr_concat.push_str("frexp(8.0) mantissa = "); __sifr_concat
            .push_str((format!("{}", mantissa)).as_str()); __sifr_concat }
        );
    }
    let ld: f64 = ldexp(0.5_f64, 4_i64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(16usize + 0usize);
        __sifr_concat.push_str("ldexp(0.5, 4) = "); __sifr_concat.push_str((format!("{}",
        ld)).as_str()); __sifr_concat }
    );
    let md: Vec<f64> = modf(3.7_f64);
    let frac: Option<f64> = {
        let __sifr_index_list = &md;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(frac) = frac {
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(21usize +
            0usize); __sifr_concat.push_str("modf(3.7) frac > 0 = "); __sifr_concat
            .push_str((format!("{}", frac > (0.0_f64))).as_str()); __sifr_concat }
        );
    }
    let na: f64 = nextafter(1.0_f64, 2.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(28usize + 0usize);
        __sifr_concat.push_str("nextafter(1.0, 2.0) > 1.0 = "); __sifr_concat
        .push_str((format!("{}", na > (1.0_f64))).as_str()); __sifr_concat }
    );
    let u: f64 = ulp(1.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(15usize + 0usize);
        __sifr_concat.push_str("ulp(1.0) > 0 = "); __sifr_concat.push_str((format!("{}",
        u > (0.0_f64))).as_str()); __sifr_concat }
    );
}
fn demo_os() {
    println!("=== os new intrinsics ===");
    let pid: i64 = getpid();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(10usize + 0usize);
        __sifr_concat.push_str("pid > 0 = "); __sifr_concat.push_str((format!("{}", pid >
        (0_i64))).as_str()); __sifr_concat }
    );
    let cpus: i64 = cpu_count();
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(17usize + 0usize);
        __sifr_concat.push_str("cpu_count >= 1 = "); __sifr_concat
        .push_str((format!("{}", cpus >= (1_i64))).as_str()); __sifr_concat }
    );
}
fn demo_hashlib() {
    println!("=== hashlib new intrinsics ===");
    let data: Vec<u8> = vec![
        (104_i64) as u8, (101_i64) as u8, (108_i64) as u8, (108_i64) as u8, (111_i64) as
        u8
    ];
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("sha224 len = "); __sifr_concat.push_str((format!("{}",
        sha224(& data).hexdigest().chars().count() as i64)).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(13usize + 0usize);
        __sifr_concat.push_str("sha384 len = "); __sifr_concat.push_str((format!("{}",
        sha384(& data).hexdigest().chars().count() as i64)).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("blake2b len = "); __sifr_concat.push_str((format!("{}",
        blake2b(& data).hexdigest().chars().count() as i64)).as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("blake2s len = "); __sifr_concat.push_str((format!("{}",
        blake2s(& data).hexdigest().chars().count() as i64)).as_str()); __sifr_concat }
    );
}
fn demo_platform() {
    println!("=== platform new intrinsics ===");
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(17usize + 0usize);
        __sifr_concat.push_str("system len > 0 = "); __sifr_concat
        .push_str((format!("{}", (system().chars().count() as i64) > (0_i64))).as_str());
        __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(18usize + 0usize);
        __sifr_concat.push_str("machine len > 0 = "); __sifr_concat
        .push_str((format!("{}", (machine().chars().count() as i64) > (0_i64)))
        .as_str()); __sifr_concat }
    );
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(20usize + 0usize);
        __sifr_concat.push_str("processor len > 0 = "); __sifr_concat
        .push_str((format!("{}", (processor().chars().count() as i64) > (0_i64)))
        .as_str()); __sifr_concat }
    );
}
fn demo_time() {
    println!("=== time new intrinsics ===");
    let gmt: __SifrStdlib_sifr_x2etime_x2estruct__time = gmtime_struct(0.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(14usize + 0usize);
        __sifr_concat.push_str("gmtime year = "); __sifr_concat.push_str((format!("{}",
        gmt.tm_year == (1970_i64))).as_str()); __sifr_concat }
    );
    let lt: __SifrStdlib_sifr_x2etime_x2estruct__time = localtime_struct(0.0_f64);
    println!(
        "{}", { let mut __sifr_concat : String = String::with_capacity(22usize + 0usize);
        __sifr_concat.push_str("localtime yday >= 1 = "); __sifr_concat
        .push_str((format!("{}", lt.tm_yday >= (1_i64))).as_str()); __sifr_concat }
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
            .push_str((format!("{}", (parsed.chars().count() as i64) > (0_i64)))
            .as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(16usize +
            0usize); __sifr_concat.push_str("strptime error: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
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
        .push_str((format!("{}", (encoded.chars().count() as i64) > (0_i64))).as_str());
        __sifr_concat }
    );
    let __sifr_try_res: Result<(), ParseError> = (|| {
        let decoded: String = b32decode(&encoded)?;
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(12usize +
            decoded.len()); __sifr_concat.push_str("b32decode = "); __sifr_concat
            .push_str((decoded).as_str()); __sifr_concat }
        );
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
            0usize); __sifr_concat.push_str("b32decode error: "); __sifr_concat
            .push_str((e.message.clone()).as_str()); __sifr_concat }
        );
    }
}
fn demo_shutil() {
    println!("=== shutil new intrinsics ===");
    let usage: Vec<i64> = disk_usage(&"/".to_string());
    let total: Option<i64> = {
        let __sifr_index_list = &usage;
        let __sifr_index_i = 0_i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).copied()
    };
    if let Some(total) = total {
        println!(
            "{}", { let mut __sifr_concat : String = String::with_capacity(17usize +
            0usize); __sifr_concat.push_str("disk_total > 0 = "); __sifr_concat
            .push_str((format!("{}", total > (0_i64))).as_str()); __sifr_concat }
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
