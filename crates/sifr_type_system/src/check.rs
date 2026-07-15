//! Type checking rules for Sifr operators and expressions.

use crate::collection_capabilities::type_check_list_repetition;
use crate::types::Type;
use crate::union::{remove_none_from_union, union_contains_none};
use sifr_diagnostics::DiagnosticCode;

type TypeCheckResult = Result<Type, (DiagnosticCode, String)>;

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

fn is_exact_or_fixed_integer_type(ty: &Type) -> bool {
    matches!(ty, Type::Int | Type::LiteralInt(_))
        || matches!(ty, Type::FixedInt(fixed) if fixed.supports_current_scalar_promotion_to_int())
}

fn is_bool_type(ty: &Type) -> bool {
    matches!(ty.resolve_alias(), Type::Bool | Type::LiteralBool(_))
}

fn is_integer_type(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::FixedInt(_)
    )
}

fn is_exact_to_float_integer_type(ty: &Type) -> bool {
    matches!(
        ty.resolve_alias(),
        Type::Int | Type::LiteralInt(_) | Type::BigInt | Type::FixedInt(_)
    )
}

fn is_bool_integer_mixed_comparison(left: &Type, right: &Type) -> bool {
    (is_bool_type(left) && is_integer_type(right)) || (is_integer_type(left) && is_bool_type(right))
}

/// Type-check a binary operation (e.g., `a + b`, `a - b`).
///
/// Returns the result type or an error.
pub fn type_check_binary_op(left: &Type, op: &str, right: &Type) -> TypeCheckResult {
    if (is_decimal_type(left) && is_bigdecimal_type(right))
        || (is_bigdecimal_type(left) && is_decimal_type(right))
    {
        return Err((
            DiagnosticCode::DECIMAL_MIXED_WITH_BIGDECIMAL,
            "cannot mix 'decimal' and 'bigdecimal' in arithmetic; use explicit Decimal(...) or BigDecimal(...) conversion".to_string(),
        ));
    }

    if (left == &Type::Float && is_decimal_family_type(right))
        || (right == &Type::Float && is_decimal_family_type(left))
    {
        return Err((
            DiagnosticCode::DECIMAL_FLOAT_MIXED,
            "cannot mix 'float' with decimal numeric types in arithmetic".to_string(),
        ));
    }

    // Mixed int/bigint arithmetic is a compile error (except bigint ** int which is allowed)
    let is_bigint_pow_int = left == &Type::BigInt && right == &Type::Int && op == "**";
    if !is_bigint_pow_int {
        if (left == &Type::Int && right == &Type::BigInt)
            || (left == &Type::BigInt && right == &Type::Int)
        {
            return Err((
                DiagnosticCode::TYPE_INT_BIGINT_MIXED,
                "cannot mix 'int' and 'bigint' in arithmetic; use bigint() or int() to convert explicitly".to_string(),
            ));
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
            if is_exact_or_fixed_integer_type(left) && is_exact_or_fixed_integer_type(right) {
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
                    if !l_elem.supports_derived_clone() && !l_elem.contains_affine_resource() {
                        return Err((
                            DiagnosticCode::TYPE_MISMATCH,
                            "cannot concatenate lists whose element clone capability is not statically known"
                                .to_string(),
                        ));
                    }
                    if l_elem.contains_affine_resource() {
                        return Err((
                            DiagnosticCode::PYZC_INVALID_DECLARATION,
                            "cannot concatenate lists containing affine Python buffers because concatenation duplicates their elements"
                                .to_string(),
                        ));
                    }
                    return Ok(Type::List(l_elem.clone()));
                }
            }
            // Bytes concatenation: bytes + bytes -> bytes
            if left == &Type::Bytes && right == &Type::Bytes {
                return Ok(Type::Bytes);
            }
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "unsupported operand type(s) for +: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
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
            if is_exact_or_fixed_integer_type(left) && is_exact_or_fixed_integer_type(right) {
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
                if let Some(result) = type_check_list_repetition(left, right)
                    .or_else(|| type_check_list_repetition(right, left))
                {
                    return result;
                }
                if left == &Type::Bytes && right == &Type::Int {
                    return Ok(Type::Bytes);
                }
                if left == &Type::Int && right == &Type::Bytes {
                    return Ok(Type::Bytes);
                }
            }
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
        }
        "/" => {
            if is_exact_to_float_integer_type(left) && is_exact_to_float_integer_type(right) {
                return Err((
                    DiagnosticCode::INT_EXACT_TO_FLOAT_REQUIRES_HANDLING,
                    "exact integer to float conversion requires handling possible overflow or precision loss".to_string(),
                ));
            }
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
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "unsupported operand type(s) for /: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
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
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
        }
        "**" => {
            // Decimal-family exponentiation currently accepts integral exponents only.
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
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "unsupported operand type(s) for **: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
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
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
        }
        "<<" | ">>" => {
            // Shift operators: int << int -> int, int >> int -> int
            if left == &Type::Int && right == &Type::Int {
                return Ok(Type::Int);
            }
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
        }
        _ => Err((
            DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
            format!("unknown binary operator: {op}"),
        )),
    }
}

