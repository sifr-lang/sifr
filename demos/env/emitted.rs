// src/main.rs
// --- stdlib: _sifr.sys ---
fn run_command(cmd: &String) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn env_get(key: &String) -> Option<String> {
    ::sifr_stdlib::sys::env_get(key)
}
fn env_set(key: &String, value: &String) {
    ::sifr_stdlib::sys::env_set(key, value);
}
fn env_unset(key: &String) {
    ::sifr_stdlib::sys::env_unset(key);
}
fn env_keys() -> Vec<String> {
    ::sifr_stdlib::sys::env_keys()
}
fn env_values() -> Vec<String> {
    ::sifr_stdlib::sys::env_values()
}
fn env_items() -> Vec<String> {
    ::sifr_stdlib::sys::env_items()
}
fn get_args() -> Vec<String> {
    ::sifr_stdlib::sys::get_args()
}
fn sys_exit(code: i64) {
    ::sifr_stdlib::sys::sys_exit(::sifr_runtime::interop::SifrIntBridge::from(code));
}
fn sys_version() -> String {
    ::sifr_stdlib::sys::sys_version()
}
fn sys_platform() -> String {
    ::sifr_stdlib::sys::sys_platform()
}
fn sys_maxsize() -> i64 {
    ::sifr_stdlib::sys::sys_maxsize().to_i64_saturating()
}
fn getpid() -> i64 {
    ::sifr_stdlib::sys::getpid().to_i64_saturating()
}
fn cpu_count() -> i64 {
    ::sifr_stdlib::sys::cpu_count().to_i64_saturating()
}
fn which(name: &String) -> Option<String> {
    ::sifr_stdlib::sys::which(name)
}
fn os_sep() -> String {
    ::sifr_stdlib::sys::os_sep()
}
fn os_linesep() -> String {
    ::sifr_stdlib::sys::os_linesep()
}
fn os_name() -> String {
    ::sifr_stdlib::sys::os_name()
}

// --- stdlib: sifr.env ---
fn getenv_opt(key: &String) -> Option<String> {
    env_get(key)
}
fn getenv(key: &String, default_value: &String) -> String {
    let val: Option<String> = env_get(key);
    let Some(val) = val else {
        return {
            let mut __sifr_concat: String = String::with_capacity(
                default_value.len() + 0usize,
            );
            __sifr_concat.push_str((default_value).as_str());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    };
    val
}
fn setenv(key: &String, value: &String) {
    env_set(key, value);
}
fn unsetenv(key: &String) {
    env_unset(key);
}

// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &Vec<bool>, expected: &Vec<bool>) {
    assert_eq!(actual.len() as i64, expected.len() as i64);
    let mut i: i64 = 0_i64;
    while i < (actual.len() as i64) {
        assert!(Some(actual[i as usize]) == expected.get(i as usize).copied());
        i += 1_i64;
    }
}
// --- end stdlib ---

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct IOError {
    message: String,
    kind: String,
}

impl IOError {
    fn new(message: String) -> Self {
        Self { message, kind: "Other".to_string() }
    }
}

impl ::std::fmt::Display for IOError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        ::std::fmt::Display::fmt(&self.message, f)
    }
}

impl ::std::error::Error for IOError {
}

fn __io_err<E: ::std::fmt::Display + 'static>(e: E) -> IOError {
    let msg = e.to_string();
    let kind = {
    let __sifr_io_kind = (&e as &dyn ::std::any::Any).downcast_ref::<std::io::Error>().map(::std::io::Error::kind);
    match __sifr_io_kind {
    Some(::std::io::ErrorKind::NotFound) => {
        "FileNotFound".to_string()
    },
    Some(::std::io::ErrorKind::PermissionDenied) => {
        "PermissionDenied".to_string()
    },
    Some(::std::io::ErrorKind::AlreadyExists) => {
        "FileExists".to_string()
    },
    Some(::std::io::ErrorKind::IsADirectory) => {
        "IsADirectory".to_string()
    },
    Some(::std::io::ErrorKind::NotADirectory) => {
        "NotADirectory".to_string()
    },
    Some(::std::io::ErrorKind::DirectoryNotEmpty) => {
        "DirectoryNotEmpty".to_string()
    },
    _ => {
        "Other".to_string()
    },
}
};
    IOError { message: msg, kind }
}

fn main() {
    unsetenv(&"SIFR_ENV_SAMPLE".to_string());
    setenv(&"SIFR_ENV_SAMPLE".to_string(), &"env".to_string());
    let with_default: String = getenv(&"SIFR_ENV_SAMPLE".to_string(), &"fallback".to_string());
    println!("{}", with_default);
    assert!((format!("{}", with_default) == "env"));
    unsetenv(&"SIFR_ENV_SAMPLE".to_string());
    let without_default: Option<String> = getenv_opt(&"SIFR_ENV_SAMPLE".to_string());
    assert!((format!("{}", without_default.is_none()) == "true"));
    assert!((format!("{}", getenv(&"SIFR_ENV_SAMPLE".to_string(), &"fallback".to_string())) == "fallback"));
    let invalid_expected_lookup_found: Vec<bool> = vec![false, false];
    let mut invalid_actual_lookup_found: Vec<bool> = vec![];
    setenv(&"".to_string(), &"x".to_string());
    invalid_actual_lookup_found.push(getenv_opt(&"".to_string()) != None);
    setenv(&"A=B".to_string(), &"x".to_string());
    invalid_actual_lookup_found.push(getenv_opt(&"A=B".to_string()) != None);
    assert_bool_vector_eq(&invalid_actual_lookup_found, &invalid_expected_lookup_found);
    println!("env env parity demo: pass");
}
