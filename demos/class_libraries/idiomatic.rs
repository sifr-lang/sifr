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
                    let __n = std::io::BufRead::read_line(__r, &mut __line).map_err(__io_err)?;
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
                        let __n =
                            std::io::BufRead::read_line(__r, &mut __line).map_err(__io_err)?;
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
            f,
            "FileHandle(_handle={}, _mode={}, _closed={})",
            self._handle, self._mode, self._closed
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
            f,
            "BinaryFileHandle(_handle={}, _mode={}, _closed={})",
            self._handle, self._mode, self._closed
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
                (self._buffer.clone())
                    .chars()
                    .skip((tail_start).max(0) as usize),
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
            f,
            "StringIO(_buffer={}, _cursor={}, _closed={})",
            self._buffer, self._cursor, self._closed
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
            _buffer: initial
                .iter()
                .map(|__byte| *__byte as i64)
                .collect::<Vec<i64>>(),
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
                                "byte out of range at index {}: {}",
                                __pair.0, *__pair.1
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
    return format!(
        "{}{}",
        "invalid whence: ".to_string(),
        format!("{}", whence)
    );
}
fn _negative_seek_error(offset: i64) -> String {
    return format!(
        "{}{}",
        "negative seek position: ".to_string(),
        format!("{}", offset)
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

// --- stdlib: sifr.logging ---
const DEBUG: i64 = 10 as i64;
const INFO: i64 = 20 as i64;
const WARNING: i64 = 30 as i64;
const ERROR: i64 = 40 as i64;
const CRITICAL: i64 = 50 as i64;
const NOTSET: i64 = 0 as i64;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Formatter {
    _fmt: String,
}
impl Formatter {
    fn new(fmt: String) -> Self {
        return Self { _fmt: fmt };
    }
    fn template(&self) -> String {
        return self._fmt.clone();
    }
    fn format(&self, level: &String, name: &String, msg: &String) -> String {
        let mut result: String = self._fmt.clone();
        result = result.replace(&"%(levelname)s".to_string(), &level);
        result = result.replace(&"%(name)s".to_string(), &name);
        result = result.replace(&"%(message)s".to_string(), &msg);
        return result;
    }
}
impl std::fmt::Display for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Formatter(_fmt={})", self._fmt);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StreamHandler {
    _level: i64,
    _formatter: Formatter,
}
impl StreamHandler {
    fn new(level: i64) -> Self {
        return Self {
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn _allows(&self, level_num: i64) -> bool {
        if self._level == NOTSET {
            return true;
        }
        return level_num >= self._level;
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
        return write!(
            f,
            "StreamHandler(_level={}, _formatter={})",
            self._level, self._formatter
        );
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
        return Self {
            _path: format!("{}{}", path, "".to_string()),
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn path(&self) -> String {
        return self._path.clone();
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn _allows(&self, level_num: i64) -> bool {
        if self._level == NOTSET {
            return true;
        }
        return level_num >= self._level;
    }
    fn emit(&mut self, level: &String, name: &String, msg: &String) {
        let level_num: i64 = _level_name_to_num(level);
        if !(self._allows(level_num)) {
            return;
        }
        let line: String = format!(
            "{}{}",
            self._formatter.clone().format(level, name, msg),
            "\n".to_string()
        );
        let __sifr_try_res: Result<(), IOError> = (|| {
            let mut fh: FileHandle = (|| {
                let __path = self._path.clone().to_string();
                let __mode = "a".to_string().to_string();
                let __handle_id = __sifr_next_file_handle_id();
                match __mode.as_str() {
                    "r" | "rt" => {
                        let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                        let __reader = std::io::BufReader::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "w" | "wt" => {
                        let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
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
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "rb" => {
                        let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                        let __reader = std::io::BufReader::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    "wb" => {
                        let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                        let __writer = std::io::BufWriter::new(__f);
                        __SIFR_FILE_HANDLES
                            .lock()
                            .unwrap_or_else(|__err| __err.into_inner())
                            .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
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
                        return Ok(FileHandle {
                            _handle: __handle_id,
                            _mode: __mode.to_string(),
                        });
                    }
                    _ => {
                        return Err(IOError {
                            message: format!("invalid mode: {}", __mode),
                            kind: "Other".to_string(),
                        });
                    }
                }
            })()?;
            let __sifr_try_res: Result<(), IOError> = (|| {
                let _: () = fh.write(&line)?;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e2 = __sifr_try_err.clone();
                let _: String = e2.message;
            }
            fh.close();
            return Ok(());
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            let _: String = e.message;
        }
    }
}
impl std::fmt::Display for FileHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "FileHandler(_path={}, _level={}, _formatter={})",
            self._path, self._level, self._formatter
        );
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NullHandler {
    _level: i64,
    _formatter: Formatter,
}
impl NullHandler {
    fn new(level: i64) -> Self {
        return Self {
            _level: level,
            _formatter: Formatter::new("%(levelname)s:%(name)s:%(message)s".to_string()),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn level(&self) -> i64 {
        return self._level;
    }
    fn set_formatter(&mut self, fmt: &Formatter) {
        self._formatter = Formatter::new(format!("{}{}", fmt._fmt, "".to_string()));
    }
    fn format_template(&mut self) -> String {
        return self._formatter.clone().template();
    }
    fn emit(&self, level: &String, name: &String, msg: &String) {
        let _: String = (level).clone();
        let _: String = (name).clone();
        let _: String = (msg).clone();
    }
}
impl std::fmt::Display for NullHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "NullHandler(_level={}, _formatter={})",
            self._level, self._formatter
        );
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
        return Self {
            _name: name,
            _level: level,
            _log_path: "".to_string(),
            _handler_kind: "".to_string(),
            _handler_path: "".to_string(),
            _handler_level: NOTSET,
            _handler_fmt: "%(levelname)s:%(name)s:%(message)s".to_string(),
        };
    }
    fn set_level(&mut self, level: i64) {
        self._level = level;
    }
    fn set_file(&mut self, path: &String) {
        self._log_path = format!("{}{}", path, "".to_string());
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
        if self._handler_level == NOTSET {
            return true;
        }
        return level_num >= self._handler_level;
    }
    fn _handler_line(&self, level: &String, msg: &String) -> String {
        let mut formatter: Formatter = Formatter::new(self._handler_fmt.clone());
        return formatter.format(level, &self._name.clone(), msg);
    }
    fn _emit(&self, level: &String, level_num: i64, msg: &String) {
        if self._level > level_num {
            return;
        }
        if self._handler_kind.clone() == "null".to_string() {
            return;
        }
        if self._handler_kind.clone() == "stream".to_string() {
            if self._handler_allows(level_num) {
                println!("{}", self._handler_line(level, msg));
            }
            return;
        }
        if self._handler_kind.clone() == "file".to_string() {
            if ((self._handler_allows(level_num)) && (self._handler_path.clone() != "".to_string()))
            {
                let line: String =
                    format!("{}{}", self._handler_line(level, msg), "\n".to_string());
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let mut fh: FileHandle = (|| {
                        let __path = self._handler_path.clone().to_string();
                        let __mode = "a".to_string().to_string();
                        let __handle_id = __sifr_next_file_handle_id();
                        match __mode.as_str() {
                            "r" | "rt" => {
                                let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                                let __reader = std::io::BufReader::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "w" | "wt" => {
                                let __f =
                                    std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
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
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "rb" => {
                                let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                                let __reader = std::io::BufReader::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            "wb" => {
                                let __f =
                                    std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                                let __writer = std::io::BufWriter::new(__f);
                                __SIFR_FILE_HANDLES
                                    .lock()
                                    .unwrap_or_else(|__err| __err.into_inner())
                                    .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
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
                                return Ok(FileHandle {
                                    _handle: __handle_id,
                                    _mode: __mode.to_string(),
                                });
                            }
                            _ => {
                                return Err(IOError {
                                    message: format!("invalid mode: {}", __mode),
                                    kind: "Other".to_string(),
                                });
                            }
                        }
                    })()?;
                    let __sifr_try_res: Result<(), IOError> = (|| {
                        let _: () = fh.write(&line)?;
                        return Ok(());
                    })();
                    if let Err(__sifr_try_err) = __sifr_try_res {
                        let e2 = __sifr_try_err.clone();
                        let _: String = e2.message;
                    }
                    fh.close();
                    return Ok(());
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e = __sifr_try_err.clone();
                    let _: String = e.message;
                }
            }
            return;
        }
        let line: String = format!(
            "{}{}{}{}{}{}",
            "[".to_string(),
            level,
            "] ".to_string(),
            self._name.clone(),
            ": ".to_string(),
            msg
        );
        println!("{}", line);
        if self._log_path.clone() != "".to_string() {
            let __sifr_try_res: Result<(), IOError> = (|| {
                let mut fh: FileHandle = (|| {
                    let __path = self._log_path.clone().to_string();
                    let __mode = "a".to_string().to_string();
                    let __handle_id = __sifr_next_file_handle_id();
                    match __mode.as_str() {
                        "r" | "rt" => {
                            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                            let __reader = std::io::BufReader::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::TextRead(__reader));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "w" | "wt" => {
                            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::TextWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
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
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "rb" => {
                            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
                            let __reader = std::io::BufReader::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        "wb" => {
                            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
                            let __writer = std::io::BufWriter::new(__f);
                            __SIFR_FILE_HANDLES
                                .lock()
                                .unwrap_or_else(|__err| __err.into_inner())
                                .insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
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
                            return Ok(FileHandle {
                                _handle: __handle_id,
                                _mode: __mode.to_string(),
                            });
                        }
                        _ => {
                            return Err(IOError {
                                message: format!("invalid mode: {}", __mode),
                                kind: "Other".to_string(),
                            });
                        }
                    }
                })()?;
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let _: () = fh.write(&format!("{}{}", line, "\n".to_string()))?;
                    return Ok(());
                })();
                if let Err(__sifr_try_err) = __sifr_try_res {
                    let e2 = __sifr_try_err.clone();
                    let _: String = e2.message;
                }
                fh.close();
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _: String = e.message;
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
        return write!(
            f,
            "Logger(_name={}, _level={}, _log_path={}, _handler_kind={}, _handler_path={}, _handler_level={}, _handler_fmt={})",
            self._name, self._level, self._log_path, self._handler_kind, self
            ._handler_path, self._handler_level, self._handler_fmt
        );
    }
}
fn _level_name_to_num(level: &String) -> i64 {
    if level.clone() == "DEBUG".to_string() {
        return DEBUG;
    }
    if level.clone() == "INFO".to_string() {
        return INFO;
    }
    if level.clone() == "WARNING".to_string() {
        return WARNING;
    }
    if level.clone() == "ERROR".to_string() {
        return ERROR;
    }
    if level.clone() == "CRITICAL".to_string() {
        return CRITICAL;
    }
    return NOTSET;
}
fn getLogger(name: &String) -> Logger {
    let level: i64 = *__SIFR_GLOBAL_LOG_LEVEL
        .lock()
        .unwrap_or_else(|__err| __err.into_inner());
    return Logger::new((name).clone(), level);
}

// --- stdlib: sifr.graphlib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CycleError {
    message: String,
}
impl CycleError {
    fn new(message: String) -> Self {
        return Self { message: message };
    }
}
impl std::fmt::Display for CycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.message);
    }
}
impl std::error::Error for CycleError {}
#[derive(Debug, Clone, PartialEq)]
struct TopologicalSorter {
    nodes: Vec<i64>,
    from_nodes: Vec<i64>,
    to_nodes: Vec<i64>,
    max_node: i64,
    _prepared: bool,
    _ready_order: Vec<i64>,
    _next_index: i64,
}
impl TopologicalSorter {
    fn new() -> Self {
        return Self {
            nodes: vec![],
            from_nodes: vec![],
            to_nodes: vec![],
            max_node: -(1 as i64),
            _prepared: false,
            _ready_order: vec![],
            _next_index: 0 as i64,
        };
    }
    fn _record_node(&mut self, node: i64) {
        if !(_contains_int(&self.nodes.clone(), node)) {
            self.nodes.push(node);
        }
        if node > self.max_node {
            self.max_node = node;
        }
    }
    fn add(&mut self, node: i64, predecessor: i64) {
        self._record_node(node);
        self._record_node(predecessor);
        self.from_nodes.push(predecessor);
        self.to_nodes.push(node);
        self._prepared = false;
        self._ready_order = vec![];
        self._next_index = 0 as i64;
    }
    fn add_many(&mut self, node: i64, predecessors: &Vec<i64>) {
        self._record_node(node);
        if (predecessors.len() as i64) == (0 as i64) {
            self._prepared = false;
            self._ready_order = vec![];
            self._next_index = 0 as i64;
            return;
        }
        for predecessor in predecessors.iter().copied() {
            self.add(node, predecessor);
        }
    }
    fn _filter_order(&self, order: &Vec<i64>) -> Vec<i64> {
        let mut filtered: Vec<i64> = vec![];
        for candidate in order.iter().copied() {
            if _contains_int(&self.nodes.clone(), candidate) {
                filtered.push(candidate);
            }
        }
        return filtered;
    }
    fn prepare(&mut self) -> Result<(), CycleError> {
        self._prepared = false;
        self._ready_order = vec![];
        self._next_index = 0 as i64;
        if self.max_node < (0 as i64) {
            self._prepared = true;
            return Ok(());
        }
        let mut prepare_ok: bool = false;
        let __sifr_try_res: Result<(), CycleError> = (|| {
            let order: Vec<i64> = topological_sort(
                self.max_node + (1 as i64),
                &self.from_nodes.clone(),
                &self.to_nodes.clone(),
            )?;
            self._ready_order = self._filter_order(&order);
            self._prepared = true;
            prepare_ok = true;
            return Ok(());
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            self._prepared = false;
            self._ready_order = vec![];
            self._next_index = 0 as i64;
            return Err(CycleError::new(e.message));
        }
        if prepare_ok {
            return Ok(());
        }
        return Ok(());
    }
    fn get_ready(&mut self) -> Result<Vec<i64>, CycleError> {
        if !(self._prepared) {
            let __sifr_try_res: Result<(), CycleError> = (|| {
                let _prepared: () = self.prepare()?;
                let _: () = _prepared;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(CycleError::new(e.message));
            }
        }
        if self._next_index < (self._ready_order.clone().len() as i64) {
            let current: Option<i64> = {
                let __sifr_index_list = &self._ready_order;
                let __sifr_index_i = self._next_index;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            if let Some(current) = current {
                return Ok(vec![current]);
            }
        }
        return Ok(vec![]);
    }
    fn done(&mut self, node: i64) {
        if !(self._prepared) {
            return;
        }
        if self._next_index >= (self._ready_order.clone().len() as i64) {
            return;
        }
        let current: Option<i64> = {
            let __sifr_index_list = &self._ready_order;
            let __sifr_index_i = self._next_index;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if ((current != None) && (current == Some(node))) {
            self._next_index = self._next_index + (1 as i64);
        }
    }
    fn is_active(&self) -> bool {
        if !(self._prepared) {
            return false;
        }
        return self._next_index < (self._ready_order.clone().len() as i64);
    }
    fn reset(&mut self) {
        self._prepared = false;
        self._ready_order = vec![];
        self._next_index = 0 as i64;
    }
    fn static_order(&self) -> Result<Vec<i64>, CycleError> {
        if self.max_node < (0 as i64) {
            return Ok(vec![]);
        }
        let __sifr_try_res: Result<Result<Vec<i64>, CycleError>, CycleError> = (|| {
            let full_order: Vec<i64> = topological_sort(
                self.max_node + (1 as i64),
                &self.from_nodes.clone(),
                &self.to_nodes.clone(),
            )?;
            return Ok(Ok(self._filter_order(&full_order)));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                return Err(CycleError::new(e.message));
            }
        }
    }
}
fn _contains_int(values: &Vec<i64>, target: i64) -> bool {
    for value in values.iter().copied() {
        if value == target {
            return true;
        }
    }
    return false;
}
fn topological_sort(
    num_nodes: i64,
    from_nodes: &Vec<i64>,
    to_nodes: &Vec<i64>,
) -> Result<Vec<i64>, CycleError> {
    let mut result: Vec<i64> = vec![];
    let mut visited: Vec<i64> = vec![];
    let mut i: i64 = 0 as i64;
    while i < num_nodes {
        visited.push(0 as i64);
        i = i + (1 as i64);
    }
    let mut processed: i64 = 0 as i64;
    while processed < num_nodes {
        let mut found_any: bool = false;
        let mut node: i64 = 0 as i64;
        while node < num_nodes {
            let v: Option<i64> = {
                let __sifr_index_list = &visited;
                let __sifr_index_i = node;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            };
            if let Some(v) = v {
                if v == (0 as i64) {
                    let mut has_dep: bool = false;
                    let mut j: i64 = 0 as i64;
                    while j < (to_nodes.len() as i64) {
                        let to_val: Option<i64> = Some(to_nodes[j as usize]);
                        let from_val: Option<i64> = {
                            let __sifr_index_list = &from_nodes;
                            let __sifr_index_i = j;
                            let __sifr_index_norm = if __sifr_index_i < 0 {
                                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                            } else {
                                __sifr_index_i as usize
                            };
                            __sifr_index_list.get(__sifr_index_norm).copied()
                        };
                        if let Some(to_val) = to_val {
                            if let Some(from_val) = from_val {
                                if to_val == node {
                                    let dep_v: Option<i64> = {
                                        let __sifr_index_list = &visited;
                                        let __sifr_index_i = from_val;
                                        let __sifr_index_norm = if __sifr_index_i < 0 {
                                            ((__sifr_index_list.len() as i64) + __sifr_index_i)
                                                as usize
                                        } else {
                                            __sifr_index_i as usize
                                        };
                                        __sifr_index_list.get(__sifr_index_norm).copied()
                                    };
                                    if let Some(dep_v) = dep_v {
                                        if dep_v == (0 as i64) {
                                            has_dep = true;
                                        }
                                    }
                                }
                            }
                        }
                        j = j + (1 as i64);
                    }
                    if !has_dep {
                        result.push(node);
                        {
                            let __idx_raw = node;
                            let __idx_norm = if __idx_raw < 0 {
                                (visited.len() as i64) + __idx_raw
                            } else {
                                __idx_raw
                            };
                            if __idx_norm >= 0 {
                                if let Some(__elem) = visited.get_mut(__idx_norm as usize) {
                                    *__elem = 1 as i64;
                                }
                            }
                        }
                        processed = processed + (1 as i64);
                        found_any = true;
                    }
                }
            }
            node = node + (1 as i64);
        }
        if !found_any {
            return Err(CycleError::new("cycle detected in graph".to_string()));
        }
    }
    return Ok(result);
}

// --- stdlib: sifr.pathlib ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Path {
    _path: String,
}
impl Path {
    fn new(path: String) -> Self {
        return Self { _path: path };
    }
    fn name(&self) -> String {
        return basename(&self._path.clone());
    }
    fn parent(&self) -> Path {
        return Path::new(dirname(&self._path.clone()));
    }
    fn suffix(&self) -> String {
        return extension(&self._path.clone());
    }
    fn stem(&self) -> String {
        return stem(&self._path.clone());
    }
    fn exists(&self) -> bool {
        return std::path::Path::new(&self._path.clone()).exists();
    }
    fn is_file(&self) -> bool {
        return std::path::Path::new(&self._path.clone()).is_file();
    }
    fn is_dir(&self) -> bool {
        return std::path::Path::new(&self._path.clone()).is_dir();
    }
    fn is_absolute(&self) -> bool {
        return is_absolute(&self._path.clone());
    }
    fn read_text(&self) -> Result<String, IOError> {
        return std::fs::read_to_string(&self._path.clone()).map_err(__io_err);
    }
    fn write_text(&self, content: &String) -> Result<(), IOError> {
        return std::fs::write(&self._path.clone(), content.as_bytes())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn mkdir(&self) -> Result<(), IOError> {
        return std::fs::create_dir_all(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn joinpath(&self, child: &String) -> Path {
        return Path::new(join_path(&self._path.clone(), child));
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._path.clone(), "".to_string());
    }
    fn touch(&self) -> Result<(), IOError> {
        return std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn unlink(&self) -> Result<(), IOError> {
        return std::fs::remove_file(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn rmdir(&self) -> Result<(), IOError> {
        return std::fs::remove_dir(&self._path.clone())
            .map(|_| ())
            .map_err(__io_err);
    }
    fn resolve(&self) -> Result<String, IOError> {
        return std::fs::canonicalize(&self._path.clone())
            .map(|p| p.to_string_lossy().to_string())
            .map_err(__io_err);
    }
    fn iterdir(&self) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        return _iterdir_to_iter(&self._path.clone());
    }
    fn with_name(&self, name: &String) -> Path {
        let parent: String = dirname(&self._path.clone());
        if parent == "".to_string() {
            return Path::new(format!("{}{}", name, "".to_string()));
        }
        return Path::new(format!(
            "{}{}",
            format!("{}{}", parent, "/".to_string()),
            name
        ));
    }
    fn with_suffix(&self, suffix: &String) -> Path {
        let s: String = stem(&self._path.clone());
        let parent: String = dirname(&self._path.clone());
        if parent == "".to_string() {
            return Path::new(format!("{}{}", s, suffix));
        }
        return Path::new(format!(
            "{}{}",
            format!("{}{}", format!("{}{}", parent, "/".to_string()), s),
            suffix
        ));
    }
    fn glob(&self, pattern: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        return _glob_to_iter(&self._path.clone(), pattern);
    }
    fn rglob(&self, pattern: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        return _rglob_to_iter(&self._path.clone(), pattern);
    }
}
impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "Path(_path={})", self._path);
    }
}
fn join_path(base: &String, child: &String) -> String {
    if (base.len() as i64) == (0 as i64) {
        return format!("{}{}", child, "".to_string());
    }
    let last: Option<String> = {
        let __sifr_index_str = &base;
        let __sifr_index_i = (base.chars().count() as i64) - (1 as i64);
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_str
            .chars()
            .nth(__sifr_index_norm)
            .map(|c| c.to_string())
    };
    if let Some(last) = last {
        if last == "/".to_string() {
            return format!("{}{}", base, child);
        }
    }
    return format!("{}{}{}", base, "/".to_string(), child);
}
fn basename(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "/".to_string() {
                return String::from_iter((path).chars().skip((i + (1 as i64)).max(0) as usize));
            }
        }
        i = i - (1 as i64);
    }
    return format!("{}{}", path, "".to_string());
}
fn dirname(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "/".to_string() {
                return String::from_iter(
                    (path)
                        .chars()
                        .skip(0 as usize)
                        .take(((i).max(0) - 0).max(0) as usize),
                );
            }
        }
        i = i - (1 as i64);
    }
    return "".to_string();
}
fn extension(path: &String) -> String {
    let mut i: i64 = (path.chars().count() as i64) - (1 as i64);
    while i >= (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = path.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == ".".to_string() {
                return String::from_iter((path).chars().skip((i).max(0) as usize));
            }
            if ch == "/".to_string() {
                return "".to_string();
            }
        }
        i = i - (1 as i64);
    }
    return "".to_string();
}
fn stem(path: &String) -> String {
    let base: String = basename(path);
    let mut i: i64 = (base.chars().count() as i64) - (1 as i64);
    while i > (0 as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = base.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == ".".to_string() {
                return String::from_iter(
                    (base)
                        .chars()
                        .skip(0 as usize)
                        .take(((i).max(0) - 0).max(0) as usize),
                );
            }
        }
        i = i - (1 as i64);
    }
    return format!("{}{}", base, "".to_string());
}
fn is_absolute(path: &String) -> bool {
    if (path.len() as i64) == (0 as i64) {
        return false;
    }
    if (path.chars().count() as i64) >= (3 as i64) {
        let colon: Option<String> = {
            let __sifr_index_str = &path;
            let __sifr_index_i = 1 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
        };
        let sep: Option<String> = {
            let __sifr_index_str = &path;
            let __sifr_index_i = 2 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
        };
        if let Some(colon) = colon {
            if let Some(sep) = sep {
                if (colon == ":".to_string())
                    && ((sep == "/".to_string()) || (sep == "\\".to_string()))
                {
                    return true;
                }
            }
        }
    }
    let first: Option<String> = Some({
        let Some(__indexed_char) = path.chars().nth((0 as i64) as usize) else {
            unreachable!("compiler-verified string index should be in range");
        };
        __indexed_char.to_string()
    });
    if let Some(first) = first {
        if (first == "/".to_string()) || (first == "\\".to_string()) {
            return true;
        }
    }
    return false;
}
fn _iter_list_str(entries: Vec<String>) -> Box<dyn Iterator<Item = String>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<String> = Vec::new().into_iter();
    return Box::new(std::iter::from_fn(move || {
        if !__sifr_generator_initialized {
            let mut _yields: Vec<String> = Vec::new();
            let mut i: i64 = 0 as i64;
            while i < (entries.len() as i64) {
                _yields.push(entries[i as usize].clone());
                i = i + (1 as i64);
            }
            __sifr_generator_iter = _yields.into_iter();
            __sifr_generator_initialized = true;
        }
        return __sifr_generator_iter.next();
    }));
}
fn _iterdir_list(path: &String) -> Result<Vec<String>, IOError> {
    return {
        let __entries = std::fs::read_dir(&path).map_err(__io_err)?;
        Ok(__entries
            .filter_map(|e| e.ok().map(|e| e.path().to_string_lossy().to_string()))
            .collect::<Vec<String>>())
    };
}
fn _glob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    return {
        let __dir = &path;
        let __pat = &pattern;
        let __include_hidden = __pat.starts_with(".");
        let __regex_src = format!(
            "^{}$",
            regex::escape(__pat)
                .replace("\\*", ".*")
                .replace("\\?", ".")
        );
        let __re = regex::Regex::new(&__regex_src).map_err(|e| IOError::new(e.to_string()))?;
        let mut __results: Vec<String> = Vec::new();
        match std::fs::read_dir(__dir) {
            Ok(__entries) => {
                for __entry in __entries {
                    if let Ok(__e) = __entry {
                        let __name = __e.file_name().to_string_lossy().to_string().to_string();
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
    };
}
fn _rglob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    return {
        let __dir = &path;
        let __pat = &pattern;
        let __include_hidden = __pat.starts_with(".");
        let __regex_src = format!(
            "^{}$",
            regex::escape(__pat)
                .replace("\\*", ".*")
                .replace("\\?", ".")
        );
        let __re = regex::Regex::new(&__regex_src).map_err(|e| IOError::new(e.to_string()))?;
        let mut __results: Vec<String> = Vec::new();
        let mut __stack: Vec<String> = vec![__dir.to_string()];
        loop {
            if let Some(__current) = __stack.pop() {
                let __entries_result = std::fs::read_dir(&__current);
                if let Ok(__entries) = __entries_result {
                    for __entry in __entries {
                        if let Ok(__e) = __entry {
                            let __path = __e.path();
                            let __name = __e.file_name().to_string_lossy().to_string().to_string();
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
    };
}
fn _iterdir_to_iter(path: &String) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
    let __sifr_try_res: Result<Result<Box<dyn Iterator<Item = String>>, IOError>, IOError> =
        (|| {
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
    let __sifr_try_res: Result<Result<Box<dyn Iterator<Item = String>>, IOError>, IOError> =
        (|| {
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
    let __sifr_try_res: Result<Result<Box<dyn Iterator<Item = String>>, IOError>, IOError> =
        (|| {
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

// --- stdlib: sifr.uuid ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct UUID {
    _hex: String,
}
impl UUID {
    fn new(hex_str: String) -> Self {
        return Self {
            _hex: format!("{}{}", hex_str, "".to_string()),
        };
    }
    fn hex(&self) -> String {
        let mut result: String = "".to_string();
        let mut i: i64 = 0 as i64;
        while i < (self._hex.clone().chars().count() as i64) {
            let ch: Option<String> = {
                let __sifr_index_str = &self._hex;
                let __sifr_index_i = i;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_str
                    .chars()
                    .nth(__sifr_index_norm)
                    .map(|c| c.to_string())
            };
            if let Some(ch) = ch {
                if ch != "-".to_string() {
                    result = format!("{}{}", result, ch);
                }
            }
            i = i + (1 as i64);
        }
        return result;
    }
    fn urn(&self) -> String {
        return format!("{}{}", "urn:uuid:".to_string(), self._hex.clone());
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._hex.clone(), "".to_string());
    }
    fn version(&self) -> i64 {
        let marker: Option<String> = {
            let __sifr_index_str = &self._hex;
            let __sifr_index_i = 14 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str
                .chars()
                .nth(__sifr_index_norm)
                .map(|c| c.to_string())
        };
        let Some(marker) = marker else {
            return -(1 as i64);
        };
        return _hex_digit_value(&marker);
    }
}
impl std::fmt::Display for UUID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "UUID(_hex={})", self._hex);
    }
}
fn _hex_digit_value(ch: &String) -> i64 {
    if ch.clone() == "0".to_string() {
        return 0 as i64;
    }
    if ch.clone() == "1".to_string() {
        return 1 as i64;
    }
    if ch.clone() == "2".to_string() {
        return 2 as i64;
    }
    if ch.clone() == "3".to_string() {
        return 3 as i64;
    }
    if ch.clone() == "4".to_string() {
        return 4 as i64;
    }
    if ch.clone() == "5".to_string() {
        return 5 as i64;
    }
    if ch.clone() == "6".to_string() {
        return 6 as i64;
    }
    if ch.clone() == "7".to_string() {
        return 7 as i64;
    }
    if ch.clone() == "8".to_string() {
        return 8 as i64;
    }
    if ch.clone() == "9".to_string() {
        return 9 as i64;
    }
    if ((ch.clone() == "a".to_string()) || (ch.clone() == "A".to_string())) {
        return 10 as i64;
    }
    if ((ch.clone() == "b".to_string()) || (ch.clone() == "B".to_string())) {
        return 11 as i64;
    }
    if ((ch.clone() == "c".to_string()) || (ch.clone() == "C".to_string())) {
        return 12 as i64;
    }
    if ((ch.clone() == "d".to_string()) || (ch.clone() == "D".to_string())) {
        return 13 as i64;
    }
    if ((ch.clone() == "e".to_string()) || (ch.clone() == "E".to_string())) {
        return 14 as i64;
    }
    if ((ch.clone() == "f".to_string()) || (ch.clone() == "F".to_string())) {
        return 15 as i64;
    }
    return -(1 as i64);
}

// --- stdlib: sifr.datetime ---
#[derive(Debug, Clone)]
struct timedelta {
    _days: i64,
    _seconds: i64,
}
impl timedelta {
    fn new(days: i64, seconds: i64) -> Self {
        return Self {
            _days: days,
            _seconds: seconds,
        };
    }
    fn total_seconds(&self) -> i64 {
        return (self._days * (86400 as i64)) + self._seconds;
    }
    fn days(&self) -> i64 {
        return self._days;
    }
    fn seconds(&self) -> i64 {
        return self._seconds;
    }
}
impl std::ops::Add<&timedelta> for &timedelta {
    type Output = timedelta;
    fn add(self, other: &timedelta) -> Self::Output {
        let total: i64 = self.total_seconds() + other.total_seconds();
        let d: i64 = total / (86400 as i64);
        let s: i64 = total % (86400 as i64);
        return timedelta::new(d, s);
    }
}
impl std::ops::Sub<&timedelta> for &timedelta {
    type Output = timedelta;
    fn sub(self, other: &timedelta) -> Self::Output {
        let total: i64 = self.total_seconds() - other.total_seconds();
        let d: i64 = total / (86400 as i64);
        let s: i64 = total % (86400 as i64);
        return timedelta::new(d, s);
    }
}
impl PartialEq for timedelta {
    fn eq(&self, other: &timedelta) -> bool {
        return self.total_seconds() == other.total_seconds();
    }
}
impl std::fmt::Display for timedelta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "timedelta(_days={}, _seconds={})",
            self._days, self._seconds
        );
    }
}

// --- stdlib: sifr.re ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Match {
    _matched: String,
    _start: i64,
    _end: i64,
}
impl Match {
    fn new(matched: String, start: i64, end: i64) -> Self {
        return Self {
            _matched: matched,
            _start: start,
            _end: end,
        };
    }
    fn group(&self) -> String {
        return format!("{}{}", self._matched.clone(), "".to_string());
    }
    fn start(&self) -> i64 {
        return self._start;
    }
    fn end(&self) -> i64 {
        return self._end;
    }
    fn span(&self) -> Vec<i64> {
        let result: Vec<i64> = vec![self._start, self._end];
        return result;
    }
    fn to_str(&self) -> String {
        return format!("{}{}", self._matched.clone(), "".to_string());
    }
}
impl std::fmt::Display for Match {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Match(_matched={}, _start={}, _end={})",
            self._matched, self._start, self._end
        );
    }
}

#[derive(Debug, Clone)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            kind: "Other".to_string(),
        };
    }
}

impl std::fmt::Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for IOError {}

fn __io_err(e: std::io::Error) -> IOError {
    let msg = e.to_string();
    let kind = if e.kind() == std::io::ErrorKind::NotFound {
        "FileNotFound".to_string()
    } else {
        if e.kind() == std::io::ErrorKind::PermissionDenied {
            "PermissionDenied".to_string()
        } else {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "FileExists".to_string()
            } else {
                "Other".to_string()
            }
        }
    };
    return IOError {
        message: msg,
        kind: kind,
    };
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

impl std::error::Error for Error {}

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

impl std::error::Error for ValueError {}

enum SifrFileHandle {
    TextRead(std::io::BufReader<std::fs::File>),
    TextWrite(std::io::BufWriter<std::fs::File>),
    BinaryRead(std::io::BufReader<std::fs::File>),
    BinaryWrite(std::io::BufWriter<std::fs::File>),
}

static __SIFR_FILE_HANDLES: std::sync::LazyLock<
    std::sync::Mutex<std::collections::HashMap<i64, SifrFileHandle>>,
> = std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

static __SIFR_NEXT_FILE_HANDLE_ID: std::sync::atomic::AtomicI64 =
    std::sync::atomic::AtomicI64::new(1);

fn __sifr_next_file_handle_id() -> i64 {
    return __SIFR_NEXT_FILE_HANDLE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
}

static __SIFR_GLOBAL_LOG_LEVEL: std::sync::LazyLock<std::sync::Mutex<i64>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(20));

fn main() {
    println!("=== TopologicalSorter ===");
    let mut ts: TopologicalSorter = TopologicalSorter::new();
    ts.add(1 as i64, 0 as i64);
    ts.add(2 as i64, 1 as i64);
    let __sifr_try_res: Result<(), CycleError> = (|| {
        let order: Vec<i64> = ts.static_order()?;
        let first: Option<i64> = {
            let __sifr_index_list = &order;
            let __sifr_index_i = 0 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let second: Option<i64> = {
            let __sifr_index_list = &order;
            let __sifr_index_i = 1 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        let third: Option<i64> = {
            let __sifr_index_list = &order;
            let __sifr_index_i = 2 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_list.get(__sifr_index_norm).copied()
        };
        if let Some(first) = first {
            println!("{}", first);
        }
        if let Some(second) = second {
            println!("{}", second);
        }
        if let Some(third) = third {
            println!("{}", third);
        }
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let err = __sifr_try_err.clone();
        println!("cycle error: {}", err.message);
    }
    println!("=== Path ===");
    let mut p: Path = Path::new("/home/user/docs/report.pdf".to_string());
    println!("{}", p.name());
    println!("{}", p.parent().to_str());
    println!("{}", p.suffix());
    println!("{}", p.stem());
    println!("{}", p.is_absolute());
    println!("=== Logger ===");
    let mut log: Logger = getLogger(&"demo".to_string());
    log.info(&"application started".to_string());
    log.warning(&"disk space low".to_string());
    log.debug(&"this should not appear at INFO level".to_string());
    log.set_level(10 as i64);
    log.debug(&"now visible after level change".to_string());
    println!("=== Match ===");
    let mut m: Match = Match::new("world".to_string(), 6 as i64, 11 as i64);
    println!("{}", m.group());
    println!("{}", m.start());
    println!("{}", m.end());
    println!("=== UUID ===");
    let mut u: UUID = UUID::new("550e8400-e29b-41d4-a716-446655440000".to_string());
    println!("{}", u.hex());
    println!("{}", u.version());
    println!("=== timedelta ===");
    let one_day: timedelta = timedelta::new(1 as i64, 0 as i64);
    let two_hours: timedelta = timedelta::new(0 as i64, 7200 as i64);
    let mut combined: timedelta = &one_day + &two_hours;
    println!("{}", combined.total_seconds());
    println!("{}", combined.days());
    let mut diff: timedelta = &one_day - &two_hours;
    println!("{}", diff.total_seconds());
    println!("{}", one_day == timedelta::new(1 as i64, 0 as i64));
}
