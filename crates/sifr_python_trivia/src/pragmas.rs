/// Returns `true` if a comment appears to be a pragma comment.
///
/// ```
/// assert!(sifr_python_trivia::is_pragma_comment("# type: ignore"));
/// assert!(sifr_python_trivia::is_pragma_comment("# noqa: F401"));
/// assert!(sifr_python_trivia::is_pragma_comment("# noqa"));
/// assert!(sifr_python_trivia::is_pragma_comment("# NoQA"));
/// assert!(sifr_python_trivia::is_pragma_comment("# nosec"));
/// assert!(sifr_python_trivia::is_pragma_comment("# nosec B602, B607"));
/// assert!(sifr_python_trivia::is_pragma_comment("# isort: off"));
/// assert!(sifr_python_trivia::is_pragma_comment("# isort: skip"));
/// ```
pub fn is_pragma_comment(comment: &str) -> bool {
    let Some(content) = comment.strip_prefix('#') else {
        return false;
    };
    let trimmed = content.trim_start();

    // Case-insensitive match against `noqa` (which doesn't require a trailing colon).
    matches!(
        trimmed.as_bytes(),
        [b'n' | b'N', b'o' | b'O', b'q' | b'Q', b'a' | b'A', ..]
    ) ||
        // Case-insensitive match against pragmas that don't require a trailing colon.
        trimmed.starts_with("nosec") ||
        // Case-sensitive match against a variety of pragmas that _do_ require a trailing colon.
        trimmed
        .split_once(':')
        .is_some_and(|(maybe_pragma, _)| matches!(maybe_pragma, "isort" | "type" | "pyright" | "pylint" | "flake8" | "ruff"))
}