/// Type-check a comparison operation (e.g., `a == b`, `a < b`).
pub fn type_check_comparison(left: &Type, op: &str, right: &Type) -> TypeCheckResult {
    if (is_decimal_type(left) && is_bigdecimal_type(right))
        || (is_bigdecimal_type(left) && is_decimal_type(right))
    {
        return Err((
            DiagnosticCode::DECIMAL_MIXED_WITH_BIGDECIMAL,
            "cannot compare 'decimal' and 'bigdecimal' without explicit conversion".to_string(),
        ));
    }
    if (left == &Type::Float && is_decimal_family_type(right))
        || (right == &Type::Float && is_decimal_family_type(left))
    {
        return Err((
            DiagnosticCode::DECIMAL_FLOAT_MIXED,
            "cannot compare 'float' with decimal numeric types".to_string(),
        ));
    }

    match op {
        "==" | "!=" => {
            if !left.supports_structural_equality() || !right.supports_structural_equality() {
                let reason = if left.contains_affine_resource() || right.contains_affine_resource()
                {
                    "affine values"
                } else {
                    "values without structural equality"
                };
                return Err((
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "cannot compare {reason} '{}' and '{}' with {op}",
                        left.display_name(),
                        right.display_name()
                    ),
                ));
            }
            if is_bool_integer_mixed_comparison(left, right) {
                return Err((
                    DiagnosticCode::INT_BOOL_INTEGER_COMPARISON,
                    "cannot compare bool and integer values without explicit conversion"
                        .to_string(),
                ));
            }
            // Block mixed int/bigint equality comparisons
            if (left == &Type::Int && right == &Type::BigInt)
                || (left == &Type::BigInt && right == &Type::Int)
            {
                return Err((
                    DiagnosticCode::TYPE_INT_BIGINT_MIXED,
                    "cannot compare 'int' and 'bigint'; use bigint() or int() to convert explicitly".to_string(),
                ));
            }
            // Equality comparison works on same types and on structurally matching
            // containers when one side still carries Any/Unknown element shape.
            if equality_comparable(left, right) {
                return Ok(Type::Bool);
            }
            // Allow T|None vs T comparisons (and T vs T|None)
            if union_contains_none(left) {
                let non_none = remove_none_from_union(left);
                if non_none == *right || type_check_comparison(&non_none, op, right).is_ok() {
                    return Ok(Type::Bool);
                }
            }
            if union_contains_none(right) {
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
            Err((
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "cannot compare '{}' and '{}' with {op}",
                    left.display_name(),
                    right.display_name()
                ),
            ))
        }
        "<" | ">" | "<=" | ">=" => {
            if is_bool_integer_mixed_comparison(left, right) {
                return Err((
                    DiagnosticCode::INT_BOOL_INTEGER_COMPARISON,
                    "cannot compare bool and integer values without explicit conversion"
                        .to_string(),
                ));
            }
            // Block mixed int/bigint comparisons
            if (left == &Type::Int && right == &Type::BigInt)
                || (left == &Type::BigInt && right == &Type::Int)
            {
                return Err((
                    DiagnosticCode::TYPE_INT_BIGINT_MIXED,
                    "cannot compare 'int' and 'bigint'; use bigint() or int() to convert explicitly".to_string(),
                ));
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
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "'{op}' not supported between instances of '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
        }
        _ => Err((
            DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
            format!("unknown comparison operator: {op}"),
        )),
    }
}

