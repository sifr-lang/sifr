//! Type inference for Sifr expressions.

use crate::types::{FixedIntType, Type};

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
        "int8" => Some(Type::FixedInt(FixedIntType::I8)),
        "int16" => Some(Type::FixedInt(FixedIntType::I16)),
        "int32" => Some(Type::FixedInt(FixedIntType::I32)),
        "int64" => Some(Type::FixedInt(FixedIntType::I64)),
        "uint8" => Some(Type::FixedInt(FixedIntType::U8)),
        "uint16" => Some(Type::FixedInt(FixedIntType::U16)),
        "uint32" => Some(Type::FixedInt(FixedIntType::U32)),
        "uint64" => Some(Type::FixedInt(FixedIntType::U64)),
        "isize" => Some(Type::FixedInt(FixedIntType::ISize)),
        "usize" => Some(Type::FixedInt(FixedIntType::USize)),
        "float" => Some(Type::Float),
        "bool" => Some(Type::Bool),
        "str" => Some(Type::Str),
        "bytes" => Some(Type::Bytes),
        "None" => Some(Type::None),
        "Any" => Some(Type::Any),
        "Iterable" => Some(Type::Iterable(Box::new(Type::Any))),
        "Iterator" => Some(Type::Iterator(Box::new(Type::Any))),
        "Reversible" => Some(Type::reversible(Type::Any)),
        "Unknown" => Some(Type::Unknown),
        "Never" => Some(Type::Never),
        "decimal" => Some(Type::Decimal),
        "bigdecimal" => Some(Type::BigDecimal),
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
        assert_eq!(
            resolve_type_annotation("int8"),
            Some(Type::FixedInt(FixedIntType::I8))
        );
        assert_eq!(
            resolve_type_annotation("int16"),
            Some(Type::FixedInt(FixedIntType::I16))
        );
        assert_eq!(
            resolve_type_annotation("int32"),
            Some(Type::FixedInt(FixedIntType::I32))
        );
        assert_eq!(
            resolve_type_annotation("int64"),
            Some(Type::FixedInt(FixedIntType::I64))
        );
        assert_eq!(
            resolve_type_annotation("uint8"),
            Some(Type::FixedInt(FixedIntType::U8))
        );
        assert_eq!(
            resolve_type_annotation("uint16"),
            Some(Type::FixedInt(FixedIntType::U16))
        );
        assert_eq!(
            resolve_type_annotation("uint32"),
            Some(Type::FixedInt(FixedIntType::U32))
        );
        assert_eq!(
            resolve_type_annotation("uint64"),
            Some(Type::FixedInt(FixedIntType::U64))
        );
        assert_eq!(
            resolve_type_annotation("isize"),
            Some(Type::FixedInt(FixedIntType::ISize))
        );
        assert_eq!(
            resolve_type_annotation("usize"),
            Some(Type::FixedInt(FixedIntType::USize))
        );
        assert_eq!(resolve_type_annotation("float"), Some(Type::Float));
        assert_eq!(resolve_type_annotation("str"), Some(Type::Str));
        assert_eq!(resolve_type_annotation("bytes"), Some(Type::Bytes));
        assert_eq!(resolve_type_annotation("bool"), Some(Type::Bool));
        assert_eq!(resolve_type_annotation("None"), Some(Type::None));
        assert_eq!(resolve_type_annotation("Any"), Some(Type::Any));
        assert_eq!(
            resolve_type_annotation("Iterable"),
            Some(Type::Iterable(Box::new(Type::Any)))
        );
        assert_eq!(
            resolve_type_annotation("Iterator"),
            Some(Type::Iterator(Box::new(Type::Any)))
        );
        assert_eq!(
            resolve_type_annotation("Reversible"),
            Some(Type::reversible(Type::Any))
        );
        assert_eq!(resolve_type_annotation("Unknown"), Some(Type::Unknown));
        assert_eq!(resolve_type_annotation("Never"), Some(Type::Never));
        assert_eq!(resolve_type_annotation("decimal"), Some(Type::Decimal));
        assert_eq!(
            resolve_type_annotation("bigdecimal"),
            Some(Type::BigDecimal)
        );
        assert_eq!(resolve_type_annotation("unknown"), None);
    }
}
