/// Built-in error class names that the compiler provides.
pub(crate) const BUILTIN_ERROR_CLASSES: &[&str] = &[
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

/// A catalog-validated builtin name. Its canonical identity is total.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct BuiltinError {
    name: &'static str,
}

impl BuiltinError {
    pub(crate) fn all() -> impl Iterator<Item = Self> {
        BUILTIN_ERROR_CLASSES.iter().map(|name| Self { name })
    }

    pub(crate) fn from_name(name: &str) -> Option<Self> {
        Self::all().find(|builtin| builtin.name == name)
    }

    pub(crate) const fn name(self) -> &'static str {
        self.name
    }

    pub(crate) fn identity(self) -> String {
        format!("sifr.builtin.{}", self.name)
    }
}

pub(crate) fn builtin_error_identity(name: &str) -> Option<String> {
    BuiltinError::from_name(name).map(BuiltinError::identity)
}
