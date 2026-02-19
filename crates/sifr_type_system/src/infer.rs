//! Type inference for Sifr expressions.

use crate::types::Type;

/// Infer the type of a literal value from its string representation and kind.
pub fn infer_literal_type(kind: LiteralKind) -> Type {
    match kind {
        LiteralKind::Int => Type::Int,
        LiteralKind::Float => Type::Float,
        LiteralKind::Str => Type::Str,
        LiteralKind::Bool => Type::Bool,
        LiteralKind::None => Type::None,
    }
}

/// The kind of literal for type inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiteralKind {
    Int,
    Float,
    Str,
    Bool,
    None,
}

/// Resolve a type annotation name (e.g., "int", "str", "Unknown") to a Type.
pub fn resolve_type_annotation(name: &str) -> Option<Type> {
    match name {
        "int" => Some(Type::Int),
        "float" => Some(Type::Float),
        "bool" => Some(Type::Bool),
        "str" => Some(Type::Str),
        "None" => Some(Type::None),
        "Any" => Some(Type::Any),
        "Unknown" => Some(Type::Unknown),
        "Never" => Some(Type::Never),
        "bigint" => Some(Type::BigInt),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_literal_types() {
        assert_eq!(infer_literal_type(LiteralKind::Int), Type::Int);
        assert_eq!(infer_literal_type(LiteralKind::Float), Type::Float);
        assert_eq!(infer_literal_type(LiteralKind::Str), Type::Str);
        assert_eq!(infer_literal_type(LiteralKind::Bool), Type::Bool);
        assert_eq!(infer_literal_type(LiteralKind::None), Type::None);
    }

    #[test]
    fn test_resolve_type_annotations() {
        assert_eq!(resolve_type_annotation("int"), Some(Type::Int));
        assert_eq!(resolve_type_annotation("float"), Some(Type::Float));
        assert_eq!(resolve_type_annotation("str"), Some(Type::Str));
        assert_eq!(resolve_type_annotation("bool"), Some(Type::Bool));
        assert_eq!(resolve_type_annotation("None"), Some(Type::None));
        assert_eq!(resolve_type_annotation("Any"), Some(Type::Any));
        assert_eq!(resolve_type_annotation("Unknown"), Some(Type::Unknown));
        assert_eq!(resolve_type_annotation("Never"), Some(Type::Never));
        assert_eq!(resolve_type_annotation("unknown"), None);
    }
}
