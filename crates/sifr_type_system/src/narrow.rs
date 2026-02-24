//! Type narrowing engine for control-flow-based type refinement.
//!
//! Inspired by TypeScript's checker narrowing and ty's intersection-based narrowing.
//! Given a type and a condition, produces the narrowed type for the then-branch
//! (condition is true) or else-branch (condition is false).

use crate::literal::LiteralValue;
use crate::types::Type;
use crate::union::{intersect_with_union, make_union, remove_none_from_union, subtract_from_union};

/// A condition that can narrow a variable's type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NarrowingCondition {
    /// Truthiness check: `if x:` -- narrows out None and falsy types
    Truthiness(String),
    /// None identity check: `if x is None:`
    IsNone(String),
    /// Not-None identity check: `if x is not None:`
    IsNotNone(String),
    /// isinstance check: `if isinstance(x, int):`
    IsInstance(String, Type),
    /// Equality check: `if x == "GET":`
    Equality(String, LiteralValue),
    /// Type predicate: user-defined guard function returned true
    TypePredicate(String, Type),
    /// Negation: the opposite of a condition
    Not(Box<NarrowingCondition>),
    /// Conjunction: all conditions are true
    And(Vec<NarrowingCondition>),
    /// Disjunction: at least one condition is true
    Or(Vec<NarrowingCondition>),
}

impl NarrowingCondition {
    /// Get the variable name being narrowed, if this is a simple condition.
    pub fn var_name(&self) -> Option<&str> {
        match self {
            NarrowingCondition::Truthiness(name)
            | NarrowingCondition::IsNone(name)
            | NarrowingCondition::IsNotNone(name)
            | NarrowingCondition::IsInstance(name, _)
            | NarrowingCondition::Equality(name, _)
            | NarrowingCondition::TypePredicate(name, _) => Some(name),
            NarrowingCondition::Not(inner) => inner.var_name(),
            NarrowingCondition::And(_) | NarrowingCondition::Or(_) => None,
        }
    }
}

/// Narrow a type based on a condition being true or false.
///
/// - `ty`: the current type of the variable
/// - `condition`: the narrowing condition
/// - `is_true`: whether we're in the then-branch (true) or else-branch (false)
///
/// Returns the narrowed type.
pub fn narrow_type(ty: &Type, condition: &NarrowingCondition, is_true: bool) -> Type {
    match condition {
        NarrowingCondition::Truthiness(_) => {
            if is_true {
                // Truthy: remove None from the type
                narrow_truthiness(ty)
            } else {
                // Falsy: could be None, 0, "", False, empty collections
                // For now, just narrow to None if it's in the union
                narrow_falsiness(ty)
            }
        }
        NarrowingCondition::IsNone(_) => {
            if is_true {
                // x is None -> x: None
                Type::None
            } else {
                // x is not None -> remove None
                remove_none_from_union(ty)
            }
        }
        NarrowingCondition::IsNotNone(_) => {
            if is_true {
                // x is not None -> remove None
                remove_none_from_union(ty)
            } else {
                // x is None
                Type::None
            }
        }
        NarrowingCondition::IsInstance(_, target_type) => {
            if is_true {
                // isinstance(x, int) is true -> narrow to int
                intersect_with_union(ty, target_type)
            } else {
                // isinstance(x, int) is false -> remove int
                subtract_from_union(ty, target_type)
            }
        }
        NarrowingCondition::Equality(_, value) => {
            if is_true {
                // x == "GET" -> narrow to LiteralStr("GET")
                value.to_type()
            } else {
                // x != "GET" -> remove that literal (if applicable)
                subtract_from_union(ty, &value.to_type())
            }
        }
        NarrowingCondition::TypePredicate(_, target_type) => {
            if is_true {
                target_type.clone()
            } else {
                subtract_from_union(ty, target_type)
            }
        }
        NarrowingCondition::Not(inner) => {
            // Negate: swap is_true
            narrow_type(ty, inner, !is_true)
        }
        NarrowingCondition::And(conditions) => {
            if is_true {
                // All conditions are true: apply each narrowing in sequence
                let mut result = ty.clone();
                for cond in conditions {
                    result = narrow_type(&result, cond, true);
                }
                result
            } else {
                // At least one is false: union of each individual false-narrowing
                // This is an approximation; for now, return the original type
                ty.clone()
            }
        }
        NarrowingCondition::Or(conditions) => {
            if is_true {
                // At least one is true: union of each individual true-narrowing
                let narrowed: Vec<Type> = conditions
                    .iter()
                    .map(|c| narrow_type(ty, c, true))
                    .collect();
                make_union(narrowed)
            } else {
                // All are false: apply each false-narrowing in sequence
                let mut result = ty.clone();
                for cond in conditions {
                    result = narrow_type(&result, cond, false);
                }
                result
            }
        }
    }
}

/// Narrow a type for truthiness (remove None and falsy singletons).
fn narrow_truthiness(ty: &Type) -> Type {
    match ty {
        Type::Union(members) => {
            let truthy: Vec<Type> = members
                .iter()
                .filter(|m| !is_always_falsy(m))
                .cloned()
                .collect();
            make_union(truthy)
        }
        other => {
            if is_always_falsy(other) {
                Type::Never
            } else {
                other.clone()
            }
        }
    }
}

