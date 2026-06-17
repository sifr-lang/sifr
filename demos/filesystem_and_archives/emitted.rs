// --- stdlib: sifr.shutil ---
fn copy(src: &String, dst: &String) -> Result<(), IOError> {
    return std::fs::copy(&src, &dst).map(|_| ()).map_err(__io_err);
}
fn move_file(src: &String, dst: &String) -> Result<(), IOError> {
    return std::fs::rename(&src, &dst).map(|_| ()).map_err(__io_err);
}
fn rmtree(path: &String) -> Result<(), IOError> {
    return std::fs::remove_dir_all(&path).map(|_| ()).map_err(__io_err);
}

// --- stdlib: sifr.fnmatch ---
fn fnmatch(name: &String, pattern: &String) -> bool {
    return _match(name, 0 as i64, pattern, 0 as i64);
}
fn _match(name: &String, mut ni: i64, pattern: &String, mut pi: i64) -> bool {
    while pi < (pattern.chars().count() as i64) {
        let pc: Option<String> = Some({
            let Some(__indexed_char) = pattern.chars().nth(pi as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(pc) = pc {
            if pc == "*".to_string() {
                pi = pi + (1 as i64);
                if pi == (pattern.len() as i64) {
                    return true;
                }
                let mut j: i64 = ni;
                while j <= (name.chars().count() as i64) {
                    if _match(name, j, pattern, pi) {
                        return true;
                    }
                    j = j + (1 as i64);
                }
                return false;
            } else {
                if pc == "?".to_string() {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                } else {
                    if ni >= (name.len() as i64) {
                        return false;
                    }
                    let nc: Option<String> = Some({
                        let Some(__indexed_char) = name.chars().nth(ni as usize) else {
                            unreachable!(
                                "compiler-verified string index should be in range"
                            );
                        };
                        __indexed_char.to_string()
                    });
                    if let Some(nc) = nc {
                        if nc != pc {
                            return false;
                        }
                    } else {
                        return false;
                    }
                    ni = ni + (1 as i64);
                    pi = pi + (1 as i64);
                }
            }
        } else {
            return false;
        }
    }
    return ni == (name.chars().count() as i64);
}
fn fnmatch_filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if fnmatch(&name, pattern) {
            result.push(name);
        }
    }
    return result;
}
fn filterfalse(names: &Vec<String>, pattern: &String) -> Vec<String> {
    let mut result: Vec<String> = vec![];
    for name in names.iter().cloned() {
        if !(fnmatch(&name, pattern)) {
            result.push(name);
        }
    }
    return result;
}
fn fnmatchcase(name: &String, pattern: &String) -> bool {
    return _match(name, 0 as i64, pattern, 0 as i64);
}
fn _translate_literal(ch: &String) -> String {
    if ch.clone() == ".".to_string() {
        return "\\.".to_string();
    }
    if ch.clone() == "^".to_string() {
        return "\\^".to_string();
    }
    if ch.clone() == "$".to_string() {
        return "\\$".to_string();
    }
    if ch.clone() == "+".to_string() {
        return "\\+".to_string();
    }
    if ch.clone() == "(".to_string() {
        return "\\(".to_string();
    }
    if ch.clone() == ")".to_string() {
        return "\\)".to_string();
    }
    if ch.clone() == "{".to_string() {
        return "\\{".to_string();
    }
    if ch.clone() == "}".to_string() {
        return "\\}".to_string();
    }
    if ch.clone() == "[".to_string() {
        return "\\[".to_string();
    }
    if ch.clone() == "]".to_string() {
        return "\\]".to_string();
    }
    if ch.clone() == "|".to_string() {
        return "\\|".to_string();
    }
    if ch.clone() == "\\".to_string() {
        return "\\\\".to_string();
    }
    return format!("{}{}", ch, "".to_string());
}
fn translate(pattern: &String) -> String {
    let mut body: String = "".to_string();
    let mut i: i64 = 0 as i64;
    while i < (pattern.chars().count() as i64) {
        let ch: Option<String> = Some({
            let Some(__indexed_char) = pattern.chars().nth(i as usize) else {
                unreachable!("compiler-verified string index should be in range");
            };
            __indexed_char.to_string()
        });
        if let Some(ch) = ch {
            if ch == "*".to_string() {
                body = format!("{}{}", body, ".*".to_string());
            } else {
                if ch == "?".to_string() {
                    body = format!("{}{}", body, ".".to_string());
                } else {
                    body = format!("{}{}", body, _translate_literal(& ch));
                }
            }
        }
        i = i + (1 as i64);
    }
    return format!("{}{}{}", "(?s:".to_string(), body, ")\\z".to_string());
}
fn filter(names: &Vec<String>, pattern: &String) -> Vec<String> {
    return fnmatch_filter(names, pattern);
}

// --- stdlib: sifr.glob ---
fn glob(directory: &String, pattern: &String) -> Vec<String> {
    let include_hidden: bool = (((pattern.chars().count() as i64) > (0 as i64))
        && (({
            let __sifr_index_str = &pattern;
            let __sifr_index_i = 0 as i64;
            let __sifr_index_norm = if __sifr_index_i < 0 {
                ((__sifr_index_str.chars().count() as i64) + __sifr_index_i) as usize
            } else {
                __sifr_index_i as usize
            };
            __sifr_index_str.chars().nth(__sifr_index_norm).map(|c| c.to_string())
        }) == Some(".".to_string())));
    let mut matches: Vec<String> = vec![];
    let __sifr_try_res: Result<(), IOError> = (|| {
        let entries: Vec<String> = std::fs::read_dir(&directory)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect::<Vec<String>>()
            })
            .map_err(__io_err)?;
        for entry in entries.iter().cloned() {
            if (entry.len() as i64) == (0 as i64) {
                continue;
            }
            if ((!(include_hidden))
                && (({
                    let __sifr_index_str = &entry;
                    let __sifr_index_i = 0 as i64;
                    let __sifr_index_norm = if __sifr_index_i < 0 {
                        ((__sifr_index_str.chars().count() as i64) + __sifr_index_i)
                            as usize
                    } else {
                        __sifr_index_i as usize
                    };
                    __sifr_index_str
                        .chars()
                        .nth(__sifr_index_norm)
                        .map(|c| c.to_string())
                }) == Some(".".to_string())))
            {
                continue;
            }
            if fnmatch(&entry, pattern) {
                matches.push(entry);
            }
        }
        return Ok(());
    })();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        return vec![];
    }
    return {
        let mut __sifr_sorted_v = (matches).iter().cloned().collect::<Vec<_>>();
        __sifr_sorted_v.sort();
        __sifr_sorted_v
    };
}

