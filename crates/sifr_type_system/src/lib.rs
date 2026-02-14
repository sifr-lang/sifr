//! Sifr Type System
//!
//! Defines the type representations, type inference, type checking,
//! and subtyping rules for the Sifr language.

mod types;
mod check;
pub mod infer;
pub mod union;
pub mod literal;

pub use types::{Type, FunctionType, OwnershipKind};
pub use check::{type_check_binary_op, type_check_unary_op, type_check_comparison, type_check_bool_op};
pub use infer::infer_literal_type;
pub use union::{make_union, subtract_from_union, intersect_with_union, remove_none_from_union, union_contains, union_contains_none};
pub use literal::{LiteralValue, widen as widen_literal};

/// A type error produced during type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub message: String,
    pub kind: TypeErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeErrorKind {
    TypeMismatch { expected: Type, actual: Type },
    UndefinedVariable { name: String },
    UndefinedFunction { name: String },
    WrongArgumentCount { expected: usize, actual: usize },
    UseAfterMove { name: String },
    MissingTypeAnnotation { name: String },
    InvalidOperator { op: String, ty: Type },
    NotCallable { ty: Type },
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeError {}
