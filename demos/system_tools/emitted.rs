// src/main.rs
use sifr_runtime::SifrInt;

use std::sync::Mutex;

// --- stdlib: sifr.env ---
fn getenv(key: &String, default_value: &String) -> String {
    let val: Option<String> = {
        let __k = key;
        if __k.is_empty() || (__k.contains('=') || __k.as_bytes().contains(&0)) {
            None
        } else {
            std::env::var(__k).ok()
        }
    };
    let Some(mut val) = val else {
        return {
            let mut __sifr_concat: String = String::with_capacity(
                default_value.len() + 0usize,
            );
            __sifr_concat.push_str((default_value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    };
    val
}
fn setenv(key: &String, value: &String) {
    {
        let __k = key;
        let __v = value;
        if !__k.is_empty()
            && (!__k.contains('=')
                && (!__k.as_bytes().contains(&0) && !__v.as_bytes().contains(&0)))
        {
            std::env::set_var(__k, __v);
        }
    };
}
fn unsetenv(key: &String) {
    {
        let __k = key;
        if !__k.is_empty() && (!__k.contains('=') && !__k.as_bytes().contains(&0)) {
            std::env::remove_var(__k);
        }
    };
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

// --- stdlib: sifr.logging ---
const DEBUG: i64 = 10_i64;
const INFO: i64 = 20_i64;
const WARNING: i64 = 30_i64;
const ERROR: i64 = 40_i64;
const CRITICAL: i64 = 50_i64;
const NOTSET: i64 = 0_i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Formatter {
    _fmt: String,
}
impl Formatter {
    fn new(fmt: String) -> Self {
        Self { _fmt: fmt }
    }
    fn template(&self) -> String {
        self._fmt.clone()
    }
    fn format(&self, level: &String, name: &String, msg: &String) -> String {
        let mut result: String = self._fmt.clone();
        result = result.replace("%(levelname)s", &level);
        result = result.replace("%(name)s", &name);
        result = result.replace("%(message)s", &msg);
        result
    }
}
impl std::fmt::Display for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Formatter(_fmt={})", self._fmt)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StreamHandler {
    _level: i64,
    _formatter: Formatter,
}
impl StreamHandler {
    fn new(level: i64) -> Self {
        Self {
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        }
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        self._level
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, ""));
    }
    fn format_template(&mut self) -> String {
        self._formatter.clone().template()
    }
    fn _allows(&self, level_num: i64) -> bool {
        if (self._level == NOTSET) {
            return true;
        }
        (level_num >= self._level)
    }
    fn emit(&mut self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = self._formatter.clone().format(level, name, msg);
        println!("{}", line);
    }
}
impl std::fmt::Display for StreamHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "StreamHandler(_level={}, _formatter={})", self._level, self._formatter
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FileHandler {
    _path: String,
    _level: i64,
    _formatter: Formatter,
}
impl FileHandler {
    fn new(path: String, level: i64) -> Self {
        Self {
            _path: {
                let mut __sifr_concat: String = String::with_capacity(
                    path.len() + 0usize,
                );
                __sifr_concat.push_str((path).as_str());
                __sifr_concat.push_str("");
                __sifr_concat
            },
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        }
    }
    fn path(&self) -> String {
        self._path.clone()
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        self._level
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, ""));
    }
    fn format_template(&mut self) -> String {
        self._formatter.clone().template()
    }
    fn _allows(&self, level_num: i64) -> bool {
        if (self._level == NOTSET) {
            return true;
        }
        (level_num >= self._level)
    }
    fn emit(&mut self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = {
            let mut __sifr_concat: String = String::with_capacity(0usize + 1usize);
            __sifr_concat
                .push_str((self._formatter.clone().format(level, name, msg)).as_str());
            __sifr_concat.push('\n');
            __sifr_concat
        };
        let __sifr_try_res: Result<(), IOError> = (|| {
            let mut fh: TextFileHandle = open_text(
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
                let _ = e2.message;
            }
            fh.close();
            Ok(())
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            let _ = e.message;
        }
    }
}
impl std::fmt::Display for FileHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "FileHandler(_path={}, _level={}, _formatter={})", self._path, self
            ._level, self._formatter
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NullHandler {
    _level: i64,
    _formatter: Formatter,
}
impl NullHandler {
    fn new(level: i64) -> Self {
        Self {
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        }
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        self._level
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, ""));
    }
    fn format_template(&mut self) -> String {
        self._formatter.clone().template()
    }
    fn emit(&self, level: &String, name: &String, msg: &String) {
        let _ = (level).clone();
        let _ = (name).clone();
        let _ = (msg).clone();
    }
}
impl std::fmt::Display for NullHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NullHandler(_level={}, _formatter={})", self._level, self._formatter)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Logger {
    _name: String,
    _level: i64,
    _log_path: String,
    _handler_kind: String,
    _handler_path: String,
    _handler_level: i64,
    _handler_fmt: String,
}
impl Logger {
    fn new(name: String, level: i64) -> Self {
        Self {
            _name: name,
            _level: level,
            _log_path: "".to_string(),
            _handler_kind: "".to_string(),
            _handler_path: "".to_string(),
            _handler_level: NOTSET,
            _handler_fmt: "%(levelname)s:%(name)s:%(message)s".to_string(),
        }
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn set_file(&mut self, path: &String) {
        self._log_path = {
            let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
            __sifr_concat.push_str((path).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    fn add_handler(&mut self, handler: &FileHandler) {
        self._handler_kind = "file".to_string();
        self._handler_path = handler.path();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn set_stream_handler(&mut self, handler: &StreamHandler) {
        self._handler_kind = "stream".to_string();
        self._handler_path = "".to_string();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn set_null_handler(&mut self, handler: &NullHandler) {
        self._handler_kind = "null".to_string();
        self._handler_path = "".to_string();
        self._handler_level = handler.level();
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn clear_handler(&mut self) {
        self._handler_kind = "".to_string();
        self._handler_path = "".to_string();
        self._handler_level = NOTSET;
        self._handler_fmt = "%(levelname)s:%(name)s:%(message)s".to_string();
    }
    fn _handler_allows(&self, level_num: i64) -> bool {
        if (self._handler_level == NOTSET) {
            return true;
        }
        (level_num >= self._handler_level)
    }
    fn _handler_line(&self, level: &String, msg: &String) -> String {
        let mut formatter: Formatter = Formatter::new(self._handler_fmt.clone());
        formatter.format(level, &self._name.clone(), msg)
    }
    fn _emit(&self, level: &String, level_num: i64, msg: &String) {
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
                    let mut fh: TextFileHandle = open_text(
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
                        let _ = e2.message;
                    }
                    fh.close();
                    Ok(())
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    let _ = e.message;
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
                let mut fh: TextFileHandle = open_text(
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
                    let _ = e2.message;
                }
                fh.close();
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _ = e.message;
            }
        }
    }
    fn debug(&self, msg: &String) {
        self._emit(&"DEBUG".to_string(), DEBUG, msg);
    }
    fn info(&self, msg: &String) {
        self._emit(&"INFO".to_string(), INFO, msg);
    }
    fn warning(&self, msg: &String) {
        self._emit(&"WARNING".to_string(), WARNING, msg);
    }
    fn error(&self, msg: &String) {
        self._emit(&"ERROR".to_string(), ERROR, msg);
    }
    fn critical(&self, msg: &String) {
        self._emit(&"CRITICAL".to_string(), CRITICAL, msg);
    }
}
impl std::fmt::Display for Logger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Logger(_name={}, _level={}, _log_path={}, _handler_kind={}, _handler_path={}, _handler_level={}, _handler_fmt={})",
            self._name, self._level, self._log_path, self._handler_kind, self
            ._handler_path, self._handler_level, self._handler_fmt
        )
    }
}
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
fn getLogger(name: &String) -> Logger {
    let level: i64 = *__SIFR_GLOBAL_LOG_LEVEL
        .lock()
        .unwrap_or_else(|__err| __err.into_inner());
    Logger::new((name).clone(), level)
}

// --- stdlib: sifr.platform ---
fn system() -> String {
    if cfg!(target_os = "windows") {
        "Windows".to_string().to_string()
    } else {
        if cfg!(target_os = "macos") {
            "Darwin".to_string().to_string()
        } else {
            if cfg!(target_os = "linux") {
                "Linux".to_string().to_string()
            } else {
                std::env::consts::OS.to_string()
            }
        }
    }
}
fn machine() -> String {
    std::env::consts::ARCH.to_string()
}
fn processor() -> String {
    std::env::consts::ARCH.to_string()
}

// --- stdlib: sifr.sys ---
fn argv() -> Vec<String> {
    std::env::args().collect::<Vec<String>>()
}
fn version() -> String {
    "sifr 0.1.0".to_string()
}
fn platform() -> String {
    std::env::consts::OS.to_string()
}

// --- stdlib: sifr.time ---
fn time() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}
fn strftime(fmt: &String, epoch: f64) -> String {
    {
        let secs = epoch as i64;
        let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default();
        dt.format(&fmt).to_string()
    }
}

// --- stdlib: sifr.timeit ---
fn _elapsed_non_negative(start: f64, end: f64) -> f64 {
    let elapsed: f64 = end - start;
    if elapsed < (0.0_f64) {
        return 0.0_f64;
    }
    elapsed
}
fn timeit(stmt: impl Fn(), number: i64) -> f64 {
    let start: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let mut i: i64 = 0_i64;
    while i < number {
        stmt();
        i += 1_i64;
    }
    let end: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    _elapsed_non_negative(start, end)
}
fn repeat(stmt: impl Fn(), count: i64, number: i64) -> Vec<f64> {
    let mut results: Vec<f64> = vec![];
    let mut r: i64 = 0_i64;
    while r < count {
        let start: f64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let mut i: i64 = 0_i64;
        while i < number {
            stmt();
            i += 1_i64;
        }
        let end: f64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let elapsed: f64 = _elapsed_non_negative(start, end);
        results.push(elapsed);
        r += 1_i64;
    }
    results
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

static __SIFR_GLOBAL_LOG_LEVEL: std::sync::LazyLock<std::sync::Mutex<i64>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(20));

fn workload() {
    let mut i: i64 = 0_i64;
    let mut total: i64 = 0_i64;
    while i < (100_i64) {
        total += i;
        i += 1_i64;
    }
}

fn main() {
    let __sifr_try_res: Result<(), IOError> = (|| {
    let shell_out: String = ({
    let __cmd = "echo system-tools-sample".to_string();
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let cwd: String = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).map_err(__io_err)?;
    let mut __sifr_chars_cwd: Vec<char> = cwd.chars().collect::<Vec<char>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + shell_out.len());
    __sifr_concat.push_str("os.run_command = ");
    __sifr_concat.push_str((shell_out).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(20usize + 0usize);
    __sifr_concat.push_str("os.getcwd len > 0 = ");
    __sifr_concat.push_str((format!("{}", (cwd.chars().count() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("os error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    setenv(&"SIFR_SYSTEM_TOOLS_DEMO".to_string(), &"ok".to_string());
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("env getenv = ");
    __sifr_concat.push_str((getenv(&"SIFR_SYSTEM_TOOLS_DEMO".to_string(), &"fallback".to_string())).as_str());
    __sifr_concat
});
    unsetenv(&"SIFR_SYSTEM_TOOLS_DEMO".to_string());
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("sys.argv len = ");
    __sifr_concat.push_str((format!("{}", argv().len() as i64)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(14usize + 0usize);
    __sifr_concat.push_str("sys.version = ");
    __sifr_concat.push_str((version()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("sys.platform = ");
    __sifr_concat.push_str((platform()).as_str());
    __sifr_concat
});
    let mut logger: Logger = getLogger(&"system-tools-sample_demo".to_string());
    logger.set_level(INFO);
    logger.info(&"logging demo line".to_string());
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(18usize + 0usize);
    __sifr_concat.push_str("platform.system = ");
    __sifr_concat.push_str((system()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(19usize + 0usize);
    __sifr_concat.push_str("platform.machine = ");
    __sifr_concat.push_str((machine()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("platform.processor = ");
    __sifr_concat.push_str((processor()).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("time.time > 0 = ");
    __sifr_concat.push_str((format!("{}", time() > (0.0_f64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(23usize + 0usize);
    __sifr_concat.push_str("time.strftime epoch0 = ");
    __sifr_concat.push_str((strftime(&"%Y-%m-%d %H:%M:%S".to_string(), 0.0_f64)).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("timeit.timeit = ");
    __sifr_concat.push_str((format!("{}", timeit(workload, 5_i64))).as_str());
    __sifr_concat
});
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(22usize + 0usize);
    __sifr_concat.push_str("timeit.repeat count = ");
    __sifr_concat.push_str((format!("{}", repeat(workload, 3_i64, 4_i64).len() as i64)).as_str());
    __sifr_concat
});
}