fn equality_comparable(left: &Type, right: &Type) -> bool {
    let left = left.resolve_alias();
    let right = right.resolve_alias();

    if left == right {
        return true;
    }
    if matches!(left, Type::Any | Type::Unknown) || matches!(right, Type::Any | Type::Unknown) {
        return true;
    }

    match (left, right) {
        (Type::List(left_elem), Type::List(right_elem))
        | (Type::Set(left_elem), Type::Set(right_elem)) => {
            equality_comparable(left_elem.as_ref(), right_elem.as_ref())
        }
        (Type::Dict(left_key, left_value), Type::Dict(right_key, right_value)) => {
            equality_comparable(left_key.as_ref(), right_key.as_ref())
                && equality_comparable(left_value.as_ref(), right_value.as_ref())
        }
        (Type::Tuple(left_items), Type::Tuple(right_items))
            if left_items.len() == right_items.len() =>
        {
            left_items
                .iter()
                .zip(right_items.iter())
                .all(|(left_item, right_item)| equality_comparable(left_item, right_item))
        }
        _ => false,
    }
}

/// Type-check a unary operation (e.g., `-x`, `not x`).
pub fn type_check_unary_op(op: &str, operand: &Type) -> TypeCheckResult {
    match op {
        "-" | "+" => {
            if operand.is_numeric() {
                return Ok(operand.clone());
            }
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "bad operand type for unary {op}: '{}'",
                    operand.display_name()
                ),
            ))
        }
        "not" => {
            if operand == &Type::Bool {
                return Ok(Type::Bool);
            }
            // Collection truthiness: `not list_var`, `not dict_var`, etc.
            match operand {
                Type::List(_)
                | Type::Dict(_, _)
                | Type::Set(_)
                | Type::Tuple(_)
                | Type::Str
                | Type::Class { .. }
                | Type::Protocol { .. } => {
                    return Ok(Type::Bool);
                }
                // Allow `not x` where x is Optional (T | None)
                Type::Union(_) if union_contains_none(operand) => {
                    return Ok(Type::Bool);
                }
                _ => {}
            }
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "bad operand type for unary not: '{}'",
                    operand.display_name()
                ),
            ))
        }
        "~" => {
            // Bitwise invert: ~int -> int, ~bool -> int
            if operand == &Type::Int || operand == &Type::Bool {
                return Ok(Type::Int);
            }
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!("bad operand type for unary ~: '{}'", operand.display_name()),
            ))
        }
        _ => Err((
            DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
            format!("unknown unary operator: {op}"),
        )),
    }
}

