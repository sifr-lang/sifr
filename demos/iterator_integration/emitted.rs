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
            f, "Match(_matched={}, _start={}, _end={})", self._matched, self._start, self
            ._end
        );
    }
}
fn _iter_matches(matches: Vec<Match>) -> Box<dyn Iterator<Item = Match>> {
    let mut __sifr_generator_initialized: bool = false;
    let mut __sifr_generator_iter: std::vec::IntoIter<Match> = Vec::new().into_iter();
    return Box::new(
        std::iter::from_fn(move || {
            if !__sifr_generator_initialized {
                let mut _yields: Vec<Match> = Vec::new();
                let mut i: i64 = 0 as i64;
                while i < (matches.len() as i64) {
                    _yields.push(matches[i as usize].clone());
                    i = i + (1 as i64);
                }
                __sifr_generator_iter = _yields.into_iter();
                __sifr_generator_initialized = true;
            }
            return __sifr_generator_iter.next();
        }),
    );
}
fn _find_index_from(text: &String, needle: &String, start: i64) -> i64 {
    if start < (0 as i64) {
        return -(1 as i64);
    }
    if (needle.len() as i64) == (0 as i64) {
        if start <= (text.len() as i64) {
            return start;
        }
        return -(1 as i64);
    }
    let max_start: i64 = (text.chars().count() as i64) - (needle.chars().count() as i64);
    let mut i: i64 = start;
    while i <= max_start {
        if String::from_iter(
            (text)
                .chars()
                .skip((i).max(0) as usize)
                .take(
                    ((i + (needle.chars().count() as i64)).max(0) - (i).max(0)).max(0)
                        as usize,
                ),
        ) == needle.clone()
        {
            return i;
        }
        i = i + (1 as i64);
    }
    return -(1 as i64);
}
fn _findall_for_finditer(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<String>, RegexError> {
    if flags != (0 as i64) {
        return {
            let __flags_val = flags;
            let mut __flag_str = String::new();
            if (__flags_val & 2) != 0 {
                __flag_str.push_str("(?i)");
            }
            if (__flags_val & 8) != 0 {
                __flag_str.push_str("(?m)");
            }
            if (__flags_val & 16) != 0 {
                __flag_str.push_str("(?s)");
            }
            if (__flags_val & 64) != 0 {
                __flag_str.push_str("(?x)");
            }
            let __pat = __flag_str + &pattern;
            let __re = regex::Regex::new(&__pat)
                .map_err(|e| RegexError {
                    message: e.to_string(),
                    detail: e.to_string(),
                })?;
            Ok(
                __re
                    .find_iter(&text)
                    .map(|m| m.as_str().to_string())
                    .collect::<Vec<String>>(),
            )
        };
    }
    return regex::Regex::new(&pattern)
        .map(|re| {
            re.find_iter(&text).map(|m| m.as_str().to_string()).collect::<Vec<String>>()
        })
        .map_err(|e| RegexError {
            message: e.to_string(),
            detail: e.to_string(),
        });
}
fn _finditer_materialize(
    pattern: &String,
    text: &String,
    flags: i64,
) -> Result<Vec<Match>, RegexError> {
    let __sifr_try_res: Result<Result<Vec<Match>, RegexError>, RegexError> = (|| {
        let found_items: Vec<String> = _findall_for_finditer(pattern, text, flags)?;
        let mut matches: Vec<Match> = vec![];
        let mut cursor: i64 = 0 as i64;
        for found in found_items.iter().cloned() {
            let mut start: i64 = _find_index_from(text, &found, cursor);
            if start < (0 as i64) {
                start = cursor;
            }
            let found_len: i64 = found.chars().count() as i64;
            let end: i64 = start + found_len;
            matches.push(Match::new(found, start, end));
            if found_len == (0 as i64) {
                cursor = end + (1 as i64);
            } else {
                cursor = end;
            }
        }
        return Ok(Ok(matches));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(RegexError::new(e.message));
        }
    }
}
fn finditer(
    pattern: &String,
    text: &String,
) -> Result<Box<dyn Iterator<Item = Match>>, RegexError> {
    let __sifr_try_res: Result<
        Result<Box<dyn Iterator<Item = Match>>, RegexError>,
        RegexError,
    > = (|| {
        let matches: Vec<Match> = _finditer_materialize(pattern, text, 0 as i64)?;
        return Ok(Ok(_iter_matches(matches)));
        unreachable!("sifr try/except return capture fell through");
    })();
    match __sifr_try_res {
        Ok(__sifr_ret_val) => {
            return __sifr_ret_val;
        }
        Err(__sifr_try_err) => {
            let e = __sifr_try_err.clone();
            return Err(RegexError::new(e.message));
        }
    }
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

// --- stdlib: sifr.itertools ---
fn _collect_iterable<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: Vec<T>,
) -> Vec<T> {
    let mut collected: Vec<T> = vec![];
    for item in data.iter().cloned() {
        collected.push(item.clone());
    }
    return collected;
}
fn islice<T: Clone + std::fmt::Display + PartialOrd + 'static>(
    data: &Vec<T>,
    start_or_stop: i64,
    stop: Option<i64>,
    step: i64,
) -> Box<dyn Iterator<Item = T>> {
    let data_owned: Vec<T> = _collect_iterable(
        ((data).iter().cloned().collect::<Vec<_>>()).clone(),
    );
    let mut actual_start: i64 = 0 as i64;
    let mut actual_stop: i64 = start_or_stop;
    if let Some(stop) = stop {
        actual_start = start_or_stop;
        actual_stop = stop;
    }
    if actual_start < (0 as i64) {
        actual_start = 0 as i64;
    }
    if actual_stop < (0 as i64) {
        actual_stop = 0 as i64;
    }
    let mut stride: i64 = step;
    if stride <= (0 as i64) {
        stride = 1 as i64;
        actual_stop = actual_start;
    }
    let mut result: Vec<T> = vec![];
    let mut index: i64 = actual_start;
    while index < actual_stop {
        if index < (data_owned.len() as i64) {
            let value: Option<T> = Some(data_owned[index as usize].clone());
            if let Some(value) = value {
                result.push(value.clone());
            }
        } else {
            index = actual_stop;
        }
        index = index + stride;
    }
    return Box::new((result).iter().cloned());
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

fn adapt_to_iterable(it: Box<dyn Iterator<Item = i64>>) -> Vec<i64> {
    return it.collect::<Vec<_>>();
}

fn collect_starts(it: Box<dyn Iterator<Item = Match>>) -> Vec<i64> {
    let mut starts: Vec<i64> = vec![];
    for m in it {
        starts.push(m.start());
    }
    return starts;
}

fn main() {
    let mut nums: Box<dyn Iterator<Item = i64>> = Box::new((vec![3 as i64, 4 as i64, 5 as i64]).into_iter());
    let via_binding: Vec<i64> = nums.collect::<Vec<_>>();
    println!("{:?}", (via_binding).iter().copied().collect::<Vec<_>>());
    let via_return: Vec<i64> = (adapt_to_iterable(Box::new((vec![7 as i64, 8 as i64]).into_iter()))).into_iter().collect::<Vec<_>>();
    println!("{:?}", (via_return).iter().copied().collect::<Vec<_>>());
    let payload: Vec<u8> = vec![(65 as i64) as u8, (90 as i64) as u8];
    let mut byte_iter: Box<dyn Iterator<Item = i64>> = Box::new((payload).iter().map(|__byte| *__byte as i64));
    let mapped_bytes: Vec<i64> = Box::new(byte_iter.map(|b| b + (1 as i64))).collect::<Vec<_>>();
    println!("{:?}", mapped_bytes);
    let __sifr_try_res: Result<(), RegexError> = (|| {
    let mut matches: Box<dyn Iterator<Item = Match>> = finditer(&"\\d+".to_string(), &"x11y222".to_string())?;
    println!("{:?}", collect_starts(matches));
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", e.message);
    }
    let base: String = format!("{}{}", "/tmp/sifr_iterator_integration".to_string(), format!("{}", std::process::id() as i64));
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _mk: String = ({
    let __cmd = format!("{}{}", format!("{}{}", "mkdir -p ".to_string(), base), "/nested".to_string());
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let _w1: () = std::fs::write(&format!("{}{}", base, "/a.txt".to_string()), "a".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let _w2: () = std::fs::write(&format!("{}{}", base, "/nested/b.txt".to_string()), "b".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    let mut root: Path = Path::new(base);
    let mut entries_it: Box<dyn Iterator<Item = String>> = root.iterdir()?;
    println!("{}", islice(&entries_it.collect::<Vec<_>>(), 2 as i64, None, 1 as i64).collect::<Vec<_>>().len() as i64);
    let mut recursive_it: Box<dyn Iterator<Item = String>> = root.rglob(&"*.txt".to_string())?;
    println!("{}", recursive_it.collect::<Vec<_>>().len() as i64);
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        println!("{}", e.message);
    }
}