// --- stdlib: sifr.gzip ---
fn compress(data: &String) -> Vec<i64> {
    return {
        let __data = &data.as_bytes();
        let mut __enc = flate2::write::GzEncoder::new(
            Vec::new(),
            flate2::Compression::default(),
        );
        std::io::Write::write_all(&mut __enc, __data).unwrap_or(());
        __enc
            .finish()
            .unwrap_or_default()
            .iter()
            .map(|b| *b as i64)
            .collect::<Vec<i64>>()
    };
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
    let base: String = format!("{}{}", "/tmp/sifr_filesystem_archive_surface_demo_".to_string(), format!("{}", std::process::id() as i64));
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _mk: String = ({
    let __cmd = format!("{}{}", "mkdir -p ".to_string(), base);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let source: String = format!("{}{}", base, "/note.txt".to_string());
    let _w: () = std::fs::write(&source, "hello d1".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let note_content: String = std::fs::read_to_string(&source).map_err(__io_err)?;
    println!("{}", format!("{}{}", "io.read_text = ".to_string(), note_content));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "io.read_text error: ".to_string(), e.message));
    }
    let mut note_path: Path = Path::new(format!("{}{}", source, "".to_string()));
    println!("{}", format!("{}{}", "pathlib.stem = ".to_string(), note_path.stem()));
    println!("{}", format!("{}{}", "glob(\"*.txt\") = ".to_string(), format!("{:?}", glob(&base, &"*.txt".to_string()))));
    let copied: String = format!("{}{}", base, "/copied.txt".to_string());
    let moved: String = format!("{}{}", base, "/moved.txt".to_string());
    let _cp: () = copy(&source, &copied)?;
    let _mv: () = move_file(&copied, &moved)?;
    println!("{}", format!("{}{}", "shutil.move_file exists = ".to_string(), format!("{}", Path::new(moved).exists())));
    let temp_file: String = mkstemp(&"sifr_filesystem_archive_surface_demo_".to_string())?;
    let temp_dir: String = mkdtemp(&"sifr_filesystem_archive_surface_demo_".to_string())?;
    println!("{}", format!("{}{}", "tempfile.mkstemp = ".to_string(), temp_file));
    println!("{}", format!("{}{}", "tempfile.mkdtemp = ".to_string(), temp_dir));
    let compressed: Vec<i64> = compress(&"archive sample".to_string());
    let __sifr_try_res: Result<(), IOError> = (|| {
    let restored: String = decompress(&compressed)?;
    println!("{}", format!("{}{}", "gzip roundtrip = ".to_string(), restored));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "gzip error: ".to_string(), e.message));
    }
    let zip_path: String = format!("{}{}", base, "/demo.zip".to_string());
    let mut archive: ZipFile = ZipFile::new(zip_path, "a".to_string(), 0 as i64);
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _zc: () = archive.create()?;
    let _zw: () = archive.write(&"inside.txt".to_string(), &"inside-zip".to_string())?;
    let inside: String = archive.read(&"inside.txt".to_string())?;
    println!("{}", format!("{}{}", "zipfile.read = ".to_string(), inside));
    println!("{}", format!("{}{}", "zipfile.namelist = ".to_string(), format!("{:?}", archive.namelist())));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "zipfile error: ".to_string(), e.message));
    }
    let _rm_temp_file: String = ({
    let __cmd = format!("{}{}", "rm -f ".to_string(), temp_file);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let _rm_temp_dir: () = rmtree(&temp_dir)?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", format!("{}{}", "filesystem_archive_surface demo error: ".to_string(), e.message));
    }
}
