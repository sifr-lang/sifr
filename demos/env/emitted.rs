// src/main.rs
use ::sifr_runtime::SifrInt;

// --- stdlib: _sifr.sys ---
fn run_command(cmd: &str) -> Result<String, IOError> {
    ::sifr_stdlib::sys::run_command(cmd)
        .map(|__sifr_bridge_ok| __sifr_bridge_ok)
        .map_err(|__sifr_bridge_error| __io_err(__sifr_bridge_error))
}
fn env_get(key: &str) -> Option<String> {
    ::sifr_stdlib::sys::env_get(key)
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
fn sys_exit(code: SifrInt) {
    ::sifr_stdlib::sys::sys_exit(::sifr_runtime::interop::SifrIntBridge::from(code));
}
fn sys_version() -> String {
    ::sifr_stdlib::sys::sys_version()
}
fn sys_platform() -> String {
    ::sifr_stdlib::sys::sys_platform()
}
fn sys_maxsize() -> SifrInt {
    ::sifr_stdlib::sys::sys_maxsize().into_sifr_int()
}
fn getpid() -> SifrInt {
    ::sifr_stdlib::sys::getpid().into_sifr_int()
}
fn cpu_count() -> SifrInt {
    ::sifr_stdlib::sys::cpu_count().into_sifr_int()
}
fn which(name: &str) -> Option<String> {
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
fn getenv_opt(key: &str) -> Option<String> {
    env_get(key)
}
fn getenv(key: &str, default_value: &str) -> String {
    let val: Option<String> = env_get(key);
    let Some(val) = val else {
        return {
            let mut __sifr_concat: String = String::with_capacity(
                default_value.len() + 0usize,
            );
            __sifr_concat.push_str((default_value).as_ref());
            __sifr_concat.push_str("");
            __sifr_concat
        };
    };
    val
}
fn keys() -> Vec<String> {
    env_keys()
}
fn values() -> Vec<String> {
    env_values()
}
fn items() -> Vec<String> {
    env_items()
}

// --- stdlib: sifr.test ---
fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(SifrInt::from(actual.len()), SifrInt::from(expected.len()));
    let mut i: SifrInt = SifrInt::from_i64(0);
    while &i < &SifrInt::from(actual.len()) {
        assert!(
            ({ let __sifr_condition_list = & actual; let __sifr_condition_index = i
            .clone(); let __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() }) == ({ let __sifr_condition_list
            = & expected; let __sifr_condition_index = i.clone(); let
            __sifr_condition_normalized = __sifr_condition_index
            .normalize_index_or_len(__sifr_condition_list.len()); __sifr_condition_list
            .get(__sifr_condition_normalized).copied() })
        );
        i = &i + &SifrInt::from_i64(1);
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
    let with_default: String = getenv(&"SIFR_ENV_SAMPLE_MISSING".to_string(), &"fallback".to_string());
    println!("{}", with_default);
    assert!((format!("{}", with_default) == "fallback"));
    let without_default: Option<String> = getenv_opt(&"SIFR_ENV_SAMPLE_MISSING".to_string());
    assert!((format!("{}", without_default.is_none()) == "true"));
    let invalid_expected_lookup_found: Vec<bool> = vec![false, false];
    let mut invalid_actual_lookup_found: Vec<bool> = vec![];
    invalid_actual_lookup_found.push(getenv_opt(&"".to_string()) != None);
    invalid_actual_lookup_found.push(getenv_opt(&"A=B".to_string()) != None);
    assert_bool_vector_eq(&invalid_actual_lookup_found, &invalid_expected_lookup_found);
    assert!((format!("{}", &SifrInt::from(keys().len()) == &SifrInt::from(values().len())) == "true"));
    assert!((format!("{}", &SifrInt::from(keys().len()) == &SifrInt::from(items().len())) == "true"));
    println!("env read-only access demo: pass");
}
