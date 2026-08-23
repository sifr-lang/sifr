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

fn main() {
    let with_default = getenv("SIFR_ENV_SAMPLE_MISSING", "fallback");
    println!("{with_default}");
    assert_eq!(with_default, "fallback");

    let without_default = getenv_opt("SIFR_ENV_SAMPLE_MISSING");
    assert!(without_default.is_none());

    let expected = [false, false];
    let actual = [getenv_opt("").is_some(), getenv_opt("A=B").is_some()];
    assert_bool_vector_eq(&actual, &expected);

    let names = std::env::vars_os().map(|(key, _)| key).collect::<Vec<_>>();
    let values = std::env::vars_os().map(|(_, value)| value).collect::<Vec<_>>();
    let items = std::env::vars_os().collect::<Vec<_>>();
    assert_eq!(names.len(), values.len());
    assert_eq!(names.len(), items.len());

    println!("env read-only access demo: pass");
}
