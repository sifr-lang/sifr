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

// --- stdlib: sifr.platform ---
fn system() -> String {
    return if cfg!(target_os = "windows") {
        "Windows".to_string().to_string()
    } else {
        if cfg!(target_os = "macos") {
            "Darwin".to_string().to_string()
        } else {
            if cfg!(target_os = "linux") {
                "Linux".to_string().to_string()
            } else {
                std::env::consts::OS.to_string()
            }
        }
    };
}
fn machine() -> String {
    return std::env::consts::ARCH.to_string();
}
fn node() -> String {
    return {
        std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("COMPUTERNAME"))
            .unwrap_or_else(|_| "localhost".to_string())
    };
}
fn release() -> String {
    return {
        std::process::Command::new("uname")
            .arg("-r")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| std::env::consts::OS.to_string())
    };
}
fn version() -> String {
    return {
        std::process::Command::new("uname")
            .arg("-v")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| std::env::consts::OS.to_string())
    };
}
fn processor() -> String {
    return std::env::consts::ARCH.to_string();
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

fn collect_core_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    let sys_name: String = system();
    actual.push(((((sys_name.chars().count() as i64) > (0 as i64)) && (sys_name != "linux".to_string())) && (sys_name != "macos".to_string())) && (sys_name != "windows".to_string()));
    actual.push((machine().chars().count() as i64) > (0 as i64));
    actual.push((processor().chars().count() as i64) > (0 as i64));
    return actual;
}

fn collect_host_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((node().chars().count() as i64) > (0 as i64));
    actual.push((release().chars().count() as i64) > (0 as i64));
    actual.push((version().chars().count() as i64) > (0 as i64));
    return actual;
}

fn collect_alias_actual() -> Vec<bool> {
    let mut actual: Vec<bool> = vec![];
    actual.push((((if cfg!(target_os = "windows") { "Windows".to_string().to_string() } else { if cfg!(target_os = "macos") { "Darwin".to_string().to_string() } else { if cfg!(target_os = "linux") { "Linux".to_string().to_string() } else { std::env::consts::OS.to_string() } } }).as_str() == (system()).as_str()) && ((std::env::consts::ARCH.to_string()).as_str() == (machine()).as_str())) && (({ std::env::var("HOSTNAME").or_else(|_| std::env::var("COMPUTERNAME")).unwrap_or_else(|_| "localhost".to_string()) }).as_str() == (node()).as_str()));
    return actual;
}

fn append_all(target: &mut Vec<bool>, values: &Vec<bool>) {
    for value in values.iter().copied() {
        target.push(value);
    }
}

fn main() {
    let expected: Vec<bool> = vec![true, true, true, true, true, true, true];
    let mut actual: Vec<bool> = vec![];
    append_all(&mut actual, &collect_core_actual());
    append_all(&mut actual, &collect_host_actual());
    append_all(&mut actual, &collect_alias_actual());
    assert_bool_vector_eq(&actual, &expected);
    println!("platform platform parity demo: pass");
}
