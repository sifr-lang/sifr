use std::fmt::Debug;

pub(crate) trait TestUnwrap<T> {
    fn test_unwrap(self, context: &str) -> T;
}

impl<T, E: Debug> TestUnwrap<T> for Result<T, E> {
    fn test_unwrap(self, context: &str) -> T {
        self.unwrap_or_else(|error| panic!("{context}: {error:?}"))
    }
}

impl<T> TestUnwrap<T> for Option<T> {
    fn test_unwrap(self, context: &str) -> T {
        self.unwrap_or_else(|| panic!("{context}"))
    }
}

pub(crate) trait TestExpectErr<E> {
    fn test_expect_err(self, context: &str) -> E;
}

impl<T: Debug, E> TestExpectErr<E> for Result<T, E> {
    fn test_expect_err(self, context: &str) -> E {
        match self {
            Err(error) => error,
            Ok(value) => panic!("{context}: {value:?}"),
        }
    }
}
