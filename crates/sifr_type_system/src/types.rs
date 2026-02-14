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
    /// List type (`list[T]` in Sifr, `Vec<T>` in Rust)
    List(Box<Type>),
    /// Dictionary type (`dict[K, V]` in Sifr, `HashMap<K, V>` in Rust)
    Dict(Box<Type>, Box<Type>),
    /// Tuple type (`tuple[A, B, ...]` in Sifr, `(A, B, ...)` in Rust)
    Tuple(Vec<Type>),
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
            Self::Str | Self::Any | Self::List(_) | Self::Dict(_, _) | Self::Tuple(_) => OwnershipKind::Move,
        }
    }

    /// Returns the Sifr source name for this type.
    pub fn display_name(&self) -> String {
        match self {
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Str => "str".to_string(),
            Self::None => "None".to_string(),
            Self::Function(_) => "function".to_string(),
            Self::List(elem) => format!("list[{}]", elem.display_name()),
            Self::Dict(key, val) => format!("dict[{}, {}]", key.display_name(), val.display_name()),
            Self::Tuple(elems) => {
                let parts: Vec<String> = elems.iter().map(Self::display_name).collect();
                format!("tuple[{}]", parts.join(", "))
            }
            Self::Range => "range".to_string(),
            Self::Any => "Any".to_string(),
            Self::Never => "Never".to_string(),
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
            Self::List(elem) => format!("Vec<{}>", elem.rust_type()),
            Self::Dict(key, val) => format!("std::collections::HashMap<{}, {}>", key.rust_type(), val.rust_type()),
            Self::Tuple(elems) => {
                let parts: Vec<String> = elems.iter().map(Self::rust_type).collect();
                format!("({})", parts.join(", "))
            }
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
            Self::List(elem) => Some(*elem.clone()),
            _ => None,
        }
    }

    /// Returns the result type of indexing this type with the given index type.
    pub fn index_result_type(&self, index_ty: &Type) -> Option<Type> {
        match self {
            Self::List(elem) => {
                if index_ty == &Type::Int {
                    Some(*elem.clone())
                } else {
                    None
                }
            }
            Self::Dict(key, val) => {
                if index_ty == key.as_ref() {
                    Some(*val.clone())
                } else {
                    None
                }
            }
            Self::Tuple(elems) => {
                // Tuple indexing requires a literal int, but at type level we just return Any
                // The actual positional type is resolved during lowering
                if index_ty == &Type::Int && !elems.is_empty() {
                    Some(elems[0].clone()) // Placeholder; real resolution happens in lowering
                } else {
                    None
                }
            }
            Self::Str => {
                if index_ty == &Type::Int {
                    Some(Type::Str) // Single char as string
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Returns the result type of the `in` operator for this collection type.
    pub fn contains_element_type(&self) -> Option<Type> {
        match self {
            Self::List(elem) => Some(*elem.clone()),
            Self::Dict(key, _) => Some(*key.clone()),
            Self::Str => Some(Type::Str),
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
        // Structural subtyping for collections
        match (self, target) {
            (Self::List(a), Self::List(b)) => a.is_assignable_to(b),
            (Self::Dict(ak, av), Self::Dict(bk, bv)) => ak.is_assignable_to(bk) && av.is_assignable_to(bv),
            (Self::Tuple(a), Self::Tuple(b)) => {
                a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.is_assignable_to(y))
            }
            _ => false,
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.display_name())
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

    #[test]
    fn test_list_type() {
        let list_int = Type::List(Box::new(Type::Int));
        assert_eq!(list_int.ownership(), OwnershipKind::Move);
        assert_eq!(list_int.display_name(), "list[int]");
        assert_eq!(list_int.rust_type(), "Vec<i64>");
        assert_eq!(list_int.iterable_element_type(), Some(Type::Int));
    }

    #[test]
    fn test_dict_type() {
        let dict_str_int = Type::Dict(Box::new(Type::Str), Box::new(Type::Int));
        assert_eq!(dict_str_int.ownership(), OwnershipKind::Move);
        assert_eq!(dict_str_int.display_name(), "dict[str, int]");
        assert_eq!(dict_str_int.rust_type(), "std::collections::HashMap<String, i64>");
    }

    #[test]
    fn test_tuple_type() {
        let tuple = Type::Tuple(vec![Type::Int, Type::Str]);
        assert_eq!(tuple.ownership(), OwnershipKind::Move);
        assert_eq!(tuple.display_name(), "tuple[int, str]");
        assert_eq!(tuple.rust_type(), "(i64, String)");
    }

    #[test]
    fn test_collection_assignability() {
        let list_int = Type::List(Box::new(Type::Int));
        let list_int2 = Type::List(Box::new(Type::Int));
        let list_str = Type::List(Box::new(Type::Str));
        assert!(list_int.is_assignable_to(&list_int2));
        assert!(!list_int.is_assignable_to(&list_str));
    }

    #[test]
    fn test_index_result_type() {
        let list_int = Type::List(Box::new(Type::Int));
        assert_eq!(list_int.index_result_type(&Type::Int), Some(Type::Int));
        assert_eq!(list_int.index_result_type(&Type::Str), None);
    }
}
