use std::sync::Mutex;

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

#[derive(Debug, Clone)]
struct TOMLDecodeError {
    message: String,
    line: i64,
    column: i64,
}

impl TOMLDecodeError {
    fn new(message: String) -> Self {
        return Self { message: message, line: 0, column: 0 };
    }
}

impl std::fmt::Display for TOMLDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
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
        return Self { message: message, detail: String::new() };
    }
}

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return std::fmt::Display::fmt(&self.message, f);
    }
}

impl std::error::Error for RegexError {
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

#[derive(Debug, Clone)]
struct FileHandle {
    _handle: i64,
    _mode: String,
}

impl FileHandle {
    fn new(_handle: i64, _mode: String) -> Self {
        return Self { _handle: _handle, _mode: _mode };
    }
    fn read(&self) -> Result<String, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __s = String::new();
                std::io::Read::read_to_string(__r, &mut __s).map_err(__io_err)?;
                return Ok(__s);
            },
            _ => {
                return Err(IOError { message: "file not open for reading".to_string(), kind: "Other".to_string() });
            },
        }
    }
    fn write(&self, data: &String) -> Result<(), IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextWrite(ref mut __w)) => {
                std::io::Write::write_all(__w, data.as_bytes()).map_err(__io_err)?;
                return Ok(());
            },
            _ => {
                return Err(IOError { message: "file not open for writing".to_string(), kind: "Other".to_string() });
            },
        }
    }
    fn readline(&self) -> Result<Option<String>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner());
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
            },
            _ => {
                return Err(IOError { message: "file not open for reading".to_string(), kind: "Other".to_string() });
            },
        }
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __lines: Vec<String> = Vec::<String>::new();
                let mut __line = String::new();
                loop {
                    __line.clear();
                    let __n = std::io::BufRead::read_line(__r, &mut __line).map_err(__io_err)?;
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
            },
            _ => {
                return Err(IOError { message: "file not open for reading".to_string(), kind: "Other".to_string() });
            },
        }
    }
    fn close(&self) {
        let __hid = self._handle;
        __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).remove(&__hid);
    }
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::BinaryRead(ref mut __r)) => {
                let mut __buf = Vec::<u8>::new();
                std::io::Read::read_to_end(__r, &mut __buf).map_err(__io_err)?;
                return Ok(__buf);
            },
            _ => {
                return Err(IOError { message: "file not open for binary reading".to_string(), kind: "Other".to_string() });
            },
        }
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::BinaryWrite(ref mut __w)) => {
                std::io::Write::write_all(__w, &data).map_err(__io_err)?;
                return Ok(());
            },
            _ => {
                return Err(IOError { message: "file not open for binary writing".to_string(), kind: "Other".to_string() });
            },
        }
    }
    fn __enter__(&self) -> &Self {
        return self;
    }
    fn __exit__(&self) {
        self.close();
    }
}

fn main() {
    let path: String = "/tmp/sifr_bytes_bytes_file_io.bin".to_string();
    let mut loaded_ok: bool = false;
    let mut ints_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let mut writer: FileHandle = (|| {
    let __path = path.to_string();
    let __mode = "wb".to_string().to_string();
    let __handle_id = __sifr_next_file_handle_id();
    match __mode.as_str() {
        "r" | "rt" => {
            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
            let __reader = std::io::BufReader::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::TextRead(__reader));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "w" | "wt" => {
            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::TextWrite(__writer));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "a" | "at" => {
            let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::TextWrite(__writer));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "rb" => {
            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
            let __reader = std::io::BufReader::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "wb" => {
            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "ab" => {
            let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        _ => {
            return Err(IOError { message: format!("invalid mode: {}", __mode), kind: "Other".to_string() });
        },
    }
})()?;
    let _w: () = writer.write_bytes(&vec![(98 as i64) as u8, (121 as i64) as u8, (116 as i64) as u8, (101 as i64) as u8, (115 as i64) as u8, (95 as i64) as u8, (102 as i64) as u8, (105 as i64) as u8, (108 as i64) as u8, (101 as i64) as u8, (95 as i64) as u8, (105 as i64) as u8, (111 as i64) as u8])?;
    writer.close();
    let mut reader: FileHandle = (|| {
    let __path = path.to_string();
    let __mode = "rb".to_string().to_string();
    let __handle_id = __sifr_next_file_handle_id();
    match __mode.as_str() {
        "r" | "rt" => {
            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
            let __reader = std::io::BufReader::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::TextRead(__reader));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "w" | "wt" => {
            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::TextWrite(__writer));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "a" | "at" => {
            let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::TextWrite(__writer));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "rb" => {
            let __f = std::fs::File::open(__path.as_str()).map_err(__io_err)?;
            let __reader = std::io::BufReader::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryRead(__reader));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "wb" => {
            let __f = std::fs::File::create(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        "ab" => {
            let __f = std::fs::OpenOptions::new().append(true).create(true).open(__path.as_str()).map_err(__io_err)?;
            let __writer = std::io::BufWriter::new(__f);
            __SIFR_FILE_HANDLES.lock().unwrap_or_else(|__err| __err.into_inner()).insert(__handle_id, SifrFileHandle::BinaryWrite(__writer));
            return Ok(FileHandle { _handle: __handle_id, _mode: __mode.to_string() });
        },
        _ => {
            return Err(IOError { message: format!("invalid mode: {}", __mode), kind: "Other".to_string() });
        },
    }
})()?;
    let loaded: Vec<u8> = reader.read_bytes()?;
    reader.close();
    loaded_ok = loaded == vec![(98 as i64) as u8, (121 as i64) as u8, (116 as i64) as u8, (101 as i64) as u8, (115 as i64) as u8, (95 as i64) as u8, (102 as i64) as u8, (105 as i64) as u8, (108 as i64) as u8, (101 as i64) as u8, (95 as i64) as u8, (105 as i64) as u8, (111 as i64) as u8];
    ints_ok = format!("{:?}", loaded.iter().map(|__byte| *__byte as i64).collect::<Vec<i64>>()) == "[98, 121, 116, 101, 115, 95, 102, 105, 108, 101, 95, 105, 111]".to_string();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    if std::path::Path::new(&path).exists() {
        let _rm: () = std::fs::remove_file(&path).map(|_| ()).map_err(__io_err)?;
    }
    cleanup_ok = !(std::path::Path::new(&path).exists());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    assert!(loaded_ok);
    assert!(ints_ok);
    assert!(cleanup_ok);
}
