/// Built-in error class names that the compiler provides.
pub const BUILTIN_ERROR_CLASSES: &[&str] = &[
    "Error",
    "IOError",
    "ParseError",
    "ValueError",
    "DivisionError",
    "KeyError",
    "JSONDecodeError",
    "JsonIntegerRangeError",
    "JsonLimitError",
    "TOMLDecodeError",
    "RegexError",
    "FileNotFoundError",
    "PermissionError",
    "FileExistsError",
    "IsADirectoryError",
    "NotADirectoryError",
    "DirectoryNotEmptyError",
    "OverflowError",
    "ArithmeticLimitError",
    "FloatOverflowError",
    "FloatPrecisionLossError",
    "IndexError",
    "AttributeError",
    "TypeError",
    "ZeroDivisionError",
    "RuntimeError",
    "NotImplementedError",
    "DecimalConversionError",
    "RustPanicError",
    "TimeoutError",
    "ScopeFailure",
    "TaskCancelled",
    "SecondaryError",
    "GeneratorCloseError",
    "WorkerRuntimeError",
    "WorkerError",
];

/// Resolve a compiler-owned builtin error name to its nominal identity.
#[must_use]
pub fn builtin_error_identity(name: &str) -> Option<String> {
    BUILTIN_ERROR_CLASSES
        .contains(&name)
        .then(|| format!("sifr.builtin.{name}"))
}

/// Test an exact builtin declaration identity without claiming user lookalikes.
#[must_use]
pub fn is_builtin_error_identity(identity: &str) -> bool {
    identity
        .strip_prefix("sifr.builtin.")
        .is_some_and(|name| BUILTIN_ERROR_CLASSES.contains(&name))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builtin_identity_preserves_rust_names_without_claiming_user_errors() {
        for name in BUILTIN_ERROR_CLASSES {
            let identity = builtin_error_identity(name).expect("catalog member");
            assert!(is_builtin_error_identity(&identity));
            assert_eq!(crate::class_rust_name(Some(&identity), name), *name);
            assert!(!is_builtin_error_identity(&format!("user.{name}")));
        }
        assert!(!is_builtin_error_identity("sifr.builtin.NotRegistered"));
        assert!(!is_builtin_error_identity("sifr.builtin.nested.ValueError"));
    }
}
