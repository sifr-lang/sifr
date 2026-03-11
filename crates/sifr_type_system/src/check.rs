//! Type checking rules for Sifr operators and expressions.

use crate::types::Type;
use crate::union::{remove_none_from_union, union_contains_none};
use crate::TypeError;

fn is_decimal_type(ty: &Type) -> bool {
    matches!(ty, Type::Decimal)
}

fn is_bigdecimal_type(ty: &Type) -> bool {
    matches!(ty, Type::BigDecimal)
}

fn is_decimal_family_type(ty: &Type) -> bool {
    is_decimal_type(ty) || is_bigdecimal_type(ty)
}

fn is_integral_numeric_type(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::LiteralInt(_) | Type::BigInt)
}

/// Type-check a binary operation (e.g., `a + b`, `a - b`).
///
/// Returns the result type or an error.
pub fn type_check_binary_op(left: &Type, op: &str, right: &Type) -> Result<Type, TypeError> {
    if (is_decimal_type(left) && is_bigdecimal_type(right))
        || (is_bigdecimal_type(left) && is_decimal_type(right))
    {
        return Err(TypeError {
            message: "[E2504] cannot mix 'decimal' and 'bigdecimal' in arithmetic; use explicit Decimal(...) or BigDecimal(...) conversion".to_string(),
            kind: crate::TypeErrorKind::InvalidOperator {
                op: op.to_string(),
                ty: Box::new(left.clone()),
            },
        });
    }

    if (left == &Type::Float && is_decimal_family_type(right))
        || (right == &Type::Float && is_decimal_family_type(left))
    {
        return Err(TypeError {
            message: "[E2503] cannot mix 'float' with decimal numeric types in arithmetic"
                .to_string(),
            kind: crate::TypeErrorKind::InvalidOperator {
                op: op.to_string(),
                ty: Box::new(left.clone()),
            },
        });
    }

    // Mixed int/bigint arithmetic is a compile error (except bigint ** int which is allowed)
    let is_bigint_pow_int = left == &Type::BigInt && right == &Type::Int && op == "**";
    if !is_bigint_pow_int {
        if (left == &Type::Int && right == &Type::BigInt)
            || (left == &Type::BigInt && right == &Type::Int)
        {
            return Err(TypeError {
                message: "cannot mix 'int' and 'bigint' in arithmetic; use bigint() or int() to convert explicitly".to_string(),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            });
        }
    }

    // TypeVar arithmetic: T op T -> T (generic code assumes the concrete type supports the operation)
    if matches!(left, Type::TypeVar(_)) && matches!(right, Type::TypeVar(_)) {
        return Ok(left.clone());
    }
    if matches!(left, Type::TypeVar(_)) {
        return Ok(left.clone());
    }
    if matches!(right, Type::TypeVar(_)) {
        return Ok(right.clone());
    }

    match op {
        "+" => {
            // Decimal-family arithmetic
            if is_decimal_type(left) && is_decimal_type(right) {
                return Ok(Type::Decimal);
            }
            if is_bigdecimal_type(left) && is_bigdecimal_type(right) {
                return Ok(Type::BigDecimal);
            }
            if (is_decimal_type(left) && is_integral_numeric_type(right))
                || (is_decimal_type(right) && is_integral_numeric_type(left))
            {
                return Ok(Type::Decimal);
            }
            if (is_bigdecimal_type(left) && is_integral_numeric_type(right))
                || (is_bigdecimal_type(right) && is_integral_numeric_type(left))
            {
                return Ok(Type::BigDecimal);
            }
            // BigInt arithmetic
            if left == &Type::BigInt && right == &Type::BigInt {
                return Ok(Type::BigInt);
            }
            // Numeric addition
            if left == &Type::Int && right == &Type::Int {
                return Ok(Type::Int);
            }
            if left.is_numeric() && right.is_numeric() {
                return Ok(Type::Float);
            }
            // String concatenation
            if left == &Type::Str && right == &Type::Str {
                return Ok(Type::Str);
            }
            // List concatenation: list[T] + list[T] -> list[T]
            if let (Type::List(l_elem), Type::List(r_elem)) = (left, right) {
                if l_elem == r_elem {
                    return Ok(Type::List(l_elem.clone()));
                }
            }
            Err(TypeError {
                message: format!(
                    "unsupported operand type(s) for +: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        "-" | "*" => {
            // Decimal-family arithmetic
            if is_decimal_type(left) && is_decimal_type(right) {
                return Ok(Type::Decimal);
            }
            if is_bigdecimal_type(left) && is_bigdecimal_type(right) {
                return Ok(Type::BigDecimal);
            }
            if (is_decimal_type(left) && is_integral_numeric_type(right))
                || (is_decimal_type(right) && is_integral_numeric_type(left))
            {
                return Ok(Type::Decimal);
            }
            if (is_bigdecimal_type(left) && is_integral_numeric_type(right))
                || (is_bigdecimal_type(right) && is_integral_numeric_type(left))
            {
                return Ok(Type::BigDecimal);
            }
            // BigInt arithmetic
            if left == &Type::BigInt && right == &Type::BigInt {
                return Ok(Type::BigInt);
            }
            if left == &Type::Int && right == &Type::Int {
                return Ok(Type::Int);
            }
            if left.is_numeric() && right.is_numeric() {
                return Ok(Type::Float);
            }
            // String repetition with *
            if op == "*" && left == &Type::Str && right == &Type::Int {
                return Ok(Type::Str);
            }
            if op == "*" && left == &Type::Int && right == &Type::Str {
                return Ok(Type::Str);
            }
            // List repetition with *
            if op == "*" {
                if let Type::List(_) = left {
                    if right == &Type::Int {
                        return Ok(left.clone());
                    }
                }
            }
            Err(TypeError {
                message: format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        "/" => {
            // Decimal-family arithmetic
            if is_decimal_type(left) && is_decimal_type(right) {
                return Ok(Type::Decimal);
            }
            if is_bigdecimal_type(left) && is_bigdecimal_type(right) {
                return Ok(Type::BigDecimal);
            }
            if (is_decimal_type(left) && is_integral_numeric_type(right))
                || (is_decimal_type(right) && is_integral_numeric_type(left))
            {
                return Ok(Type::Decimal);
            }
            if (is_bigdecimal_type(left) && is_integral_numeric_type(right))
                || (is_bigdecimal_type(right) && is_integral_numeric_type(left))
            {
                return Ok(Type::BigDecimal);
            }
            // BigInt division returns BigInt (floor division semantics for bigint)
            if left == &Type::BigInt && right == &Type::BigInt {
                return Ok(Type::BigInt);
            }
            // Division always returns float in Sifr (like Python 3)
            if left.is_numeric() && right.is_numeric() {
                return Ok(Type::Float);
            }
            Err(TypeError {
                message: format!(
                    "unsupported operand type(s) for /: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        "//" | "%" => {
            // Decimal-family arithmetic
            if is_decimal_type(left) && is_decimal_type(right) {
                return Ok(Type::Decimal);
            }
            if is_bigdecimal_type(left) && is_bigdecimal_type(right) {
                return Ok(Type::BigDecimal);
            }
            if (is_decimal_type(left) && is_integral_numeric_type(right))
                || (is_decimal_type(right) && is_integral_numeric_type(left))
            {
                return Ok(Type::Decimal);
            }
            if (is_bigdecimal_type(left) && is_integral_numeric_type(right))
                || (is_bigdecimal_type(right) && is_integral_numeric_type(left))
            {
                return Ok(Type::BigDecimal);
            }
            // BigInt floor division and modulo
            if left == &Type::BigInt && right == &Type::BigInt {
                return Ok(Type::BigInt);
            }
            // Floor division and modulo
            if left == &Type::Int && right == &Type::Int {
                return Ok(Type::Int);
            }
            if left.is_numeric() && right.is_numeric() {
                return Ok(Type::Float);
            }
            Err(TypeError {
                message: format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        "**" => {
            // Decimal-family exponentiation (integral exponents only in this phase)
            if is_decimal_type(left) && is_integral_numeric_type(right) {
                return Ok(Type::Decimal);
            }
            if is_bigdecimal_type(left) && is_integral_numeric_type(right) {
                return Ok(Type::BigDecimal);
            }
            // BigInt power: bigint ** bigint -> bigint, bigint ** int -> bigint
            if left == &Type::BigInt && (right == &Type::BigInt || right == &Type::Int) {
                return Ok(Type::BigInt);
            }
            // Power: int ** int -> int, otherwise float
            if left == &Type::Int && right == &Type::Int {
                return Ok(Type::Int);
            }
            if left.is_numeric() && right.is_numeric() {
                return Ok(Type::Float);
            }
            Err(TypeError {
                message: format!(
                    "unsupported operand type(s) for **: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        "&" | "|" | "^" => {
            // Bitwise operators: int & int -> int, int | int -> int, int ^ int -> int
            // Also bool & bool -> bool, bool | bool -> bool, bool ^ bool -> bool
            if left == &Type::Int && right == &Type::Int {
                return Ok(Type::Int);
            }
            if left == &Type::Bool && right == &Type::Bool {
                return Ok(Type::Bool);
            }
            Err(TypeError {
                message: format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        "<<" | ">>" => {
            // Shift operators: int << int -> int, int >> int -> int
            if left == &Type::Int && right == &Type::Int {
                return Ok(Type::Int);
            }
            Err(TypeError {
                message: format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        _ => Err(TypeError {
            message: format!("unknown binary operator: {op}"),
            kind: crate::TypeErrorKind::InvalidOperator {
                op: op.to_string(),
                ty: Box::new(left.clone()),
            },
        }),
    }
}

/// Type-check a comparison operation (e.g., `a == b`, `a < b`).
pub fn type_check_comparison(left: &Type, op: &str, right: &Type) -> Result<Type, TypeError> {
    if (is_decimal_type(left) && is_bigdecimal_type(right))
        || (is_bigdecimal_type(left) && is_decimal_type(right))
    {
        return Err(TypeError {
            message:
                "[E2504] cannot compare 'decimal' and 'bigdecimal' without explicit conversion"
                    .to_string(),
            kind: crate::TypeErrorKind::TypeMismatch {
                expected: Box::new(left.clone()),
                actual: Box::new(right.clone()),
            },
        });
    }
    if (left == &Type::Float && is_decimal_family_type(right))
        || (right == &Type::Float && is_decimal_family_type(left))
    {
        return Err(TypeError {
            message: "[E2503] cannot compare 'float' with decimal numeric types".to_string(),
            kind: crate::TypeErrorKind::TypeMismatch {
                expected: Box::new(left.clone()),
                actual: Box::new(right.clone()),
            },
        });
    }

    match op {
        "==" | "!=" => {
            // Block mixed int/bigint equality comparisons
            if (left == &Type::Int && right == &Type::BigInt)
                || (left == &Type::BigInt && right == &Type::Int)
            {
                return Err(TypeError {
                    message: "cannot compare 'int' and 'bigint'; use bigint() or int() to convert explicitly".to_string(),
                    kind: crate::TypeErrorKind::TypeMismatch {
                        expected: Box::new(left.clone()),
                        actual: Box::new(right.clone()),
                    },
                });
            }
            // Equality comparison works on same types
            if left == right || left == &Type::Any || right == &Type::Any {
                return Ok(Type::Bool);
            }
            // Allow T|None vs T comparisons (and T vs T|None)
            if let Type::Union(_) = left {
                let non_none = remove_none_from_union(left);
                if non_none == *right || type_check_comparison(&non_none, op, right).is_ok() {
                    return Ok(Type::Bool);
                }
            }
            if let Type::Union(_) = right {
                let non_none = remove_none_from_union(right);
                if non_none == *left || type_check_comparison(left, op, &non_none).is_ok() {
                    return Ok(Type::Bool);
                }
            }
            // Allow comparing union members with each other
            if let (Type::Union(left_members), _) = (left, right) {
                if left_members.iter().any(|m| m == right) {
                    return Ok(Type::Bool);
                }
            }
            if let (_, Type::Union(right_members)) = (left, right) {
                if right_members.iter().any(|m| m == left) {
                    return Ok(Type::Bool);
                }
            }
            Err(TypeError {
                message: format!(
                    "cannot compare '{}' and '{}' with {op}",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::TypeMismatch {
                    expected: Box::new(left.clone()),
                    actual: Box::new(right.clone()),
                },
            })
        }
        "<" | ">" | "<=" | ">=" => {
            // Block mixed int/bigint comparisons
            if (left == &Type::Int && right == &Type::BigInt)
                || (left == &Type::BigInt && right == &Type::Int)
            {
                return Err(TypeError {
                    message: "cannot compare 'int' and 'bigint'; use bigint() or int() to convert explicitly".to_string(),
                    kind: crate::TypeErrorKind::TypeMismatch {
                        expected: Box::new(left.clone()),
                        actual: Box::new(right.clone()),
                    },
                });
            }
            // Ordering comparison works on numeric types and strings
            if left.is_numeric() && right.is_numeric() {
                return Ok(Type::Bool);
            }
            if left == &Type::Str && right == &Type::Str {
                return Ok(Type::Bool);
            }
            // Allow TypeVar comparisons (generic code)
            if matches!(left, Type::TypeVar(_)) || matches!(right, Type::TypeVar(_)) {
                return Ok(Type::Bool);
            }
            // Allow T|None vs T ordering comparisons (unwrap the union)
            if union_contains_none(left) {
                let non_none = remove_none_from_union(left);
                if type_check_comparison(&non_none, op, right).is_ok() {
                    return Ok(Type::Bool);
                }
            }
            if union_contains_none(right) {
                let non_none = remove_none_from_union(right);
                if type_check_comparison(left, op, &non_none).is_ok() {
                    return Ok(Type::Bool);
                }
            }
            Err(TypeError {
                message: format!(
                    "'{op}' not supported between instances of '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        _ => Err(TypeError {
            message: format!("unknown comparison operator: {op}"),
            kind: crate::TypeErrorKind::InvalidOperator {
                op: op.to_string(),
                ty: Box::new(left.clone()),
            },
        }),
    }
}

/// Type-check a unary operation (e.g., `-x`, `not x`).
pub fn type_check_unary_op(op: &str, operand: &Type) -> Result<Type, TypeError> {
    match op {
        "-" | "+" => {
            if operand.is_numeric() {
                return Ok(operand.clone());
            }
            Err(TypeError {
                message: format!(
                    "bad operand type for unary {op}: '{}'",
                    operand.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(operand.clone()),
                },
            })
        }
        "not" => {
            if operand == &Type::Bool {
                return Ok(Type::Bool);
            }
            // Collection truthiness: `not list_var`, `not dict_var`, etc.
            match operand {
                Type::List(_) | Type::Dict(_, _) | Type::Set(_) | Type::Tuple(_) | Type::Str => {
                    return Ok(Type::Bool);
                }
                // Allow `not x` where x is Optional (T | None)
                Type::Union(_) if union_contains_none(operand) => {
                    return Ok(Type::Bool);
                }
                _ => {}
            }
            Err(TypeError {
                message: format!(
                    "bad operand type for unary not: '{}'",
                    operand.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(operand.clone()),
                },
            })
        }
        "~" => {
            // Bitwise invert: ~int -> int, ~bool -> int
            if operand == &Type::Int || operand == &Type::Bool {
                return Ok(Type::Int);
            }
            Err(TypeError {
                message: format!("bad operand type for unary ~: '{}'", operand.display_name()),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(operand.clone()),
                },
            })
        }
        _ => Err(TypeError {
            message: format!("unknown unary operator: {op}"),
            kind: crate::TypeErrorKind::InvalidOperator {
                op: op.to_string(),
                ty: Box::new(operand.clone()),
            },
        }),
    }
}

/// Type-check a boolean operation (e.g., `a and b`, `a or b`).
pub fn type_check_bool_op(left: &Type, op: &str, right: &Type) -> Result<Type, TypeError> {
    fn supports_truthiness(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Bool
                | Type::Int
                | Type::Float
                | Type::List(_)
                | Type::Dict(_, _)
                | Type::Set(_)
                | Type::Tuple(_)
                | Type::Str
                | Type::Any
                | Type::Unknown
        ) || union_contains_none(ty)
    }

    match op {
        "and" | "or" => {
            if supports_truthiness(left) && supports_truthiness(right) {
                return Ok(Type::Bool);
            }
            Err(TypeError {
                message: format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
                kind: crate::TypeErrorKind::InvalidOperator {
                    op: op.to_string(),
                    ty: Box::new(left.clone()),
                },
            })
        }
        _ => Err(TypeError {
            message: format!("unknown boolean operator: {op}"),
            kind: crate::TypeErrorKind::InvalidOperator {
                op: op.to_string(),
                ty: Box::new(left.clone()),
            },
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_arithmetic() {
        assert_eq!(
            type_check_binary_op(&Type::Int, "+", &Type::Int).unwrap(),
            Type::Int
        );
        assert_eq!(
            type_check_binary_op(&Type::Int, "-", &Type::Int).unwrap(),
            Type::Int
        );
        assert_eq!(
            type_check_binary_op(&Type::Int, "*", &Type::Int).unwrap(),
            Type::Int
        );
        assert_eq!(
            type_check_binary_op(&Type::Int, "//", &Type::Int).unwrap(),
            Type::Int
        );
        assert_eq!(
            type_check_binary_op(&Type::Int, "%", &Type::Int).unwrap(),
            Type::Int
        );
    }

    #[test]
    fn test_division_returns_float() {
        assert_eq!(
            type_check_binary_op(&Type::Int, "/", &Type::Int).unwrap(),
            Type::Float
        );
    }

    #[test]
    fn test_mixed_numeric() {
        assert_eq!(
            type_check_binary_op(&Type::Int, "+", &Type::Float).unwrap(),
            Type::Float
        );
        assert_eq!(
            type_check_binary_op(&Type::Float, "*", &Type::Int).unwrap(),
            Type::Float
        );
    }

    #[test]
    fn test_string_concat() {
        assert_eq!(
            type_check_binary_op(&Type::Str, "+", &Type::Str).unwrap(),
            Type::Str
        );
    }

    #[test]
    fn test_invalid_binary_op() {
        assert!(type_check_binary_op(&Type::Str, "-", &Type::Str).is_err());
        assert!(type_check_binary_op(&Type::Int, "+", &Type::Str).is_err());
        assert!(type_check_binary_op(&Type::Bool, "+", &Type::Bool).is_err());
    }

    #[test]
    fn test_optional_arithmetic_requires_narrowing() {
        let optional_int = Type::Union(vec![Type::None, Type::Int]);
        assert!(type_check_binary_op(&optional_int, "+", &Type::Int).is_err());
        assert!(type_check_binary_op(&Type::Int, "+", &optional_int).is_err());
        assert!(type_check_binary_op(&optional_int, "-", &Type::Int).is_err());
        assert!(type_check_binary_op(&optional_int, "*", &Type::Int).is_err());
        assert!(type_check_binary_op(&optional_int, "/", &Type::Int).is_err());
    }

    #[test]
    fn test_comparison() {
        assert_eq!(
            type_check_comparison(&Type::Int, "==", &Type::Int).unwrap(),
            Type::Bool
        );
        assert_eq!(
            type_check_comparison(&Type::Int, "<", &Type::Int).unwrap(),
            Type::Bool
        );
        assert_eq!(
            type_check_comparison(&Type::Str, "==", &Type::Str).unwrap(),
            Type::Bool
        );
        assert!(type_check_comparison(&Type::Int, "==", &Type::Str).is_err());
    }

    #[test]
    fn test_unary_ops() {
        assert_eq!(type_check_unary_op("-", &Type::Int).unwrap(), Type::Int);
        assert_eq!(type_check_unary_op("-", &Type::Float).unwrap(), Type::Float);
        assert_eq!(type_check_unary_op("not", &Type::Bool).unwrap(), Type::Bool);
        assert!(type_check_unary_op("-", &Type::Str).is_err());
        assert!(type_check_unary_op("not", &Type::Int).is_err());
    }

    #[test]
    fn test_mixed_int_bigint_comparison_blocked() {
        assert!(type_check_comparison(&Type::Int, "==", &Type::BigInt).is_err());
        assert!(type_check_comparison(&Type::BigInt, "==", &Type::Int).is_err());
        assert!(type_check_comparison(&Type::Int, "<", &Type::BigInt).is_err());
        assert!(type_check_comparison(&Type::BigInt, ">", &Type::Int).is_err());
        // Same-type comparisons should still work
        assert!(type_check_comparison(&Type::BigInt, "==", &Type::BigInt).is_ok());
        assert!(type_check_comparison(&Type::BigInt, "<", &Type::BigInt).is_ok());
    }

    #[test]
    fn test_bool_ops() {
        assert_eq!(
            type_check_bool_op(&Type::Bool, "and", &Type::Bool).unwrap(),
            Type::Bool
        );
        assert_eq!(
            type_check_bool_op(&Type::Bool, "or", &Type::Bool).unwrap(),
            Type::Bool
        );
        assert_eq!(
            type_check_bool_op(&Type::Int, "and", &Type::Int).unwrap(),
            Type::Bool
        );
        assert_eq!(
            type_check_bool_op(&Type::List(Box::new(Type::Int)), "and", &Type::Bool).unwrap(),
            Type::Bool
        );
    }
}
