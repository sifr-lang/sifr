// src/main.rs
use std::collections::HashMap;

use sifr_runtime::SifrInt;

use std::sync::Mutex;

// --- stdlib: sifr.calendar ---
fn isleap(year: i64) -> bool {
    {
        let __y = year;
        (((__y % 4) == 0) && ((__y % 100) != 0)) || ((__y % 400) == 0)
    }
}
fn weekday(year: i64, month: i64, day: i64) -> i64 {
    {
        let __y0 = year;
        let __additional_modules = month;
        let __d0 = day;
        {
            let __t = vec![0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
            let __y = if __additional_modules < 3 { __y0 - 1 } else { __y0 };
            let __wd_raw = ((((((__y + (__y / 4)) - (__y / 100)) + (__y / 400))
                + __t[(__additional_modules - 1) as usize]) + __d0) % 7) + 6;
            __wd_raw % 7
        }
    }
}
fn monthrange(year: i64, month: i64) -> Vec<i64> {
    {
        let __y = year;
        let __m = month;
        let __days = if ((((((__m == 1) || (__m == 3)) || (__m == 5)) || (__m == 7))
            || (__m == 8)) || (__m == 10)) || (__m == 12)
        {
            31
        } else {
            if (((__m == 4) || (__m == 6)) || (__m == 9)) || (__m == 11) {
                30
            } else {
                if __m == 2 {
                    if (((__y % 4) == 0) && ((__y % 100) != 0)) || ((__y % 400) == 0) {
                        29
                    } else {
                        28
                    }
                } else {
                    30
                }
            }
        };
        let __wd = {
            let __t = vec![0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
            let __y = if __m < 3 { __y - 1 } else { __y };
            let __wd_raw = ((((((__y + (__y / 4)) - (__y / 100)) + (__y / 400))
                + __t[(__m - 1) as usize]) + 1) % 7) + 6;
            __wd_raw % 7
        };
        vec![__wd, __days]
    }
}

// --- stdlib: sifr.encoding ---
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DecodeError {
    message: String,
}
impl DecodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for DecodeError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EncodeError {
    message: String,
}
impl EncodeError {
    fn new(message: String) -> Self {
        Self { message }
    }
}
impl std::fmt::Display for EncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for EncodeError {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Encoding {
    label: String,
}
impl Encoding {
    fn new(label: String) -> Self {
        Self {
            label: {
                let mut __sifr_concat: String = String::with_capacity(
                    label.len() + 0usize,
                );
                __sifr_concat.push_str((label).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
        }
    }
    fn canonical_label(&self) -> Result<String, DecodeError> {
        sifr_runtime::encoding::canonical_label(&self.label.clone())
            .map_err(|__message| DecodeError { message: __message })
    }
    fn is_supported(&self) -> bool {
        sifr_runtime::encoding::is_supported_encoding(&self.label.clone())
    }
}
impl std::fmt::Display for Encoding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Encoding(label={})", self.label)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct DecodeErrorHandler {
    name: String,
}
impl DecodeErrorHandler {
    fn new(name: String) -> Self {
        Self {
            name: {
                let mut __sifr_concat: String = String::with_capacity(
                    name.len() + 0usize,
                );
                __sifr_concat.push_str((name).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
        }
    }
}
impl std::fmt::Display for DecodeErrorHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DecodeErrorHandler(name={})", self.name)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EncodeErrorHandler {
    name: String,
}
impl EncodeErrorHandler {
    fn new(name: String) -> Self {
        Self {
            name: {
                let mut __sifr_concat: String = String::with_capacity(
                    name.len() + 0usize,
                );
                __sifr_concat.push_str((name).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
        }
    }
}
impl std::fmt::Display for EncodeErrorHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EncodeErrorHandler(name={})", self.name)
    }
}
#[derive(Debug, Clone, PartialEq)]
struct DecodeOutcome {
    text: String,
    recoveries: Vec<String>,
}
impl DecodeOutcome {
    fn new(text: String, recoveries: Vec<String>) -> Self {
        Self {
            text: {
                let mut __sifr_concat: String = String::with_capacity(
                    text.len() + 0usize,
                );
                __sifr_concat.push_str((text).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
            recoveries,
        }
    }
    fn get_text(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self.text.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    fn get_recoveries(&self) -> Vec<String> {
        self.recoveries.clone()
    }
}
#[derive(Debug, Clone, PartialEq)]
struct EncodeOutcome {
    data: Vec<u8>,
    recoveries: Vec<String>,
}
impl EncodeOutcome {
    fn new(data: Vec<u8>, recoveries: Vec<String>) -> Self {
        Self { data, recoveries }
    }
    fn get_data(&self) -> Vec<u8> {
        self.data.clone()
    }
    fn get_recoveries(&self) -> Vec<String> {
        self.recoveries.clone()
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Decoder {
    _encoding: Encoding,
    _errors: DecodeErrorHandler,
    _exhausted: bool,
    _pending: Vec<u8>,
}
impl Decoder {
    fn new(enc: Encoding, errors: Option<DecodeErrorHandler>) -> Self {
        Self {
            _encoding: enc,
            _errors: _decode_handler_or_strict(&errors),
            _exhausted: false,
            _pending: vec![],
        }
    }
    fn decode(
        &mut self,
        data: &Vec<u8>,
        r#final: bool,
    ) -> Result<DecodeOutcome, DecodeError> {
        if self._exhausted {
            return Err(DecodeError::new("decoder is exhausted".to_string()));
        }
        let __sifr_try_res: Result<Result<DecodeOutcome, DecodeError>, DecodeError> = (||
        {
            let outcome: DecodeOutcome = sifr_runtime::encoding::incremental_decode_with_recoveries(
                    &data,
                    &self._pending.clone(),
                    &self._encoding.clone().label,
                    &self._errors.clone().name,
                    r#final,
                )
                .map(|__parts| DecodeOutcome {
                    text: __parts.0,
                    recoveries: __parts.1,
                })
                .map_err(|__message| DecodeError { message: __message })?;
            let next_pending: Vec<u8> = sifr_runtime::encoding::incremental_decode_pending(
                    &data,
                    &self._pending.clone(),
                    &self._encoding.clone().label,
                    r#final,
                )
                .map_err(|__message| DecodeError { message: __message })?;
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
                return Err(DecodeError::new(e.message));
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Encoder {
    _encoding: Encoding,
    _errors: EncodeErrorHandler,
    _exhausted: bool,
}
impl Encoder {
    fn new(enc: Encoding, errors: Option<EncodeErrorHandler>) -> Self {
        Self {
            _encoding: enc,
            _errors: _encode_handler_or_strict(&errors),
            _exhausted: false,
        }
    }
    fn encode(
        &mut self,
        text: &String,
        r#final: bool,
    ) -> Result<EncodeOutcome, EncodeError> {
        if self._exhausted {
            return Err(EncodeError::new("encoder is exhausted".to_string()));
        }
        let __sifr_try_res: Result<Result<EncodeOutcome, EncodeError>, EncodeError> = (||
        {
            let outcome: EncodeOutcome = encode_outcome(
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
                return Err(EncodeError::new(e.message));
            }
        }
    }
}
impl std::fmt::Display for Encoder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "Encoder(_encoding={}, _errors={}, _exhausted={})", self._encoding, self
            ._errors, self._exhausted
        )
    }
}
fn encoding(label: &String) -> Encoding {
    Encoding::new((label).clone())
}
fn utf8() -> Encoding {
    Encoding::new(__const_ENCODING_UTF8())
}
fn utf8_sig() -> Encoding {
    Encoding::new(__const_ENCODING_UTF8_SIG())
}
fn ascii() -> Encoding {
    Encoding::new(__const_ENCODING_ASCII())
}
fn latin1() -> Encoding {
    Encoding::new(__const_ENCODING_LATIN1())
}
fn utf16_le() -> Encoding {
    Encoding::new(__const_ENCODING_UTF16_LE())
}
fn utf16_be() -> Encoding {
    Encoding::new(__const_ENCODING_UTF16_BE())
}
fn windows1252() -> Encoding {
    Encoding::new(__const_ENCODING_WINDOWS_1252())
}
fn strict_decode_handler() -> DecodeErrorHandler {
    DecodeErrorHandler::new(__const_DECODE_ERRORS_STRICT())
}
fn replace_decode_handler() -> DecodeErrorHandler {
    DecodeErrorHandler::new(__const_DECODE_ERRORS_REPLACE())
}
fn ignore_decode_handler() -> DecodeErrorHandler {
    DecodeErrorHandler::new(__const_DECODE_ERRORS_IGNORE())
}
fn backslash_replace_decode_handler() -> DecodeErrorHandler {
    DecodeErrorHandler::new(__const_DECODE_ERRORS_BACKSLASH_REPLACE())
}
fn strict_encode_handler() -> EncodeErrorHandler {
    EncodeErrorHandler::new(__const_ENCODE_ERRORS_STRICT())
}
fn replace_encode_handler() -> EncodeErrorHandler {
    EncodeErrorHandler::new(__const_ENCODE_ERRORS_REPLACE())
}
fn ignore_encode_handler() -> EncodeErrorHandler {
    EncodeErrorHandler::new(__const_ENCODE_ERRORS_IGNORE())
}
fn backslash_replace_encode_handler() -> EncodeErrorHandler {
    EncodeErrorHandler::new(__const_ENCODE_ERRORS_BACKSLASH_REPLACE())
}
fn xmlcharref_replace_encode_handler() -> EncodeErrorHandler {
    EncodeErrorHandler::new(__const_ENCODE_ERRORS_XMLCHARREF_REPLACE())
}
fn name_replace_encode_handler() -> EncodeErrorHandler {
    EncodeErrorHandler::new(__const_ENCODE_ERRORS_NAME_REPLACE())
}
fn _decode_handler_name(errors: &Option<DecodeErrorHandler>) -> String {
    if let Some(errors) = errors.as_ref() {
        return {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((errors.name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    __const_DECODE_ERRORS_STRICT()
}
fn _encode_handler_name(errors: &Option<EncodeErrorHandler>) -> String {
    if let Some(errors) = errors.as_ref() {
        return {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((errors.name).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    __const_ENCODE_ERRORS_STRICT()
}
fn _decode_handler_or_strict(errors: &Option<DecodeErrorHandler>) -> DecodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return DecodeErrorHandler::new(format!("{}{}", errors.name, ""));
    }
    strict_decode_handler()
}
fn _encode_handler_or_strict(errors: &Option<EncodeErrorHandler>) -> EncodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return EncodeErrorHandler::new(format!("{}{}", errors.name, ""));
    }
    strict_encode_handler()
}
fn decode_outcome(
    data: &Vec<u8>,
    enc: &Encoding,
    errors: &Option<DecodeErrorHandler>,
) -> Result<DecodeOutcome, DecodeError> {
    let handler_name: String = _decode_handler_name(errors);
    let __sifr_try_res: Result<Result<DecodeOutcome, DecodeError>, DecodeError> = (|| {
        return Ok(
            sifr_runtime::encoding::decode_with_recoveries(
                    &data,
                    &enc.label,
                    &handler_name,
                )
                .map(|__parts| DecodeOutcome {
                    text: __parts.0,
                    recoveries: __parts.1,
                })
                .map_err(|__message| DecodeError { message: __message }),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(DecodeError::new(e.message));
        }
    }
}
fn decode(
    data: &Vec<u8>,
    enc: &Encoding,
    errors: &Option<DecodeErrorHandler>,
) -> Result<String, DecodeError> {
    let __sifr_try_res: Result<Result<String, DecodeError>, DecodeError> = (|| {
        let mut outcome: DecodeOutcome = decode_outcome(data, enc, errors)?;
        return Ok(Ok(outcome.get_text()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(DecodeError::new(e.message));
        }
    }
}
fn encode_outcome(
    text: &String,
    enc: &Encoding,
    errors: &Option<EncodeErrorHandler>,
) -> Result<EncodeOutcome, EncodeError> {
    let handler_name: String = _encode_handler_name(errors);
    let __sifr_try_res: Result<Result<EncodeOutcome, EncodeError>, EncodeError> = (|| {
        return Ok(
            sifr_runtime::encoding::encode_with_recoveries(
                    &text,
                    &enc.label,
                    &handler_name,
                )
                .map(|__parts| EncodeOutcome {
                    data: __parts.0,
                    recoveries: __parts.1,
                })
                .map_err(|__message| EncodeError { message: __message }),
        );
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(EncodeError::new(e.message));
        }
    }
}
fn encode(
    text: &String,
    enc: &Encoding,
    errors: &Option<EncodeErrorHandler>,
) -> Result<Vec<u8>, EncodeError> {
    let __sifr_try_res: Result<Result<Vec<u8>, EncodeError>, EncodeError> = (|| {
        let mut outcome: EncodeOutcome = encode_outcome(text, enc, errors)?;
        return Ok(Ok(outcome.get_data()));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(EncodeError::new(e.message));
        }
    }
}

// --- stdlib: sifr.io ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IOBase {
    _closed: bool,
}
impl IOBase {
    fn new() -> Self {
        Self { _closed: false }
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        self._closed
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
    fn tell(&self) -> Result<i64, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
    fn readable(&self) -> bool {
        false
    }
    fn writable(&self) -> bool {
        false
    }
    fn seekable(&self) -> bool {
        false
    }
}
impl std::fmt::Display for IOBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "IOBase(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextIOBase {
    iobase: IOBase,
}
impl TextIOBase {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BinaryIOBase {
    iobase: IOBase,
}
impl BinaryIOBase {}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileHandle {
    _handle: i64,
    _mode: String,
    _closed: bool,
}
impl FileHandle {
    fn new(handle: i64, mode: String) -> Self {
        Self {
            _handle: handle,
            _mode: mode,
            _closed: false,
        }
    }
    fn close(&mut self) {
        if self._closed {
            return;
        }
        {
            let __hid = self._handle;
            __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner())
                .remove(&__hid);
            ()
        };
        self._closed = true;
    }
    fn closed(&self) -> bool {
        self._closed
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
    fn read(&self) -> Result<String, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::TextRead(ref mut __r)) => {
                    let mut __s = String::new();
                    std::io::Read::read_to_string(__r, &mut __s).map_err(__io_err)?;
                    return Ok(__s);
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()
    }
    fn write(&self, data: &String) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::TextWrite(ref mut __w)) => {
                    let __data = data.as_str();
                    std::io::Write::write_all(__w, __data.as_bytes()).map_err(__io_err)?;
                    return Ok(());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for writing".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()
    }
    fn readline(&self) -> Result<Option<String>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::TextRead(ref mut __r)) => {
                    let mut __line = String::new();
                    let __n = std::io::BufRead::read_line(__r, &mut __line)
                        .map_err(__io_err)?;
                    if __n == 0 {
                        return Ok(None);
                    }
                    if __line.ends_with('\n') {
                        __line.pop();
                        if __line.ends_with('\r') {
                            __line.pop();
                        }
                    }
                    return Ok(Some(__line));
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::TextRead(ref mut __r)) => {
                    let mut __lines: Vec<String> = Vec::new();
                    let mut __line = String::new();
                    loop {
                        __line.clear();
                        let __n = std::io::BufRead::read_line(__r, &mut __line)
                            .map_err(__io_err)?;
                        if __n == 0 {
                            break;
                        }
                        let mut __l = __line.clone();
                        if __l.ends_with('\n') {
                            __l.pop();
                            if __l.ends_with('\r') {
                                __l.pop();
                            }
                        }
                        __lines.push(__l);
                    }
                    return Ok(__lines);
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()
    }
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::BinaryRead(ref mut __r)) => {
                    let mut __buf = Vec::new();
                    std::io::Read::read_to_end(__r, &mut __buf).map_err(__io_err)?;
                    return Ok(__buf.to_vec());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for binary reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::BinaryWrite(ref mut __w)) => {
                    std::io::Write::write_all(__w, &data).map_err(__io_err)?;
                    return Ok(());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for binary writing".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
    fn tell(&self) -> Result<i64, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
    fn readable(&self) -> bool {
        _mode_is_readable(&self._mode)
    }
    fn writable(&self) -> bool {
        _mode_is_writable(&self._mode)
    }
    fn seekable(&self) -> bool {
        false
    }
    fn __enter__(&self) -> FileHandle {
        self.clone()
    }
    fn __exit__(&mut self) {
        self.close();
    }
}
impl std::fmt::Display for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "FileHandle(_handle={}, _mode={}, _closed={})", self._handle, self._mode,
            self._closed
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct BinaryFileHandle {
    _handle: i64,
    _mode: String,
    _closed: bool,
}
impl BinaryFileHandle {
    fn new(handle: i64, mode: String) -> Self {
        Self {
            _handle: handle,
            _mode: mode,
            _closed: false,
        }
    }
    fn close(&mut self) {
        if self._closed {
            return;
        }
        {
            let __hid = self._handle;
            __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner())
                .remove(&__hid);
            ()
        };
        self._closed = true;
    }
    fn closed(&self) -> bool {
        self._closed
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
    fn read_bytes(&self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        let _ = size;
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::BinaryRead(ref mut __r)) => {
                    let mut __buf = Vec::new();
                    std::io::Read::read_to_end(__r, &mut __buf).map_err(__io_err)?;
                    return Ok(__buf.to_vec());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for binary reading".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        (|| {
            let __hid = self._handle;
            let mut __handles = __SIFR_FILE_HANDLES
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            match __handles.get_mut(&__hid) {
                Some(SifrFileHandle::BinaryWrite(ref mut __w)) => {
                    std::io::Write::write_all(__w, &data).map_err(__io_err)?;
                    return Ok(());
                }
                _ => {
                    return Err(IOError {
                        message: "file not open for binary writing".to_string(),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _ = offset;
        let _ = whence;
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
    fn tell(&self) -> Result<i64, IOError> {
        Err(IOError::new(_unsupported_seek_tell_error()))
    }
    fn readable(&self) -> bool {
        _mode_is_readable(&self._mode)
    }
    fn writable(&self) -> bool {
        _mode_is_writable(&self._mode)
    }
    fn seekable(&self) -> bool {
        false
    }
    fn __enter__(&self) -> BinaryFileHandle {
        self.clone()
    }
    fn __exit__(&mut self) {
        self.close();
    }
}
impl std::fmt::Display for BinaryFileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "BinaryFileHandle(_handle={}, _mode={}, _closed={})", self._handle, self
            ._mode, self._closed
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextFileHandle {
    _binary: BinaryFileHandle,
    _encoding: Encoding,
    _decode_errors: DecodeErrorHandler,
    _encode_errors: EncodeErrorHandler,
}
impl TextFileHandle {
    fn new(
        binary: BinaryFileHandle,
        enc: Encoding,
        decode_errors: DecodeErrorHandler,
        encode_errors: EncodeErrorHandler,
    ) -> Self {
        Self {
            _binary: binary,
            _encoding: enc,
            _decode_errors: decode_errors,
            _encode_errors: encode_errors,
        }
    }
    fn close(&mut self) {
        self._binary.clone().close();
    }
    fn closed(&mut self) -> bool {
        self._binary.clone().closed()
    }
    fn flush(&mut self) -> Result<(), IOError> {
        self._binary.clone().flush()
    }
    fn read(&mut self) -> Result<String, IOError> {
        let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
            let data: Vec<u8> = self._binary.clone().read_bytes(None)?;
            let text: String = (decode(
                &data,
                &self._encoding,
                &Some((self._decode_errors.clone()).clone()),
            ))
                .map_err(|__e| IOError::new(__e.to_string()))?;
            return Ok(Ok(text));
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
    fn write(&mut self, text: &String) -> Result<(), IOError> {
        let __sifr_try_res: Result<Result<(), IOError>, IOError> = (|| {
            let data: Vec<u8> = (encode(
                text,
                &self._encoding,
                &Some((self._encode_errors.clone()).clone()),
            ))
                .map_err(|__e| IOError::new(__e.to_string()))?;
            let result: () = self._binary.clone().write_bytes(&data)?;
            return Ok(Ok(result));
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
    fn readline(&self) -> Result<Option<String>, IOError> {
        Err(
            IOError::new(
                "TextFileHandle.readline is deferred; use read().split(\"\\n\")"
                    .to_string(),
            ),
        )
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        Err(
            IOError::new(
                "TextFileHandle.readlines is deferred; use read().split(\"\\n\")"
                    .to_string(),
            ),
        )
    }
    fn readable(&mut self) -> bool {
        self._binary.clone().readable()
    }
    fn writable(&mut self) -> bool {
        self._binary.clone().writable()
    }
    fn seekable(&mut self) -> bool {
        self._binary.clone().seekable()
    }
    fn __enter__(&self) -> TextFileHandle {
        self.clone()
    }
    fn __exit__(&mut self) {
        self.close();
    }
}
impl std::fmt::Display for TextFileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TextFileHandle(_binary={}, _encoding={:?}, _decode_errors={:?}, _encode_errors={:?})",
            self._binary, self._encoding, self._decode_errors, self._encode_errors
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextReader {
    _closed: bool,
}
impl TextReader {
    fn new() -> Self {
        Self { _closed: false }
    }
    fn read(&self) -> Result<String, IOError> {
        Err(
            IOError::new(
                "TextReader direct construction is unsupported; use open_text"
                    .to_string(),
            ),
        )
    }
    fn readline(&self) -> Result<Option<String>, IOError> {
        Err(
            IOError::new(
                "TextReader.readline is deferred; use read().split(\"\\n\")".to_string(),
            ),
        )
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        Err(
            IOError::new(
                "TextReader.readlines is deferred; use read().split(\"\\n\")".to_string(),
            ),
        )
    }
    fn close(&mut self) {
        self._closed = true;
    }
}
impl std::fmt::Display for TextReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TextReader(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct TextWriter {
    _closed: bool,
}
impl TextWriter {
    fn new() -> Self {
        Self { _closed: false }
    }
    fn write(&self, text: &String) -> Result<(), IOError> {
        let _ = (text).clone();
        Err(
            IOError::new(
                "TextWriter direct construction is unsupported; use open_text"
                    .to_string(),
            ),
        )
    }
    fn close(&mut self) {
        self._closed = true;
    }
}
impl std::fmt::Display for TextWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TextWriter(_closed={})", self._closed)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StringIO {
    _buffer: String,
    _cursor: i64,
    _closed: bool,
}
impl StringIO {
    fn new(initial: String) -> Self {
        Self {
            _buffer: {
                let mut __sifr_concat: String = String::with_capacity(
                    initial.len() + 0usize,
                );
                __sifr_concat.push_str((initial).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
            _cursor: 0_i64,
            _closed: false,
        }
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        self._closed
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
    fn read(&mut self, size: Option<i64>) -> Result<String, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let start: i64 = self._cursor;
        let mut end: i64 = self._buffer.clone().chars().count() as i64;
        if let Some(mut size) = size {
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
        if (tail_start < (self._buffer.clone().chars().count() as i64)) {
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
    fn getvalue(&self) -> String {
        self._buffer.clone()
    }
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
                    origin = self._buffer.clone().chars().count() as i64;
                } else {
                    return Err(IOError::new(_invalid_whence_error(whence)));
                }
            }
        }
        let mut next_pos: i64 = origin + offset;
        if next_pos < (0_i64) {
            return Err(IOError::new(_negative_seek_error(next_pos)));
        }
        let end: i64 = self._buffer.clone().chars().count() as i64;
        if next_pos > end {
            next_pos = end;
        }
        self._cursor = next_pos;
        Ok(self._cursor)
    }
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(self._cursor)
    }
    fn readable(&self) -> bool {
        !(self._closed)
    }
    fn writable(&self) -> bool {
        !(self._closed)
    }
    fn seekable(&self) -> bool {
        !(self._closed)
    }
}
impl std::fmt::Display for StringIO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "StringIO(_buffer={}, _cursor={}, _closed={})", self._buffer, self
            ._cursor, self._closed
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
struct BytesIO {
    _buffer: Vec<i64>,
    _cursor: i64,
    _closed: bool,
}
impl BytesIO {
    fn new(initial: Vec<u8>) -> Self {
        Self {
            _buffer: initial.iter().map(|__byte| *__byte as i64).collect::<Vec<i64>>(),
            _cursor: 0_i64,
            _closed: false,
        }
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        self._closed
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(())
    }
    fn _slice_to_bytes(&self, values: &Vec<i64>) -> Result<Vec<u8>, IOError> {
        let __sifr_try_res: Result<Result<Vec<u8>, IOError>, ValueError> = (|| {
            let built: Vec<u8> = ({
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
            })?;
            return Ok(Ok(built));
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
    fn read_bytes(&mut self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let start: i64 = self._cursor;
        let mut end: i64 = self._buffer.len() as i64;
        if let Some(mut size) = size {
            let maybe_size: i64 = size;
            if maybe_size >= (0_i64) {
                let requested: i64 = start + maybe_size;
                if requested < end {
                    end = requested;
                }
            }
        }
        let chunk: Vec<i64> = {
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
        self._slice_to_bytes(&chunk)
    }
    fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let values: Vec<i64> = data
            .iter()
            .map(|__byte| *__byte as i64)
            .collect::<Vec<i64>>();
        let mut i: i64 = 0_i64;
        while (i < (values.len() as i64)) {
            let maybe_value: Option<i64> = Some(values[i as usize]);
            let Some(mut maybe_value) = maybe_value else {
                return Err(IOError::new("bytes write invariant violation".to_string()));
            };
            let idx: i64 = self._cursor + i;
            if (idx < (self._buffer.len() as i64)) {
                {
                    let __idx_raw = idx;
                    let __idx_norm = if __idx_raw < 0 {
                        (self._buffer.len() as i64) + __idx_raw
                    } else {
                        __idx_raw
                    };
                    if __idx_norm >= 0 {
                        if let Some(__elem) = self._buffer.get_mut(__idx_norm as usize) {
                            *__elem = maybe_value;
                        }
                    }
                }
            } else {
                self._buffer.push(maybe_value);
            }
            i += 1_i64;
        }
        self._cursor += values.len() as i64;
        Ok(())
    }
    fn getvalue(&self) -> Result<Vec<u8>, IOError> {
        self._slice_to_bytes(&self._buffer.clone())
    }
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
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        Ok(self._cursor)
    }
    fn readable(&self) -> bool {
        !(self._closed)
    }
    fn writable(&self) -> bool {
        !(self._closed)
    }
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
fn _text_encoding_or_default(enc: &Option<Encoding>) -> Encoding {
    if let Some(enc) = enc.as_ref() {
        return Encoding::new(format!("{}{}", enc.label, ""));
    }
    Encoding::new("utf-8".to_string())
}
fn _decode_errors_or_default(errors: &Option<DecodeErrorHandler>) -> DecodeErrorHandler {
    if let Some(errors) = errors.as_ref() {
        return DecodeErrorHandler::new(format!("{}{}", errors.name, ""));
    }
    strict_decode_handler()
}
fn _encode_errors_from_decode_errors(errors: &DecodeErrorHandler) -> EncodeErrorHandler {
    EncodeErrorHandler::new(format!("{}{}", errors.name, ""))
}
fn open(path: &String, mode: &String) -> Result<FileHandle, IOError> {
    let __sifr_try_res: Result<Result<FileHandle, IOError>, IOError> = (|| {
        let handle: i64 = (|| {
            let __path = path.to_string();
            let __mode = mode.to_string();
            let __handle_id = __sifr_next_file_handle_id();
            match __mode.as_str() {
                "r" | "rt" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                    return Ok(__handle_id);
                }
                "w" | "wt" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(__handle_id);
                }
                "a" | "at" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(__handle_id);
                }
                "rb" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                    return Ok(__handle_id);
                }
                "wb" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(__handle_id);
                }
                "ab" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(__handle_id);
                }
                _ => {
                    return Err(IOError {
                        message: format!("invalid mode: {}", __mode),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()?;
        return Ok(Ok(FileHandle::new(handle, (mode).clone())));
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
fn open_binary(path: &String, mode: &String) -> Result<BinaryFileHandle, IOError> {
    if !(mode.contains(&"b".to_string())) {
        return Err(IOError::new("open_binary requires binary mode".to_string()));
    }
    let __sifr_try_res: Result<Result<BinaryFileHandle, IOError>, IOError> = (|| {
        let handle: i64 = (|| {
            let __path = path.to_string();
            let __mode = mode.to_string();
            let __handle_id = __sifr_next_file_handle_id();
            match __mode.as_str() {
                "r" | "rt" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                    return Ok(__handle_id);
                }
                "w" | "wt" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(__handle_id);
                }
                "a" | "at" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                    return Ok(__handle_id);
                }
                "rb" => {
                    let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                    let __reader = std::io::BufReader::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                    return Ok(__handle_id);
                }
                "wb" => {
                    let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(__handle_id);
                }
                "ab" => {
                    let __f = std::fs::OpenOptions::new()
                        .append(true)
                        .create(true)
                        .open(__path.as_str())
                        .map_err(__io_err)?;
                    let __writer = std::io::BufWriter::new(__f);
                    __SIFR_FILE_HANDLES
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner())
                        .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                    return Ok(__handle_id);
                }
                _ => {
                    return Err(IOError {
                        message: format!("invalid mode: {}", __mode),
                        kind: "Other".to_string(),
                    });
                }
            }
        })()?;
        return Ok(Ok(BinaryFileHandle::new(handle, (mode).clone())));
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
    encoding: &Option<Encoding>,
    errors: &Option<DecodeErrorHandler>,
) -> Result<TextFileHandle, IOError> {
    let __sifr_try_res: Result<Result<TextFileHandle, IOError>, IOError> = (|| {
        let binary_mode: String = _text_binary_mode(mode)?;
        let text_encoding: Encoding = _text_encoding_or_default(encoding);
        let decode_errors: DecodeErrorHandler = _decode_errors_or_default(errors);
        let encode_errors: EncodeErrorHandler = _encode_errors_from_decode_errors(
            &decode_errors,
        );
        let binary: BinaryFileHandle = open_binary(path, &binary_mode)?;
        return Ok(
            Ok(TextFileHandle::new(binary, text_encoding, decode_errors, encode_errors)),
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

// --- stdlib: sifr.configparser ---
fn __const_DEFAULTSECT() -> String {
    "DEFAULT".to_string().to_string()
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParsingError {
    line: i64,
    message: String,
}
impl ParsingError {
    fn new(line: i64, message: String) -> Self {
        Self { line, message }
    }
}
impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl std::error::Error for ParsingError {}
#[derive(Debug, Clone, PartialEq)]
struct SectionProxy {
    name: String,
    _values: HashMap<String, Option<String>>,
}
impl SectionProxy {
    fn new(name: String, values: HashMap<String, Option<String>>) -> Self {
        Self {
            name: {
                let mut __sifr_concat: String = String::with_capacity(
                    name.len() + 0usize,
                );
                __sifr_concat.push_str((name).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
            _values: _copy_values(&values),
        }
    }
    fn has_option(&self, option: &String) -> bool {
        _has_option_key(&self._values, &_normalize_option(option))
    }
    fn get(
        &self,
        option: &String,
        fallback: &Option<String>,
        raw: bool,
    ) -> Option<String> {
        let normalized: String = _normalize_option(option);
        if _has_option_key(&self._values, &normalized) {
            let value: Option<String> = _lookup_option(&self._values, &normalized);
            let Some(mut value) = value else {
                return None;
            };
            if raw {
                return Some(value);
            }
            return Some(_resolve_interpolation(&value, &self._values, 0_i64));
        }
        _copy_optional_str(fallback)
    }
    fn options(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![];
        for key in self._values.clone().keys().cloned() {
            names.push(key.clone());
        }
        names
    }
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
struct ConfigParser {
    _defaults: HashMap<String, Option<String>>,
    _sections: HashMap<String, HashMap<String, Option<String>>>,
    strict: bool,
    allow_no_value: bool,
}
impl ConfigParser {
    fn new(
        defaults: Option<HashMap<String, Option<String>>>,
        strict: bool,
        allow_no_value: bool,
    ) -> Self {
        let mut defaults_map: HashMap<String, Option<String>> = HashMap::from([]);
        let sections_map: HashMap<String, HashMap<String, Option<String>>> = HashMap::from([]);
        if let Some(mut defaults) = defaults {
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
        Self {
            strict,
            allow_no_value,
            _defaults: defaults_map,
            _sections: sections_map,
        }
    }
    fn defaults(&self) -> HashMap<String, Option<String>> {
        _copy_values(&self._defaults)
    }
    fn read_string(&mut self, text: &String) -> Result<(), ParsingError> {
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
                        ParsingError::new(line_no, "section name is empty".to_string()),
                    );
                }
                if section_name == default_section {
                    current_section = _default_section();
                    continue;
                }
                if self.strict && (self._sections).contains_key(&(section_name)) {
                    return Err(
                        ParsingError::new(
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
            let __sifr_try_res: Result<(), ParsingError> = (|| {
                let parsed_option_pair: (String, Option<String>) = _split_option_line(
                    &line,
                    self.allow_no_value,
                    line_no,
                )?;
                let (option_name, option_value) = parsed_option_pair;
                let mut __sifr_chars_option_name: Vec<char> = option_name
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
                                ParsingError::new(
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
    fn read(&mut self, path: &String) -> Result<Vec<String>, IOError> {
        let __sifr_try_res: Result<Result<Vec<String>, IOError>, IOError> = (|| {
            let content: String = std::fs::read_to_string(&path).map_err(__io_err)?;
            let __sifr_try_res: Result<(), ParsingError> = (|| {
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
            return Ok(Ok(vec![loaded_path]));
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
    fn sections(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![];
        for section in self._sections.clone().keys().cloned() {
            names.push(section.clone());
        }
        names
    }
    fn has_section(&self, section: &String) -> bool {
        (self._sections).contains_key((section).as_str())
    }
    fn options(&self, section: &String) -> Vec<String> {
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        let mut names: Vec<String> = vec![];
        for option in merged.keys().cloned() {
            names.push(option.clone());
        }
        names
    }
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
            let Some(mut raw_value) = raw_value else {
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
                let Some(mut default_value) = default_value else {
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
        let Some(mut raw_value2) = raw_value2 else {
            return None;
        };
        if raw {
            return Some(raw_value2);
        }
        Some(_resolve_interpolation(&raw_value2, &merged, 0_i64))
    }
    fn getint(
        &self,
        section: &String,
        option: &String,
        fallback: Option<i64>,
    ) -> Option<i64> {
        let raw: Option<String> = self.get(section, option, &None, false);
        let Some(mut raw) = raw else {
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
    fn getfloat(
        &self,
        section: &String,
        option: &String,
        fallback: Option<f64>,
    ) -> Option<f64> {
        let raw: Option<String> = self.get(section, option, &None, false);
        let Some(mut raw) = raw else {
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
    fn getboolean(
        &self,
        section: &String,
        option: &String,
        fallback: Option<bool>,
    ) -> Option<bool> {
        let raw: Option<String> = self.get(section, option, &None, false);
        let Some(mut raw) = raw else {
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
    fn proxy(&self, section: &String) -> Option<SectionProxy> {
        let default_section: String = _default_section();
        if (*section != default_section) && !(self.has_section(section)) {
            return None;
        }
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        Some(SectionProxy::new((section).clone(), merged))
    }
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
                    if let Some(mut value) = value {
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
                    if let Some(mut value) = value {
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
    fn write(&self, path: &String) -> Result<(), IOError> {
        let payload: String = self.to_ini_string();
        std::fs::write(&path, payload.as_bytes()).map(|_| ()).map_err(__io_err)
    }
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
) -> Result<(String, Option<String>), ParsingError> {
    let delimiter: Option<String> = _find_delimiter(line);
    let Some(mut delimiter) = delimiter else {
        if allow_no_value {
            return Ok((line.trim().to_string(), None));
        }
        return Err(
            ParsingError::new(
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
        return Err(ParsingError::new(line_no, "invalid option line".to_string()));
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
    let Some(mut raw_key) = raw_key else {
        return Err(ParsingError::new(line_no, "option name is missing".to_string()));
    };
    let key: String = _normalize_option(&raw_key);
    if key == "" {
        return Err(ParsingError::new(line_no, "option name is empty".to_string()));
    }
    let Some(mut raw_value) = raw_value else {
        return Ok((key, None));
    };
    let stripped_value: Option<String> = _some_str(&raw_value.trim().to_string());
    Ok((key.clone(), stripped_value.clone()))
}
fn _char_at(text: &String, index: i64) -> String {
    let mut __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
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
    let Some(mut ch) = ch else {
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
                        if let Some(mut replacement) = replacement {
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

// --- stdlib: sifr.gzip ---
fn compress(data: &String) -> Vec<i64> {
    {
        let __data = &data.as_bytes();
        let mut __enc = flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        );
        std::io::Write::write_all(&mut __enc, __data).unwrap_or(());
        __enc
            .finish()
            .unwrap_or_default()
            .iter()
            .map(|b| *b as i64)
            .collect::<Vec<i64>>()
    }
}
fn decompress(data: &Vec<i64>) -> Result<String, IOError> {
    {
        let __bytes = data.iter().map(|b| *b as u8).collect::<Vec<u8>>();
        let mut __dec = flate2::read::GzDecoder::new(__bytes.as_slice());
        let mut __out = String::new();
        std::io::Read::read_to_string(&mut __dec, &mut __out).map_err(__io_err)?;
        Ok(__out)
    }
}

// --- stdlib: sifr.html ---
fn escape(s: &String, quote: bool) -> String {
    let escaped: String = s
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#x27;");
    if quote {
        return escaped;
    }
    escaped.replace("&quot;", "\"").replace("&#x27;", "\'")
}
fn unescape(s: &String) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#X27;", "'")
        .replace("&#39;", "'")
        .replace("&#60;", "<")
        .replace("&#x3C;", "<")
        .replace("&#x3c;", "<")
        .replace("&#X3C;", "<")
        .replace("&#X3c;", "<")
        .replace("&#62;", ">")
        .replace("&#x3E;", ">")
        .replace("&#x3e;", ">")
        .replace("&#X3E;", ">")
        .replace("&#X3e;", ">")
}

// --- stdlib: sifr.operator ---
fn add(a: i64, b: i64) -> i64 {
    a + b
}
fn sub(a: i64, b: i64) -> i64 {
    a - b
}
fn mul(a: i64, b: i64) -> i64 {
    a * b
}
fn floordiv(a: i64, b: i64) -> i64 {
    a / b
}
fn mod_val(a: i64, b: i64) -> i64 {
    a % b
}
fn neg(a: i64) -> i64 {
    -a
}
fn lt(a: i64, b: i64) -> bool {
    a < b
}
fn eq(a: i64, b: i64) -> bool {
    a == b
}
fn getitem<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    index: i64,
) -> Option<T> {
    {
        let __sifr_index_list = &items;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    }
}
fn itemgetter<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    index: i64,
) -> Option<T> {
    getitem(items, index)
}

// --- stdlib: sifr.sys ---
fn version() -> String {
    "sifr 0.1.0".to_string()
}
fn maxsize() -> i64 {
    i64::MAX
}

// --- stdlib: sifr.zipfile ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ZipInfo {
    filename: String,
    file_size: i64,
    compress_type: i64,
}
impl ZipInfo {
    fn new(filename: String, file_size: i64, compress_type: i64) -> Self {
        Self {
            filename: {
                let mut __sifr_concat: String = String::with_capacity(
                    filename.len() + 0usize,
                );
                __sifr_concat.push_str((filename).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
            file_size,
            compress_type,
        }
    }
}
impl std::fmt::Display for ZipInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "ZipInfo(filename={}, file_size={}, compress_type={})", self.filename,
            self.file_size, self.compress_type
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
struct ZipReadHandle {
    _data: Vec<u8>,
    _cursor: i64,
    _closed: bool,
}
impl ZipReadHandle {
    fn new(data: Vec<u8>) -> Self {
        Self {
            _data: data,
            _cursor: 0_i64,
            _closed: false,
        }
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        self._closed
    }
    fn read_bytes(&mut self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let mut end: i64 = self._data.len() as i64;
        if let Some(mut size) = size {
            let requested_size: i64 = size;
            if requested_size < (0_i64) {
                end = self._data.len() as i64;
            } else {
                let requested_end: i64 = self._cursor + requested_size;
                if requested_end < end {
                    end = requested_end;
                }
            }
        }
        let out: Vec<u8> = {
            let _slice_src = &self._data.clone();
            let _slice_len_i64 = _slice_src.len() as i64;
            let _slice_start_i64 = if self._cursor < 0 {
                (_slice_len_i64 + self._cursor).max(0)
            } else {
                self._cursor.min(_slice_len_i64)
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
        Ok(out)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ZipFile {
    path: String,
    mode: String,
    compression: i64,
}
impl ZipFile {
    fn new(path: String, mode: String, compression: i64) -> Self {
        Self {
            path: {
                let mut __sifr_concat: String = String::with_capacity(
                    path.len() + 0usize,
                );
                __sifr_concat.push_str((path).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
            mode: {
                let mut __sifr_concat: String = String::with_capacity(
                    mode.len() + 0usize,
                );
                __sifr_concat.push_str((mode).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
            compression,
        }
    }
    fn _writable_mode(&self) -> bool {
        (((((self.mode.clone() == "w")) || ((self.mode.clone() == "a")))
            || ((self.mode.clone() == "wb"))) || ((self.mode.clone() == "ab")))
    }
    fn create(&self) -> Result<(), IOError> {
        {
            let __f = std::fs::File::create(&self.path.clone()).map_err(__io_err)?;
            drop(zip::ZipWriter::new(__f));
            Ok(())
        }
    }
    fn write(&self, name: &String, content: &String) -> Result<(), IOError> {
        if !(self._writable_mode()) {
            return Err(IOError::new(_zip_read_only_error()));
        }
        {
            let __path = self.path.clone().clone();
            let __name = name;
            let __content = content;
            let __f = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&__path)
                .map_err(__io_err)?;
            let mut __zip = zip::ZipWriter::new_append(__f)
                .map_err(|e| IOError::new(e.to_string()))?;
            let __opts = zip::write::SimpleFileOptions::default();
            __zip
                .start_file(__name.to_string(), __opts)
                .map_err(|e| IOError::new(e.to_string()))?;
            std::io::Write::write_all(&mut __zip, __content.as_bytes())
                .map_err(__io_err)?;
            __zip.finish().map_err(|e| IOError::new(e.to_string()))?;
            Ok(())
        }
    }
    fn write_bytes(&self, name: &String, content: &Vec<u8>) -> Result<(), IOError> {
        if !(self._writable_mode()) {
            return Err(IOError::new(_zip_read_only_error()));
        }
        {
            let __path = self.path.clone().clone();
            let __name = name;
            let __content = content;
            let __f = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&__path)
                .map_err(__io_err)?;
            let mut __zip = zip::ZipWriter::new_append(__f)
                .map_err(|e| IOError::new(e.to_string()))?;
            let __opts = zip::write::SimpleFileOptions::default();
            __zip
                .start_file(__name.to_string(), __opts)
                .map_err(|e| IOError::new(e.to_string()))?;
            std::io::Write::write_all(&mut __zip, &__content).map_err(__io_err)?;
            __zip.finish().map_err(|e| IOError::new(e.to_string()))?;
            Ok(())
        }
    }
    fn read(&self, name: &String) -> Result<String, IOError> {
        {
            let __f = std::fs::File::open(&self.path.clone()).map_err(__io_err)?;
            let mut __zip = zip::ZipArchive::new(__f)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __file = __zip
                .by_name(&name)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __content = String::new();
            std::io::Read::read_to_string(&mut __file, &mut __content)
                .map_err(__io_err)?;
            Ok(__content)
        }
    }
    fn read_bytes(&self, name: &String) -> Result<Vec<u8>, IOError> {
        {
            let __f = std::fs::File::open(&self.path.clone()).map_err(__io_err)?;
            let mut __zip = zip::ZipArchive::new(__f)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __file = __zip
                .by_name(&name)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __content = Vec::new();
            std::io::Read::read_to_end(&mut __file, &mut __content).map_err(__io_err)?;
            Ok(__content.to_vec())
        }
    }
    fn namelist(&self) -> Result<Vec<String>, IOError> {
        {
            let __f = std::fs::File::open(&self.path.clone()).map_err(__io_err)?;
            let mut __zip = zip::ZipArchive::new(__f)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __names = Vec::new();
            for __i in 0..__zip.len() {
                if let Ok(__file) = __zip.by_index(__i) {
                    __names.push(__file.name().to_string());
                }
            }
            Ok(__names)
        }
    }
    fn infolist(&self) -> Result<Vec<ZipInfo>, IOError> {
        Err(IOError::new(_zip_unimplemented_error(&"infolist".to_string())))
    }
    fn getinfo(&self, name: &String) -> Result<ZipInfo, IOError> {
        let _ = (name).clone();
        Err(IOError::new(_zip_unimplemented_error(&"getinfo".to_string())))
    }
    fn open(&self, name: &String, mode: &String) -> Result<ZipReadHandle, IOError> {
        let _ = (name).clone();
        if ((mode).as_str() != "r") && ((mode).as_str() != "rb") {
            return Err(IOError::new(_zip_open_mode_error(mode)));
        }
        Err(IOError::new(_zip_unimplemented_error(&"open".to_string())))
    }
    fn extract(&self, name: &String, path: &String) -> Result<String, IOError> {
        let _ = (name).clone();
        let _ = (path).clone();
        Err(IOError::new(_zip_unimplemented_error(&"extract".to_string())))
    }
    fn extractall(&self, path: &String) -> Result<Vec<String>, IOError> {
        let _ = (path).clone();
        Err(IOError::new(_zip_unimplemented_error(&"extractall".to_string())))
    }
    fn __enter__(&self) -> ZipFile {
        self.clone()
    }
    fn __exit__(&self) {}
}
impl std::fmt::Display for ZipFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "ZipFile(path={}, mode={}, compression={})", self.path, self.mode, self
            .compression
        )
    }
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
            (8usize + feature.len()) + 32usize,
        );
        __sifr_concat.push_str("zipfile ");
        __sifr_concat.push_str((feature).as_str());
        __sifr_concat.push_str(" is not implemented in this compatibility surface");
        __sifr_concat
    }
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        Self { message, kind: "Other".to_string() }
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    IOError { message: msg, kind }
}

#[derive(Debug, Clone)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for Error {
}

#[derive(Debug, Clone)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for ParseError {
}

#[derive(Debug, Clone)]
struct ValueError {
    message: String,
}

impl ValueError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for ValueError {
}

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        Self { message, line: 0, column: 0 }
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for JSONDecodeError {
}

#[derive(Debug, Clone)]
struct JsonIntegerRangeError {
    message: String,
    path: String,
    profile: String,
}

impl JsonIntegerRangeError {
    fn new(message: String) -> Self {
        Self { message, path: String::new(), profile: String::new() }
    }
}

impl std::fmt::Display for JsonIntegerRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for JsonIntegerRangeError {
}

#[derive(Debug, Clone)]
struct JsonLimitError {
    message: String,
    limit: i64,
}

impl JsonLimitError {
    fn new(message: String) -> Self {
        Self { message, limit: 0 }
    }
}

impl std::fmt::Display for JsonLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for JsonLimitError {
}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        Self { message, line: 0, column: 0 }
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for TOMLDecodeError {
}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        Self { message, detail: String::new() }
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for RegexError {
}

#[derive(Debug, Clone)]
struct TimeoutError {
    message: String,
}

impl TimeoutError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for TimeoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for TimeoutError {
}

#[derive(Debug, Clone)]
struct ScopeFailure {
    message: String,
}

impl ScopeFailure {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for ScopeFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.message, f)
    }
}

impl std::error::Error for ScopeFailure {
}

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

enum SifrFileHandle {
    TextRead(std::io::BufReader<std::fs::File>),
    TextWrite(std::io::BufWriter<std::fs::File>),
    BinaryRead(std::io::BufReader<std::fs::File>),
    BinaryWrite(std::io::BufWriter<std::fs::File>),
}

static __SIFR_FILE_HANDLES: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<i64, SifrFileHandle>>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

static __SIFR_NEXT_FILE_HANDLE_ID: std::sync::atomic::AtomicI64 = std::sync::atomic::AtomicI64::new(1);

fn __sifr_next_file_handle_id() -> i64 {
    __SIFR_NEXT_FILE_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

fn demo_operator() {
    println!("=== operator ===");
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("add(10, 5) = ");
    __sifr_concat.push_str((format!("{}", add(10_i64, 5_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("sub(10, 5) = ");
    __sifr_concat.push_str((format!("{}", sub(10_i64, 5_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("mul(3, 4) = ");
    __sifr_concat.push_str((format!("{}", mul(3_i64, 4_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + 0usize);
    __sifr_concat.push_str("floordiv(7, 2) = ");
    __sifr_concat.push_str((format!("{}", floordiv(7_i64, 2_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("mod_val(7, 2) = ");
    __sifr_concat.push_str((format!("{}", mod_val(7_i64, 2_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("neg(42) = ");
    __sifr_concat.push_str((format!("{}", neg(42_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("lt(3, 5) = ");
    __sifr_concat.push_str((format!("{}", lt(3_i64, 5_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("eq(5, 5) = ");
    __sifr_concat.push_str((format!("{}", eq(5_i64, 5_i64))).as_str());
    __sifr_concat
});
    let items: Vec<i64> = vec![1_i64, 2_i64, 3_i64];
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(25usize + 0usize);
    __sifr_concat.push_str("itemgetter([1,2,3], 1) = ");
    __sifr_concat.push_str(((itemgetter(&items, 1_i64)).map_or("None".to_string().to_string(), |__v| format!("{}", __v))).as_str());
    __sifr_concat
});
}

fn demo_calendar() {
    println!("=== calendar ===");
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("isleap(2000) = ");
    __sifr_concat.push_str((format!("{}", isleap(2000_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("isleap(1900) = ");
    __sifr_concat.push_str((format!("{}", isleap(1900_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("isleap(2024) = ");
    __sifr_concat.push_str((format!("{}", isleap(2024_i64))).as_str());
    __sifr_concat
});
    let wd: i64 = weekday(2024_i64, 1_i64, 1_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(20usize + 0usize);
    __sifr_concat.push_str("weekday(2024,1,1) = ");
    __sifr_concat.push_str((format!("{}", wd)).as_str());
    __sifr_concat
});
    let mr: Vec<i64> = monthrange(2024_i64, 2_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
    __sifr_concat.push_str("monthrange(2024,2)[1] = ");
    __sifr_concat.push_str((({
    let __sifr_index_list = &mr;
    let __sifr_index_i = 1_i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).copied()
}).map_or("None".to_string().to_string(), |__v| format!("{}", __v))).as_str());
    __sifr_concat
});
}

fn demo_html() {
    println!("=== html ===");
    let s: String = "<b>Hi & Bye</b>".to_string();
    let esc: String = escape(&s, true);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(26usize + esc.len());
    __sifr_concat.push_str("escape(<b>Hi & Bye</b>) = ");
    __sifr_concat.push_str((esc).as_str());
    __sifr_concat
});
    let unesc: String = unescape(&esc);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(44usize + unesc.len());
    __sifr_concat.push_str("unescape(&lt;b&gt;Hi &amp; Bye&lt;/b&gt;) = ");
    __sifr_concat.push_str((unesc).as_str());
    __sifr_concat
});
}

fn demo_sys() {
    println!("=== sys ===");
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("version = ");
    __sifr_concat.push_str((version()).as_str());
    __sifr_concat
});
    let ms: i64 = maxsize();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("maxsize > 0 = ");
    __sifr_concat.push_str((format!("{}", ms > (0_i64))).as_str());
    __sifr_concat
});
}

fn demo_configparser() {
    println!("=== configparser ===");
    let mut config: ConfigParser = ConfigParser::new(None, false, false);
    let __sifr_try_res: Result<(), ParsingError> = (|| {
    let _ = config.read_string(&"[database]\nhost = db.example.com\nport = 5432\n".to_string())?;
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", e.message);
        return;
    }
    let host_value: Option<String> = config.get(&"database".to_string(), &"host".to_string(), &None, false);
    let port_value: Option<String> = config.get(&"database".to_string(), &"port".to_string(), &None, false);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(7usize + 0usize);
    __sifr_concat.push_str("host = ");
    __sifr_concat.push_str(((host_value).map_or("None".to_string().to_string(), |__v| format!("{}", __v))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(7usize + 0usize);
    __sifr_concat.push_str("port = ");
    __sifr_concat.push_str(((port_value).map_or("None".to_string().to_string(), |__v| format!("{}", __v))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("has_host = ");
    __sifr_concat.push_str((format!("{}", config.has_option(&"database".to_string(), &"host".to_string()))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("has_missing = ");
    __sifr_concat.push_str((format!("{}", config.has_option(&"database".to_string(), &"missing".to_string()))).as_str());
    __sifr_concat
});
}

fn demo_gzip() {
    println!("=== gzip ===");
    let data: String = "Sifr stdlib gzip compression!".to_string();
    let compressed: Vec<i64> = compress(&data);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("compressed len > 0 = ");
    __sifr_concat.push_str((format!("{}", (compressed.len() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    let __sifr_try_res: Result<(), IOError> = (|| {
    let decompressed: String = decompress(&compressed)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + decompressed.len());
    __sifr_concat.push_str("decompressed = ");
    __sifr_concat.push_str((decompressed).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(7usize + 0usize);
    __sifr_concat.push_str("error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
}

fn demo_zipfile() {
    println!("=== zipfile ===");
    let mut zf: ZipFile = ZipFile::new("/tmp/sifr_demo_zipfile.zip".to_string(), "a".to_string(), 0_i64);
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _c: () = zf.create()?;
    println!("zip created = true");
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("create error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _w: () = zf.write(&"demo.txt".to_string(), &"Hello from ZipFile!".to_string())?;
    let content: String = zf.read(&"demo.txt".to_string())?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + content.len());
    __sifr_concat.push_str("zip content = ");
    __sifr_concat.push_str((content).as_str());
    __sifr_concat
});
    let names: Vec<String> = zf.namelist()?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(19usize + 0usize);
    __sifr_concat.push_str("zip namelist len = ");
    __sifr_concat.push_str((format!("{}", names.len() as i64)).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("zip error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _r: () = std::fs::remove_file(&"/tmp/sifr_demo_zipfile.zip".to_string()).map(|_| ()).map_err(__io_err)?;
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
