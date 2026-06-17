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
        return std::fs::remove_file(&self._path.clone()).map(|_| ()).map_err(__io_err);
    }
    fn rmdir(&self) -> Result<(), IOError> {
        return std::fs::remove_dir(&self._path.clone()).map(|_| ()).map_err(__io_err);
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
        return Path::new(
            format!("{}{}", format!("{}{}", parent, "/".to_string()), name),
        );
    }
    fn with_suffix(&self, suffix: &String) -> Path {
        let s: String = stem(&self._path.clone());
        let parent: String = dirname(&self._path.clone());
        if parent == "".to_string() {
            return Path::new(format!("{}{}", s, suffix));
        }
        return Path::new(
            format!(
                "{}{}", format!("{}{}", format!("{}{}", parent, "/".to_string()), s),
                suffix
            ),
        );
    }
    fn glob(
        &self,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
        return _glob_to_iter(&self._path.clone(), pattern);
    }
    fn rglob(
        &self,
        pattern: &String,
    ) -> Result<Box<dyn Iterator<Item = String>>, IOError> {
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
        __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
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
                return String::from_iter(
                    (path).chars().skip((i + (1 as i64)).max(0) as usize),
                );
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
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        };
        let sep: Option<String> = {
            let __sifr_index_str = &path;
            let __sifr_index_i = 2 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
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
    return Box::new(
        std::iter::from_fn(move || {
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
        }),
    );
}
fn _iterdir_list(path: &String) -> Result<Vec<String>, IOError> {
    return {
        let __entries = std::fs::read_dir(&path).map_err(__io_err)?;
        Ok(
            __entries
                .filter_map(|e| e.ok().map(|e| e.path().to_string_lossy().to_string()))
                .collect::<Vec<String>>(),
        )
    };
}
fn _glob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    return {
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
    };
}
fn _rglob_list(path: &String, pattern: &String) -> Result<Vec<String>, IOError> {
    return {
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
    };
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

fn collect_path_helpers_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((basename(&"/tmp/demo.txt".to_string())).as_str() == ("demo.txt".to_string()).as_str());
    actual.push((join_path(&"/tmp".to_string(), &"demo.txt".to_string())).as_str() == ("/tmp/demo.txt".to_string()).as_str());
    return actual;
}

fn collect_path_class_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let base: String = format!("{}{}", "/tmp/sifr_pathlib_pathlib_demo_".to_string(), format!("{}", std::process::id() as i64));
    let mut filep: Path = Path::new(format!("{}{}", base, "/demo.txt".to_string()));
    let mut dirp: Path = Path::new(format!("{}{}", base, "".to_string()));
    let mut path_flow_ok: bool = false;
    let mut glob_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _m: () = dirp.mkdir()?;
    let _w: () = filep.write_text(&"hello".to_string())?;
    let content: String = filep.read_text()?;
    path_flow_ok = (((filep.exists()) && (filep.is_file())) && (content == "hello".to_string()));
    let mut matches_it: Box<dyn Iterator<Item = String>> = dirp.glob(&"*.txt".to_string())?;
    let matches: Vec<String> = matches_it.collect::<Vec<_>>();
    glob_ok = (matches.len() as i64) >= (1 as i64);
    let _u: () = filep.unlink()?;
    let _r: () = dirp.rmdir()?;
    cleanup_ok = !(dirp.exists());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    actual.push(path_flow_ok);
    actual.push(glob_ok);
    actual.push(cleanup_ok);
    return actual;
}

fn collect_missing_path_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut missing_rejected: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _bad: String = Path::new("/tmp/sifr_pathlib_pathlib_demo_missing.txt".to_string()).read_text()?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        missing_rejected = true;
    }
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
    append_all(&mut actual, &collect_path_helpers_actual());
    append_all(&mut actual, &collect_path_class_actual());
    append_all(&mut actual, &collect_missing_path_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("pathlib pathlib parity demo: pass");
}
