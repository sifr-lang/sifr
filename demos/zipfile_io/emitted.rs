// --- stdlib: sifr.tempfile ---
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct NamedTemporaryFile {
    _path: String,
    _mode: String,
    _delete: bool,
    _closed: bool,
    _cleaned: bool,
}
impl NamedTemporaryFile {
    fn new(mode: String, delete: bool, prefix: String) -> Self {
        let mut candidate: String = mktemp_path(&prefix);
        while std::path::Path::new(&candidate).exists() {
            candidate = mktemp_path(&prefix);
        }
        let _created_result: Result<(), IOError> = std::fs::write(
                &candidate,
                "".to_string().as_bytes(),
            )
            .map(|_| ())
            .map_err(__io_err);
        return Self {
            _path: format!("{}{}", candidate, "".to_string()),
            _mode: format!("{}{}", mode, "".to_string()),
            _delete: delete,
            _closed: false,
            _cleaned: false,
        };
    }
    fn name(&self) -> String {
        return format!("{}{}", self._path.clone(), "".to_string());
    }
    fn closed(&self) -> bool {
        return self._closed;
    }
    fn _cleanup_path(&mut self) -> Result<(), IOError> {
        if self._cleaned {
            return Ok(());
        }
        if std::path::Path::new(&self._path.clone()).exists() {
            let __sifr_try_res: Result<(), IOError> = (|| {
                let _rm_done: () = std::fs::remove_file(&self._path.clone())
                    .map(|_| ())
                    .map_err(__io_err)?;
                return Ok(());
            })();
            if let Err(__sifr_try_err) = __sifr_try_res {
                let e = __sifr_try_err.clone();
                return Err(IOError::new(e.message));
            }
        }
        self._cleaned = true;
        return Ok(());
    }
    fn close(&mut self) -> Result<(), IOError> {
        self._closed = true;
        if self._delete {
            return self._cleanup_path();
        }
        return Ok(());
    }
    fn cleanup(&mut self) -> Result<(), IOError> {
        self._closed = true;
        return self._cleanup_path();
    }
    fn __enter__(&self) -> NamedTemporaryFile {
        return self.clone();
    }
    fn __exit__(&mut self) {
        let __sifr_try_res: Result<(), IOError> = (|| {
            let _closed_done: () = self.close()?;
            return Ok(());
        })();
        if let Err(__sifr_try_err) = __sifr_try_res {
            let e = __sifr_try_err.clone();
            let _: String = e.message;
        }
    }
}
impl std::fmt::Display for NamedTemporaryFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "NamedTemporaryFile(_path={}, _mode={}, _delete={}, _closed={}, _cleaned={})",
            self._path, self._mode, self._delete, self._closed, self._cleaned
        );
    }
}
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

// --- stdlib: sifr.zipfile ---
const ZIP_STORED: i64 = 0 as i64;
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
fn is_zipfile(path: &String) -> bool {
    let __sifr_try_res: Result<bool, IOError> = (|| {
        let _names: Vec<String> = ({
            let __f = std::fs::File::open(&path).map_err(__io_err)?;
            let mut __zip = zip::ZipArchive::new(__f)
                .map_err(|e| IOError::new(e.to_string()))?;
            let mut __names = Vec::new();
            for __i in 0..__zip.len() {
                if let Ok(__file) = __zip.by_index(__i) {
                    __names.push(__file.name().to_string());
                }
            }
            Ok(__names)
        })?;
        return Ok(true);
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            let _: String = e.message;
            return false;
        }
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

fn main() {
    let zip_path: String = "/tmp/sifr_runtime_zipfile_io.zip".to_string();
    let mut demo_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let mut temp_file: NamedTemporaryFile = NamedTemporaryFile::new("wb".to_string(), false, "sifr_runtime_zipfile_io_".to_string());
    let tmp_path: String = temp_file.name();
    let _close_tmp: () = temp_file.close()?;
    let _cleanup_tmp: () = temp_file.cleanup()?;
    let tempfile_ok: bool = !(std::path::Path::new(&tmp_path).exists());
    if std::path::Path::new(&zip_path).exists() {
        let _rm_old: () = std::fs::remove_file(&zip_path).map(|_| ()).map_err(__io_err)?;
    }
    let mut writer: ZipFile = ZipFile::new(format!("{}{}", zip_path, "".to_string()), "w".to_string(), ZIP_STORED);
    let _create: () = writer.create()?;
    let _write_text: () = writer.write(&"note.txt".to_string(), &"runtime-zipfile_io".to_string())?;
    let _write_bytes: () = writer.write_bytes(&"bin/raw.bin".to_string(), &vec![(0 as i64) as u8, (1 as i64) as u8, (2 as i64) as u8])?;
    let mut reader: ZipFile = ZipFile::new(format!("{}{}", zip_path, "".to_string()), "r".to_string(), ZIP_STORED);
    let payload: Vec<u8> = reader.read_bytes(&"bin/raw.bin".to_string())?;
    let mut handle: ZipReadHandle = ZipReadHandle::new(vec![(97 as i64) as u8, (98 as i64) as u8, (99 as i64) as u8]);
    let read_all: Vec<u8> = handle.read_bytes(Some(-(1 as i64)))?;
    let handle_negative_ok: bool = read_all == vec![(97 as i64) as u8, (98 as i64) as u8, (99 as i64) as u8];
    let _open_handle_result: Result<ZipReadHandle, IOError> = reader.open(&"bin/raw.bin".to_string(), &"rb".to_string());
    let mut open_rejected: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _open_handle: ZipReadHandle = _open_handle_result?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
        open_rejected = true;
    }
    let mut bad_mode_writer: ZipFile = ZipFile::new(format!("{}{}", zip_path, "".to_string()), "rw".to_string(), ZIP_STORED);
    let _bad_mode_write_result: Result<(), IOError> = bad_mode_writer.write(&"bad.txt".to_string(), &"bad-mode".to_string());
    let mut bad_mode_rejected: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _bad_mode_ok: () = _bad_mode_write_result?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
        bad_mode_rejected = true;
    }
    demo_ok = ((((((tempfile_ok) && (is_zipfile(&zip_path))) && (payload == vec![(0 as i64) as u8, (1 as i64) as u8, (2 as i64) as u8])) && (handle_negative_ok)) && (open_rejected)) && (bad_mode_rejected));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    let __sifr_try_res: Result<(), IOError> = (|| {
    if std::path::Path::new(&zip_path).exists() {
        let _rm_zip: () = std::fs::remove_file(&zip_path).map(|_| ()).map_err(__io_err)?;
    }
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = e.message;
    }
    assert!(demo_ok);
    println!("runtime_zipfile_io_zipfile_lifecycle_demo: ok");
}
