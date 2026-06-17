use std::collections::HashMap;

use std::sync::Mutex;

// --- stdlib: sifr.io ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IOBase {
    _closed: bool,
}
impl IOBase {
    fn new() -> Self {
        return Self { _closed: false };
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _: i64 = offset;
        let _: i64 = whence;
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn tell(&self) -> Result<i64, IOError> {
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn readable(&self) -> bool {
        return false;
    }
    fn writable(&self) -> bool {
        return false;
    }
    fn seekable(&self) -> bool {
        return false;
    }
}
impl std::fmt::Display for IOBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "IOBase(_closed={})", self._closed);
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
        return Self {
            _handle: handle,
            _mode: mode,
            _closed: false,
        };
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
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn read(&self) -> Result<String, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
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
        })();
    }
    fn write(&self, data: &String) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        return (|| {
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
        })();
    }
    fn readline(&self) -> Result<Option<String>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
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
        })();
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
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
        })();
    }
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
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
        })();
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        return (|| {
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
        })();
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _: i64 = offset;
        let _: i64 = whence;
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn tell(&self) -> Result<i64, IOError> {
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn readable(&self) -> bool {
        return _mode_is_readable(&self._mode.clone());
    }
    fn writable(&self) -> bool {
        return _mode_is_writable(&self._mode.clone());
    }
    fn seekable(&self) -> bool {
        return false;
    }
    fn __enter__(&self) -> FileHandle {
        return self.clone();
    }
    fn __exit__(&mut self) {
        self.close();
    }
}
impl std::fmt::Display for FileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "FileHandle(_handle={}, _mode={}, _closed={})", self._handle, self._mode,
            self._closed
        );
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
        return Self {
            _handle: handle,
            _mode: mode,
            _closed: false,
        };
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
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn read_bytes(&self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        let _: Option<i64> = size;
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.readable()) {
            return Err(IOError::new("stream is not readable".to_string()));
        }
        return (|| {
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
        })();
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        if !(self.writable()) {
            return Err(IOError::new("stream is not writable".to_string()));
        }
        return (|| {
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
        })();
    }
    fn seek(&self, offset: i64, whence: i64) -> Result<i64, IOError> {
        let _: i64 = offset;
        let _: i64 = whence;
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn tell(&self) -> Result<i64, IOError> {
        return Err(IOError::new(_unsupported_seek_tell_error()));
    }
    fn readable(&self) -> bool {
        return _mode_is_readable(&self._mode.clone());
    }
    fn writable(&self) -> bool {
        return _mode_is_writable(&self._mode.clone());
    }
    fn seekable(&self) -> bool {
        return false;
    }
    fn __enter__(&self) -> BinaryFileHandle {
        return self.clone();
    }
    fn __exit__(&mut self) {
        self.close();
    }
}
impl std::fmt::Display for BinaryFileHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "BinaryFileHandle(_handle={}, _mode={}, _closed={})", self._handle, self
            ._mode, self._closed
        );
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
        return Self {
            _buffer: format!("{}{}", initial, "".to_string()),
            _cursor: 0 as i64,
            _closed: false,
        };
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
    }
    fn read(&mut self, size: Option<i64>) -> Result<String, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let start: i64 = self._cursor;
        let mut end: i64 = self._buffer.clone().chars().count() as i64;
        if let Some(size) = size {
            let maybe_size: i64 = size;
            if maybe_size >= (0 as i64) {
                let requested: i64 = start + maybe_size;
                if requested < end {
                    end = requested;
                }
            }
        }
        let piece: String = String::from_iter(
            (self._buffer.clone())
                .chars()
                .skip((start).max(0) as usize)
                .take(((end).max(0) - (start).max(0)).max(0) as usize),
        );
        self._cursor = end;
        return Ok(piece);
    }
    fn write(&mut self, data: &String) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let left: String = String::from_iter(
            (self._buffer.clone())
                .chars()
                .skip(0 as usize)
                .take(((self._cursor).max(0) - 0).max(0) as usize),
        );
        let tail_start: i64 = self._cursor + (data.chars().count() as i64);
        let mut right: String = "".to_string();
        if tail_start < (self._buffer.clone().chars().count() as i64) {
            right = String::from_iter(
                (self._buffer.clone()).chars().skip((tail_start).max(0) as usize),
            );
        }
        self._buffer = format!("{}{}{}", left, data, right);
        self._cursor = self._cursor + (data.chars().count() as i64);
        return Ok(());
    }
    fn getvalue(&self) -> String {
        return self._buffer.clone();
    }
    fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let mut origin: i64 = 0 as i64;
        if whence == (0 as i64) {
            origin = 0 as i64;
        } else {
            if whence == (1 as i64) {
                origin = self._cursor;
            } else {
                if whence == (2 as i64) {
                    origin = self._buffer.clone().chars().count() as i64;
                } else {
                    return Err(IOError::new(_invalid_whence_error(whence)));
                }
            }
        }
        let mut next_pos: i64 = origin + offset;
        if next_pos < (0 as i64) {
            return Err(IOError::new(_negative_seek_error(next_pos)));
        }
        let end: i64 = self._buffer.clone().chars().count() as i64;
        if next_pos > end {
            next_pos = end;
        }
        self._cursor = next_pos;
        return Ok(self._cursor);
    }
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(self._cursor);
    }
    fn readable(&self) -> bool {
        return !(self._closed);
    }
    fn writable(&self) -> bool {
        return !(self._closed);
    }
    fn seekable(&self) -> bool {
        return !(self._closed);
    }
}
impl std::fmt::Display for StringIO {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "StringIO(_buffer={}, _cursor={}, _closed={})", self._buffer, self
            ._cursor, self._closed
        );
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
        return Self {
            _buffer: initial.iter().map(|__byte| *__byte as i64).collect::<Vec<i64>>(),
            _cursor: 0 as i64,
            _closed: false,
        };
    }
    fn close(&mut self) {
        self._closed = true;
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn flush(&self) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(());
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
                Ok(__out)
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
        let mut end: i64 = self._buffer.clone().len() as i64;
        if let Some(size) = size {
            let maybe_size: i64 = size;
            if maybe_size >= (0 as i64) {
                let requested: i64 = start + maybe_size;
                if requested < end {
                    end = requested;
                }
            }
        }
        let chunk: Vec<i64> = Vec::from_iter(
            (self._buffer.clone())
                .iter()
                .skip((start).max(0) as usize)
                .take(((end).max(0) - (start).max(0)).max(0) as usize)
                .cloned(),
        );
        self._cursor = end;
        return self._slice_to_bytes(&chunk);
    }
    fn write_bytes(&mut self, data: &Vec<u8>) -> Result<(), IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let values: Vec<i64> = data
            .iter()
            .map(|__byte| *__byte as i64)
            .collect::<Vec<i64>>();
        let mut i: i64 = 0 as i64;
        while i < (values.len() as i64) {
            let maybe_value: Option<i64> = Some(values[i as usize]);
            let Some(maybe_value) = maybe_value else {
                return Err(IOError::new("bytes write invariant violation".to_string()));
            };
            let idx: i64 = self._cursor + i;
            if idx < (self._buffer.clone().len() as i64) {
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
            i = i + (1 as i64);
        }
        self._cursor = self._cursor + (values.len() as i64);
        return Ok(());
    }
    fn getvalue(&self) -> Result<Vec<u8>, IOError> {
        return self._slice_to_bytes(&self._buffer.clone());
    }
    fn seek(&mut self, offset: i64, whence: i64) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let mut origin: i64 = 0 as i64;
        if whence == (0 as i64) {
            origin = 0 as i64;
        } else {
            if whence == (1 as i64) {
                origin = self._cursor;
            } else {
                if whence == (2 as i64) {
                    origin = self._buffer.clone().len() as i64;
                } else {
                    return Err(IOError::new(_invalid_whence_error(whence)));
                }
            }
        }
        let mut next_pos: i64 = origin + offset;
        if next_pos < (0 as i64) {
            return Err(IOError::new(_negative_seek_error(next_pos)));
        }
        let end: i64 = self._buffer.clone().len() as i64;
        if next_pos > end {
            next_pos = end;
        }
        self._cursor = next_pos;
        return Ok(self._cursor);
    }
    fn tell(&self) -> Result<i64, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        return Ok(self._cursor);
    }
    fn readable(&self) -> bool {
        return !(self._closed);
    }
    fn writable(&self) -> bool {
        return !(self._closed);
    }
    fn seekable(&self) -> bool {
        return !(self._closed);
    }
}
fn _closed_stream_error() -> String {
    return "I/O operation on closed stream".to_string();
}
fn _invalid_whence_error(whence: i64) -> String {
    return format!("{}{}", "invalid whence: ".to_string(), format!("{}", whence));
}
fn _negative_seek_error(offset: i64) -> String {
    return format!(
        "{}{}", "negative seek position: ".to_string(), format!("{}", offset)
    );
}
fn _unsupported_seek_tell_error() -> String {
    return "seek/tell is unsupported for this stream".to_string();
}
fn _mode_is_readable(mode: &String) -> bool {
    return mode.contains(&"r".to_string()) || mode.contains(&"+".to_string());
}
fn _mode_is_writable(mode: &String) -> bool {
    return (mode.contains(&"w".to_string()) || mode.contains(&"a".to_string()))
        || mode.contains(&"+".to_string());
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

// --- stdlib: sifr.csv ---
const QUOTE_NONE: i64 = 3 as i64;
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
        if quotechar != "".to_string() {
            _validate_char(&"quotechar".to_string(), &quotechar);
        }
        if escapechar != "".to_string() {
            _validate_char(&"escapechar".to_string(), &escapechar);
        }
        if (quotechar == "".to_string()) && (resolved_quoting != QUOTE_NONE) {
            resolved_quoting = QUOTE_NONE;
        }
        return Self {
            delimiter: delimiter,
            quotechar: quotechar,
            escapechar: escapechar,
            doublequote: doublequote,
            skipinitialspace: skipinitialspace,
            lineterminator: lineterminator,
            quoting: resolved_quoting,
        };
    }
}
impl std::fmt::Display for Dialect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Dialect(delimiter={}, quotechar={}, escapechar={}, doublequote={}, skipinitialspace={}, lineterminator={}, quoting={})",
            self.delimiter, self.quotechar, self.escapechar, self.doublequote, self
            .skipinitialspace, self.lineterminator, self.quoting
        );
    }
}
#[derive(Debug, Clone, PartialEq)]
struct DialectRegistry {
    _dialects: HashMap<String, Dialect>,
}
impl DialectRegistry {
    fn new() -> Self {
        return Self {
            _dialects: {
                let mut __dict = HashMap::new();
                __dict.insert("excel".to_string(), excel());
                __dict.insert("excel-tab".to_string(), excel_tab());
                __dict.insert("unix".to_string(), unix_dialect());
                __dict
            },
        };
    }
    fn register(&mut self, name: &String, dialect: &Dialect) {
        self._dialects
            .insert(format!("{}{}", name, "".to_string()), _copy_dialect(dialect));
    }
    fn unregister(&mut self, name: &String) -> bool {
        if (self._dialects.clone()).contains_key((name).as_str()) {
            let _: Option<Dialect> = self._dialects.remove((name).as_str());
            return true;
        }
        return false;
    }
    fn get(&self, name: &String) -> Option<Dialect> {
        if !((self._dialects.clone()).contains_key((name).as_str())) {
            return None;
        }
        for (key, value) in self
            ._dialects
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if key != *name {
                continue;
            }
            return Some(_copy_dialect(&value));
        }
        return None;
    }
    fn names(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![];
        for key in self._dialects.clone().keys().cloned() {
            names.push(format!("{}{}", key, "".to_string()));
        }
        return names;
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
            &format!("{}{}", resolved_dialect.delimiter, "".to_string()),
            &format!("{}{}", resolved_dialect.quotechar, "".to_string()),
            &format!("{}{}", resolved_dialect.escapechar, "".to_string()),
            resolved_dialect.doublequote,
            resolved_dialect.skipinitialspace,
            resolved_dialect.quoting,
        );
        return Self {
            dialect: resolved_dialect,
            _rows: rows,
            _pos: 0 as i64,
        };
    }
    fn __next__(&mut self) -> Option<Vec<String>> {
        if self._pos >= (self._rows.clone().len() as i64) {
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
        self._pos = self._pos + (1 as i64);
        let Some(row) = row else {
            return None;
        };
        let mut result: Vec<String> = vec![];
        for field in row.iter().cloned() {
            result.push(format!("{}{}", field, "".to_string()));
        }
        return Some(result);
    }
    fn rows(&self) -> Vec<Vec<String>> {
        let mut result: Vec<Vec<String>> = vec![];
        for row in self._rows.clone().iter().cloned() {
            let mut copied: Vec<String> = vec![];
            for field in row.iter().cloned() {
                copied.push(format!("{}{}", field, "".to_string()));
            }
            result.push(copied);
        }
        return result;
    }
    fn line_num(&self) -> i64 {
        return self._pos;
    }
}
fn excel() -> Dialect {
    return Dialect::new(
        ",".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        "\n".to_string(),
        0 as i64,
    );
}
fn excel_tab() -> Dialect {
    return Dialect::new(
        "\t".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        "\n".to_string(),
        0 as i64,
    );
}
fn unix_dialect() -> Dialect {
    return Dialect::new(
        ",".to_string(),
        "\"".to_string(),
        "".to_string(),
        true,
        false,
        "\n".to_string(),
        1 as i64,
    );
}
fn _copy_dialect(dialect: &Dialect) -> Dialect {
    return Dialect::new(
        format!("{}{}", dialect.delimiter, "".to_string()),
        format!("{}{}", dialect.quotechar, "".to_string()),
        format!("{}{}", dialect.escapechar, "".to_string()),
        dialect.doublequote,
        dialect.skipinitialspace,
        format!("{}{}", dialect.lineterminator, "".to_string()),
        dialect.quoting,
    );
}
fn dialect_registry() -> DialectRegistry {
    return DialectRegistry::new();
}
fn _validate_char(name: &String, value: &String) {
    let _: String = (name).clone();
    let _: String = (value).clone();
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
    return Dialect::new(
        (delimiter).clone(),
        (quotechar).clone(),
        (escapechar).clone(),
        doublequote,
        skipinitialspace,
        (lineterminator).clone(),
        quoting,
    );
}
fn _quotechar_value(dialect: &Dialect) -> String {
    let quotechar: String = format!("{}{}", dialect.quotechar, "".to_string());
    if quotechar == "".to_string() {
        return "\"".to_string();
    }
    return quotechar;
}
fn _append_field(row: &mut Vec<String>, field: String) {
    row.push(format!("{}{}", field, "".to_string()));
}
fn _append_row(rows: &mut Vec<Vec<String>>, row: Vec<String>) {
    rows.push(row);
}
fn _char_at(text: &String, index: i64) -> String {
    if ((index < (0 as i64)) || (index >= (text.chars().count() as i64))) {
        return "".to_string();
    }
    let ch: Option<String> = Some({
        let Some(__indexed_char) = text.chars().nth(index as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    });
    let Some(ch) = ch else {
        return "".to_string();
    };
    return ch;
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
    let mut i: i64 = 0 as i64;
    while i < (text.chars().count() as i64) {
        let ch_value: String = _char_at(text, i);
        if in_quotes {
            if ((resolved.escapechar != "".to_string())
                && (ch_value == resolved.escapechar))
            {
                if (i + (1 as i64)) < (text.chars().count() as i64) {
                    let escaped_value: String = _char_at(text, i + (1 as i64));
                    field = format!("{}{}", field, escaped_value);
                    i = i + (2 as i64);
                    continue;
                }
                field = format!("{}{}", field, ch_value);
                i = i + (1 as i64);
                continue;
            }
            if ((resolved.quotechar != "".to_string())
                && (ch_value == resolved.quotechar))
            {
                let quotechar: String = _quotechar_value(&resolved);
                if (((resolved.doublequote)
                    && ((i + (1 as i64)) < (text.chars().count() as i64)))
                    && (_char_at(text, i + (1 as i64)) == quotechar.clone()))
                {
                    field = format!("{}{}", field, quotechar);
                    i = i + (2 as i64);
                    continue;
                }
                in_quotes = false;
                i = i + (1 as i64);
                continue;
            }
            field = format!("{}{}", field, ch_value);
            i = i + (1 as i64);
            continue;
        }
        if (((!(field_started)) && (resolved.skipinitialspace))
            && (ch_value == " ".to_string()))
        {
            i = i + (1 as i64);
            continue;
        }
        if ((resolved.escapechar != "".to_string()) && (ch_value == resolved.escapechar))
        {
            if (i + (1 as i64)) < (text.chars().count() as i64) {
                let escaped_plain_value: String = _char_at(text, i + (1 as i64));
                field = format!("{}{}", field, escaped_plain_value);
                field_started = true;
                i = i + (2 as i64);
                continue;
            }
            field = format!("{}{}", field, ch_value);
            field_started = true;
            i = i + (1 as i64);
            continue;
        }
        if ((resolved.quoting != QUOTE_NONE) && (resolved.quotechar != "".to_string())) {
            let quotechar2: String = _quotechar_value(&resolved);
            if ch_value == quotechar2 {
                in_quotes = true;
                field_started = true;
                i = i + (1 as i64);
                continue;
            }
        }
        if ch_value == resolved.delimiter {
            _append_field(&mut row, field);
            field = "".to_string();
            field_started = false;
            i = i + (1 as i64);
            continue;
        }
        if (ch_value == "\n".to_string()) || (ch_value == "\r".to_string()) {
            if (((ch_value == "\r".to_string())
                && ((i + (1 as i64)) < (text.chars().count() as i64)))
                && (_char_at(text, i + (1 as i64)) == "\n".to_string()))
            {
                i = i + (1 as i64);
            }
            if (((row.len() as i64) == (0 as i64)) && (field == "".to_string())) {
                _append_row(&mut rows, vec![]);
            } else {
                _append_field(&mut row, field);
                _append_row(&mut rows, row);
            }
            row = vec![];
            field = "".to_string();
            field_started = false;
            i = i + (1 as i64);
            continue;
        }
        field = format!("{}{}", field, ch_value);
        field_started = true;
        i = i + (1 as i64);
    }
    if in_quotes {
        in_quotes = false;
    }
    if (((row.len() as i64) > (0 as i64)) || (field != "".to_string())) {
        _append_field(&mut row, field);
        _append_row(&mut rows, row);
    }
    return rows;
}

// --- stdlib: sifr.json ---
#[derive(Debug, Clone, PartialEq)]
struct JsonValue {
    kind: String,
    bool_value: Option<bool>,
    int_value: Option<i64>,
    float_value: Option<f64>,
    str_value: Option<String>,
    array_items: Box<Vec<JsonValue>>,
    object_items: Box<Vec<(String, JsonValue)>>,
}
impl JsonValue {
    fn new(
        kind: String,
        bool_value: Option<bool>,
        int_value: Option<i64>,
        float_value: Option<f64>,
        str_value: Option<String>,
    ) -> Self {
        return Self {
            kind: kind,
            bool_value: bool_value,
            int_value: int_value,
            float_value: float_value,
            str_value: str_value,
            array_items: Box::new(vec![]),
            object_items: Box::new(vec![]),
        };
    }
    fn is_null(&self) -> bool {
        return self.kind.clone() == "null".to_string();
    }
    fn is_bool(&self) -> bool {
        return self.kind.clone() == "bool".to_string();
    }
    fn is_int(&self) -> bool {
        return self.kind.clone() == "int".to_string();
    }
    fn is_float(&self) -> bool {
        return self.kind.clone() == "float".to_string();
    }
    fn is_str(&self) -> bool {
        return self.kind.clone() == "str".to_string();
    }
    fn is_array(&self) -> bool {
        return self.kind.clone() == "array".to_string();
    }
    fn is_object(&self) -> bool {
        return self.kind.clone() == "object".to_string();
    }
    fn as_bool(&self) -> Option<bool> {
        return self.bool_value;
    }
    fn as_int(&self) -> Option<i64> {
        return self.int_value;
    }
    fn as_float(&self) -> Option<f64> {
        return self.float_value;
    }
    fn as_str(&self) -> Option<String> {
        return self.str_value.clone();
    }
    fn as_array(&self) -> Option<Vec<JsonValue>> {
        if !(self.is_array()) {
            return None;
        }
        let mut result: Vec<JsonValue> = vec![];
        for item in (self.array_items).as_ref().clone().iter().cloned() {
            result.push(item);
        }
        return Some(result);
    }
    fn as_object(&self) -> Option<Vec<(String, JsonValue)>> {
        if !(self.is_object()) {
            return None;
        }
        let mut result: Vec<(String, JsonValue)> = vec![];
        for (key, value) in (self.object_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return Some(result);
    }
    fn at(&self, index: i64) -> Option<JsonValue> {
        if !(self.is_array()) {
            return None;
        }
        if ((index < (0 as i64))
            || (index >= ((self.array_items).as_ref().clone().len() as i64)))
        {
            return None;
        }
        let value: Option<JsonValue> = {
            let __sifr_index_list = &self.array_items;
            let __sifr_index_i = index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).cloned()
        };
        return value;
    }
    fn get(&self, key: &String) -> Option<JsonValue> {
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
        return None;
    }
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
            result.push(item_key);
        }
        return result;
    }
    fn values(&self) -> Vec<JsonValue> {
        let mut result: Vec<JsonValue> = vec![];
        if !(self.is_object()) {
            return result;
        }
        for (_item_key, item_value) in (self.object_items)
            .as_ref()
            .clone()
            .iter()
            .cloned()
        {
            result.push(item_value);
        }
        return result;
    }
    fn items(&self) -> Vec<(String, JsonValue)> {
        if !(self.is_object()) {
            return vec![];
        }
        let mut result: Vec<(String, JsonValue)> = vec![];
        for (key, value) in (self.object_items).as_ref().clone().iter().cloned() {
            result.push((key, value));
        }
        return result;
    }
}
impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f, "{}", { let __json_value = self; fn __sifr_json_value_to_serde(value : &
            JsonValue) -> serde_json::Value { match value.kind.as_str() { "null" => {
            return serde_json::Value::Null; }, "bool" => { if let Some(v) = value
            .bool_value { return serde_json::Value::from(v); } return
            serde_json::Value::Null; }, "int" => { if let Some(v) = value.int_value {
            return serde_json::Value::from(v); } return serde_json::Value::Null; },
            "float" => { if let Some(v) = value.float_value { return
            serde_json::Value::from(v); } return serde_json::Value::Null; }, "str" => {
            if let Some(v) = value.str_value.clone() { return
            serde_json::Value::String(v); } return serde_json::Value::Null; }, "array" =>
            { let mut converted = vec![]; for item in value.array_items.as_ref().iter()
            .cloned() { converted.push(__sifr_json_value_to_serde(& item)); } return
            serde_json::Value::Array(converted); }, "object" => { let mut converted =
            serde_json::Map::new(); for entry in value.object_items.as_ref().iter()
            .cloned() { let entry_key = entry.0; let entry_value = entry.1; converted
            .insert(entry_key, __sifr_json_value_to_serde(& entry_value)); } return
            serde_json::Value::Object(converted); }, _ => { return
            serde_json::Value::Null; }, } } serde_json::to_string(&
            __sifr_json_value_to_serde(& __json_value)).unwrap_or_else(| _err | "null"
            .to_string().to_string()) }
        );
    }
}
#[derive(Debug, Clone, PartialEq)]
struct JSONEncoder {
    indent: Option<i64>,
    sort_keys: bool,
    ensure_ascii: bool,
}
impl JSONEncoder {
    fn new(indent: Option<i64>, sort_keys: bool, ensure_ascii: bool) -> Self {
        return Self {
            indent: indent,
            sort_keys: sort_keys,
            ensure_ascii: ensure_ascii,
        };
    }
    fn encode(&self, value: &JsonValue) -> String {
        let _: Option<i64> = self.indent;
        let _: bool = self.sort_keys;
        let _: bool = self.ensure_ascii;
        return dumps(value);
    }
    fn dump(&self, value: &JsonValue, path: &String) -> Result<(), IOError> {
        return std::fs::write(&path, self.encode(value).as_bytes())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn dump_handle(&self, value: &JsonValue, fh: &FileHandle) -> Result<(), IOError> {
        return fh.write(&self.encode(value));
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct JSONDecoder {}
impl JSONDecoder {
    fn new() -> Self {
        return Self {};
    }
    fn decode(&self, s: &String) -> Result<JsonValue, JSONDecodeError> {
        return loads(s);
    }
    fn load(&self, path: &String) -> Result<JsonValue, Error> {
        return load(path);
    }
    fn load_handle(&self, fh: &FileHandle) -> Result<JsonValue, Error> {
        return load_handle(fh);
    }
}
fn from_int(value: i64) -> JsonValue {
    let int_value: Option<i64> = Some(value);
    return JsonValue::new("int".to_string(), None, int_value, None, None);
}
fn from_str(value: &String) -> JsonValue {
    let str_value: Option<String> = Some(format!("{}{}", value, "".to_string()));
    return JsonValue::new("str".to_string(), None, None, None, str_value);
}
fn _append_object_item(
    mut value: JsonValue,
    key: String,
    item_value: JsonValue,
) -> JsonValue {
    value.object_items.push((key, item_value));
    return value;
}
fn from_object(items: &Vec<(String, JsonValue)>) -> JsonValue {
    let mut value: JsonValue = JsonValue::new(
        "object".to_string(),
        None,
        None,
        None,
        None,
    );
    for (key, item_value) in items.iter().cloned() {
        value = _append_object_item(value, key, item_value);
    }
    return value;
}
fn loads(s: &String) -> Result<JsonValue, JSONDecodeError> {
    return {
        let __json_input = s;
        fn __sifr_json_value_from_serde(
            value: serde_json::Value,
        ) -> Result<JsonValue, JSONDecodeError> {
            match value {
                serde_json::Value::Null => {
                    return Ok(JsonValue {
                        kind: "null".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        array_items: Box::new(vec![]),
                        object_items: Box::new(vec![]),
                    });
                }
                serde_json::Value::Bool(b) => {
                    return Ok(JsonValue {
                        kind: "bool".to_string().to_string(),
                        bool_value: Some(b),
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        array_items: Box::new(vec![]),
                        object_items: Box::new(vec![]),
                    });
                }
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        return Ok(JsonValue {
                            kind: "int".to_string().to_string(),
                            bool_value: None,
                            int_value: Some(i),
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    if n.is_u64() {
                        return Err(JSONDecodeError {
                            message: "json integer out of range for sifr int"
                                .to_string()
                                .to_string(),
                            line: 0,
                            column: 0,
                        });
                    }
                    if let Some(f) = n.as_f64() {
                        return Ok(JsonValue {
                            kind: "float".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: Some(f),
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    return Err(JSONDecodeError {
                        message: "unsupported json number representation"
                            .to_string()
                            .to_string(),
                        line: 0,
                        column: 0,
                    });
                }
                serde_json::Value::String(s) => {
                    return Ok(JsonValue {
                        kind: "str".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: Some(s),
                        array_items: Box::new(vec![]),
                        object_items: Box::new(vec![]),
                    });
                }
                serde_json::Value::Array(items) => {
                    let mut converted = vec![];
                    for item in items {
                        converted.push(__sifr_json_value_from_serde(item)?);
                    }
                    return Ok(JsonValue {
                        kind: "array".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        array_items: Box::new(converted),
                        object_items: Box::new(vec![]),
                    });
                }
                serde_json::Value::Object(entries) => {
                    let mut converted = vec![];
                    for entry in entries {
                        let entry_key = entry.0;
                        let entry_value = entry.1;
                        let converted_value = __sifr_json_value_from_serde(entry_value)?;
                        converted.push((entry_key, converted_value));
                    }
                    return Ok(JsonValue {
                        kind: "object".to_string().to_string(),
                        bool_value: None,
                        int_value: None,
                        float_value: None,
                        str_value: None,
                        array_items: Box::new(vec![]),
                        object_items: Box::new(converted),
                    });
                }
            }
        }
        serde_json::from_str::<serde_json::Value>(__json_input.as_ref())
            .map_err(|e| JSONDecodeError {
                message: e.to_string(),
                line: e.line() as i64,
                column: e.column() as i64,
            })
            .and_then(|parsed| __sifr_json_value_from_serde(parsed))
    };
}
fn _decode_loaded_json(content: &String) -> Result<JsonValue, Error> {
    let __sifr_try_res: Result<Result<JsonValue, Error>, JSONDecodeError> = (|| {
        let value: JsonValue = ({
            let __json_input = content;
            fn __sifr_json_value_from_serde(
                value: serde_json::Value,
            ) -> Result<JsonValue, JSONDecodeError> {
                match value {
                    serde_json::Value::Null => {
                        return Ok(JsonValue {
                            kind: "null".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    serde_json::Value::Bool(b) => {
                        return Ok(JsonValue {
                            kind: "bool".to_string().to_string(),
                            bool_value: Some(b),
                            int_value: None,
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            return Ok(JsonValue {
                                kind: "int".to_string().to_string(),
                                bool_value: None,
                                int_value: Some(i),
                                float_value: None,
                                str_value: None,
                                array_items: Box::new(vec![]),
                                object_items: Box::new(vec![]),
                            });
                        }
                        if n.is_u64() {
                            return Err(JSONDecodeError {
                                message: "json integer out of range for sifr int"
                                    .to_string()
                                    .to_string(),
                                line: 0,
                                column: 0,
                            });
                        }
                        if let Some(f) = n.as_f64() {
                            return Ok(JsonValue {
                                kind: "float".to_string().to_string(),
                                bool_value: None,
                                int_value: None,
                                float_value: Some(f),
                                str_value: None,
                                array_items: Box::new(vec![]),
                                object_items: Box::new(vec![]),
                            });
                        }
                        return Err(JSONDecodeError {
                            message: "unsupported json number representation"
                                .to_string()
                                .to_string(),
                            line: 0,
                            column: 0,
                        });
                    }
                    serde_json::Value::String(s) => {
                        return Ok(JsonValue {
                            kind: "str".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: None,
                            str_value: Some(s),
                            array_items: Box::new(vec![]),
                            object_items: Box::new(vec![]),
                        });
                    }
                    serde_json::Value::Array(items) => {
                        let mut converted = vec![];
                        for item in items {
                            converted.push(__sifr_json_value_from_serde(item)?);
                        }
                        return Ok(JsonValue {
                            kind: "array".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(converted),
                            object_items: Box::new(vec![]),
                        });
                    }
                    serde_json::Value::Object(entries) => {
                        let mut converted = vec![];
                        for entry in entries {
                            let entry_key = entry.0;
                            let entry_value = entry.1;
                            let converted_value = __sifr_json_value_from_serde(
                                entry_value,
                            )?;
                            converted.push((entry_key, converted_value));
                        }
                        return Ok(JsonValue {
                            kind: "object".to_string().to_string(),
                            bool_value: None,
                            int_value: None,
                            float_value: None,
                            str_value: None,
                            array_items: Box::new(vec![]),
                            object_items: Box::new(converted),
                        });
                    }
                }
            }
            serde_json::from_str::<serde_json::Value>(__json_input.as_ref())
                .map_err(|e| JSONDecodeError {
                    message: e.to_string(),
                    line: e.line() as i64,
                    column: e.column() as i64,
                })
                .and_then(|parsed| __sifr_json_value_from_serde(parsed))
        })?;
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
fn load_handle(fh: &FileHandle) -> Result<JsonValue, Error> {
    let content_result: Result<String, IOError> = fh.read();
    let __sifr_try_res: Result<Result<JsonValue, Error>, IOError> = (|| {
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
fn load(path: &String) -> Result<JsonValue, Error> {
    let content_result: Result<String, IOError> = std::fs::read_to_string(&path)
        .map_err(__io_err);
    let __sifr_try_res: Result<Result<JsonValue, Error>, IOError> = (|| {
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
fn dumps(value: &JsonValue) -> String {
    return {
        let __json_value = value;
        fn __sifr_json_value_to_serde(value: &JsonValue) -> serde_json::Value {
            match value.kind.as_str() {
                "null" => {
                    return serde_json::Value::Null;
                }
                "bool" => {
                    if let Some(v) = value.bool_value {
                        return serde_json::Value::from(v);
                    }
                    return serde_json::Value::Null;
                }
                "int" => {
                    if let Some(v) = value.int_value {
                        return serde_json::Value::from(v);
                    }
                    return serde_json::Value::Null;
                }
                "float" => {
                    if let Some(v) = value.float_value {
                        return serde_json::Value::from(v);
                    }
                    return serde_json::Value::Null;
                }
                "str" => {
                    if let Some(v) = value.str_value.clone() {
                        return serde_json::Value::String(v);
                    }
                    return serde_json::Value::Null;
                }
                "array" => {
                    let mut converted = vec![];
                    for item in value.array_items.as_ref().iter().cloned() {
                        converted.push(__sifr_json_value_to_serde(&item));
                    }
                    return serde_json::Value::Array(converted);
                }
                "object" => {
                    let mut converted = serde_json::Map::new();
                    for entry in value.object_items.as_ref().iter().cloned() {
                        let entry_key = entry.0;
                        let entry_value = entry.1;
                        converted
                            .insert(entry_key, __sifr_json_value_to_serde(&entry_value));
                    }
                    return serde_json::Value::Object(converted);
                }
                _ => {
                    return serde_json::Value::Null;
                }
            }
        }
        serde_json::to_string(&__sifr_json_value_to_serde(&__json_value))
            .unwrap_or_else(|_err| "null".to_string().to_string())
    };
}

// --- stdlib: sifr.configparser ---
fn __const_DEFAULTSECT() -> String {
    return "DEFAULT".to_string().to_string();
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParsingError {
    line: i64,
    message: String,
}
impl ParsingError {
    fn new(line: i64, message: String) -> Self {
        return Self {
            line: line,
            message: message,
        };
    }
}
impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
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
        return Self {
            name: format!("{}{}", name, "".to_string()),
            _values: _copy_values(&values),
        };
    }
    fn has_option(&self, option: &String) -> bool {
        return _has_option_key(&self._values.clone(), &_normalize_option(option));
    }
    fn get(
        &self,
        option: &String,
        fallback: &Option<String>,
        raw: bool,
    ) -> Option<String> {
        let normalized: String = _normalize_option(option);
        if _has_option_key(&self._values.clone(), &normalized) {
            let value: Option<String> = _lookup_option(
                &self._values.clone(),
                &normalized,
            );
            let Some(value) = value else {
                return None;
            };
            if raw {
                return Some(value);
            }
            return Some(_resolve_interpolation(&value, &self._values.clone(), 0 as i64));
        }
        return _copy_optional_str(fallback);
    }
    fn options(&self) -> Vec<String> {
        let mut names: Vec<String> = vec![];
        for key in self._values.clone().keys().cloned() {
            names.push(key);
        }
        return names;
    }
    fn items(&self) -> Vec<(String, Option<String>)> {
        let mut pairs: Vec<(String, Option<String>)> = vec![];
        for (key, value) in self
            ._values
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            pairs.push((key, _copy_optional_str(&value)));
        }
        return pairs;
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
        if let Some(defaults) = defaults {
            for (key, value) in defaults
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                let normalized: String = _normalize_option(&key);
                {
                    let __assign_key = normalized;
                    let __assign_value = _copy_optional_str(&value);
                    defaults_map.insert(__assign_key, __assign_value);
                }
            }
        }
        return Self {
            strict: strict,
            allow_no_value: allow_no_value,
            _defaults: defaults_map,
            _sections: sections_map,
        };
    }
    fn defaults(&self) -> HashMap<String, Option<String>> {
        return _copy_values(&self._defaults.clone());
    }
    fn read_string(&mut self, text: &String) -> Result<(), ParsingError> {
        let mut current_section: String = "".to_string();
        let default_section: String = _default_section();
        for (line_no, raw_line) in Box::new(
            (text
                .split(&"\n".to_string())
                .map(|s| s.to_string())
                .collect::<Vec<String>>())
                .into_iter()
                .enumerate()
                .map(|__pair| ((__pair.0 as i64) + (1 as i64), __pair.1)),
        ) {
            let line: String = raw_line.trim().to_string();
            if (((line == "".to_string()) || (line.starts_with(&"#".to_string())))
                || (line.starts_with(&";".to_string())))
            {
                continue;
            }
            if ((line.starts_with(&"[".to_string()))
                && (line.ends_with(&"]".to_string())))
            {
                let section_name: String = line
                    .chars()
                    .skip((1 as i64) as usize)
                    .take(
                        (((line.chars().count() as i64) - (1 as i64)) as usize)
                            - ((1 as i64) as usize),
                    )
                    .collect::<String>()
                    .trim()
                    .to_string();
                if section_name == "".to_string() {
                    return Err(
                        ParsingError::new(line_no, "section name is empty".to_string()),
                    );
                }
                if section_name == default_section {
                    current_section = _default_section();
                    continue;
                }
                if ((self.strict)
                    && ((self._sections.clone()).contains_key(&(section_name))))
                {
                    return Err(
                        ParsingError::new(
                            line_no,
                            format!(
                                "{}{}", "duplicate section: ".to_string(), section_name
                            ),
                        ),
                    );
                }
                current_section = format!("{}{}", section_name, "".to_string());
                if !((self._sections.clone()).contains_key(&(section_name))) {
                    self._sections.insert(section_name, HashMap::from([]));
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
                if (current_section == "".to_string())
                    || (current_section == default_section)
                {
                    self._defaults
                        .insert(option_name, _copy_optional_str(&option_value));
                } else {
                    let section_key: String = format!(
                        "{}{}", current_section, "".to_string()
                    );
                    let mut section_found: bool = false;
                    for (section_name, section_values) in self
                        ._sections
                        .clone()
                        .iter()
                        .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                        .collect::<Vec<_>>()
                    {
                        if section_name != section_key {
                            continue;
                        }
                        if ((self.strict)
                            && (_has_option_key(&section_values, &option_name)))
                        {
                            return Err(
                                ParsingError::new(
                                    line_no,
                                    format!(
                                        "{}{}", "duplicate option: ".to_string(), option_name
                                    ),
                                ),
                            );
                        }
                        let mut updated_section: HashMap<String, Option<String>> = _copy_values(
                            &section_values,
                        );
                        {
                            let __assign_key = option_name;
                            let __assign_value = _copy_optional_str(&option_value);
                            updated_section.insert(__assign_key, __assign_value);
                        }
                        self._sections.insert(section_name, updated_section);
                        section_found = true;
                        break;
                    }
                }
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(e);
            }
        }
        return Ok(());
    }
    fn read(&mut self, path: &String) -> Result<Vec<String>, IOError> {
        let __sifr_try_res: Result<Result<Vec<String>, IOError>, IOError> = (|| {
            let content: String = std::fs::read_to_string(&path).map_err(__io_err)?;
            let __sifr_try_res: Result<(), ParsingError> = (|| {
                let _: () = self.read_string(&content)?;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(
                    IOError::new(
                        format!(
                            "{}{}{}{}", "parse error on line ".to_string(), format!("{}",
                            e.line), ": ".to_string(), e.message
                        ),
                    ),
                );
            }
            let loaded_path: String = format!("{}{}", path, "".to_string());
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
            names.push(section);
        }
        return names;
    }
    fn has_section(&self, section: &String) -> bool {
        return (self._sections.clone()).contains_key((section).as_str());
    }
    fn options(&self, section: &String) -> Vec<String> {
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        let mut names: Vec<String> = vec![];
        for option in merged.keys().cloned() {
            names.push(option);
        }
        return names;
    }
    fn items(&self, section: &String) -> Vec<(String, Option<String>)> {
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        let mut items: Vec<(String, Option<String>)> = vec![];
        for (option, value) in merged
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            items.push((option, _copy_optional_str(&value)));
        }
        return items;
    }
    fn _merged_section(&self, section: &String) -> HashMap<String, Option<String>> {
        let mut merged: HashMap<String, Option<String>> = _copy_values(
            &self._defaults.clone(),
        );
        let default_section: String = _default_section();
        if *section == default_section {
            return merged;
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
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
                    let __assign_key = option;
                    let __assign_value = _copy_optional_str(&value);
                    merged.insert(__assign_key, __assign_value);
                }
            }
            return merged;
        }
        return merged;
    }
    fn has_option(&self, section: &String, option: &String) -> bool {
        let normalized: String = _normalize_option(option);
        let default_section: String = _default_section();
        if *section == default_section {
            return (self._defaults.clone()).contains_key(&(normalized));
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
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
            return (self._defaults.clone()).contains_key(&(normalized));
        }
        return false;
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
            let Some(raw_value) = raw_value else {
                return None;
            };
            if raw {
                return Some(raw_value);
            }
            return Some(_resolve_interpolation(&raw_value, &merged, 0 as i64));
        }
        if !(self.has_section(section)) {
            if _has_option_key(&self._defaults.clone(), &normalized) {
                let default_value: Option<String> = _lookup_option(
                    &self._defaults.clone(),
                    &normalized,
                );
                let Some(default_value) = default_value else {
                    return None;
                };
                if raw {
                    return Some(default_value);
                }
                return Some(_resolve_interpolation(&default_value, &merged, 0 as i64));
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
        return Some(_resolve_interpolation(&raw_value2, &merged, 0 as i64));
    }
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
        if (((normalized == "1".to_string()) || (normalized == "yes".to_string()))
            || (normalized == "true".to_string())) || (normalized == "on".to_string())
        {
            return Some(true);
        }
        if (((normalized == "0".to_string()) || (normalized == "no".to_string()))
            || (normalized == "false".to_string())) || (normalized == "off".to_string())
        {
            return Some(false);
        }
        return fallback;
    }
    fn set(&mut self, section: &String, option: &String, value: &Option<String>) {
        let normalized: String = _normalize_option(option);
        let default_section: String = _default_section();
        if *section == default_section {
            self._defaults.insert(normalized, _copy_optional_str(value));
            return;
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
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
                let __assign_key = normalized;
                let __assign_value = _copy_optional_str(value);
                updated_section.insert(__assign_key, __assign_value);
            }
            self._sections.insert(section_name, updated_section);
            return;
        }
        if !((self._sections.clone()).contains_key((section).as_str())) {
            self._sections.insert(section.clone(), HashMap::from([]));
        }
        let mut created_section: HashMap<String, Option<String>> = HashMap::from([]);
        for (section_name, section_values) in self
            ._sections
            .clone()
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
            let __assign_key = normalized;
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
        if (self._sections.clone()).contains_key((section).as_str()) {
            return;
        }
        self._sections.insert(section.clone(), HashMap::from([]));
    }
    fn remove_option(&mut self, section: &String, option: &String) -> bool {
        let normalized: String = _normalize_option(option);
        let default_section: String = _default_section();
        if *section == default_section {
            if (self._defaults.clone()).contains_key(&(normalized)) {
                self._defaults = _without_option(&self._defaults.clone(), &normalized);
                return true;
            }
            return false;
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            if section_name != *section {
                continue;
            }
            if _has_option_key(&section_values, &normalized) {
                self._sections
                    .insert(section_name, _without_option(&section_values, &normalized));
                return true;
            }
            return false;
        }
        return false;
    }
    fn remove_section(&mut self, section: &String) -> bool {
        let default_section: String = _default_section();
        if *section == default_section {
            return false;
        }
        if (self._sections.clone()).contains_key((section).as_str()) {
            self._sections = _without_section(&self._sections.clone(), section);
            return true;
        }
        return false;
    }
    fn proxy(&self, section: &String) -> Option<SectionProxy> {
        let default_section: String = _default_section();
        if ((section.clone() != default_section) && (!(self.has_section(section)))) {
            return None;
        }
        let merged: HashMap<String, Option<String>> = self._merged_section(section);
        return Some(SectionProxy::new((section).clone(), merged));
    }
    fn to_ini_string(&self) -> String {
        let mut lines: Vec<String> = vec![];
        if (self._defaults.clone().len() as i64) > (0 as i64) {
            lines
                .push(
                    format!(
                        "{}{}", format!("{}{}", "[".to_string(), _default_section()), "]"
                        .to_string()
                    ),
                );
            for (key, value) in self
                ._defaults
                .clone()
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if value.is_none() {
                    lines.push(key);
                } else {
                    if let Some(value) = value {
                        lines
                            .push(
                                format!(
                                    "{}{}", format!("{}{}", key, " = ".to_string()), value
                                ),
                            );
                    }
                }
            }
            lines.push("".to_string());
        }
        for (section_name, section_values) in self
            ._sections
            .clone()
            .iter()
            .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
            .collect::<Vec<_>>()
        {
            lines
                .push(
                    format!(
                        "{}{}", format!("{}{}", "[".to_string(), section_name), "]"
                        .to_string()
                    ),
                );
            for (key, value) in section_values
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if value.is_none() {
                    lines.push(key);
                } else {
                    if let Some(value) = value {
                        lines
                            .push(
                                format!(
                                    "{}{}", format!("{}{}", key, " = ".to_string()), value
                                ),
                            );
                    }
                }
            }
            lines.push("".to_string());
        }
        if (lines.len() as i64) > (0 as i64) {
            let maybe_last: Option<String> = {
                let __sifr_index_list = &lines;
                let __sifr_index_i = (lines.len() as i64) - (1 as i64);
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).cloned()
            };
            if ((maybe_last != None) && (maybe_last == Some("".to_string()))) {
                let _: String = {
                    let Some(__sifr_nonempty_pop_value) = lines.pop() else {
                        unreachable!(
                            "compiler-verified non-empty pop should return Some"
                        );
                    };
                    __sifr_nonempty_pop_value
                };
            }
        }
        return lines.join(&"\n".to_string());
    }
    fn write(&self, path: &String) -> Result<(), IOError> {
        let payload: String = self.to_ini_string();
        return std::fs::write(&path, payload.as_bytes()).map(|_| ()).map_err(__io_err);
    }
}
fn _default_section() -> String {
    return format!("{}{}", __const_DEFAULTSECT(), "".to_string());
}
fn _normalize_option(option: &String) -> String {
    return option.to_lowercase().trim().to_string();
}
fn _some_str(value: &String) -> Option<String> {
    return Some(format!("{}{}", value, "".to_string()));
}
fn _copy_optional_str(value: &Option<String>) -> Option<String> {
    if let Some(value) = value.as_ref() {
        return _some_str(value);
    }
    return None;
}
fn _has_option_key(values: &HashMap<String, Option<String>>, key: &String) -> bool {
    for current_key in values.keys().cloned() {
        if current_key == *key {
            return true;
        }
    }
    return false;
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
    return None;
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
            let __assign_key = key;
            let __assign_value = _copy_optional_str(&value);
            copied.insert(__assign_key, __assign_value);
        }
    }
    return copied;
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
            let __assign_key = key;
            let __assign_value = _copy_optional_str(&value);
            copied.insert(__assign_key, __assign_value);
        }
    }
    return copied;
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
            let __assign_key = key;
            let __assign_value = _copy_values(&section);
            copied.insert(__assign_key, __assign_value);
        }
    }
    return copied;
}
fn _find_delimiter(line: &String) -> Option<String> {
    if line.contains(&"=".to_string()) {
        return Some("=".to_string());
    }
    if line.contains(&":".to_string()) {
        return Some(":".to_string());
    }
    return None;
}
fn _split_option_line(
    line: &String,
    allow_no_value: bool,
    line_no: i64,
) -> Result<(String, Option<String>), ParsingError> {
    let delimiter: Option<String> = _find_delimiter(line);
    let Some(delimiter) = delimiter else {
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
    let parts: Vec<String> = if (1 as i64) < 0 {
        line.split(&delimiter).map(|s| s.to_string()).collect::<Vec<String>>()
    } else {
        line.splitn(((1 as i64) + 1) as usize, &delimiter)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
    if (parts.len() as i64) != (2 as i64) {
        return Err(ParsingError::new(line_no, "invalid option line".to_string()));
    }
    let raw_key: Option<String> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 0 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let raw_value: Option<String> = {
        let __sifr_index_list = &parts;
        let __sifr_index_i = 1 as i64;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
    let Some(raw_key) = raw_key else {
        return Err(ParsingError::new(line_no, "option name is missing".to_string()));
    };
    let key: String = _normalize_option(&raw_key);
    if key == "".to_string() {
        return Err(ParsingError::new(line_no, "option name is empty".to_string()));
    }
    let Some(raw_value) = raw_value else {
        return Ok((key, None));
    };
    let stripped_value: Option<String> = _some_str(&raw_value.trim().to_string());
    return Ok((key, stripped_value));
}
fn _resolve_interpolation(
    value: &String,
    merged: &HashMap<String, Option<String>>,
    depth: i64,
) -> String {
    if depth >= (8 as i64) {
        return format!("{}{}", value, "".to_string());
    }
    if !value.contains(&"%(".to_string()) {
        return format!("{}{}", value, "".to_string());
    }
    let mut result: String = "".to_string();
    let mut replaced: bool = false;
    let mut i: i64 = 0 as i64;
    while i < (value.chars().count() as i64) {
        let ch: String = _char_at(value, i);
        if (((ch == "%".to_string())
            && ((i + (1 as i64)) < (value.chars().count() as i64)))
            && (_char_at(value, i + (1 as i64)) == "(".to_string()))
        {
            let mut j: i64 = i + (2 as i64);
            let mut key: String = "".to_string();
            let mut matched: bool = false;
            while j < (value.chars().count() as i64) {
                let part: String = _char_at(value, j);
                if (((part == ")".to_string())
                    && ((j + (1 as i64)) < (value.chars().count() as i64)))
                    && (_char_at(value, j + (1 as i64)) == "s".to_string()))
                {
                    matched = true;
                    let normalized_key: String = _normalize_option(&key);
                    let replacement: Option<String> = _lookup_option(
                        merged,
                        &normalized_key,
                    );
                    if replacement.is_none() {
                        result = format!(
                            "{}{}{}{}", result, "%(".to_string(), key, ")s".to_string()
                        );
                    } else {
                        if let Some(replacement) = replacement {
                            replaced = true;
                            result = format!("{}{}", result, replacement);
                        }
                    }
                    i = j + (2 as i64);
                    break;
                }
                key = format!("{}{}", key, part);
                j = j + (1 as i64);
            }
            if matched {
                continue;
            }
        }
        result = format!("{}{}", result, ch);
        i = i + (1 as i64);
    }
    if replaced {
        return _resolve_interpolation(&result, merged, depth + (1 as i64));
    }
    return result;
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self { message: message, kind: "Other".to_string() };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {
}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound { "FileNotFound".to_string() } else { if e.kind() == std::io::ErrorKind::PermissionDenied { "PermissionDenied".to_string() } else { if e.kind() == std::io::ErrorKind::AlreadyExists { "FileExists".to_string() } else { "Other".to_string() } } };
    return IOError { message: msg, kind: kind };
}

#[derive(Debug, Clone)]
struct Error {
    message: String,
}

impl Error {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
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
        return Self { message: message };
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
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
        return Self { message: message };
    }
}

impl std::fmt::Display for ValueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
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
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {
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
    return __SIFR_NEXT_FILE_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

fn main() {
    let mut encoder: JSONEncoder = JSONEncoder::new(Some(2 as i64), false, true);
    let mut decoder: JSONDecoder = JSONDecoder::new();
    let payload: JsonValue = from_object(&vec![("module".to_string(), from_str(&"config_json_csv".to_string())), ("version".to_string(), from_int(1 as i64))]);
    let encoded: String = encoder.encode(&payload);
    assert!(encoded == "{\"module\":\"config_json_csv\",\"version\":1}".to_string());
    let mut decoded_ok: bool = false;
    let __sifr_try_res: Result<(), Error> = (|| {
    let decoded_value: JsonValue = (decoder.decode(&encoded)).map_err(|__e| Error::new(__e.to_string()))?;
    decoded_ok = format!("{}", decoded_value) == encoded;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    assert!(decoded_ok);
    let mut parser: ConfigParser = ConfigParser::new(None, false, false);
    let __sifr_try_res: Result<(), ParsingError> = (|| {
    let _: () = parser.read_string(&"[DEFAULT]\nbase=/tmp\n[paths]\ncache=%(base)s/cache\n".to_string())?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        assert!(false);
    }
    assert!(parser.get(&"paths".to_string(), &"cache".to_string(), &None, false) == Some("/tmp/cache".to_string()));
    let mut registry: DialectRegistry = dialect_registry();
    registry.register(&"pipe".to_string(), &Dialect::new("|".to_string(), "\"".to_string(), "".to_string(), true, false, "\n".to_string(), 0 as i64));
    let d: Option<Dialect> = registry.get(&"pipe".to_string());
    assert!(d.is_some());
    if let Some(d) = d {
        let mut r: reader = reader::new("a|b\n1|2".to_string(), Some(d), ",".to_string(), "\"".to_string(), "".to_string(), true, false, 0 as i64);
        assert!(format!("{:?}", r.rows()) == "[[\"a\", \"b\"], [\"1\", \"2\"]]".to_string());
    }
    assert!(registry.unregister(&"pipe".to_string()));
}