/// Type-check a boolean operation (e.g., `a and b`, `a or b`).
pub fn type_check_bool_op(left: &Type, op: &str, right: &Type) -> TypeCheckResult {
    fn supports_truthiness(ty: &Type) -> bool {
        matches!(
            ty,
            Type::Bool
                | Type::Int
                | Type::BigInt
                | Type::Float
                | Type::List(_)
                | Type::Dict(_, _)
                | Type::Set(_)
                | Type::Tuple(_)
                | Type::Str
                | Type::Class { .. }
                | Type::Protocol { .. }
                | Type::Any
                | Type::Unknown
        ) || union_contains_none(ty)
    }

    match op {
        "and" | "or" => {
            if supports_truthiness(left) && supports_truthiness(right) {
                return Ok(Type::Bool);
            }
            Err((
                DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
                format!(
                    "unsupported operand type(s) for {op}: '{}' and '{}'",
                    left.display_name(),
                    right.display_name()
                ),
            ))
        }
        _ => Err((
            DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR,
            format!("unknown boolean operator: {op}"),
        )),
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
    fn test_exact_integer_true_division_requires_float_handling() {
        for (left, right) in [
            (Type::Int, Type::Int),
            (Type::LiteralInt(10), Type::LiteralInt(2)),
            (Type::FixedInt(crate::FixedIntType::I32), Type::Int),
            (Type::BigInt, Type::BigInt),
        ] {
            let err = type_check_binary_op(&left, "/", &right).unwrap_err();
            assert_eq!(err.0, DiagnosticCode::INT_EXACT_TO_FLOAT_REQUIRES_HANDLING);
            assert_eq!(
                err.1,
                "exact integer to float conversion requires handling possible overflow or precision loss"
            );
        }
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
    fn test_fixed_width_integer_add_sub_mul_promote_to_int() {
        for fixed in [
            crate::FixedIntType::I8,
            crate::FixedIntType::I16,
            crate::FixedIntType::I32,
            crate::FixedIntType::I64,
            crate::FixedIntType::U8,
            crate::FixedIntType::U16,
            crate::FixedIntType::U32,
            crate::FixedIntType::ISize,
        ] {
            let ty = Type::FixedInt(fixed);

            assert_eq!(type_check_binary_op(&ty, "+", &ty).unwrap(), Type::Int);
            assert_eq!(
                type_check_binary_op(&ty, "-", &Type::Int).unwrap(),
                Type::Int
            );
            assert_eq!(
                type_check_binary_op(&Type::LiteralInt(2), "*", &ty).unwrap(),
                Type::Int
            );
        }
    }

    #[test]
    fn test_uint64_and_usize_integer_add_wait_for_sifrint_promotion() {
        for fixed in [crate::FixedIntType::U64, crate::FixedIntType::USize] {
            let ty = Type::FixedInt(fixed);

            assert!(type_check_binary_op(&ty, "+", &ty).is_err());
        }
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
        let str_sub = type_check_binary_op(&Type::Str, "-", &Type::Str).unwrap_err();
        assert_eq!(str_sub.0, DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR);
        let int_str_add = type_check_binary_op(&Type::Int, "+", &Type::Str).unwrap_err();
        assert_eq!(int_str_add.0, DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR);
        let bool_add = type_check_binary_op(&Type::Bool, "+", &Type::Bool).unwrap_err();
        assert_eq!(bool_add.0, DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR);
    }

    #[test]
    fn test_optional_arithmetic_requires_narrowing() {
        let optional_int = Type::Union(vec![Type::None, Type::Int]);
        let optional_plus_int = type_check_binary_op(&optional_int, "+", &Type::Int).unwrap_err();
        assert_eq!(
            optional_plus_int.0,
            DiagnosticCode::TYPE_UNSUPPORTED_OPERATOR
        );
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
    fn test_union_member_equality_terminates_for_both_operand_orders() {
        let int_or_str = Type::Union(vec![Type::Int, Type::Str]);

        assert_eq!(
            type_check_comparison(&Type::Int, "==", &int_or_str).unwrap(),
            Type::Bool
        );
        assert_eq!(
            type_check_comparison(&int_or_str, "!=", &Type::Int).unwrap(),
            Type::Bool
        );
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
        let int_bigint_eq = type_check_comparison(&Type::Int, "==", &Type::BigInt).unwrap_err();
        assert_eq!(int_bigint_eq.0, DiagnosticCode::TYPE_INT_BIGINT_MIXED);
        assert!(type_check_comparison(&Type::BigInt, "==", &Type::Int).is_err());
        assert!(type_check_comparison(&Type::Int, "<", &Type::BigInt).is_err());
        assert!(type_check_comparison(&Type::BigInt, ">", &Type::Int).is_err());
        // Same-type comparisons should still work
        assert!(type_check_comparison(&Type::BigInt, "==", &Type::BigInt).is_ok());
        assert!(type_check_comparison(&Type::BigInt, "<", &Type::BigInt).is_ok());
    }

    #[test]
    fn test_bool_integer_comparison_blocked_with_int_diagnostic() {
        for (left, op, right) in [
            (Type::Bool, "==", Type::Int),
            (Type::LiteralBool(true), "!=", Type::LiteralInt(1)),
            (Type::FixedInt(crate::FixedIntType::U8), "<", Type::Bool),
            (Type::BigInt, ">=", Type::LiteralBool(false)),
        ] {
            let err = type_check_comparison(&left, op, &right).unwrap_err();
            assert_eq!(err.0, DiagnosticCode::INT_BOOL_INTEGER_COMPARISON);
            assert_eq!(
                err.1,
                "cannot compare bool and integer values without explicit conversion"
            );
        }
        assert!(type_check_comparison(&Type::Bool, "==", &Type::Bool).is_ok());
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
            type_check_bool_op(&Type::BigInt, "and", &Type::Bool).unwrap(),
            Type::Bool
        );
        assert_eq!(
            type_check_bool_op(&Type::List(Box::new(Type::Int)), "and", &Type::Bool).unwrap(),
            Type::Bool
        );
    }
}
