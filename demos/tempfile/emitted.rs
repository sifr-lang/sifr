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

// --- stdlib: sifr.pathlib ---
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

fn collect_tempfile_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let preview_path: String = mktemp_path(&"sifr_tempfile_preview_".to_string());
    let __sifr_try_res: Result<(), IOError> = (|| {
    let file_path: String = mkstemp(&"sifr_tempfile_tmp_".to_string())?;
    let dir_path: String = mkdtemp(&"sifr_tempfile_tmpd_".to_string())?;
    actual.push(std::path::Path::new(&file_path).exists());
    actual.push(std::path::Path::new(&dir_path).exists());
    let preview_name: String = basename(&preview_path);
    let file_name: String = basename(&file_path);
    let dir_name: String = basename(&dir_path);
    let preview_has_prefix: bool = (((preview_name.chars().count() as i64) > ("sifr_tempfile_preview_".to_string().chars().count() as i64)) && (String::from_iter((preview_name).chars().skip((0 as i64).max(0) as usize).take((("sifr_tempfile_preview_".to_string().chars().count() as i64).max(0) - (0 as i64).max(0)).max(0) as usize)) == "sifr_tempfile_preview_".to_string()));
    let file_has_prefix: bool = (((file_name.chars().count() as i64) > ("sifr_tempfile_tmp_".to_string().chars().count() as i64)) && (String::from_iter((file_name).chars().skip((0 as i64).max(0) as usize).take((("sifr_tempfile_tmp_".to_string().chars().count() as i64).max(0) - (0 as i64).max(0)).max(0) as usize)) == "sifr_tempfile_tmp_".to_string()));
    let dir_has_prefix: bool = (((dir_name.chars().count() as i64) > ("sifr_tempfile_tmpd_".to_string().chars().count() as i64)) && (String::from_iter((dir_name).chars().skip((0 as i64).max(0) as usize).take((("sifr_tempfile_tmpd_".to_string().chars().count() as i64).max(0) - (0 as i64).max(0)).max(0) as usize)) == "sifr_tempfile_tmpd_".to_string()));
    actual.push((preview_has_prefix && file_has_prefix) && dir_has_prefix);
    let temp_root: String = dirname(&preview_path);
    let missing_parent_name: String = "__sifr_tempfile_missing_parent__".to_string();
    let missing_parent_path: String = format!("{}{}{}", temp_root, "/".to_string(), missing_parent_name);
    let missing_prefix: String = format!("{}{}", missing_parent_name, "/bad_".to_string());
    let _rm_missing: String = ({
    let __cmd = format!("{}{}", "rm -rf ".to_string(), missing_parent_path);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let mut missing_error: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let unexpected_file: String = mkstemp(&missing_prefix)?;
    let _rm_unexpected: String = ({
    let __cmd = format!("{}{}", "rm -f ".to_string(), unexpected_file);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    missing_error = false;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        missing_error = true;
    }
    actual.push(missing_error);
    let _c1: String = ({
    let __cmd = format!("{}{}", "rm -f ".to_string(), file_path);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let _c2: String = ({
    let __cmd = format!("{}{}", "rm -rf ".to_string(), dir_path);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let _c3: String = ({
    let __cmd = format!("{}{}", "rm -rf ".to_string(), missing_parent_path);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    let cleaned: bool = ((!(std::path::Path::new(&file_path).exists())) && (!(std::path::Path::new(&dir_path).exists())));
    actual.push(cleaned);
    let next_path: String = mkstemp(&"sifr_tempfile_tmp_".to_string())?;
    actual.push(next_path != file_path);
    let _c4: String = ({
    let __cmd = format!("{}{}", "rm -f ".to_string(), next_path);
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
        actual = vec![false, false, false, false, false, false];
    }
    return actual;
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true];
    let actual: Vec<bool> = collect_tempfile_actual();
    assert_bool_vector_eq(&actual, &expected);
    println!("tempfile tempfile parity demo: pass");
}
