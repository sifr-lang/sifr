use std::sync::Mutex;

// --- stdlib: sifr.test ---
fn assert_eq<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    actual: &T,
    expected: &T,
) {
    assert!(* actual == * expected);
}
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0 as i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i = i + (1 as i64);
    }
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

fn collect_io_roundtrip_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let path: String = "/tmp/sifr_io_io_demo.txt".to_string();
    let mut text_roundtrip_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _w: () = std::fs::write(&path, "hello".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _a: () = ({
    let mut _f = std::fs::OpenOptions::new().append(true).create(true).open(&path).map_err(__io_err)?;
    std::io::Write::write_all(&mut _f, "\nworld".to_string().as_bytes()).map_err(__io_err)?;
    Ok(())
})?;
    let content: String = std::fs::read_to_string(&path).map_err(__io_err)?;
    text_roundtrip_ok = content == "hello\nworld".to_string();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    actual.push(text_roundtrip_ok);
    actual.push(std::path::Path::new(&path).exists());
    return actual;
}

fn collect_open_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let path: String = "/tmp/sifr_io_io_demo.txt".to_string();
    let mut first_ok: bool = false;
    let mut second_ok: bool = false;
    let mut eof_ok: bool = false;
    let mut missing_rejected: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let mut f: FileHandle = (|| {
    let __path = path.to_string();
    let __mode = "r".to_string().to_string();
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
    let first: Option<String> = f.readline()?;
    let second: Option<String> = f.readline()?;
    let third: Option<String> = f.readline()?;
    f.close();
    first_ok = (first).map_or("None".to_string().to_string(), |__v| format!("{}", __v)) == "hello".to_string();
    second_ok = (second).map_or("None".to_string().to_string(), |__v| format!("{}", __v)) == "world".to_string();
    eof_ok = third.is_none();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _: FileHandle = (|| {
    let __path = "/tmp/sifr_io_io_demo_missing.txt".to_string().to_string();
    let __mode = "r".to_string().to_string();
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
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        missing_rejected = true;
    }
    actual.push(first_ok);
    actual.push(second_ok);
    actual.push(eof_ok);
    actual.push(missing_rejected);
    return actual;
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_io_roundtrip_actual());
    append_all(&mut actual, &collect_open_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("io io parity demo: pass");
}
