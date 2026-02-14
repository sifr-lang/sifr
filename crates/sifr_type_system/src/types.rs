//! Core type definitions for the Sifr type system.

/// Represents a type in the Sifr language.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// 64-bit integer (`int` in Sifr, `i64` in Rust)
    Int,
    /// 64-bit float (`float` in Sifr, `f64` in Rust)
    Float,
    /// Boolean (`bool`)
    Bool,
    /// String (`str` in Sifr, `String` in Rust)
    Str,
    /// None type (unit type `()` in Rust)
    None,
    /// Function type with parameter types and return type
    Function(FunctionType),
    /// Range type (maps to `std::ops::Range<i64>` in Rust)
    Range,
    /// Explicit opt-out of type checking
    Any,
    /// Bottom type (function that never returns)
    Never,
}

/// Represents a function's type signature.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionType {
    /// Parameter names and types
    pub params: Vec<(String, Type)>,
    /// Return type
    pub return_type: Box<Type>,
}

/// Describes how a type behaves with respect to ownership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OwnershipKind {
    /// Value is copied on assignment (primitives: int, float, bool)
    Copy,
    /// Value is moved on assignment (str, compound types, classes)
    Move,
}

impl Type {
    /// Returns the ownership kind for this type.
    ///
    /// - Primitives (`Int`, `Float`, `Bool`) are `Copy`.
    /// - `Str` and compound types are `Move`.
    /// - `None` is `Copy` (it's a zero-sized type).
    /// - `Any` is `Move` (conservative).
    /// - `Never` is `Copy` (unreachable).
    /// - `Function` is `Copy` (function pointers).
    pub fn ownership(&self) -> OwnershipKind {
        match self {
            Self::Int | Self::Float | Self::Bool | Self::None | Self::Never | Self::Range => OwnershipKind::Copy,
            Self::Function(_) => OwnershipKind::Copy,
            Self::Str | Self::Any => OwnershipKind::Move,
        }
    }

    /// Returns the Sifr source name for this type.
    pub fn display_name(&self) -> &str {
        match self {
            Self::Int => "int",
            Self::Float => "float",
            Self::Bool => "bool",
            Self::Str => "str",
            Self::None => "None",
            Self::Function(_) => "function",
            Self::Range => "range",
            Self::Any => "Any",
            Self::Never => "Never",
        }
    }

    /// Returns the Rust type name for code generation.
    pub fn rust_type(&self) -> String {
        match self {
            Self::Int => "i64".to_string(),
            Self::Float => "f64".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Str => "String".to_string(),
            Self::None => "()".to_string(),
            Self::Range => "std::ops::Range<i64>".to_string(),
            Self::Any => "Box<dyn std::any::Any>".to_string(),
            Self::Never => "!".to_string(),
            Self::Function(ft) => {
                let params: Vec<String> = ft.params.iter().map(|(_, t)| t.rust_type()).collect();
                let ret = ft.return_type.rust_type();
                format!("fn({}) -> {}", params.join(", "), ret)
            }
        }
    }

    /// Check if this type is a numeric type (int or float).
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::Int | Self::Float)
    }

    /// Returns the element type if this type is iterable, or None otherwise.
    pub fn iterable_element_type(&self) -> Option<Type> {
        match self {
            Self::Range => Some(Type::Int),
            _ => None,
        }
    }

    /// Check if a value of type `self` can be assigned to a target of type `target`.
    pub fn is_assignable_to(&self, target: &Type) -> bool {
        if self == target {
            return true;
        }
        // Any is compatible with everything
        if matches!(self, Self::Any) || matches!(target, Self::Any) {
            return true;
        }
        // Never is assignable to everything
        if matches!(self, Self::Never) {
            return true;
        }
        false
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ownership_primitives_are_copy() {
        assert_eq!(Type::Int.ownership(), OwnershipKind::Copy);
        assert_eq!(Type::Float.ownership(), OwnershipKind::Copy);
        assert_eq!(Type::Bool.ownership(), OwnershipKind::Copy);
        assert_eq!(Type::None.ownership(), OwnershipKind::Copy);
    }

    #[test]
    fn test_ownership_str_is_move() {
        assert_eq!(Type::Str.ownership(), OwnershipKind::Move);
    }

    #[test]
    fn test_rust_type_mapping() {
        assert_eq!(Type::Int.rust_type(), "i64");
        assert_eq!(Type::Float.rust_type(), "f64");
        assert_eq!(Type::Bool.rust_type(), "bool");
        assert_eq!(Type::Str.rust_type(), "String");
        assert_eq!(Type::None.rust_type(), "()");
    }

    #[test]
    fn test_assignability() {
        assert!(Type::Int.is_assignable_to(&Type::Int));
        assert!(!Type::Int.is_assignable_to(&Type::Str));
        assert!(Type::Any.is_assignable_to(&Type::Int));
        assert!(Type::Int.is_assignable_to(&Type::Any));
        assert!(Type::Never.is_assignable_to(&Type::Int));
    }
}
