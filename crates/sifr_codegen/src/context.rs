//! Shared context and error types for IR lowering.

use sifr_type_system::Type;

#[derive(Debug, Clone, Default)]
pub struct CodegenContext {
    pub pub_mode: bool,
    pub test_mode: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ScopeContext {
    pub function_return_type: Option<Type>,
    pub in_generator_closure: bool,
    pub in_display_impl: bool,
    pub in_loop_with_else: bool,
    pub class_scope: ClassScope,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ClassScope {
    #[default]
    Outside,
    Inside,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodegenError {
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
}

impl CodegenError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
            column: None,
        }
    }

    pub fn with_location(
        message: impl Into<String>,
        line: Option<usize>,
        column: Option<usize>,
    ) -> Self {
        Self {
            message: message.into(),
            line,
            column,
        }
    }

    #[must_use]
    pub fn in_context(self, context: impl AsRef<str>) -> Self {
        Self {
            message: format!("{}: {}", context.as_ref(), self.message),
            line: self.line,
            column: self.column,
        }
    }
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(column)) => write!(f, "{} (at {}:{})", self.message, line, column),
            (Some(line), None) => write!(f, "{} (at line {})", self.message, line),
            _ => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for CodegenError {}

pub type CodegenOutcome<T> = Result<T, CodegenError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codegen_error_formats_with_location() {
        let err = CodegenError::with_location("unsupported node", Some(10), Some(3));
        assert_eq!(err.to_string(), "unsupported node (at 10:3)");
    }
}
