//! Literal type handling and widening.
//!
//! Literal types represent specific values used as types:
//! - `LiteralInt(42)` -- the type of exactly the value 42
//! - `LiteralStr("GET")` -- the type of exactly the string "GET"
//! - `LiteralBool(true)` -- the type of exactly the value true
//!
//! Literal types widen to their base type at mutable assignment
//! (like TypeScript's fresh literal behavior).

use crate::types::Type;

/// Widen a literal type to its base type.
///
/// - `LiteralInt(42)` -> `Int`
/// - `LiteralStr("GET")` -> `Str`
/// - `LiteralBool(true)` -> `Bool`
/// - Non-literal types are returned unchanged.
pub fn widen(ty: &Type) -> Type {
    match ty {
        Type::LiteralInt(_) => Type::Int,
        Type::LiteralStr(_) => Type::Str,
        Type::LiteralBool(_) => Type::Bool,
        Type::Union(members) => {
            let widened: Vec<Type> = members.iter().map(widen).collect();
            crate::union::make_union(widened)
        }
        other => other.clone(),
    }
}

/// Get the base type of a literal type (or the type itself if not a literal).
pub fn base_type(ty: &Type) -> Type {
    match ty {
        Type::LiteralInt(_) => Type::Int,
        Type::LiteralStr(_) => Type::Str,
        Type::LiteralBool(_) => Type::Bool,
        other => other.clone(),
    }
}

/// Check if a type is a literal type.
pub fn is_literal(ty: &Type) -> bool {
    matches!(
        ty,
        Type::LiteralInt(_) | Type::LiteralStr(_) | Type::LiteralBool(_)
    )
}

/// Check if a literal value matches a literal type.
pub fn literal_matches(ty: &Type, value: &LiteralValue) -> bool {
    match (ty, value) {
        (Type::LiteralInt(expected), LiteralValue::Int(actual)) => expected == actual,
        (Type::LiteralStr(expected), LiteralValue::Str(actual)) => expected == actual,
        (Type::LiteralBool(expected), LiteralValue::Bool(actual)) => expected == actual,
        // Base types accept any literal of that kind
        (Type::Int, LiteralValue::Int(_)) => true,
        (Type::Str, LiteralValue::Str(_)) => true,
        (Type::Bool, LiteralValue::Bool(_)) => true,
        _ => false,
    }
}

/// A concrete literal value (used in narrowing conditions).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LiteralValue {
    Int(i64),
    Str(String),
    Bool(bool),
}

impl LiteralValue {
    /// Convert a literal value to its corresponding literal type.
    pub fn to_type(&self) -> Type {
        match self {
            LiteralValue::Int(v) => Type::LiteralInt(*v),
            LiteralValue::Str(v) => Type::LiteralStr(v.clone()),
            LiteralValue::Bool(v) => Type::LiteralBool(*v),
        }
    }
}

impl std::fmt::Display for LiteralValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LiteralValue::Int(v) => write!(f, "{v}"),
            LiteralValue::Str(v) => write!(f, "\"{v}\""),
            LiteralValue::Bool(v) => write!(f, "{v}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widen_literal_int() {
        assert_eq!(widen(&Type::LiteralInt(42)), Type::Int);
    }

    #[test]
    fn test_widen_literal_str() {
        assert_eq!(widen(&Type::LiteralStr("GET".to_string())), Type::Str);
    }

    #[test]
    fn test_widen_literal_bool() {
        assert_eq!(widen(&Type::LiteralBool(true)), Type::Bool);
    }

    #[test]
    fn test_widen_non_literal() {
        assert_eq!(widen(&Type::Int), Type::Int);
        assert_eq!(widen(&Type::Str), Type::Str);
    }

    #[test]
    fn test_base_type() {
        assert_eq!(base_type(&Type::LiteralInt(42)), Type::Int);
        assert_eq!(base_type(&Type::Int), Type::Int);
    }

    #[test]
    fn test_is_literal() {
        assert!(is_literal(&Type::LiteralInt(42)));
        assert!(is_literal(&Type::LiteralStr("GET".to_string())));
        assert!(is_literal(&Type::LiteralBool(true)));
        assert!(!is_literal(&Type::Int));
        assert!(!is_literal(&Type::Str));
    }

    #[test]
    fn test_literal_matches() {
        assert!(literal_matches(
            &Type::LiteralInt(42),
            &LiteralValue::Int(42)
        ));
        assert!(!literal_matches(
            &Type::LiteralInt(42),
            &LiteralValue::Int(99)
        ));
        assert!(literal_matches(&Type::Int, &LiteralValue::Int(42)));
    }

    #[test]
    fn test_literal_value_to_type() {
        assert_eq!(LiteralValue::Int(42).to_type(), Type::LiteralInt(42));
        assert_eq!(
            LiteralValue::Str("GET".to_string()).to_type(),
            Type::LiteralStr("GET".to_string())
        );
        assert_eq!(LiteralValue::Bool(true).to_type(), Type::LiteralBool(true));
    }

    #[test]
    fn test_widen_union_of_literals() {
        let u = Type::Union(vec![
            Type::LiteralStr("GET".to_string()),
            Type::LiteralStr("POST".to_string()),
        ]);
        // Widening a union of literals should produce the base type
        assert_eq!(widen(&u), Type::Str);
    }
}
