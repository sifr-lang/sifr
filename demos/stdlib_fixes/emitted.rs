// src/main.rs
use sifr_runtime::SifrInt;

use std::sync::Mutex;

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

// --- stdlib: sifr.csv ---
const QUOTE_ALL: i64 = 1_i64;
const QUOTE_NONNUMERIC: i64 = 2_i64;
const QUOTE_NONE: i64 = 3_i64;
const QUOTE_STRINGS: i64 = 4_i64;
const QUOTE_NOTNULL: i64 = 5_i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Dialect {
    delimiter: String,
    quotechar: String,
    escapechar: String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: String,
    quoting: i64,
}
impl Dialect {
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
        Self {
            delimiter,
            quotechar,
            escapechar,
            doublequote,
            skipinitialspace,
            lineterminator,
            quoting: resolved_quoting,
        }
    }
}
impl std::fmt::Display for Dialect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Dialect(delimiter={}, quotechar={}, escapechar={}, doublequote={}, skipinitialspace={}, lineterminator={}, quoting={})",
            self.delimiter, self.quotechar, self.escapechar, self.doublequote, self
            .skipinitialspace, self.lineterminator, self.quoting
        )
    }
}
#[derive(Debug, Clone, PartialEq)]
struct reader {
    _rows: Vec<Vec<String>>,
    _pos: i64,
    dialect: Dialect,
}
impl reader {
    fn new(
        text: String,
        dialect: Option<Dialect>,
        delimiter: String,
        quotechar: String,
        escapechar: String,
        doublequote: bool,
        skipinitialspace: bool,
        quoting: i64,
    ) -> Self {
        let resolved_dialect: Dialect = _resolve_dialect(
            &dialect,
            &delimiter,
            &quotechar,
            &escapechar,
            doublequote,
            skipinitialspace,
            &"\n".to_string(),
            quoting,
        );
        let rows: Vec<Vec<String>> = parse_csv(
            &text,
            &None,
            &format!("{}{}", resolved_dialect.delimiter, ""),
            &format!("{}{}", resolved_dialect.quotechar, ""),
            &format!("{}{}", resolved_dialect.escapechar, ""),
            resolved_dialect.doublequote,
            resolved_dialect.skipinitialspace,
            resolved_dialect.quoting,
        );
        Self {
            dialect: resolved_dialect,
            _rows: rows,
            _pos: 0_i64,
        }
    }
    fn __next__(&mut self) -> Option<Vec<String>> {
        if (self._pos >= (self._rows.len() as i64)) {
            return None;
        }
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
        let Some(mut row) = row else {
            return None;
        };
        let mut result: Vec<String> = vec![];
        for field in row.iter().cloned() {
            result.push(format!("{}{}", field, ""));
        }
        Some(result)
    }
    fn rows(&self) -> Vec<Vec<String>> {
        let mut result: Vec<Vec<String>> = vec![];
        for row in self._rows.clone().iter().cloned() {
            let mut copied: Vec<String> = vec![];
            for field in row.iter().cloned() {
                copied.push(format!("{}{}", field, ""));
            }
            result.push(copied.clone());
        }
        result
    }
    fn line_num(&self) -> i64 {
        self._pos
    }
}
fn _copy_dialect(dialect: &Dialect) -> Dialect {
    Dialect::new(
        format!("{}{}", dialect.delimiter, ""),
        format!("{}{}", dialect.quotechar, ""),
        format!("{}{}", dialect.escapechar, ""),
        dialect.doublequote,
        dialect.skipinitialspace,
        format!("{}{}", dialect.lineterminator, ""),
        dialect.quoting,
    )
}
fn _validate_char(name: &String, value: &String) {
    let _ = (name).clone();
    let _ = (value).clone();
}
fn _resolve_dialect(
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> Dialect {
    if let Some(dialect) = dialect.as_ref() {
        return _copy_dialect(dialect);
    }
    Dialect::new(
        (delimiter).clone(),
        (quotechar).clone(),
        (escapechar).clone(),
        doublequote,
        skipinitialspace,
        (lineterminator).clone(),
        quoting,
    )
}
fn _quotechar_value(dialect: &Dialect) -> String {
    let quotechar: String = {
        let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
        __sifr_concat.push_str((dialect.quotechar).as_str());
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
fn _first_char(text: &String) -> String {
    _char_at(text, 0_i64)
}
fn _last_char(text: &String) -> String {
    let mut __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    _char_at(text, (text.chars().count() as i64) - (1_i64))
}
fn parse_csv(
    text: &String,
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> Vec<Vec<String>> {
    let mut __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let resolved: Dialect = _resolve_dialect(
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
fn _needs_quote(field: &String, dialect: &Dialect) -> bool {
    let mut __sifr_chars_field: Vec<char> = field.chars().collect::<Vec<char>>();
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
    if (field).contains((dialect.delimiter).as_str()) {
        return true;
    }
    if field.contains(&"\n".to_string()) || field.contains(&"\r".to_string()) {
        return true;
    }
    if (dialect.quotechar != "") {
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
fn _quote_field(field: &String, dialect: &Dialect) -> String {
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
            if (dialect.escapechar != "") {
                let escapechar_value: String = {
                    let mut __sifr_concat: String = String::with_capacity(
                        0usize + 0usize,
                    );
                    __sifr_concat.push_str((dialect.escapechar).as_str());
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
fn _escape_unquoted_field(field: &String, dialect: &Dialect) -> String {
    let mut result: String = {
        let mut __sifr_concat: String = String::with_capacity(field.len() + 0usize);
        __sifr_concat.push_str((field).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    };
    if (result).contains((dialect.delimiter).as_str()) {
        if (dialect.escapechar != "") {
            result = result
                .replace(
                    &dialect.delimiter,
                    &format!("{}{}", dialect.escapechar, dialect.delimiter),
                );
        }
    }
    if result.contains(&"\n".to_string()) {
        if (dialect.escapechar != "") {
            result = result.replace('\n', &format!("{}{}", dialect.escapechar, "\n"));
        }
    }
    if result.contains(&"\r".to_string()) {
        if (dialect.escapechar != "") {
            result = result.replace('\r', &format!("{}{}", dialect.escapechar, "\r"));
        }
    }
    if (dialect.quotechar != "") {
        let quotechar2: String = _quotechar_value(dialect);
        if result.contains(&quotechar2) {
            if (dialect.escapechar != "") {
                result = result
                    .replace(
                        &quotechar2,
                        &format!("{}{}", dialect.escapechar, quotechar2),
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
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> String {
    let resolved: Dialect = _resolve_dialect(
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
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> String {
    let resolved: Dialect = _resolve_dialect(
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
fn reader_from_path(
    path: &String,
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    quoting: i64,
) -> Result<reader, IOError> {
    let __sifr_try_res: Result<Result<reader, IOError>, IOError> = (|| {
        let text: String = std::fs::read_to_string(&path).map_err(__io_err)?;
        return Ok(
            Ok(
                reader::new(
                    text,
                    (dialect).clone(),
                    (delimiter).clone(),
                    (quotechar).clone(),
                    (escapechar).clone(),
                    doublequote,
                    skipinitialspace,
                    quoting,
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
            return Err(e);
        }
    }
}
fn writer_to_path(
    path: &String,
    rows: &Vec<Vec<String>>,
    dialect: &Option<Dialect>,
    delimiter: &String,
    quotechar: &String,
    escapechar: &String,
    doublequote: bool,
    skipinitialspace: bool,
    lineterminator: &String,
    quoting: i64,
) -> Result<(), IOError> {
    let payload: String = format_csv(
        rows,
        dialect,
        delimiter,
        quotechar,
        escapechar,
        doublequote,
        skipinitialspace,
        lineterminator,
        quoting,
    );
    std::fs::write(&path, payload.as_bytes()).map(|_| ()).map_err(__io_err)
}

// --- stdlib: sifr.datetime ---
#[derive(Debug, Clone)]
struct timezone {
    _offset: i64,
}
impl timezone {
    fn new(offset: i64) -> Self {
        Self { _offset: offset }
    }
    fn offset(&self) -> i64 {
        self._offset
    }
    fn iso_suffix(&self) -> String {
        let mut sign: String = "+".to_string();
        if (self._offset < (0_i64)) {
            sign = "-".to_string();
        }
        let mut abs_offset: i64 = self._offset;
        if abs_offset < (0_i64) {
            abs_offset = -abs_offset;
        }
        let h: i64 = abs_offset / (3600_i64);
        let m: i64 = (abs_offset % (3600_i64)) / (60_i64);
        let mut hs: String = format!("{}", h);
        if ((hs.chars().count() as i64) < (2_i64)) {
            hs = {
                let mut __sifr_concat: String = String::with_capacity(1usize + hs.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((hs).as_str());
                __sifr_concat
            };
        }
        let mut ms: String = format!("{}", m);
        if ((ms.chars().count() as i64) < (2_i64)) {
            ms = {
                let mut __sifr_concat: String = String::with_capacity(1usize + ms.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((ms).as_str());
                __sifr_concat
            };
        }
        {
            let mut __sifr_concat: String = String::with_capacity(
                ((sign.len() + hs.len()) + 1usize) + ms.len(),
            );
            __sifr_concat.push_str((sign).as_str());
            __sifr_concat.push_str((hs).as_str());
            __sifr_concat.push(':');
            __sifr_concat.push_str((ms).as_str());
            __sifr_concat
        }
    }
}
impl PartialEq for timezone {
    fn eq(&self, other: &timezone) -> bool {
        (self._offset == other._offset)
    }
}
impl std::fmt::Display for timezone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if (self._offset == (0_i64)) {
            return write!(f, "{}", "UTC".to_string());
        }
        write!(
            f, "{}", { let mut __sifr_concat : String = String::with_capacity(3usize +
            0usize); __sifr_concat.push_str("UTC"); __sifr_concat.push_str((self
            .iso_suffix()).as_str()); __sifr_concat }
        )
    }
}
#[derive(Debug, Clone)]
struct datetime {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
    _tz_offset: Option<i64>,
}
impl datetime {
    fn new(
        year: i64,
        month: i64,
        day: i64,
        hour: i64,
        minute: i64,
        second: i64,
        tz_offset: Option<i64>,
    ) -> Self {
        Self {
            year,
            month,
            day,
            hour,
            minute,
            second,
            _tz_offset: tz_offset,
        }
    }
    fn isoformat(&self) -> String {
        let y: String = format!("{}", self.year);
        let mut mo: String = format!("{}", self.month);
        if ((mo.chars().count() as i64) < (2_i64)) {
            mo = {
                let mut __sifr_concat: String = String::with_capacity(1usize + mo.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((mo).as_str());
                __sifr_concat
            };
        }
        let mut d: String = format!("{}", self.day);
        if ((d.chars().count() as i64) < (2_i64)) {
            d = {
                let mut __sifr_concat: String = String::with_capacity(1usize + d.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((d).as_str());
                __sifr_concat
            };
        }
        let mut h: String = format!("{}", self.hour);
        if ((h.chars().count() as i64) < (2_i64)) {
            h = {
                let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((h).as_str());
                __sifr_concat
            };
        }
        let mut mi: String = format!("{}", self.minute);
        if ((mi.chars().count() as i64) < (2_i64)) {
            mi = {
                let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((mi).as_str());
                __sifr_concat
            };
        }
        let mut s: String = format!("{}", self.second);
        if ((s.chars().count() as i64) < (2_i64)) {
            s = {
                let mut __sifr_concat: String = String::with_capacity(1usize + s.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((s).as_str());
                __sifr_concat
            };
        }
        let base: String = {
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
        };
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(mut tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            let mut sign: String = "+".to_string();
            let mut abs_offset: i64 = offset;
            if abs_offset < (0_i64) {
                sign = "-".to_string();
                abs_offset = -abs_offset;
            }
            let h_off: i64 = abs_offset / (3600_i64);
            let m_off: i64 = (abs_offset % (3600_i64)) / (60_i64);
            let mut hs_off: String = format!("{}", h_off);
            if ((hs_off.chars().count() as i64) < (2_i64)) {
                hs_off = {
                    let mut __sifr_concat: String = String::with_capacity(
                        1usize + hs_off.len(),
                    );
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((hs_off).as_str());
                    __sifr_concat
                };
            }
            let mut ms_off: String = format!("{}", m_off);
            if ((ms_off.chars().count() as i64) < (2_i64)) {
                ms_off = {
                    let mut __sifr_concat: String = String::with_capacity(
                        1usize + ms_off.len(),
                    );
                    __sifr_concat.push('0');
                    __sifr_concat.push_str((ms_off).as_str());
                    __sifr_concat
                };
            }
            return {
                let mut __sifr_concat: String = String::with_capacity(
                    (((base.len() + sign.len()) + hs_off.len()) + 1usize) + ms_off.len(),
                );
                __sifr_concat.push_str((base).as_str());
                __sifr_concat.push_str((sign).as_str());
                __sifr_concat.push_str((hs_off).as_str());
                __sifr_concat.push(':');
                __sifr_concat.push_str((ms_off).as_str());
                __sifr_concat
            };
        }
        base
    }
    fn timestamp(&self) -> i64 {
        let mut days: i64 = 0_i64;
        if (self.year >= (1970_i64)) {
            let mut y: i64 = 1970_i64;
            while (y < self.year) {
                days += _days_in_year(y);
                y += 1_i64;
            }
        } else {
            let mut y: i64 = 1969_i64;
            while (y >= self.year) {
                days -= _days_in_year(y);
                y -= 1_i64;
            }
        }
        let mut m: i64 = 1_i64;
        while (m < self.month) {
            days += _days_in_month(self.year, m);
            m += 1_i64;
        }
        days = (days + self.day) - (1_i64);
        let naive_timestamp: i64 = (((days * (86400_i64)) + (self.hour * (3600_i64)))
            + (self.minute * (60_i64))) + self.second;
        let tz_offset_opt: Option<i64> = self._tz_offset;
        if let Some(mut tz_offset_opt) = tz_offset_opt {
            let offset: i64 = tz_offset_opt;
            return naive_timestamp - offset;
        }
        naive_timestamp
    }
    fn astimezone(&self, tz: &Option<timezone>) -> Result<datetime, ValueError> {
        let mut target: timezone = timezone::new(0_i64);
        if let Some(tz) = tz.as_ref() {
            let __sifr_try_res: Result<(), ValueError> = (|| {
                let tz_text: String = format!("{}", tz);
                let target_offset: i64 = _timezone_offset_from_text(&tz_text)?;
                target = timezone::new(target_offset);
                Ok(())
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(ValueError::new(e.message));
            }
        }
        from_timestamp(self.timestamp() as f64, &Some((target).clone()))
    }
}
impl PartialEq for datetime {
    fn eq(&self, other: &datetime) -> bool {
        let same_tz: bool = (self._tz_offset == other._tz_offset);
        ((((((((self.year == other.year)) && ((self.month == other.month)))
            && ((self.day == other.day))) && ((self.hour == other.hour)))
            && ((self.minute == other.minute))) && ((self.second == other.second)))
            && (same_tz))
    }
}
impl std::fmt::Display for datetime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.isoformat())
    }
}
#[derive(Debug, Clone)]
struct time {
    hour: i64,
    minute: i64,
    second: i64,
}
impl time {
    fn new(hour: i64, minute: i64, second: i64) -> Self {
        Self { hour, minute, second }
    }
    fn isoformat(&self) -> String {
        let mut h: String = format!("{}", self.hour);
        if ((h.chars().count() as i64) < (2_i64)) {
            h = {
                let mut __sifr_concat: String = String::with_capacity(1usize + h.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((h).as_str());
                __sifr_concat
            };
        }
        let mut mi: String = format!("{}", self.minute);
        if ((mi.chars().count() as i64) < (2_i64)) {
            mi = {
                let mut __sifr_concat: String = String::with_capacity(1usize + mi.len());
                __sifr_concat.push('0');
                __sifr_concat.push_str((mi).as_str());
                __sifr_concat
            };
        }
        let mut s: String = format!("{}", self.second);
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
                (((h.len() + 1usize) + mi.len()) + 1usize) + s.len(),
            );
            __sifr_concat.push_str((h).as_str());
            __sifr_concat.push(':');
            __sifr_concat.push_str((mi).as_str());
            __sifr_concat.push(':');
            __sifr_concat.push_str((s).as_str());
            __sifr_concat
        }
    }
}
impl PartialEq for time {
    fn eq(&self, other: &time) -> bool {
        ((((self.hour == other.hour)) && ((self.minute == other.minute)))
            && ((self.second == other.second)))
    }
}
impl std::fmt::Display for time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.isoformat())
    }
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
    if let Some(mut d) = d {
        return d;
    }
    0_i64
}
fn _substring(value: &String, start: i64, end: i64) -> String {
    let mut __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    let mut result: String = "".to_string();
    let mut i: i64 = start;
    while i < end {
        let ch: Option<String> = __sifr_chars_value
            .get(i as usize)
            .map(|c| c.to_string());
        if let Some(mut ch) = ch {
            result.push_str((ch).as_str());
        }
        i += 1_i64;
    }
    result
}
fn _parse_datetime_iso(
    value: &String,
) -> Result<(i64, i64, i64, i64, i64, i64), ValueError> {
    let mut __sifr_chars_value: Vec<char> = value.chars().collect::<Vec<char>>();
    if ((__sifr_chars_value.len() as i64) < (19_i64)) {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    if ((((({
        let Some(__indexed_char) = __sifr_chars_value
            .get((4_i64) as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    }) != "-")
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((7_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "-"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((10_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != "T"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((13_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":"))
        || (({
            let Some(__indexed_char) = __sifr_chars_value
                .get((16_i64) as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        }) != ":")
    {
        return Err(ValueError::new("invalid datetime string".to_string()));
    }
    let __sifr_try_res: Result<
        Result<(i64, i64, i64, i64, i64, i64), ValueError>,
        ParseError,
    > = (|| {
        let year: i64 = (_substring(value, 0_i64, 4_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let month: i64 = (_substring(value, 5_i64, 7_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let day: i64 = (_substring(value, 8_i64, 10_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let hour: i64 = (_substring(value, 11_i64, 13_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minute: i64 = (_substring(value, 14_i64, 16_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let second: i64 = (_substring(value, 17_i64, 19_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        return Ok(Ok((year, month, day, hour, minute, second)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid datetime string".to_string()));
        }
    }
}
fn _timezone_offset_from_text(text: &String) -> Result<i64, ValueError> {
    let mut __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    if (text).as_str() == "UTC" {
        return Ok(0_i64);
    }
    if ((__sifr_chars_text.len() as i64) != (9_i64)) {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (_substring(text, 0_i64, 3_i64) != "UTC") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let sign_value: String = _substring(text, 3_i64, 4_i64);
    if (sign_value != "+") && (sign_value != "-") {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    if (__sifr_chars_text.get((6_i64) as usize).map(|c| c.to_string())
        != Some(":".to_string()))
    {
        return Err(ValueError::new("invalid timezone string".to_string()));
    }
    let __sifr_try_res: Result<Result<i64, ValueError>, ParseError> = (|| {
        let hours: i64 = (_substring(text, 4_i64, 6_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let minutes: i64 = (_substring(text, 7_i64, 9_i64))
            .parse::<i64>()
            .map_err(|e| ParseError {
                message: e.to_string(),
            })?;
        let mut offset: i64 = (hours * (3600_i64)) + (minutes * (60_i64));
        if sign_value == "-" {
            offset = -offset;
        }
        return Ok(Ok(offset));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            return Err(ValueError::new("invalid timezone string".to_string()));
        }
    }
}
fn _from_timestamp_with_tz(
    ts: f64,
    tz: &Option<timezone>,
) -> Result<datetime, ValueError> {
    let __sifr_try_res: Result<Result<datetime, ValueError>, ValueError> = (|| {
        let whole_seconds: i64 = ts as i64;
        let mut adjusted_seconds: i64 = whole_seconds;
        let mut tz_offset_value: i64 = 0_i64;
        let mut tz_has_offset: bool = false;
        if let Some(tz) = tz.as_ref() {
            let tz_text: String = format!("{}", tz);
            let tz_offset: i64 = _timezone_offset_from_text(&tz_text)?;
            adjusted_seconds = whole_seconds + tz_offset;
            tz_offset_value = tz_offset;
            tz_has_offset = true;
        }
        let rendered: String = ({
            let __ts = (adjusted_seconds as f64) as i64;
            chrono::DateTime::from_timestamp(__ts, 0)
                .map(|dt| dt.format(&"%Y-%m-%dT%H:%M:%S".to_string()).to_string())
                .ok_or_else(|| ValueError {
                    message: "invalid timestamp".to_string(),
                })
        })?;
        let parts: (i64, i64, i64, i64, i64, i64) = _parse_datetime_iso(&rendered)?;
        let year_part: Option<i64> = Some((parts).0);
        let month_part: Option<i64> = Some((parts).1);
        let day_part: Option<i64> = Some((parts).2);
        let hour_part: Option<i64> = Some((parts).3);
        let minute_part: Option<i64> = Some((parts).4);
        let second_part: Option<i64> = Some((parts).5);
        let mut year: i64 = 0_i64;
        let mut month: i64 = 1_i64;
        let mut day: i64 = 1_i64;
        let mut hour: i64 = 0_i64;
        let mut minute: i64 = 0_i64;
        let mut second: i64 = 0_i64;
        if let Some(mut year_part) = year_part {
            year = year_part;
        }
        if let Some(mut month_part) = month_part {
            month = month_part;
        }
        if let Some(mut day_part) = day_part {
            day = day_part;
        }
        if let Some(mut hour_part) = hour_part {
            hour = hour_part;
        }
        if let Some(mut minute_part) = minute_part {
            minute = minute_part;
        }
        if let Some(mut second_part) = second_part {
            second = second_part;
        }
        if tz_has_offset {
            return Ok(
                Ok(
                    datetime::new(
                        year,
                        month,
                        day,
                        hour,
                        minute,
                        second,
                        Some(tz_offset_value),
                    ),
                ),
            );
        }
        return Ok(Ok(datetime::new(year, month, day, hour, minute, second, None)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(ValueError::new(e.message));
        }
    }
}
fn now(tz: &Option<timezone>) -> datetime {
    let current_epoch: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let __sifr_try_res: Result<datetime, ValueError> = (|| {
        let current: datetime = _from_timestamp_with_tz(current_epoch, tz)?;
        return Ok(current);
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let _e = __sifr_try_err.clone();
            let parts: Vec<i64> = {
                let __dt = chrono::Local::now();
                vec![
                    chrono::Datelike::year(& __dt) as i64, chrono::Datelike::month(&
                    __dt) as i64, chrono::Datelike::day(& __dt) as i64,
                    chrono::Timelike::hour(& __dt) as i64, chrono::Timelike::minute(&
                    __dt) as i64, chrono::Timelike::second(& __dt) as i64
                ]
            };
            let mut yr: i64 = 0_i64;
            let mut mo: i64 = 1_i64;
            let mut dy: i64 = 1_i64;
            let mut hr: i64 = 0_i64;
            let mut mn: i64 = 0_i64;
            let mut sc: i64 = 0_i64;
            for (i, v) in Box::new(
                (parts)
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
            ) {
                if i == (0_i64) {
                    yr = v;
                }
                if i == (1_i64) {
                    mo = v;
                }
                if i == (2_i64) {
                    dy = v;
                }
                if i == (3_i64) {
                    hr = v;
                }
                if i == (4_i64) {
                    mn = v;
                }
                if i == (5_i64) {
                    sc = v;
                }
            }
            if let Some(tz) = tz.as_ref() {
                let __sifr_try_res: Result<datetime, ValueError> = (|| {
                    let parsed_offset: i64 = _timezone_offset_from_text(
                        &format!("{}", tz),
                    )?;
                    return Ok(
                        datetime::new(yr, mo, dy, hr, mn, sc, Some(parsed_offset)),
                    );
                    unreachable!("sifr try/except return capture fell through");
                })();
                match __sifr_try_res {
                    Ok(__sifr_ret_val) => {
                        return __sifr_ret_val;
                    }
                    Err(__sifr_try_err) => {
                        let _e = __sifr_try_err.clone();
                        return datetime::new(yr, mo, dy, hr, mn, sc, None);
                    }
                }
            }
            return datetime::new(yr, mo, dy, hr, mn, sc, None);
        }
    }
}
fn from_timestamp(ts: f64, tz: &Option<timezone>) -> Result<datetime, ValueError> {
    _from_timestamp_with_tz(ts, tz)
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
fn basicConfig(level: i64) -> Logger {
    {
        *__SIFR_GLOBAL_LOG_LEVEL.lock().unwrap_or_else(|__err| __err.into_inner()) = level;
        ()
    };
    Logger::new("root".to_string(), level)
}
fn getLogger(name: &String) -> Logger {
    let level: i64 = *__SIFR_GLOBAL_LOG_LEVEL
        .lock()
        .unwrap_or_else(|__err| __err.into_inner());
    Logger::new((name).clone(), level)
}

// --- stdlib: sifr.pathlib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Path {
    _path: String,
}
impl Path {
    fn new(path: String) -> Self {
        Self { _path: path }
    }
    fn name(&self) -> String {
        basename(&self._path)
    }
    fn parent(&self) -> Path {
        Path::new(dirname(&self._path))
    }
    fn suffix(&self) -> String {
        extension(&self._path)
    }
    fn stem(&self) -> String {
        stem(&self._path)
    }
    fn exists(&self) -> bool {
        std::path::Path::new(&self._path.clone()).exists()
    }
    fn is_file(&self) -> bool {
        std::path::Path::new(&self._path.clone()).is_file()
    }
    fn is_dir(&self) -> bool {
        std::path::Path::new(&self._path.clone()).is_dir()
    }
    fn is_absolute(&self) -> bool {
        is_absolute(&self._path)
    }
    fn read_text(&self) -> Result<String, IOError> {
        std::fs::read_to_string(&self._path.clone()).map_err(__io_err)
    }
    fn write_text(&self, content: &String) -> Result<(), IOError> {
        std::fs::write(&self._path.clone(), content.as_bytes())
            .map(|_| ())
            .map_err(__io_err)
    }
    fn mkdir(&self) -> Result<(), IOError> {
        std::fs::create_dir_all(&self._path.clone()).map(|_| ()).map_err(__io_err)
    }
    fn joinpath(&self, child: &String) -> Path {
        Path::new(join_path(&self._path, child))
    }
    fn to_str(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._path.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    fn touch(&self) -> Result<(), IOError> {
        std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err)
    }
    fn unlink(&self) -> Result<(), IOError> {
        std::fs::remove_file(&self._path.clone()).map(|_| ()).map_err(__io_err)
    }
    fn rmdir(&self) -> Result<(), IOError> {
        std::fs::remove_dir(&self._path.clone()).map(|_| ()).map_err(__io_err)
    }
    fn resolve(&self) -> Result<String, IOError> {
        std::fs::canonicalize(&self._path.clone())
            .map(|p| p.to_string_lossy().to_string())
            .map_err(__io_err)
    }
    fn iterdir(&self) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        _iterdir_to_iter(&self._path)
    }
    fn with_name(&self, name: &String) -> Path {
        let parent: String = dirname(&self._path);
        if parent == "" {
            return Path::new(format!("{}{}", name, ""));
        }
        Path::new(format!("{}{}", format!("{}{}", parent, "/"), name))
    }
    fn with_suffix(&self, suffix: &String) -> Path {
        let s: String = stem(&self._path);
        let parent: String = dirname(&self._path);
        if parent == "" {
            return Path::new(format!("{}{}", s, suffix));
        }
        Path::new(
            format!("{}{}", format!("{}{}", format!("{}{}", parent, "/"), s), suffix),
        )
    }
    fn glob(
        &self,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        _glob_to_iter(&self._path, pattern)
    }
    fn rglob(
        &self,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        _rglob_to_iter(&self._path, pattern)
    }
}
impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Path(_path={})", self._path)
    }
}
fn join_path(base: &String, child: &String) -> String {
    let mut __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    if ((__sifr_chars_base.len() as i64) == (0_i64)) {
        return {
            let mut __sifr_concat: String = String::with_capacity(child.len() + 0usize);
            __sifr_concat.push_str((child).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    }
    let last: Option<String> = __sifr_chars_base
        .get(((base.chars().count() as i64) - (1_i64)) as usize)
        .map(|c| c.to_string());
    if let Some(mut last) = last {
        if last == "/" {
            return {
                let mut __sifr_concat: String = String::with_capacity(
                    base.len() + child.len(),
                );
                __sifr_concat.push_str((base).as_str());
                __sifr_concat.push_str((child).as_str());
                __sifr_concat
            };
        }
    }
    {
        let mut __sifr_concat: String = String::with_capacity(
            (base.len() + 1usize) + child.len(),
        );
        __sifr_concat.push_str((base).as_str());
        __sifr_concat.push('/');
        __sifr_concat.push_str((child).as_str());
        __sifr_concat
    }
}
fn basename(path: &String) -> String {
    let mut __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(mut ch) = ch {
            if ch == "/" {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = if (i + (1_i64)) < 0 {
                        (_slice_len_i64 + (i + (1_i64))).max(0)
                    } else {
                        (i + (1_i64)).min(_slice_len_i64)
                    };
                    let _slice_stop_i64 = _slice_len_i64;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
    }
    {
        let mut __sifr_concat: String = String::with_capacity(path.len() + 0usize);
        __sifr_concat.push_str((path).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn dirname(path: &String) -> String {
    let mut __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(mut ch) = ch {
            if ch == "/" {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = 0;
                    let _slice_stop_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
    }
    "".to_string()
}
fn extension(path: &String) -> String {
    let mut __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_path.len() as i64) - (1_i64);
    while i >= (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_path
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(mut ch) = ch {
            if ch == "." {
                return {
                    let _slice_src = &__sifr_chars_path;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    let _slice_stop_i64 = _slice_len_i64;
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
            if ch == "/" {
                return "".to_string();
            }
        }
        i -= 1_i64;
    }
    "".to_string()
}
fn stem(path: &String) -> String {
    let base: String = basename(path);
    let mut __sifr_chars_base: Vec<char> = base.chars().collect::<Vec<char>>();
    let mut i: i64 = (__sifr_chars_base.len() as i64) - (1_i64);
    while i > (0_i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = __sifr_chars_base
                .get(i as usize)
                .map(|c| c.to_string()) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char
        });
        if let Some(mut ch) = ch {
            if ch == "." {
                return {
                    let _slice_src = &__sifr_chars_base;
                    let _slice_len_i64 = _slice_src.len() as i64;
                    let _slice_start_i64 = 0;
                    let _slice_stop_i64 = if i < 0 {
                        (_slice_len_i64 + i).max(0)
                    } else {
                        i.min(_slice_len_i64)
                    };
                    String::from_iter(
                        _slice_src
                            .iter()
                            .skip(_slice_start_i64 as usize)
                            .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                            .copied(),
                    )
                };
            }
        }
        i -= 1_i64;
    }
    {
        let mut __sifr_concat: String = String::with_capacity(base.len() + 0usize);
        __sifr_concat.push_str((base).as_str());
        __sifr_concat.push_str("");
        __sifr_concat
    }
}
fn is_absolute(path: &String) -> bool {
    let mut __sifr_chars_path: Vec<char> = path.chars().collect::<Vec<char>>();
    if ((__sifr_chars_path.len() as i64) == (0_i64)) {
        return false;
    }
    if ((__sifr_chars_path.len() as i64) >= (3_i64)) {
        let colon: Option<String> = __sifr_chars_path
            .get((1_i64) as usize)
            .map(|c| c.to_string());
        let sep: Option<String> = __sifr_chars_path
            .get((2_i64) as usize)
            .map(|c| c.to_string());
        if let Some(mut colon) = colon {
            if let Some(mut sep) = sep {
                if (colon == ":") && ((sep == "/") || (sep == "\\")) {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = Some({
        let Some(__indexed_char) = __sifr_chars_path
            .get((0_i64) as usize)
            .map(|c| c.to_string()) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char
    });
    if let Some(mut first) = first {
        if (first == "/") || (first == "\\") {
            return true;
        }
    }
    false
}
fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<String> = Vec::new().into_iter();
    Box::new(
        std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<String> = Vec::new();
                let mut i: i64 = 0_i64;
                while (i < (entries.len() as i64)) {
                    _yields.push(entries[i as usize].clone());
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn _iterdir_list(path: &String) -> Result<Vec<String>, IOError> {
    {
        let __entries = std::fs::read_dir(&path).map_err(__io_err)?;
        Ok(
            __entries
                .filter_map(|e| e.ok().map(|e| e.path().to_string_lossy().to_string()))
                .collect::<Vec<String>>(),
        )
    }
}
fn _glob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    {
        let __dir = &path;
        let __pat = &pattern;
        let __include_hidden = __pat.starts_with(".");
        let __regex_src = format!(
            "^{}$", regex::escape(__pat).replace("\\*", ".*").replace("\\?", ".")
        );
        let __re = regex::Regex::new(&__regex_src)
            .map_err(|e| IOError::new(e.to_string()))?;
        let mut __results: Vec<String> = Vec::new();
        match std::fs::read_dir(__dir) {
            Ok(__entries) => {
                for __entry in __entries {
                    if let Ok(__e) = __entry {
                        let __name = __e
                            .file_name()
                            .to_string_lossy()
                            .to_string()
                            .to_string();
                        if !__include_hidden && __name.starts_with(".") {
                            continue;
                        }
                        if __re.is_match(&__name) {
                            __results.push(__e.path().to_string_lossy().to_string());
                        }
                    } else {
                        continue;
                    }
                }
            }
            Err(_) => {
                return Ok(Vec::new());
            }
        }
        __results.sort();
        Ok(__results)
    }
}
fn _rglob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    {
        let __dir = &path;
        let __pat = &pattern;
        let __include_hidden = __pat.starts_with(".");
        let __regex_src = format!(
            "^{}$", regex::escape(__pat).replace("\\*", ".*").replace("\\?", ".")
        );
        let __re = regex::Regex::new(&__regex_src)
            .map_err(|e| IOError::new(e.to_string()))?;
        let mut __results: Vec<String> = Vec::new();
        let mut __stack: Vec<String> = vec![__dir.to_string()];
        loop {
            if let Some(__current) = __stack.pop() {
                let __entries_result = std::fs::read_dir(&__current);
                if let Ok(__entries) = __entries_result {
                    for __entry in __entries {
                        if let Ok(__e) = __entry {
                            let __path = __e.path();
                            let __name = __e
                                .file_name()
                                .to_string_lossy()
                                .to_string()
                                .to_string();
                            if !__include_hidden && __name.starts_with(".") {
                                continue;
                            }
                            if __path.is_dir() {
                                __stack.push(__path.to_string_lossy().to_string());
                            }
                            if __re.is_match(&__name) {
                                __results.push(__path.to_string_lossy().to_string());
                            }
                        } else {
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            } else {
                break;
            }
        }
        __results.sort();
        Ok(__results)
    }
}
fn _iterdir_to_iter(path: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _iterdir_list(path)?;
        return Ok(Ok(_iter_list_str(entries)));
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
fn _glob_to_iter(
    path: &String,
    pattern: &String,
) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _glob_list(path, pattern)?;
        return Ok(Ok(_iter_list_str(entries)));
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
fn _rglob_to_iter(
    path: &String,
    pattern: &String,
) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = String>>, IOError>,
        IOError,
    > = (|| {
        let entries: Vec<String> = _rglob_list(path, pattern)?;
        return Ok(Ok(_iter_list_str(entries)));
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

// --- stdlib: sifr.bytes ---
fn decode_utf8(data: &Vec<u8>) -> Result<String, ParseError> {
    sifr_runtime::encoding::decode_text(
            &data,
            &"utf-8".to_string(),
            &"strict".to_string(),
        )
        .map_err(|__message| ParseError { message: __message })
}
fn bytes_from_hex(s: &String) -> Result<Vec<u8>, ParseError> {
    {
        let s: String = s.to_string();
        let mut cleaned = String::new();
        for ch in s.chars() {
            if ch.is_ascii_whitespace() {
                continue;
            }
            if !ch.is_ascii_hexdigit() {
                return Err(ParseError {
                    message: format!("invalid hex character: {}", ch),
                });
            }
            cleaned.push(ch);
        }
        if (cleaned.len() % 2) != 0 {
            return Err(ParseError {
                message: "fromhex() arg must contain an even number of hexadecimal digits"
                    .to_string()
                    .to_string(),
            });
        }
        let mut result = Vec::new();
        for pair in cleaned.as_bytes().chunks(2) {
            let pair_str = std::str::from_utf8(pair)
                .map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
            result
                .push(
                    u8::from_str_radix(pair_str, 16)
                        .map_err(|e| ParseError {
                            message: e.to_string(),
                        })?,
                );
        }
        Ok::<Vec<u8>, ParseError>(result)
    }
}
fn bytes_from_ints(values: &Vec<i64>) -> Result<Vec<u8>, ValueError> {
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
fn bytes_with_size(size: i64) -> Result<Vec<u8>, ValueError> {
    {
        let __size = size;
        if __size < 0 {
            return Err(ValueError {
                message: "bytes(size) requires a non-negative size"
                    .to_string()
                    .to_string(),
            });
        }
        Ok::<Vec<u8>, ValueError>((0..__size).map(|_| 0_u8).collect::<Vec<u8>>())
    }
}
fn encode_utf8_result(s: &String) -> Result<Vec<u8>, ParseError> {
    sifr_runtime::encoding::encode_bytes(&s, &"utf-8".to_string(), &"strict".to_string())
        .map_err(|__message| ParseError { message: __message })
}
fn count_byte(data: &Vec<u8>, value: i64) -> i64 {
    let mut count: i64 = 0_i64;
    for b in data.iter().map(|__byte| *__byte as u8) {
        if ((b as i64) == value) {
            count += 1_i64;
        }
    }
    count
}
fn find_byte(data: &Vec<u8>, value: i64) -> Option<i64> {
    let mut idx: i64 = 0_i64;
    for b in data.iter().map(|__byte| *__byte as u8) {
        if ((b as i64) == value) {
            return Some(idx);
        }
        idx += 1_i64;
    }
    None
}
fn starts_with(data: &Vec<u8>, prefix: &Vec<u8>) -> bool {
    if (prefix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let mut i: i64 = 0_i64;
    while (i < (prefix.len() as i64)) {
        let a: Option<u8> = data.get(i as usize).map(|__byte| *__byte as u8);
        let b: Option<u8> = Some(prefix[i as usize] as u8);
        let Some(mut a) = a else {
            return false;
        };
        let Some(mut b) = b else {
            return false;
        };
        if (a != b) {
            return false;
        }
        i += 1_i64;
    }
    true
}
fn ends_with(data: &Vec<u8>, suffix: &Vec<u8>) -> bool {
    if (suffix.len() as i64) > (data.len() as i64) {
        return false;
    }
    let offset: i64 = (data.len() as i64) - (suffix.len() as i64);
    let mut i: i64 = 0_i64;
    while (i < (suffix.len() as i64)) {
        let a: Option<u8> = data.get((offset + i) as usize).map(|__byte| *__byte as u8);
        let b: Option<u8> = Some(suffix[i as usize] as u8);
        let Some(mut a) = a else {
            return false;
        };
        let Some(mut b) = b else {
            return false;
        };
        if (a != b) {
            return false;
        }
        i += 1_i64;
    }
    true
}

// --- stdlib: sifr.math ---
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
    (x).ln() / (base).ln()
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
    if (a).is_nan() || (b).is_nan() {
        return false;
    }
    if (a).is_infinite() || (b).is_infinite() {
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
    let mut rel_bound: f64 = rel_tol * (a_abs).max(b_abs);
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
fn frexp_mantissa(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1_u64) << 63;
                    let __frac_mask: u64 = ((1_u64) << 52) - (1_u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047_u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0_f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047_u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022_u64) << 52)) | __sfrac,
                            );
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022_u64) << 52)) | __frac,
                            );
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
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
    let Some(mut m) = m else {
        return f64::NAN;
    };
    m
}
fn frexp_exponent(x: f64) -> i64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x == 0.0 {
            vec![__x, 0.0]
        } else {
            if !__x.is_finite() {
                vec![__x, 0.0]
            } else {
                {
                    let __bits: u64 = __x.to_bits();
                    let __sign_mask: u64 = (1_u64) << 63;
                    let __frac_mask: u64 = ((1_u64) << 52) - (1_u64);
                    let __sign: u64 = __bits & __sign_mask;
                    let __exp: i32 = ((__bits >> 52) & (2047_u64)) as i32;
                    let __frac: u64 = __bits & __frac_mask;
                    if __exp == 0 {
                        {
                            let __scaled: f64 = __x * (2.0_f64).powi(54);
                            let __sbits: u64 = __scaled.to_bits();
                            let __sexp: i32 = ((__sbits >> 52) & (2047_u64)) as i32;
                            let __sfrac: u64 = __sbits & __frac_mask;
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022_u64) << 52)) | __sfrac,
                            );
                            let __e: i32 = (__sexp - 1022) - 54;
                            vec![__mant, __e as f64]
                        }
                    } else {
                        {
                            let __mant: f64 = f64::from_bits(
                                (__sign | ((1022_u64) << 52)) | __frac,
                            );
                            let __e: i32 = __exp - 1022;
                            vec![__mant, __e as f64]
                        }
                    }
                }
            }
        }
    };
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
    let Some(mut exp_val) = exp_val else {
        return 0_i64;
    };
    (exp_val).trunc() as i64
}
fn modf_fractional(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0_f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0_f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
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
    let Some(mut f) = f else {
        return f64::NAN;
    };
    f
}
fn modf_integral(x: f64) -> f64 {
    let parts: Vec<f64> = {
        let __x: f64 = x as f64;
        if __x.is_nan() {
            vec![f64::NAN, f64::NAN]
        } else {
            if __x.is_infinite() {
                vec![(0.0_f64).copysign(__x), __x]
            } else {
                {
                    let __int = __x.trunc();
                    let mut __frac = __x - __int;
                    if __frac == 0.0 {
                        __frac = (0.0_f64).copysign(__x);
                    }
                    vec![__frac, __int]
                }
            }
        }
    };
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
    let Some(mut i) = i else {
        return f64::NAN;
    };
    i
}
fn pow(x: f64, y: f64) -> f64 {
    (x).powf(y)
}

// --- stdlib: sifr.random ---
#[derive(Debug, Clone)]
struct __SifrRandomModuleState {
    words: Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
}
static __SIFR_RANDOM_MODULE_STATE: std::sync::LazyLock<
    std::sync::Mutex<__SifrRandomModuleState>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(__SifrRandomModuleState {
    words: Vec::new(),
    index: 0,
    gauss_next: None,
}));
const _MT_N: i64 = 624_i64;
const _MT_M: i64 = 397_i64;
const _MT_MATRIX_A: i64 = 2567483615_i64;
const _MT_UPPER_MASK: i64 = 2147483648_i64;
const _MT_LOWER_MASK: i64 = 2147483647_i64;
const _MT_F: i64 = 1812433253_i64;
const _MT_WORD_MASK: i64 = 4294967295_i64;
#[derive(Debug, Clone, PartialEq)]
struct RandomState {
    version: i64,
    state_words: Vec<i64>,
    index: i64,
    gauss_next: Option<f64>,
}
impl RandomState {
    fn new(
        version: i64,
        state_words: Vec<i64>,
        index: i64,
        gauss_next: Option<f64>,
    ) -> Self {
        Self {
            version,
            state_words,
            index,
            gauss_next,
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
struct Random {
    _state_words: Vec<i64>,
    _index: i64,
    _gauss_next: Option<f64>,
}
impl Random {
    fn new(seed_value: Option<i64>) -> Self {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        Self {
            _state_words: _seed_words_from_seed(normalized_seed),
            _index: _MT_N,
            _gauss_next: None,
        }
    }
    fn seed(&mut self, seed_value: Option<i64>) {
        let normalized_seed: i64 = _normalize_seed_input(seed_value);
        self._state_words = _seed_words_from_seed(normalized_seed);
        self._index = _MT_N;
        self._gauss_next = None;
    }
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
    fn random(&mut self) -> f64 {
        (self._next_u32() as f64) / (4294967296.0_f64)
    }
    fn uniform(&mut self, minimum: f64, maximum: f64) -> f64 {
        minimum + ((maximum - minimum) * self.random())
    }
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
            if let Some(mut stop) = stop {
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
    fn randint(&mut self, minimum: i64, maximum: i64) -> Result<i64, ValueError> {
        if minimum > maximum {
            return Err(ValueError::new("randint: min must be <= max".to_string()));
        }
        self.randrange(minimum, Some(maximum + (1_i64)), 1_i64)
    }
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
    fn gauss(&mut self, mu: f64, sigma: f64) -> f64 {
        let cached: Option<f64> = self._gauss_next;
        if let Some(mut cached) = cached {
            self._gauss_next = None;
            return mu + (sigma * cached);
        }
        let mut u1: f64 = self.random();
        if u1 <= (0.0_f64) {
            u1 = 0.000000000001_f64;
        }
        let u2: f64 = self.random();
        let radius: f64 = (-(2.0_f64) * (u1).ln()).sqrt();
        let theta: f64 = ((2.0_f64) * std::f64::consts::PI) * u2;
        let z0: f64 = radius * (theta).cos();
        let z1: f64 = radius * (theta).sin();
        let next_cached: Option<f64> = Some(z1);
        self._gauss_next = next_cached;
        mu + (sigma * z0)
    }
    fn getstate(&self) -> RandomState {
        RandomState::new(
            3_i64,
            _clone_words(&self._state_words),
            self._index,
            self._gauss_next,
        )
    }
    fn setstate(&mut self, state: &RandomState) -> Result<(), ValueError> {
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
        for word in state.state_words.iter().copied() {
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
    if let Some(mut value) = value {
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
    if let Some(mut seed_value) = seed_value {
        return seed_value;
    }
    (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64() * (1000000.0_f64)) as i64
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
fn _build_state_from_module_storage() -> RandomState {
    RandomState::new(
        3_i64,
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.words.clone()
        },
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.index
        },
        {
            let __state = __SIFR_RANDOM_MODULE_STATE
                .lock()
                .unwrap_or_else(|__err| __err.into_inner());
            __state.gauss_next.clone()
        },
    )
}
fn _store_state_into_module_storage(state: &RandomState) {
    let _set_result: Result<(), ValueError> = {
        let __words = _clone_words(&state.state_words);
        let __index = state.index;
        let __gauss_next = state.gauss_next;
        if (__index < 0) || (__index > 624) {
            Err(ValueError {
                message: "random module state index must be in range [0, 624]"
                    .to_string(),
            })
        } else {
            if __words.len() != 624 {
                Err(ValueError {
                    message: "random module state words must have length 624".to_string(),
                })
            } else {
                {
                    let mut __state = __SIFR_RANDOM_MODULE_STATE
                        .lock()
                        .unwrap_or_else(|__err| __err.into_inner());
                    __state.words = __words;
                    __state.index = __index;
                    __state.gauss_next = __gauss_next;
                    Ok(())
                }
            }
        }
    };
    let _ = _set_result;
}
fn _ensure_module_state_initialized() {
    let words: Vec<i64> = {
        let __state = __SIFR_RANDOM_MODULE_STATE
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        __state.words.clone()
    };
    if (words.len() as i64) == _MT_N {
        return;
    }
    let mut bootstrap: Random = Random::new(Some(5489_i64));
    _store_state_into_module_storage(&bootstrap.getstate());
}
fn _module_random() -> Random {
    _ensure_module_state_initialized();
    let mut r: Random = Random::new(Some(0_i64));
    let __sifr_try_res: Result<(), ValueError> = (|| {
        let _set_result: Result<(), ValueError> = r
            .setstate(&_build_state_from_module_storage());
        let _ = _set_result;
        Ok(())
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _ = e.message;
    }
    r
}
fn _sync_module_random(generator: &mut Random) {
    _store_state_into_module_storage(&generator.getstate());
}
fn choice<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
) -> Result<T, ValueError> {
    if ((items.len() as i64) == (0_i64)) {
        return Err(ValueError::new("choice: items must not be empty".to_string()));
    }
    let mut generator: Random = _module_random();
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
    if let Some(mut picked) = picked {
        return Ok(picked);
    }
    Err(ValueError::new("choice: index out of range".to_string()))
}

// --- stdlib: sifr.re ---
const IGNORECASE: i64 = 2_i64;
const MULTILINE: i64 = 8_i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Match {
    _matched: String,
    _start: i64,
    _end: i64,
}
impl Match {
    fn new(matched: String, start: i64, end: i64) -> Self {
        Self {
            _matched: matched,
            _start: start,
            _end: end,
        }
    }
    fn group(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._matched.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
    fn start(&self) -> i64 {
        self._start
    }
    fn end(&self) -> i64 {
        self._end
    }
    fn span(&self) -> Vec<i64> {
        let result: Vec<i64> = vec![self._start, self._end];
        result
    }
    fn to_str(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._matched.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "Match(_matched={}, _start={}, _end={})", self._matched, self._start, self
            ._end
        )
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pattern {
    _pattern: String,
    _flags: i64,
}
impl Pattern {
    fn new(pattern: String, flags: i64) -> Self {
        Self {
            _pattern: pattern,
            _flags: flags,
        }
    }
    fn search(&self, text: &String) -> Result<Option<String>, RegexError> {
        if (self._flags != (0_i64)) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat)
                    .map_err(|e| RegexError {
                        message: e.to_string(),
                        detail: e.to_string(),
                    })?;
                Ok(__re.find(&text).map(|m| m.as_str().to_string()))
            };
        }
        regex::Regex::new(&self._pattern.clone())
            .map(|re| re.find(&text).map(|m| m.as_str().to_string()))
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            })
    }
    fn is_match(&self, text: &String) -> Result<bool, RegexError> {
        if (self._flags != (0_i64)) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat)
                    .map_err(|e| RegexError {
                        message: e.to_string(),
                        detail: e.to_string(),
                    })?;
                Ok(__re.is_match(&text))
            };
        }
        regex::Regex::new(&self._pattern.clone())
            .map(|re| re.is_match(&text))
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            })
    }
    fn sub(&self, replacement: &String, text: &String) -> Result<String, RegexError> {
        if (self._flags != (0_i64)) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat)
                    .map_err(|e| RegexError {
                        message: e.to_string(),
                        detail: e.to_string(),
                    })?;
                Ok(__re.replace_all(&text, &*replacement).to_string())
            };
        }
        regex::Regex::new(&self._pattern.clone())
            .map(|re| re.replace_all(&text, &*replacement).to_string())
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            })
    }
    fn findall(&self, text: &String) -> Result<Vec<String>, RegexError> {
        if (self._flags != (0_i64)) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat)
                    .map_err(|e| RegexError {
                        message: e.to_string(),
                        detail: e.to_string(),
                    })?;
                Ok(
                    __re
                        .find_iter(&text)
                        .map(|m| m.as_str().to_string())
                        .collect::<Vec<String>>(),
                )
            };
        }
        regex::Regex::new(&self._pattern.clone())
            .map(|re| {
                re.find_iter(&text)
                    .map(|m| m.as_str().to_string())
                    .collect::<Vec<String>>()
            })
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            })
    }
    fn finditer(
        &self,
        text: &String,
    ) -> Result<Box<dyn Iterator<Item = Match>>, RegexError> {
        let __sifr_try_res: Result<
            Result<Box<dyn Iterator<Item = Match>>, RegexError>,
            RegexError,
        > = (|| {
            let matches: Vec<Match> = _finditer_materialize(
                &self._pattern,
                text,
                self._flags,
            )?;
            return Ok(Ok(_iter_matches(matches)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(RegexError::new(e.message));
            }
        }
    }
    fn split(&self, text: &String) -> Result<Vec<String>, RegexError> {
        if (self._flags != (0_i64)) {
            return {
                let __flags_val = self._flags;
                let mut __flag_str = String::new();
                if (__flags_val & 2) != 0 {
                    __flag_str.push_str("(?i)");
                }
                if (__flags_val & 8) != 0 {
                    __flag_str.push_str("(?m)");
                }
                if (__flags_val & 16) != 0 {
                    __flag_str.push_str("(?s)");
                }
                if (__flags_val & 64) != 0 {
                    __flag_str.push_str("(?x)");
                }
                let __pat = __flag_str + &self._pattern.clone();
                let __re = regex::Regex::new(&__pat)
                    .map_err(|e| RegexError {
                        message: e.to_string(),
                        detail: e.to_string(),
                    })?;
                Ok(__re.split(&text).map(|s| s.to_string()).collect::<Vec<String>>())
            };
        }
        regex::Regex::new(&self._pattern.clone())
            .map(|re| re.split(&text).map(|s| s.to_string()).collect::<Vec<String>>())
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            })
    }
    fn pattern(&self) -> String {
        {
            let mut __sifr_concat: String = String::with_capacity(0usize + 0usize);
            __sifr_concat.push_str((self._pattern.clone()).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        }
    }
}
impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Pattern(_pattern={}, _flags={})", self._pattern, self._flags)
    }
}
fn search_flags(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Option<String>, RegexError> {
    {
        let __flags_val = flags;
        let mut __flag_str = String::new();
        if (__flags_val & 2) != 0 {
            __flag_str.push_str("(?i)");
        }
        if (__flags_val & 8) != 0 {
            __flag_str.push_str("(?m)");
        }
        if (__flags_val & 16) != 0 {
            __flag_str.push_str("(?s)");
        }
        if (__flags_val & 64) != 0 {
            __flag_str.push_str("(?x)");
        }
        let __pat = __flag_str + &pattern;
        let __re = regex::Regex::new(&__pat)
            .map_err(|e| RegexError {
                message: e.to_string(),
                detail: e.to_string(),
            })?;
        Ok(__re.find(&text).map(|m| m.as_str().to_string()))
    }
}
fn _iter_matches(matches: Vec<Match>) -> Box<dyn Iterator<Item = Match>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<Match> = Vec::new().into_iter();
    Box::new(
        std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<Match> = Vec::new();
                let mut i: i64 = 0_i64;
                while (i < (matches.len() as i64)) {
                    _yields.push(matches[i as usize].clone());
                    i += 1_i64;
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            __sifr_generator_iter.next()
        }),
    )
}
fn _find_index_from(text: &String, needle: &String, start: i64) -> i64 {
    let mut __sifr_chars_text: Vec<char> = text.chars().collect::<Vec<char>>();
    let mut __sifr_chars_needle: Vec<char> = needle.chars().collect::<Vec<char>>();
    if start < (0_i64) {
        return -(1_i64);
    }
    if ((__sifr_chars_needle.len() as i64) == (0_i64)) {
        if (start <= (__sifr_chars_text.len() as i64)) {
            return start;
        }
        return -(1_i64);
    }
    let max_start: i64 = (__sifr_chars_text.len() as i64)
        - (__sifr_chars_needle.len() as i64);
    let mut i: i64 = start;
    while i <= max_start {
        if (({
            let _slice_src = &__sifr_chars_text;
            let _slice_len_i64 = _slice_src.len() as i64;
            let _slice_start_i64 = if i < 0 {
                (_slice_len_i64 + i).max(0)
            } else {
                i.min(_slice_len_i64)
            };
            let _slice_stop_i64 = if (i + (__sifr_chars_needle.len() as i64)) < 0 {
                (_slice_len_i64 + (i + (__sifr_chars_needle.len() as i64))).max(0)
            } else {
                (i + (__sifr_chars_needle.len() as i64)).min(_slice_len_i64)
            };
            String::from_iter(
                _slice_src
                    .iter()
                    .skip(_slice_start_i64 as usize)
                    .take((_slice_stop_i64 - _slice_start_i64).max(0) as usize)
                    .copied(),
            )
        }) == needle.clone())
        {
            return i;
        }
        i += 1_i64;
    }
    -(1_i64)
}
fn _findall_for_finditer(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<String>, RegexError> {
    if flags != (0_i64) {
        return {
            let __flags_val = flags;
            let mut __flag_str = String::new();
            if (__flags_val & 2) != 0 {
                __flag_str.push_str("(?i)");
            }
            if (__flags_val & 8) != 0 {
                __flag_str.push_str("(?m)");
            }
            if (__flags_val & 16) != 0 {
                __flag_str.push_str("(?s)");
            }
            if (__flags_val & 64) != 0 {
                __flag_str.push_str("(?x)");
            }
            let __pat = __flag_str + &pattern;
            let __re = regex::Regex::new(&__pat)
                .map_err(|e| RegexError {
                    message: e.to_string(),
                    detail: e.to_string(),
                })?;
            Ok(
                __re
                    .find_iter(&text)
                    .map(|m| m.as_str().to_string())
                    .collect::<Vec<String>>(),
            )
        };
    }
    regex::Regex::new(&pattern)
        .map(|re| {
            re.find_iter(&text).map(|m| m.as_str().to_string()).collect::<Vec<String>>()
        })
        .map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        })
}
fn _finditer_materialize(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<Match>, RegexError> {
    let __sifr_try_res: Result<Result<Vec<Match>, RegexError>, RegexError> = (|| {
        let found_items: Vec<String> = _findall_for_finditer(pattern, text, flags)?;
        let mut matches: Vec<Match> = vec![];
        let mut cursor: i64 = 0_i64;
        for found in found_items.iter().cloned() {
            let mut __sifr_chars_found: Vec<char> = found.chars().collect::<Vec<char>>();
            let mut start: i64 = _find_index_from(text, &found, cursor);
            if start < (0_i64) {
                start = cursor;
            }
            let found_len: i64 = __sifr_chars_found.len() as i64;
            let end: i64 = start + found_len;
            matches.push(Match::new(found, start, end));
            if found_len == (0_i64) {
                cursor = end + (1_i64);
            } else {
                cursor = end;
            }
        }
        return Ok(Ok(matches));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(RegexError::new(e.message));
        }
    }
}
fn compile_flags(pattern: &String, flags: i64) -> Pattern {
    Pattern::new((pattern).clone(), flags)
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

fn main() {
    let path: String = "/tmp/sifr_demo_remediation.txt".to_string();
    let __sifr_try_res: Result<(), IOError> = (|| {
    let mut f: TextFileHandle = (|| {
    let __path = path.to_string();
    let __mode = "w".to_string().to_string();
    let __encoding = "utf-8".to_string().to_string();
    let __errors = "strict".to_string().to_string();
    let __handle_id = __sifr_next_file_handle_id();
    match __mode.as_str() {
        "r" | "rt" => {
            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
            let __reader = std::io::BufReader::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        "w" | "wt" => {
            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        "a" | "at" => {
            let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        _ => {
            return Err(IOError { message: format!("invalid mode: {}", __mode), kind: "Other".to_string() });
        },
    }
})()?;
    let _ = f.write(&"hello from open()\n".to_string())?;
    let _2: () = f.write(&"second line\n".to_string())?;
    f.close();
    let content: String = std::fs::read_to_string(&path).map_err(__io_err)?;
    let mut __sifr_chars_content: Vec<char> = content.chars().collect::<Vec<char>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("open write ok = ");
    __sifr_concat.push_str((format!("{}", (content.chars().count() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(18usize + 0usize);
    __sifr_concat.push_str("open write error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let path2: String = "/tmp/sifr_demo_ctx.txt".to_string();
    let __sifr_try_res: Result<(), IOError> = (|| {
    {
        let mut __ctx_0 = (|| {
    let __path = path2.to_string();
    let __mode = "w".to_string().to_string();
    let __encoding = "utf-8".to_string().to_string();
    let __errors = "strict".to_string().to_string();
    let __handle_id = __sifr_next_file_handle_id();
    match __mode.as_str() {
        "r" | "rt" => {
            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
            let __reader = std::io::BufReader::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        "w" | "wt" => {
            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        "a" | "at" => {
            let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        _ => {
            return Err(IOError { message: format!("invalid mode: {}", __mode), kind: "Other".to_string() });
        },
    }
})()?;
        struct __WithGuard0 { ctx: TextFileHandle }
        impl Drop for __WithGuard0 {
            fn drop(&mut self) { self.ctx.__exit__(); }
        }
        let mut __guard_0 = __WithGuard0 { ctx: __ctx_0 };
        let fw = __guard_0.ctx.__enter__();
        let _3: () = fw.write(&"context manager works".to_string())?;
    }
    let result: String = std::fs::read_to_string(&path2).map_err(__io_err)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("context manager ok = ");
    __sifr_concat.push_str((format!("{}", result == "context manager works")).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(23usize + 0usize);
    __sifr_concat.push_str("context manager error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    let mut fr: TextFileHandle = (|| {
    let __path = path.to_string();
    let __mode = "r".to_string().to_string();
    let __encoding = "utf-8".to_string().to_string();
    let __errors = "strict".to_string().to_string();
    let __handle_id = __sifr_next_file_handle_id();
    match __mode.as_str() {
        "r" | "rt" => {
            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
            let __reader = std::io::BufReader::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        "w" | "wt" => {
            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        "a" | "at" => {
            let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(TextFileHandle::new(BinaryFileHandle::new(__handle_id, __mode.to_string()), Encoding::new(__encoding), DecodeErrorHandler::new(__errors.clone()), EncodeErrorHandler::new(__errors)));
        },
        _ => {
            return Err(IOError { message: format!("invalid mode: {}", __mode), kind: "Other".to_string() });
        },
    }
})()?;
    let content2: String = fr.read()?;
    let mut __sifr_chars_content2: Vec<char> = content2.chars().collect::<Vec<char>>();
    fr.close();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("open read ok = ");
    __sifr_concat.push_str((format!("{}", (content2.chars().count() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + 0usize);
    __sifr_concat.push_str("open read error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let mut t: time = time::new(10_i64, 30_i64, 45_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + 0usize);
    __sifr_concat.push_str("time isoformat = ");
    __sifr_concat.push_str((t.isoformat()).as_str());
    __sifr_concat
});
    let t2: time = time::new(10_i64, 30_i64, 45_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("time eq = ");
    __sifr_concat.push_str((format!("{}", t == t2)).as_str());
    __sifr_concat
});
    let tz: timezone = timezone::new(0_i64);
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("timezone utc = ");
    __sifr_concat.push_str((format!("{}", tz)).as_str());
    __sifr_concat
});
    let mut dt: datetime = now(&None);
    let iso: String = dt.isoformat();
    let mut __sifr_chars_iso: Vec<char> = iso.chars().collect::<Vec<char>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(19usize + 0usize);
    __sifr_concat.push_str("now isoformat ok = ");
    __sifr_concat.push_str((format!("{}", (iso.chars().count() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    let mut tmp: Path = Path::new("/tmp".to_string());
    let __sifr_try_res: Result<(), IOError> = (|| {
    let mut matches_it: Box<dyn Iterator<Item = String>> = tmp.glob(&"sifr_demo_*".to_string())?;
    let matches: Vec<String> = matches_it.collect::<Vec<_>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(13usize + 0usize);
    __sifr_concat.push_str("glob found = ");
    __sifr_concat.push_str((format!("{}", (matches.len() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(12usize + 0usize);
    __sifr_concat.push_str("glob error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let __sifr_try_res: Result<(), RegexError> = (|| {
    let found: Option<String> = search_flags(&"hello".to_string(), &"HELLO WORLD".to_string(), IGNORECASE)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(16usize + 0usize);
    __sifr_concat.push_str("re ignorecase = ");
    __sifr_concat.push_str((format!("{}", found.is_some())).as_str());
    __sifr_concat
});
    let mut pat: Pattern = compile_flags(&"^line".to_string(), MULTILINE);
    let found2: Option<String> = pat.search(&"line1\nline2".to_string())?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("re multiline = ");
    __sifr_concat.push_str((format!("{}", found2.is_some())).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(10usize + 0usize);
    __sifr_concat.push_str("re error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    let cwd: String = std::env::current_dir().map(|p| p.to_string_lossy().to_string()).map_err(__io_err)?;
    let mut __sifr_chars_cwd: Vec<char> = cwd.chars().collect::<Vec<char>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(15usize + 0usize);
    __sifr_concat.push_str("os getcwd ok = ");
    __sifr_concat.push_str((format!("{}", (cwd.chars().count() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(17usize + 0usize);
    __sifr_concat.push_str("os getcwd error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let items: Vec<i64> = vec![1_i64, 2_i64, 3_i64, 4_i64, 5_i64];
    let __sifr_try_res: Result<(), ValueError> = (|| {
    let picked: i64 = choice(&items)?;
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(19usize + 0usize);
    __sifr_concat.push_str("random choice ok = ");
    __sifr_concat.push_str((format!("{}", (picked >= (1_i64)) && (picked <= (5_i64)))).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(21usize + 0usize);
    __sifr_concat.push_str("random choice error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let mut root: Logger = basicConfig(WARNING);
    root.info(&"should not print".to_string());
    root.warning(&"root warning visible".to_string());
    let mut logger2: Logger = getLogger(&"myapp".to_string());
    logger2.info(&"should not print either".to_string());
    logger2.warning(&"myapp warning visible".to_string());
    println!("basicConfig global level ok");
    let mut handler: FileHandler = FileHandler::new("/tmp/sifr_demo_fh_log.txt".to_string(), 0_i64);
    handler.emit(&"INFO".to_string(), &"demo".to_string(), &"file handler test".to_string());
    let __sifr_try_res: Result<(), IOError> = (|| {
    let log_content: String = std::fs::read_to_string(&"/tmp/sifr_demo_fh_log.txt".to_string()).map_err(__io_err)?;
    let mut __sifr_chars_log_content: Vec<char> = log_content.chars().collect::<Vec<char>>();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(24usize + 0usize);
    __sifr_concat.push_str("file handler wrote ok = ");
    __sifr_concat.push_str((format!("{}", (log_content.chars().count() as i64) > (0_i64))).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(20usize + 0usize);
    __sifr_concat.push_str("file handler error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
    let csv_path: String = "/tmp/sifr_demo_csv.csv".to_string();
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _4: () = std::fs::write(&csv_path, "name,age\nalice,30\nbob,25".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let mut r: reader = reader_from_path(&csv_path, &None, &",".to_string(), &"\"".to_string(), &"".to_string(), true, false, 0_i64)?;
    let rows: Vec<Vec<String>> = r.rows();
    println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(28usize + 0usize);
    __sifr_concat.push_str("csv reader_from_path rows = ");
    __sifr_concat.push_str((format!("{}", rows.len() as i64)).as_str());
    __sifr_concat
});
    Ok(())
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", {
    let mut __sifr_concat: String = String::with_capacity(11usize + 0usize);
    __sifr_concat.push_str("csv error: ");
    __sifr_concat.push_str((e.message).as_str());
    __sifr_concat
});
    }
}
