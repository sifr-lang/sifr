//! Sifr Type System
//!
//! Defines the type representations, type inference, type checking,
//! and subtyping rules for the Sifr language.

mod check;
pub mod infer;
pub mod literal;
mod types;
pub mod union;

pub use check::{
    type_check_binary_op, type_check_bool_op, type_check_comparison, type_check_unary_op,
};
pub use infer::infer_literal_type;
pub use types::{
    FunctionType, OwnershipKind, ParamConvention, ParamMutability, ParamOwnership, Type,
};
pub mod narrow;
pub use literal::{widen as widen_literal, LiteralValue};
pub use narrow::{narrow_type, NarrowingCondition};
pub use union::{
    intersect_with_union, make_union, remove_none_from_union, subtract_from_union, union_contains,
    union_contains_none,
};

/// A type error produced during type checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    pub message: String,
    pub kind: TypeErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeErrorKind {
    TypeMismatch {
        expected: Box<Type>,
        actual: Box<Type>,
    },
    UndefinedVariable {
        name: String,
    },
    UndefinedFunction {
        name: String,
    },
    WrongArgumentCount {
        expected: usize,
        actual: usize,
    },
    UseAfterMove {
        name: String,
    },
    MissingTypeAnnotation {
        name: String,
    },
    InvalidOperator {
        op: String,
        ty: Box<Type>,
    },
    NotCallable {
        ty: Box<Type>,
    },
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for TypeError {}
