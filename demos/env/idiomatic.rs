fn assert_bool_vector_eq(actual: &[bool], expected: &[bool]) {
    assert_eq!(actual, expected);
}

fn valid_env_key(key: &str) -> bool {
    !key.is_empty() && !key.contains('=') && !key.as_bytes().contains(&0)
}

fn getenv_opt(key: &str) -> Option<String> {
    valid_env_key(key)
        .then(|| std::env::var(key).ok())
        .flatten()
}

fn getenv(key: &str, default: &str) -> String {
    getenv_opt(key).unwrap_or_else(|| default.to_string())
}

fn setenv(key: &str, value: &str) {
    if valid_env_key(key) && !value.as_bytes().contains(&0) {
        std::env::set_var(key, value);
    }
}

fn unsetenv(key: &str) {
    if valid_env_key(key) {
        std::env::remove_var(key);
    }
}

fn main() {
    unsetenv("SIFR_ENV_SAMPLE");
    setenv("SIFR_ENV_SAMPLE", "env");

    let with_default = getenv("SIFR_ENV_SAMPLE", "fallback");
    println!("{with_default}");
    assert_eq!(with_default, "env");

    unsetenv("SIFR_ENV_SAMPLE");
    let without_default = getenv_opt("SIFR_ENV_SAMPLE");
    assert!(without_default.is_none());
    assert_eq!(getenv("SIFR_ENV_SAMPLE", "fallback"), "fallback");

    let expected = [false, false];
    setenv("", "x");
    setenv("A=B", "x");
    let actual = [getenv_opt("").is_some(), getenv_opt("A=B").is_some()];
    assert_bool_vector_eq(&actual, &expected);

    println!("env env parity demo: pass");
}
