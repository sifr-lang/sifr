use std::collections::HashMap;

use std::sync::Mutex;

// --- stdlib: sifr.zipfile ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ZipInfo {
    filename: String,
    file_size: i64,
    compress_type: i64,
}
impl ZipInfo {
    fn new(filename: String, file_size: i64, compress_type: i64) -> Self {
        return Self {
            filename: format!("{}{}", filename, "".to_string()),
            file_size: file_size,
            compress_type: compress_type,
        };
    }
}
impl std::fmt::Display for ZipInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "ZipInfo(filename={}, file_size={}, compress_type={})",
            self.filename, self.file_size, self.compress_type
        );
    }
}
fn _zip_rewrite_with_entry(path: &String, name: &String, content: &[u8]) -> Result<(), IOError> {
    let existing_entries: Vec<(String, Vec<u8>)> = if std::path::Path::new(path).exists() {
        let __f = std::fs::File::open(path).map_err(__io_err)?;
        let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?;
        let mut __entries: Vec<(String, Vec<u8>)> = Vec::new();
        for __i in 0..__zip.len() {
            let mut __file = __zip
                .by_index(__i)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __data: Vec<u8> = Vec::new();
            std::io::Read::read_to_end(&mut __file, &mut __data).map_err(__io_err)?;
            __entries.push((__file.name().to_string(), __data));
        }
        __entries
    } else {
        Vec::new()
    };
    let temp_path = format!("{}.tmp", path);
    let temp_file = std::fs::File::create(&temp_path).map_err(__io_err)?;
    let mut writer = zip::ZipWriter::new(temp_file);
    for (existing_name, existing_data) in existing_entries {
        writer
            .start_file(existing_name, zip::write::FileOptions::default())
            .map_err(|e| IOError::new(e.to_string()))?;
        std::io::Write::write_all(&mut writer, &existing_data).map_err(__io_err)?;
    }
    writer
        .start_file(name.to_string(), zip::write::FileOptions::default())
        .map_err(|e| IOError::new(e.to_string()))?;
    std::io::Write::write_all(&mut writer, content).map_err(__io_err)?;
    writer.finish().map_err(|e| IOError::new(e.to_string()))?;
    if std::path::Path::new(path).exists() {
        std::fs::remove_file(path).map_err(__io_err)?;
    }
    std::fs::rename(&temp_path, path).map_err(__io_err)
}
#[derive(Debug, Clone, PartialEq)]
struct ZipReadHandle {
    _data: Vec<u8>,
    _cursor: i64,
    _closed: bool,
}
impl ZipReadHandle {
    fn new(data: Vec<u8>) -> Self {
        return Self {
            _data: data,
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
    fn read_bytes(&mut self, size: Option<i64>) -> Result<Vec<u8>, IOError> {
        if self._closed {
            return Err(IOError::new(_closed_stream_error()));
        }
        let mut end: i64 = self._data.clone().len() as i64;
        if let Some(size) = size {
            let requested_size: i64 = size;
            if requested_size < (0 as i64) {
                end = self._data.clone().len() as i64;
            } else {
                let requested_end: i64 = self._cursor + requested_size;
                if requested_end < end {
                    end = requested_end;
                }
            }
        }
        let out: Vec<u8> = Vec::from_iter(
            (self._data.clone())
                .iter()
                .skip((self._cursor).max(0) as usize)
                .take(((end).max(0) - (self._cursor).max(0)).max(0) as usize)
                .cloned(),
        );
        self._cursor = end;
        return Ok(out);
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ZipFile {
    path: String,
    mode: String,
    compression: i64,
}
impl ZipFile {
    fn new(path: String) -> Self {
        return Self::with_options(path, "a".to_string(), 0 as i64);
    }
    fn with_options(path: String, mode: String, compression: i64) -> Self {
        return Self {
            path: format!("{}{}", path, "".to_string()),
            mode: format!("{}{}", mode, "".to_string()),
            compression: compression,
        };
    }
    fn _writable_mode(&self) -> bool {
        return ((((self.mode.clone() == "w".to_string())
            || (self.mode.clone() == "a".to_string()))
            || (self.mode.clone() == "wb".to_string()))
            || (self.mode.clone() == "ab".to_string()));
    }
    fn create(&self) -> Result<(), IOError> {
        return {
            let __f = std::fs::File::create(&self.path.clone()).map_err(__io_err)?;
            drop(zip::ZipWriter::new(__f));
            Ok(())
        };
    }
    fn write(&self, name: &String, content: &String) -> Result<(), IOError> {
        if !(self._writable_mode()) {
            return Err(IOError::new(_zip_read_only_error()));
        }
        return _zip_rewrite_with_entry(&self.path, name, content.as_bytes());
    }
    fn write_bytes(&self, name: &String, content: &Vec<u8>) -> Result<(), IOError> {
        if !(self._writable_mode()) {
            return Err(IOError::new(_zip_read_only_error()));
        }
        return _zip_rewrite_with_entry(&self.path, name, content.as_slice());
    }
    fn read(&self, name: &String) -> Result<String, IOError> {
        return {
            let __f = std::fs::File::open(&self.path.clone()).map_err(__io_err)?;
            let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?;
            let mut __file = __zip
                .by_name(&name)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __content = String::new();
            std::io::Read::read_to_string(&mut __file, &mut __content).map_err(__io_err)?;
            Ok(__content)
        };
    }
    fn read_bytes(&self, name: &String) -> Result<Vec<u8>, IOError> {
        return {
            let __f = std::fs::File::open(&self.path.clone()).map_err(__io_err)?;
            let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?;
            let mut __file = __zip
                .by_name(&name)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __content = Vec::new();
            std::io::Read::read_to_end(&mut __file, &mut __content).map_err(__io_err)?;
            Ok(__content.to_vec())
        };
    }
    fn namelist(&self) -> Result<Vec<String>, IOError> {
        return {
            let __f = std::fs::File::open(&self.path.clone()).map_err(__io_err)?;
            let mut __zip = zip::ZipArchive::new(__f).map_err(|e| IOError::new(e.to_string()))?;
            let mut __names = Vec::new();
            for __i in 0..__zip.len() {
                if let Ok(__file) = __zip.by_index(__i) {
                    __names.push(__file.name().to_string());
                }
            }
            Ok(__names)
        };
    }
    fn infolist(&self) -> Result<Vec<ZipInfo>, IOError> {
        return Err(IOError::new(_zip_unimplemented_error(
            &"infolist".to_string(),
        )));
    }
    fn getinfo(&self, name: &String) -> Result<ZipInfo, IOError> {
        let _: String = (name).clone();
        return Err(IOError::new(_zip_unimplemented_error(
            &"getinfo".to_string(),
        )));
    }
    fn open(&self, name: &String, mode: &String) -> Result<ZipReadHandle, IOError> {
        let _: String = (name).clone();
        if ((mode.clone() != "r".to_string()) && (mode.clone() != "rb".to_string())) {
            return Err(IOError::new(_zip_open_mode_error(mode)));
        }
        return Err(IOError::new(_zip_unimplemented_error(&"open".to_string())));
    }
    fn extract(&self, name: &String, path: &String) -> Result<String, IOError> {
        let _: String = (name).clone();
        let _: String = (path).clone();
        return Err(IOError::new(_zip_unimplemented_error(
            &"extract".to_string(),
        )));
    }
    fn extractall(&self, path: &String) -> Result<Vec<String>, IOError> {
        let _: String = (path).clone();
        return Err(IOError::new(_zip_unimplemented_error(
            &"extractall".to_string(),
        )));
    }
    fn __enter__(&self) -> ZipFile {
        return self.clone();
    }
    fn __exit__(&self) {
        return;
    }
}
impl std::fmt::Display for ZipFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "ZipFile(path={}, mode={}, compression={})",
            self.path, self.mode, self.compression
        );
    }
}
fn _zip_read_only_error() -> String {
    return "zipfile operation requires write or append mode".to_string();
}
fn _zip_open_mode_error(mode: &String) -> String {
    return format!(
        "{}{}",
        "zipfile open supports read-only mode only, got: ".to_string(),
        mode
    );
}
fn _closed_stream_error() -> String {
    return "I/O operation on closed stream".to_string();
}
fn _zip_unimplemented_error(feature: &String) -> String {
    return format!(
        "{}{}{}",
        "zipfile ".to_string(),
        feature,
        " is not implemented in this wave".to_string()
    );
}

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
    fn get(&self, option: &String, fallback: &Option<String>, raw: bool) -> Option<String> {
        let normalized: String = _normalize_option(option);
        if _has_option_key(&self._values.clone(), &normalized) {
            let value: Option<String> = _lookup_option(&self._values.clone(), &normalized);
            let Some(value) = value else {
                return None;
            };
            if raw {
                return Some(value);
            }
            return Some(_resolve_interpolation(
                &value,
                &self._values.clone(),
                0 as i64,
            ));
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
    fn new() -> Self {
        return Self::with_options(None, false, false);
    }
    fn with_options(
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
            if ((line.starts_with(&"[".to_string())) && (line.ends_with(&"]".to_string()))) {
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
                    return Err(ParsingError::new(
                        line_no,
                        "section name is empty".to_string(),
                    ));
                }
                if section_name == default_section {
                    current_section = _default_section();
                    continue;
                }
                if ((self.strict) && ((self._sections.clone()).contains_key(&(section_name)))) {
                    return Err(ParsingError::new(
                        line_no,
                        format!("{}{}", "duplicate section: ".to_string(), section_name),
                    ));
                }
                current_section = format!("{}{}", section_name, "".to_string());
                if !((self._sections.clone()).contains_key(&(section_name))) {
                    self._sections.insert(section_name, HashMap::from([]));
                }
                continue;
            }
            let __sifr_try_res: Result<(), ParsingError> = (|| {
                let parsed_option_pair: (String, Option<String>) =
                    _split_option_line(&line, self.allow_no_value, line_no)?;
                let (option_name, option_value) = parsed_option_pair;
                if (current_section == "".to_string()) || (current_section == default_section) {
                    self._defaults
                        .insert(option_name, _copy_optional_str(&option_value));
                } else {
                    let section_key: String = format!("{}{}", current_section, "".to_string());
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
                        if ((self.strict) && (_has_option_key(&section_values, &option_name))) {
                            return Err(ParsingError::new(
                                line_no,
                                format!("{}{}", "duplicate option: ".to_string(), option_name),
                            ));
                        }
                        let mut updated_section: HashMap<String, Option<String>> =
                            _copy_values(&section_values);
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
                return Err(IOError::new(format!(
                    "{}{}{}{}",
                    "parse error on line ".to_string(),
                    format!("{}", e.line),
                    ": ".to_string(),
                    e.message
                )));
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
        let mut merged: HashMap<String, Option<String>> = _copy_values(&self._defaults.clone());
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
    fn get(&self, section: &String, option: &String) -> Option<String> {
        return self.get_with_options(section, option, &None, false);
    }
    fn get_with_options(
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
                let default_value: Option<String> =
                    _lookup_option(&self._defaults.clone(), &normalized);
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
    fn getint(&self, section: &String, option: &String, fallback: Option<i64>) -> Option<i64> {
        let raw: Option<String> = self.get_with_options(section, option, &None, false);
        let Some(raw) = raw else {
            return fallback;
        };
        let __sifr_try_res: Result<Option<i64>, ParseError> = (|| {
            let parsed: i64 = (raw).parse::<i64>().map_err(|e| ParseError {
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
    fn getfloat(&self, section: &String, option: &String, fallback: Option<f64>) -> Option<f64> {
        let raw: Option<String> = self.get_with_options(section, option, &None, false);
        let Some(raw) = raw else {
            return fallback;
        };
        let __sifr_try_res: Result<Option<f64>, ParseError> = (|| {
            let parsed: f64 = (raw).parse::<f64>().map_err(|e| ParseError {
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
        let raw: Option<String> = self.get_with_options(section, option, &None, false);
        let Some(raw) = raw else {
            return fallback;
        };
        let normalized: String = raw.to_lowercase();
        if (((normalized == "1".to_string()) || (normalized == "yes".to_string()))
            || (normalized == "true".to_string()))
            || (normalized == "on".to_string())
        {
            return Some(true);
        }
        if (((normalized == "0".to_string()) || (normalized == "no".to_string()))
            || (normalized == "false".to_string()))
            || (normalized == "off".to_string())
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
            let mut updated_section: HashMap<String, Option<String>> =
                _copy_values(&section_values);
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
            lines.push(format!(
                "{}{}",
                format!("{}{}", "[".to_string(), _default_section()),
                "]".to_string()
            ));
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
                        lines.push(format!(
                            "{}{}",
                            format!("{}{}", key, " = ".to_string()),
                            value
                        ));
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
            lines.push(format!(
                "{}{}",
                format!("{}{}", "[".to_string(), section_name),
                "]".to_string()
            ));
            for (key, value) in section_values
                .iter()
                .map(|__kv| (__kv.0.clone(), __kv.1.clone()))
                .collect::<Vec<_>>()
            {
                if value.is_none() {
                    lines.push(key);
                } else {
                    if let Some(value) = value {
                        lines.push(format!(
                            "{}{}",
                            format!("{}{}", key, " = ".to_string()),
                            value
                        ));
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
                        unreachable!("compiler-verified non-empty pop should return Some");
                    };
                    __sifr_nonempty_pop_value
                };
            }
        }
        return lines.join(&"\n".to_string());
    }
    fn write(&self, path: &String) -> Result<(), IOError> {
        let payload: String = self.to_ini_string();
        return std::fs::write(&path, payload.as_bytes())
            .map(|_| ())
            .map_err(__io_err);
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
fn _lookup_option(values: &HashMap<String, Option<String>>, key: &String) -> Option<String> {
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
fn _copy_values(values: &HashMap<String, Option<String>>) -> HashMap<String, Option<String>> {
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
        return Err(ParsingError::new(
            line_no,
            "expected key=value or key:value entry".to_string(),
        ));
    };
    let parts: Vec<String> = if (1 as i64) < 0 {
        line.split(&delimiter)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    } else {
        line.splitn(((1 as i64) + 1) as usize, &delimiter)
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
    };
    if (parts.len() as i64) != (2 as i64) {
        return Err(ParsingError::new(
            line_no,
            "invalid option line".to_string(),
        ));
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
        return Err(ParsingError::new(
            line_no,
            "option name is missing".to_string(),
        ));
    };
    let key: String = _normalize_option(&raw_key);
    if key == "".to_string() {
        return Err(ParsingError::new(
            line_no,
            "option name is empty".to_string(),
        ));
    }
    let Some(raw_value) = raw_value else {
        return Ok((key, None));
    };
    let stripped_value: Option<String> = _some_str(&raw_value.trim().to_string());
    return Ok((key, stripped_value));
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
        if (((ch == "%".to_string()) && ((i + (1 as i64)) < (value.chars().count() as i64)))
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
                    let replacement: Option<String> = _lookup_option(merged, &normalized_key);
                    if replacement.is_none() {
                        result =
                            format!("{}{}{}{}", result, "%(".to_string(), key, ")s".to_string());
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

// --- stdlib: sifr.operator ---
fn add(a: i64, b: i64) -> i64 {
    return a + b;
}
fn sub(a: i64, b: i64) -> i64 {
    return a - b;
}
fn mul(a: i64, b: i64) -> i64 {
    return a * b;
}
fn floordiv(a: i64, b: i64) -> i64 {
    return a / b;
}
fn mod_val(a: i64, b: i64) -> i64 {
    return a % b;
}
fn neg(a: i64) -> i64 {
    return -a;
}
fn lt(a: i64, b: i64) -> bool {
    return a < b;
}
fn eq(a: i64, b: i64) -> bool {
    return a == b;
}
fn getitem<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    index: i64,
) -> Option<T> {
    return {
        let __sifr_index_list = &items;
        let __sifr_index_i = index;
        let __sifr_index_norm = if __sifr_index_i < 0 {
            ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
        } else {
            __sifr_index_i as usize
        };
        __sifr_index_list.get(__sifr_index_norm).cloned()
    };
}
fn itemgetter<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    items: &Vec<T>,
    index: i64,
) -> Option<T> {
    return getitem(items, index);
}

// --- stdlib: sifr.gzip ---
fn compress(data: &String) -> Vec<i64> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    std::io::Write::write_all(&mut encoder, data.as_bytes())
        .expect("writing gzip data into an in-memory buffer should not fail");
    encoder
        .finish()
        .expect("finishing gzip encoding into an in-memory buffer should not fail")
        .into_iter()
        .map(|byte| byte as i64)
        .collect::<Vec<i64>>()
}
fn decompress(data: &Vec<i64>) -> Result<String, IOError> {
    return {
        let __bytes = data.iter().map(|b| *b as u8).collect::<Vec<u8>>();
        let mut __dec = flate2::read::GzDecoder::new(__bytes.as_slice());
        let mut __out = String::new();
        std::io::Read::read_to_string(&mut __dec, &mut __out).map_err(__io_err)?;
        Ok(__out)
    };
}

// --- stdlib: sifr.calendar ---
fn isleap(year: i64) -> bool {
    return {
        let __y = year;
        (((__y % 4) == 0) && ((__y % 100) != 0)) || ((__y % 400) == 0)
    };
}
fn weekday(year: i64, month: i64, day: i64) -> i64 {
    return {
        let __y0 = year;
        let __m0 = month;
        let __d0 = day;
        {
            let __t = vec![0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
            let __y = if __m0 < 3 { __y0 - 1 } else { __y0 };
            let __wd_raw = ((((((__y + (__y / 4)) - (__y / 100)) + (__y / 400))
                + __t[(__m0 - 1) as usize])
                + __d0)
                % 7)
                + 6;
            __wd_raw % 7
        }
    };
}
fn monthrange(year: i64, month: i64) -> Vec<i64> {
    return {
        let __y = year;
        let __m = month;
        let __days = if ((((((__m == 1) || (__m == 3)) || (__m == 5)) || (__m == 7)) || (__m == 8))
            || (__m == 10))
            || (__m == 12)
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
                + __t[(__m - 1) as usize])
                + 1)
                % 7)
                + 6;
            __wd_raw % 7
        };
        vec![__wd, __days]
    };
}

// --- stdlib: sifr.subprocess ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CompletedProcess {
    returncode: i64,
    stdout: String,
    stderr: String,
}
impl CompletedProcess {
    fn new(returncode: i64, stdout: String, stderr: String) -> Self {
        return Self {
            returncode: returncode,
            stdout: stdout,
            stderr: stderr,
        };
    }
}
impl std::fmt::Display for CompletedProcess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "CompletedProcess(returncode={}, stdout={}, stderr={})",
            self.returncode, self.stdout, self.stderr
        );
    }
}
fn run(cmd: &String) -> Result<CompletedProcess, IOError> {
    let __sifr_try_res: Result<Result<CompletedProcess, IOError>, IOError> = (|| {
        let result: Vec<String> = ({
            let __output = std::process::Command::new("sh".to_string())
                .arg("-c".to_string())
                .arg(&cmd)
                .output()
                .map_err(__io_err)?;
            let __stdout = String::from_utf8_lossy(&__output.stdout).to_string();
            let __stderr = String::from_utf8_lossy(&__output.stderr).to_string();
            let __returncode = __output.status.code().unwrap_or(-1).to_string();
            Ok(vec![__stdout, __stderr, __returncode])
        })?;
        let mut stdout: String = "".to_string();
        let mut stderr: String = "".to_string();
        let mut rc_str: String = "".to_string();
        let mut rc: i64 = 0 as i64;
        for (i, value) in Box::new(
            (result)
                .iter()
                .cloned()
                .enumerate()
                .map(|__pair| ((__pair.0 as i64) + 0, __pair.1)),
        ) {
            if i == (0 as i64) {
                stdout = format!("{}{}", value, "".to_string());
            }
            if i == (1 as i64) {
                stderr = format!("{}{}", value, "".to_string());
            }
            if i == (2 as i64) {
                rc_str = format!("{}{}", value, "".to_string());
            }
        }
        if rc_str != "".to_string() {
            let __sifr_try_res: Result<(), ParseError> = (|| {
                let parsed: i64 = (rc_str).parse::<i64>().map_err(|e| ParseError {
                    message: e.to_string(),
                })?;
                rc = parsed;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                let _: String = e.message;
                rc = -(1 as i64);
            }
        }
        return Ok(Ok(CompletedProcess::new(rc, stdout, stderr)));
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

// --- stdlib: sifr.html ---
fn escape_with_quote(s: &str, quote: bool) -> String {
    let escaped: String = s
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('\"', "&quot;")
        .replace('\'', "&#x27;");
    if quote {
        return escaped;
    }
    return escaped
        .replace(&"&quot;".to_string(), &"\"".to_string())
        .replace(&"&#x27;".to_string(), &"\'".to_string());
}
fn escape(s: &str) -> String {
    escape_with_quote(s, false)
}
fn unescape(s: &str) -> String {
    return s
        .replace("&amp;", "&")
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
        .replace("&#X3e;", ">");
}

// --- stdlib: sifr.sys ---
fn version() -> String {
    return "sifr 0.1.0".to_string();
}
fn maxsize() -> i64 {
    return i64::MAX;
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

impl std::error::Error for ParseError {}

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

#[derive(Debug, Clone)]
struct JSONDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl JSONDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for JSONDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for JSONDecodeError {}

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            line: 0,
            column: 0,
        };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for TOMLDecodeError {}

#[derive(Debug, Clone)]
struct RegexError {
    message: String,
    detail: String,
}

impl RegexError {
    fn new(message: String) -> Self {
        return Self {
            message: message,
            detail: String::new(),
        };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {}

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

fn remove_file(path: &str) -> Result<(), IOError> {
    std::fs::remove_file(path).map(|_| ()).map_err(__io_err)
}

fn demo_operator() {
    println!("=== operator ===");
    println!(
        "{}",
        format!(
            "{}{}",
            "add(10, 5) = ".to_string(),
            format!("{}", add(10 as i64, 5 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "sub(10, 5) = ".to_string(),
            format!("{}", sub(10 as i64, 5 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "mul(3, 4) = ".to_string(),
            format!("{}", mul(3 as i64, 4 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "floordiv(7, 2) = ".to_string(),
            format!("{}", floordiv(7 as i64, 2 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "mod_val(7, 2) = ".to_string(),
            format!("{}", mod_val(7 as i64, 2 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "neg(42) = ".to_string(),
            format!("{}", neg(42 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "lt(3, 5) = ".to_string(),
            format!("{}", lt(3 as i64, 5 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "eq(5, 5) = ".to_string(),
            format!("{}", eq(5 as i64, 5 as i64))
        )
    );
    let items: Vec<i64> = vec![1 as i64, 2 as i64, 3 as i64];
    println!(
        "{}",
        format!(
            "{}{}",
            "itemgetter([1,2,3], 1) = ".to_string(),
            (itemgetter(&items, 1 as i64))
                .map_or("None".to_string().to_string(), |__v| format!("{}", __v))
        )
    );
}

fn demo_calendar() {
    println!("=== calendar ===");
    println!(
        "{}",
        format!(
            "{}{}",
            "isleap(2000) = ".to_string(),
            format!("{}", isleap(2000 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "isleap(1900) = ".to_string(),
            format!("{}", isleap(1900 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "isleap(2024) = ".to_string(),
            format!("{}", isleap(2024 as i64))
        )
    );
    let wd: i64 = weekday(2024 as i64, 1 as i64, 1 as i64);
    println!(
        "{}",
        format!(
            "{}{}",
            "weekday(2024,1,1) = ".to_string(),
            format!("{}", wd)
        )
    );
    let mr: Vec<i64> = monthrange(2024 as i64, 2 as i64);
    println!(
        "{}",
        format!(
            "{}{}",
            "monthrange(2024,2)[1] = ".to_string(),
            ({
                let __sifr_index_list = &mr;
                let __sifr_index_i = 1 as i64;
                let __sifr_index_norm = if __sifr_index_i < 0 {
                    ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize
                } else {
                    __sifr_index_i as usize
                };
                __sifr_index_list.get(__sifr_index_norm).copied()
            })
            .map_or("None".to_string().to_string(), |__v| format!("{}", __v))
        )
    );
}

fn demo_html() {
    println!("=== html ===");
    let s: String = "<b>Hi & Bye</b>".to_string();
    let esc: String = escape(&s);
    println!(
        "{}",
        format!("{}{}", "escape(<b>Hi & Bye</b>) = ".to_string(), esc)
    );
    let unesc: String = unescape(&esc);
    println!(
        "{}",
        format!(
            "{}{}",
            "unescape(&lt;b&gt;Hi &amp; Bye&lt;/b&gt;) = ".to_string(),
            unesc
        )
    );
}

fn demo_sys() {
    println!("=== sys ===");
    println!("{}", format!("{}{}", "version = ".to_string(), version()));
    let ms: i64 = maxsize();
    println!(
        "{}",
        format!(
            "{}{}",
            "maxsize > 0 = ".to_string(),
            format!("{}", ms > (0 as i64))
        )
    );
}

fn demo_subprocess() {
    println!("=== subprocess ===");
    let __sifr_try_res: Result<(), IOError> = (|| {
        let result: CompletedProcess = run(&"echo hello".to_string())?;
        println!(
            "{}",
            format!("{}{}", "echo hello = ".to_string(), result.stdout)
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "error: ".to_string(), e.message));
    }
}

fn demo_configparser() {
    println!("=== configparser ===");
    let mut config: ConfigParser = ConfigParser::new();
    let __sifr_try_res: Result<(), ParsingError> = (|| {
        let _: () =
            config.read_string(&"[database]\nhost = db.example.com\nport = 5432\n".to_string())?;
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", e.message);
        return;
    }
    let host_value: Option<String> = config.get(&"database".to_string(), &"host".to_string());
    let port_value: Option<String> = config.get(&"database".to_string(), &"port".to_string());
    println!(
        "{}",
        format!(
            "{}{}",
            "host = ".to_string(),
            (host_value).map_or("None".to_string().to_string(), |__v| format!("{}", __v))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "port = ".to_string(),
            (port_value).map_or("None".to_string().to_string(), |__v| format!("{}", __v))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "has_host = ".to_string(),
            format!(
                "{}",
                config.has_option(&"database".to_string(), &"host".to_string())
            )
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "has_missing = ".to_string(),
            format!(
                "{}",
                config.has_option(&"database".to_string(), &"missing".to_string())
            )
        )
    );
}

fn demo_gzip() {
    println!("=== gzip ===");
    let data: String = "Sifr stdlib gzip compression!".to_string();
    let compressed: Vec<i64> = compress(&data);
    println!(
        "{}",
        format!(
            "{}{}",
            "compressed len > 0 = ".to_string(),
            format!("{}", (compressed.len() as i64) > (0 as i64))
        )
    );
    let __sifr_try_res: Result<(), IOError> = (|| {
        let decompressed: String = decompress(&compressed)?;
        println!(
            "{}",
            format!("{}{}", "decompressed = ".to_string(), decompressed)
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "error: ".to_string(), e.message));
    }
}

fn demo_zipfile() {
    println!("=== zipfile ===");
    let mut zf: ZipFile = ZipFile::new("/tmp/sifr_demo_zipfile.zip".to_string());
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _c: () = zf.create()?;
        println!("zip created = true");
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "create error: ".to_string(), e.message)
        );
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _w: () = zf.write(&"demo.txt".to_string(), &"Hello from ZipFile!".to_string())?;
        let content: String = zf.read(&"demo.txt".to_string())?;
        println!("{}", format!("{}{}", "zip content = ".to_string(), content));
        let names: Vec<String> = zf.namelist()?;
        println!(
            "{}",
            format!(
                "{}{}",
                "zip namelist len = ".to_string(),
                format!("{}", names.len() as i64)
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "zip error: ".to_string(), e.message));
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
        let _r: () = remove_file("/tmp/sifr_demo_zipfile.zip")?;
        return Ok(());
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
    demo_subprocess();
    demo_configparser();
    demo_gzip();
    demo_zipfile();
}
