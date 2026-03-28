use std::sync::Mutex;

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
fn _nonzero_exit_error(cmd: &String, returncode: i64) -> String {
    return format!(
        "{}{}{}{}",
        "command returned non-zero exit status ".to_string(),
        format!("{}", returncode),
        ": ".to_string(),
        cmd
    );
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
fn check_call(cmd: &String) -> Result<i64, IOError> {
    let __sifr_try_res: Result<Result<i64, IOError>, IOError> = (|| {
        let result: CompletedProcess = run(cmd)?;
        if result.returncode != (0 as i64) {
            return Err(IOError::new(_nonzero_exit_error(cmd, result.returncode)));
        }
        return Ok(Ok(result.returncode));
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
fn check_output(cmd: &String) -> Result<String, IOError> {
    let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
        let result: CompletedProcess = run(cmd)?;
        if result.returncode != (0 as i64) {
            return Err(IOError::new(_nonzero_exit_error(cmd, result.returncode)));
        }
        return Ok(Ok(result.stdout));
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
fn run_with_input(cmd: &String, stdin_data: &String) -> Result<String, IOError> {
    return {
        let mut __child = std::process::Command::new("sh".to_string())
            .arg("-c".to_string())
            .arg(&cmd)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .map_err(__io_err)?;
        if let Some(mut stdin) = __child.stdin.take() {
            std::io::Write::write_all(&mut stdin, stdin_data.as_bytes()).map_err(__io_err)?;
        }
        let __output = __child.wait_with_output().map_err(__io_err)?;
        Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
    };
}

// --- stdlib: sifr.platform ---
fn system() -> String {
    return if cfg!(target_os = "windows") {
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
    };
}
fn machine() -> String {
    return std::env::consts::ARCH.to_string();
}
fn processor() -> String {
    return std::env::consts::ARCH.to_string();
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

// --- stdlib: sifr.sys ---
fn argv() -> Vec<String> {
    return std::env::args().collect::<Vec<String>>();
}
fn version() -> String {
    return "sifr 0.1.0".to_string();
}
fn platform() -> String {
    return std::env::consts::OS.to_string();
}

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
    let Some(val) = val else {
        return format!("{}{}", default_value, "".to_string());
    };
    return val;
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

// --- stdlib: sifr.time ---
fn time() -> f64 {
    return std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
}
fn strftime(fmt: &String, epoch: f64) -> String {
    return {
        let secs = epoch as i64;
        let dt = chrono::DateTime::from_timestamp(secs, 0).unwrap_or_default();
        dt.format(&fmt).to_string()
    };
}

// --- stdlib: sifr.timeit ---
fn _elapsed_non_negative(start: f64, end: f64) -> f64 {
    let elapsed: f64 = end - start;
    if elapsed < (0.0 as f64) {
        return 0.0 as f64;
    }
    return elapsed;
}
fn timeit(stmt: impl Fn(), number: i64) -> f64 {
    let start: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    let mut i: i64 = 0 as i64;
    while i < number {
        stmt();
        i = i + (1 as i64);
    }
    let end: f64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    return _elapsed_non_negative(start, end);
}
fn repeat(stmt: impl Fn(), count: i64, number: i64) -> Vec<f64> {
    let mut results: Vec<f64> = vec![];
    let mut r: i64 = 0 as i64;
    while r < count {
        let start: f64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let mut i: i64 = 0 as i64;
        while i < number {
            stmt();
            i = i + (1 as i64);
        }
        let end: f64 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();
        let elapsed: f64 = _elapsed_non_negative(start, end);
        results.push(elapsed);
        r = r + (1 as i64);
    }
    return results;
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

static __SIFR_GLOBAL_LOG_LEVEL: std::sync::LazyLock<std::sync::Mutex<i64>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(20));

fn workload() {
    let mut i: i64 = 0 as i64;
    let mut total: i64 = 0 as i64;
    while i < (100 as i64) {
        total = total + i;
        i = i + (1 as i64);
    }
}

fn main() {
    let __sifr_try_res: Result<(), IOError> = (|| {
        let shell_out: String = ({
            let __cmd = "echo wave_psp_d2".to_string();
            let __output = std::process::Command::new("sh".to_string())
                .arg("-c".to_string())
                .arg(&__cmd)
                .output()
                .map_err(__io_err)?;
            Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
        })?;
        let cwd: String = std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .map_err(__io_err)?;
        println!(
            "{}",
            format!("{}{}", "os.run_command = ".to_string(), shell_out)
        );
        println!(
            "{}",
            format!(
                "{}{}",
                "os.getcwd len > 0 = ".to_string(),
                format!("{}", (cwd.chars().count() as i64) > (0 as i64))
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "os error: ".to_string(), e.message));
    }
    setenv(&"SIFR_WAVE_D2_DEMO".to_string(), &"ok".to_string());
    println!(
        "{}",
        format!(
            "{}{}",
            "env getenv = ".to_string(),
            getenv(&"SIFR_WAVE_D2_DEMO".to_string(), &"fallback".to_string())
        )
    );
    unsetenv(&"SIFR_WAVE_D2_DEMO".to_string());
    println!(
        "{}",
        format!(
            "{}{}",
            "sys.argv len = ".to_string(),
            format!("{}", argv().len() as i64)
        )
    );
    println!(
        "{}",
        format!("{}{}", "sys.version = ".to_string(), version())
    );
    println!(
        "{}",
        format!("{}{}", "sys.platform = ".to_string(), platform())
    );
    let __sifr_try_res: Result<(), IOError> = (|| {
        let cp: CompletedProcess = run(&"echo subprocess_demo".to_string())?;
        println!(
            "{}",
            format!(
                "{}{}",
                "subprocess.run rc = ".to_string(),
                format!("{}", cp.returncode)
            )
        );
        println!(
            "{}",
            format!(
                "{}{}",
                "subprocess.run stdout = ".to_string(),
                cp.stdout.trim().to_string()
            )
        );
        let echoed: String = run_with_input(&"cat".to_string(), &"stdin_demo".to_string())?;
        println!(
            "{}",
            format!("{}{}", "subprocess.run_with_input = ".to_string(), echoed)
        );
        let checked_rc: i64 = check_call(&"echo subprocess_check_call_demo".to_string())?;
        let checked_out: String = check_output(&"echo subprocess_check_output_demo".to_string())?;
        println!(
            "{}",
            format!(
                "{}{}",
                "subprocess.check_call rc = ".to_string(),
                format!("{}", checked_rc)
            )
        );
        println!(
            "{}",
            format!(
                "{}{}",
                "subprocess.check_output = ".to_string(),
                checked_out.trim().to_string()
            )
        );
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!(
            "{}",
            format!("{}{}", "subprocess error: ".to_string(), e.message)
        );
    }
    let mut logger: Logger = getLogger(&"wave_psp_d2_demo".to_string());
    logger.set_level(INFO);
    logger.info(&"logging demo line".to_string());
    println!(
        "{}",
        format!("{}{}", "platform.system = ".to_string(), system())
    );
    println!(
        "{}",
        format!("{}{}", "platform.machine = ".to_string(), machine())
    );
    println!(
        "{}",
        format!("{}{}", "platform.processor = ".to_string(), processor())
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "time.time > 0 = ".to_string(),
            format!("{}", time() > (0.0 as f64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "time.strftime epoch0 = ".to_string(),
            strftime(&"%Y-%m-%d %H:%M:%S".to_string(), 0.0 as f64)
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "timeit.timeit = ".to_string(),
            format!("{}", timeit(workload, 5 as i64))
        )
    );
    println!(
        "{}",
        format!(
            "{}{}",
            "timeit.repeat count = ".to_string(),
            format!("{}", repeat(workload, 3 as i64, 4 as i64).len() as i64)
        )
    );
}
