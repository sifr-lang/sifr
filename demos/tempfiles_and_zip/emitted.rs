// --- stdlib: sifr.tempfile ---
fn _random_suffix() -> String {
    let n: i64 = {
        let __start = 100000 as i64;
        let __end = 999999 as i64;
        __start + rand::RngExt::random_range(&mut rand::rng(), 0..(__end - __start) + 1)
    };
    return format!("{}", n);
}
fn mktemp_path(prefix: &String) -> String {
    let suffix: String = _random_suffix();
    let mut root: String = std::env::temp_dir().display().to_string();
    if (root.chars().count() as i64) == (0 as i64) {
        root = "/tmp".to_string();
    } else {
        let last: Option<String> = {
            let __sifr_index_str = &root;
            let __sifr_index_i = (root.chars().count() as i64) - (1 as i64);
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        if let Some(last) = last {
            if last == "/".to_string() {
                root = String::from_iter(
                    (root)
                        .chars()
                        .skip(0 as usize)
                        .take(
                            (((root.chars().count() as i64) - (1 as i64)).max(0) - 0)
                                .max(0) as usize,
                        ),
                );
            }
        }
    }
    return format!("{}{}{}{}", root, "/".to_string(), prefix, suffix);
}
fn _next_candidate(prefix: &String) -> String {
    return mktemp_path(prefix);
}
fn _collision_message(kind: &String, attempts: i64) -> String {
    return format!(
        "{}{}{}{}{}", "tempfile.".to_string(), kind,
        ": failed to create unique path after ".to_string(), format!("{}", attempts),
        " attempts".to_string()
    );
}
fn mkstemp(prefix: &String) -> Result<String, IOError> {
    let mut attempts: i64 = 0 as i64;
    let max_attempts: i64 = 64 as i64;
    while attempts < max_attempts {
        let path: String = _next_candidate(prefix);
        let path_for_check: String = format!("{}{}", path, "".to_string());
        if std::path::Path::new(&path).exists() {
            attempts = attempts + (1 as i64);
            continue;
        }
        let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
            let wrt: () = std::fs::write(&path, "".to_string().as_bytes())
                .map(|_| ())
                .map_err(__io_err)?;
            return Ok(Ok(path));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                if std::path::Path::new(&path_for_check).exists() {
                    attempts = attempts + (1 as i64);
                    continue;
                }
                return Err(IOError::new(e.message));
            }
        }
    }
    return Err(IOError::new(_collision_message(&"mkstemp".to_string(), max_attempts)));
}
fn mkdtemp(prefix: &String) -> Result<String, IOError> {
    let mut attempts: i64 = 0 as i64;
    let max_attempts: i64 = 64 as i64;
    while attempts < max_attempts {
        let path: String = _next_candidate(prefix);
        let path_for_check: String = format!("{}{}", path, "".to_string());
        if std::path::Path::new(&path).exists() {
            attempts = attempts + (1 as i64);
            continue;
        }
        let __sifr_try_res: Result<Result<String, IOError>, IOError> = (|| {
            let md: () = std::fs::create_dir_all(&path).map(|_| ()).map_err(__io_err)?;
            return Ok(Ok(path));
            unreachable!("sifr try/except return capture fell through");
        })();
        match __sifr_try_res {
            Ok(__sifr_ret_val) => {
                return __sifr_ret_val;
            }
            Err(__sifr_try_err) => {
                let e = __sifr_try_err.clone();
                if std::path::Path::new(&path_for_check).exists() {
                    attempts = attempts + (1 as i64);
                    continue;
                }
                return Err(IOError::new(e.message));
            }
        }
    }
    return Err(IOError::new(_collision_message(&"mkdtemp".to_string(), max_attempts)));
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
            f, "ZipInfo(filename={}, file_size={}, compress_type={})", self.filename,
            self.file_size, self.compress_type
        );
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
    fn new(path: String, mode: String, compression: i64) -> Self {
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
        return {
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
        };
    }
    fn write_bytes(&self, name: &String, content: &Vec<u8>) -> Result<(), IOError> {
        if !(self._writable_mode()) {
            return Err(IOError::new(_zip_read_only_error()));
        }
        return {
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
        };
    }
    fn read(&self, name: &String) -> Result<String, IOError> {
        return {
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
        };
    }
    fn read_bytes(&self, name: &String) -> Result<Vec<u8>, IOError> {
        return {
            let __f = std::fs::File::open(&self.path.clone()).map_err(__io_err)?;
            let mut __zip = zip::ZipArchive::new(__f)
                .map_err(|e| IOError::new(e.to_string()))?;
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
            let mut __zip = zip::ZipArchive::new(__f)
                .map_err(|e| IOError::new(e.to_string()))?;
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
        return Err(IOError::new(_zip_unimplemented_error(&"infolist".to_string())));
    }
    fn getinfo(&self, name: &String) -> Result<ZipInfo, IOError> {
        let _: String = (name).clone();
        return Err(IOError::new(_zip_unimplemented_error(&"getinfo".to_string())));
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
        return Err(IOError::new(_zip_unimplemented_error(&"extract".to_string())));
    }
    fn extractall(&self, path: &String) -> Result<Vec<String>, IOError> {
        let _: String = (path).clone();
        return Err(IOError::new(_zip_unimplemented_error(&"extractall".to_string())));
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
            f, "ZipFile(path={}, mode={}, compression={})", self.path, self.mode, self
            .compression
        );
    }
}
fn _zip_read_only_error() -> String {
    return "zipfile operation requires write or append mode".to_string();
}
fn _zip_open_mode_error(mode: &String) -> String {
    return format!(
        "{}{}", "zipfile open supports read-only mode only, got: ".to_string(), mode
    );
}
fn _closed_stream_error() -> String {
    return "I/O operation on closed stream".to_string();
}
fn _zip_unimplemented_error(feature: &String) -> String {
    return format!(
        "{}{}{}", "zipfile ".to_string(), feature, " is not implemented in this compatibility surface"
        .to_string()
    );
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

fn main() {
    let mut temp_file: String = "".to_string();
    let mut temp_dir: String = "".to_string();
    let mut zip_path: String = "".to_string();
    let mut tempfile_ok: bool = false;
    let mut zip_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let temp_file_created: String = mkstemp(&"sifr_runtime_tempfiles_and_zip_".to_string())?;
    let temp_dir_created: String = mkdtemp(&"sifr_runtime_tempfiles_and_zip_".to_string())?;
    temp_file = format!("{}{}", temp_file_created, "".to_string());
    temp_dir = format!("{}{}", temp_dir_created, "".to_string());
    tempfile_ok = ((std::path::Path::new(&temp_file).exists()) && (std::path::Path::new(&temp_dir).exists()));
    let _w: () = std::fs::write(&temp_file, "payload".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    zip_path = format!("{}{}", temp_file, ".zip".to_string());
    let mut archive: ZipFile = ZipFile::new(format!("{}{}", zip_path, "".to_string()), "a".to_string(), 0 as i64);
    let _create: () = archive.create()?;
    let _add: () = archive.write(&"entry.txt".to_string(), &"payload".to_string())?;
    let names: Vec<String> = archive.namelist()?;
    let content: String = archive.read(&"entry.txt".to_string())?;
    zip_ok = ((((names.len() as i64) == (1 as i64)) && (({
    let __sifr_index_list = &names;
    let __sifr_index_i = 0 as i64;
    let __sifr_index_norm = if __sifr_index_i < 0 { ((__sifr_index_list.len() as i64) + __sifr_index_i) as usize } else { __sifr_index_i as usize };
    __sifr_index_list.get(__sifr_index_norm).cloned()
}) == Some("entry.txt".to_string()))) && (content == "payload".to_string()));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    if (((zip_path.chars().count() as i64) > (0 as i64)) && (std::path::Path::new(&zip_path).exists())) {
        let _rm_zip: () = std::fs::remove_file(&zip_path).map(|_| ()).map_err(__io_err)?;
    }
    if (((temp_file.chars().count() as i64) > (0 as i64)) && (std::path::Path::new(&temp_file).exists())) {
        let _rm_file: () = std::fs::remove_file(&temp_file).map(|_| ()).map_err(__io_err)?;
    }
    if (((temp_dir.chars().count() as i64) > (0 as i64)) && (std::path::Path::new(&temp_dir).exists())) {
        let _rm_dir: () = std::fs::remove_dir(&temp_dir).map(|_| ()).map_err(__io_err)?;
    }
    cleanup_ok = ((((((temp_file.chars().count() as i64) == (0 as i64)) || (!(std::path::Path::new(&temp_file).exists())))) && ((((temp_dir.chars().count() as i64) == (0 as i64)) || (!(std::path::Path::new(&temp_dir).exists()))))) && ((((zip_path.chars().count() as i64) == (0 as i64)) || (!(std::path::Path::new(&zip_path).exists())))));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    assert!(tempfile_ok);
    assert!(zip_ok);
    assert!(cleanup_ok);
    println!("runtime_tempfiles_and_zip_zip_lifecycle_demo: ok");
}