/// Narrow a type for falsiness (keep only None and falsy types).
fn narrow_falsiness(ty: &Type) -> Type {
    match ty {
        Type::Union(members) => {
            let falsy: Vec<Type> = members
                .iter()
                .filter(|m| can_be_falsy(m))
                .cloned()
                .collect();
            make_union(falsy)
        }
        other => {
            if can_be_falsy(other) {
                other.clone()
            } else {
                Type::Never
            }
        }
    }
}

/// Check if a type is always falsy.
fn is_always_falsy(ty: &Type) -> bool {
    matches!(
        ty,
        Type::None | Type::LiteralBool(false) | Type::LiteralInt(0)
    ) || matches!(ty, Type::LiteralStr(s) if s.is_empty())
}

/// Check if a type can be falsy (has falsy values in its domain).
fn can_be_falsy(ty: &Type) -> bool {
    matches!(
        ty,
        Type::None
            | Type::Bool
            | Type::Int
            | Type::Float
            | Type::Str
            | Type::LiteralBool(false)
            | Type::LiteralInt(0)
    ) || matches!(ty, Type::LiteralStr(s) if s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_isinstance_narrowing_true() {
        let ty = make_union(vec![Type::Int, Type::Str]);
        let cond = NarrowingCondition::IsInstance("x".to_string(), Type::Int);
        let result = narrow_type(&ty, &cond, true);
        assert_eq!(result, Type::Int);
    }

    #[test]
    fn test_isinstance_narrowing_false() {
        let ty = make_union(vec![Type::Int, Type::Str]);
        let cond = NarrowingCondition::IsInstance("x".to_string(), Type::Int);
        let result = narrow_type(&ty, &cond, false);
        assert_eq!(result, Type::Str);
    }

    #[test]
    fn test_is_none_narrowing() {
        let ty = make_union(vec![Type::Str, Type::None]);
        let cond = NarrowingCondition::IsNone("x".to_string());
        assert_eq!(narrow_type(&ty, &cond, true), Type::None);
        assert_eq!(narrow_type(&ty, &cond, false), Type::Str);
    }

    #[test]
    fn test_is_not_none_narrowing() {
        let ty = make_union(vec![Type::Str, Type::None]);
        let cond = NarrowingCondition::IsNotNone("x".to_string());
        assert_eq!(narrow_type(&ty, &cond, true), Type::Str);
        assert_eq!(narrow_type(&ty, &cond, false), Type::None);
    }

    #[test]
    fn test_truthiness_narrowing() {
        let ty = make_union(vec![Type::Str, Type::None]);
        let cond = NarrowingCondition::Truthiness("x".to_string());
        let result = narrow_type(&ty, &cond, true);
        assert_eq!(result, Type::Str); // None removed
    }

    #[test]
    fn test_equality_narrowing() {
        let ty = Type::Str;
        let cond =
            NarrowingCondition::Equality("x".to_string(), LiteralValue::Str("GET".to_string()));
        let result = narrow_type(&ty, &cond, true);
        assert_eq!(result, Type::LiteralStr("GET".to_string()));
    }

    #[test]
    fn test_equality_narrowing_false() {
        let ty = make_union(vec![
            Type::LiteralStr("GET".to_string()),
            Type::LiteralStr("POST".to_string()),
        ]);
        let cond =
            NarrowingCondition::Equality("x".to_string(), LiteralValue::Str("GET".to_string()));
        let result = narrow_type(&ty, &cond, false);
        assert_eq!(result, Type::LiteralStr("POST".to_string()));
    }

    #[test]
    fn test_type_predicate_narrowing() {
        let ty = make_union(vec![Type::Int, Type::Str]);
        let cond = NarrowingCondition::TypePredicate("x".to_string(), Type::Str);
        assert_eq!(narrow_type(&ty, &cond, true), Type::Str);
        assert_eq!(narrow_type(&ty, &cond, false), Type::Int);
    }

    #[test]
    fn test_not_negation() {
        let ty = make_union(vec![Type::Int, Type::Str]);
        let inner = NarrowingCondition::IsInstance("x".to_string(), Type::Int);
        let cond = NarrowingCondition::Not(Box::new(inner));
        // Not(isinstance(x, int)) true -> isinstance false -> Str
        assert_eq!(narrow_type(&ty, &cond, true), Type::Str);
        // Not(isinstance(x, int)) false -> isinstance true -> Int
        assert_eq!(narrow_type(&ty, &cond, false), Type::Int);
    }

    #[test]
    fn test_isinstance_three_types() {
        let ty = make_union(vec![Type::Bool, Type::Int, Type::Str]);
        let cond = NarrowingCondition::IsInstance("x".to_string(), Type::Int);
        assert_eq!(narrow_type(&ty, &cond, true), Type::Int);
        // False branch: Bool and Str remain
        let false_result = narrow_type(&ty, &cond, false);
        assert_eq!(false_result, make_union(vec![Type::Bool, Type::Str]));
    }

    #[test]
    fn test_narrow_unknown_with_isinstance() {
        let ty = Type::Unknown;
        let cond = NarrowingCondition::IsInstance("x".to_string(), Type::Int);
        // Unknown narrowed with isinstance(x, int) -> Int
        let result = narrow_type(&ty, &cond, true);
        assert_eq!(result, Type::Int);
        // False branch: Unknown minus Int is still Unknown
        let false_result = narrow_type(&ty, &cond, false);
        assert_eq!(false_result, Type::Unknown);
    }
}
