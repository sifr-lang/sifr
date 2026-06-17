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

// --- stdlib: sifr.os ---
fn stat(path: &String) -> Result<i64, IOError> {
    return std::fs::metadata(&path).map(|m| m.len() as i64).map_err(__io_err);
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

fn collect_runtime_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut shell_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let output: String = ({
    let __cmd = "echo sifr_os_demo".to_string();
    let __output = std::process::Command::new("sh".to_string()).arg("-c".to_string()).arg(&__cmd).output().map_err(__io_err)?;
    Ok(String::from_utf8_lossy(&__output.stdout).trim().to_string())
})?;
    shell_ok = output == "sifr_os_demo".to_string();
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    actual.push(shell_ok);
    return actual;
}

fn collect_filesystem_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let base: String = format!("{}{}", "/tmp/sifr_os_os_demo_".to_string(), format!("{}", std::process::id() as i64));
    let file_path: String = format!("{}{}", base, "/demo.txt".to_string());
    let mut os_flow_ok: bool = false;
    let mut list_ok: bool = false;
    let mut stat_ok: bool = false;
    let mut cleanup_ok: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _mk: () = std::fs::create_dir_all(&base).map(|_| ()).map_err(__io_err)?;
    let _w: () = std::fs::write(&file_path, "demo".to_string().as_bytes()).map(|_| ()).map_err(__io_err)?;
    os_flow_ok = ((std::path::Path::new(&base).is_dir()) && (std::path::Path::new(&file_path).is_file()));
    let entries: Vec<String> = std::fs::read_dir(&base).map(|rd| rd.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().to_string()).collect::<Vec<String>>()).map_err(__io_err)?;
    list_ok = (entries.len() as i64) >= (1 as i64);
    let size: i64 = stat(&file_path)?;
    stat_ok = size > (0 as i64);
    let _rm: () = std::fs::remove_file(&file_path).map(|_| ()).map_err(__io_err)?;
    let _rd: () = std::fs::remove_dir(&base).map(|_| ()).map_err(__io_err)?;
    cleanup_ok = !(std::path::Path::new(&base).is_dir());
    return Ok(());
})();
    if let Err(__sifr_try_err) = __sifr_try_res {
        let e = __sifr_try_err.clone();
        let _: String = format!("{}", e.message);
    }
    actual.push(os_flow_ok);
    actual.push(list_ok);
    actual.push(stat_ok);
    actual.push(cleanup_ok);
    return actual;
}

fn collect_missing_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let mut missing_rejected: bool = false;
    let __sifr_try_res: Result<(), IOError> = (|| {
    let _bad: () = std::fs::remove_dir(&"/tmp/sifr_os_os_demo_missing".to_string()).map(|_| ()).map_err(__io_err)?;
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
    append_all(&mut actual, &collect_runtime_actual());
    append_all(&mut actual, &collect_filesystem_actual());
    append_all(&mut actual, &collect_missing_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("os os parity demo: pass");
}
