use std::sync::Mutex;

// --- stdlib: sifr.logging ---
#[derive(Debug, Clone)]
struct FileHandle {
    _handle: i64,
    _mode: String,
}
impl FileHandle {
    fn new(_handle: i64, _mode: String) -> Self {
        return Self {
            _handle: _handle,
            _mode: _mode,
        };
    }
    fn read(&self) -> Result<String, IOError> {
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
    }
    fn write(&self, data: &String) -> Result<(), IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextWrite(ref mut __w)) => {
                std::io::Write::write_all(__w, data.as_bytes()).map_err(__io_err)?;
                return Ok(());
            }
            _ => {
                return Err(IOError {
                    message: "file not open for writing".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn readline(&self) -> Result<Option<String>, IOError> {
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
    }
    fn readlines(&self) -> Result<Vec<String>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::TextRead(ref mut __r)) => {
                let mut __lines: Vec<String> = Vec::<String>::new();
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
    }
    fn close(&self) {
        let __hid = self._handle;
        __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner())
            .remove(&__hid);
    }
    fn read_bytes(&self) -> Result<Vec<u8>, IOError> {
        let __hid = self._handle;
        let mut __handles = __SIFR_FILE_HANDLES
            .lock()
            .unwrap_or_else(|__err| __err.into_inner());
        match __handles.get_mut(&__hid) {
            Some(SifrFileHandle::BinaryRead(ref mut __r)) => {
                let mut __buf = Vec::<u8>::new();
                std::io::Read::read_to_end(__r, &mut __buf).map_err(__io_err)?;
                return Ok(__buf);
            }
            _ => {
                return Err(IOError {
                    message: "file not open for binary reading".to_string(),
                    kind: "Other".to_string(),
                });
            }
        }
    }
    fn write_bytes(&self, data: &Vec<u8>) -> Result<(), IOError> {
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
    }
    fn __enter__(&self) -> &Self {
        return self;
    }
    fn __exit__(&self) {
        self.close();
    }
}
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
            f, "StreamHandler(_level={}, _formatter={})", self._level, self._formatter
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
            "{}{}", self._formatter.clone().format(level, name, msg), "\n".to_string()
        );
        let __sifr_try_res: Result<(), IOError> = (|| {
            let mut fh: FileHandle = (|| {
                let __path = self._path.clone().to_string();
                let __mode = "a".to_string().to_string();
                let __handle_id = __sifr_next_file_handle_id();
                match __mode.as_str() {
                    "r" | "rt" => {
                        let __f = std::fs::File::open(__path.as_str())
                            .map_err(__io_err)?;
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
                        let __f = std::fs::File::create(__path.as_str())
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
                        let __f = std::fs::File::open(__path.as_str())
                            .map_err(__io_err)?;
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
                        let __f = std::fs::File::create(__path.as_str())
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
            f, "FileHandler(_path={}, _level={}, _formatter={})", self._path, self
            ._level, self._formatter
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
            f, "NullHandler(_level={}, _formatter={})", self._level, self._formatter
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
            if ((self._handler_allows(level_num))
                && (self._handler_path.clone() != "".to_string()))
            {
                let line: String = format!(
                    "{}{}", self._handler_line(level, msg), "\n".to_string()
                );
                let __sifr_try_res: Result<(), IOError> = (|| {
                    let mut fh: FileHandle = (|| {
                        let __path = self._handler_path.clone().to_string();
                        let __mode = "a".to_string().to_string();
                        let __handle_id = __sifr_next_file_handle_id();
                        match __mode.as_str() {
                            "r" | "rt" => {
                                let __f = std::fs::File::open(__path.as_str())
                                    .map_err(__io_err)?;
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
                                let __f = std::fs::File::create(__path.as_str())
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
                                let __f = std::fs::File::open(__path.as_str())
                                    .map_err(__io_err)?;
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
                                let __f = std::fs::File::create(__path.as_str())
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
            "{}{}{}{}{}{}", "[".to_string(), level, "] ".to_string(), self._name.clone(),
            ": ".to_string(), msg
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
                            let __f = std::fs::File::open(__path.as_str())
                                .map_err(__io_err)?;
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
                            let __f = std::fs::File::create(__path.as_str())
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
                            let __f = std::fs::File::open(__path.as_str())
                                .map_err(__io_err)?;
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
                            let __f = std::fs::File::create(__path.as_str())
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

static __SIFR_GLOBAL_LOG_LEVEL: std::sync::LazyLock<std::sync::Mutex<i64>> = std::sync::LazyLock::new(|| std::sync::Mutex::new(20));

fn main() {
    let path: String = "/tmp/sifr_codegen_preamble_demo.txt".to_string();
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _: () = std::fs::write(&path, "codegen preamble".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
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
    let text: String = f.read()?;
    f.close();
    println!("{}", format!("{}{}", "file = ".to_string(), text));
    assert!(format!("{}", format!("{}{}", "file = ".to_string(), text)) == "file = codegen preamble".to_string());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "ioerror = ".to_string(), e.message));
        assert!(format!("{}", format!("{}{}", "ioerror = ".to_string(), e.message)) == "preamble demo complete".to_string());
    }
    let mut log: Logger = getLogger(&"codegen".to_string());
    log.set_level(20 as i64);
    log.info(&"preamble logging alive".to_string());
    println!("preamble demo complete");
}
