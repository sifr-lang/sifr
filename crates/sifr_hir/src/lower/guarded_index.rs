use super::LowerCtx;
use sifr_python_ast::{Expr, ExprSubscript, Number};
use sifr_type_system::Type;

pub(super) fn guarded_sequence_index_result_type(
    sub: &ExprSubscript,
    object_ty: &Type,
    ctx: &LowerCtx,
) -> Option<Type> {
    let Expr::Name(sequence_name) = sub.value.as_ref() else {
        return None;
    };
    match object_ty.resolve_alias() {
        Type::List(elem_ty) => {
            guarded_element_type(sequence_name.id.as_str(), elem_ty, &sub.slice, ctx)
        }
        Type::Str => guarded_string_index_type(sequence_name.id.as_str(), &sub.slice, ctx),
        _ => None,
    }
}

fn guarded_element_type(
    sequence_name: &str,
    elem_ty: &Type,
    index_expr: &Expr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if has_guarded_sequence_index(sequence_name, index_expr, ctx) {
        Some(elem_ty.clone())
    } else {
        None
    }
}

fn guarded_string_index_type(
    sequence_name: &str,
    index_expr: &Expr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if has_guarded_sequence_index(sequence_name, index_expr, ctx) {
        Some(Type::Str)
    } else {
        None
    }
}

fn has_guarded_sequence_index(sequence_name: &str, index_expr: &Expr, ctx: &LowerCtx) -> bool {
    match index_expr {
        Expr::Name(index_name) => {
            let index_var = index_name.id.as_str();
            ctx.has_index_var_guard(sequence_name, index_var)
                || (ctx.is_zero_based_pointer(index_var) && ctx.min_length_guard(sequence_name) > 0)
                || ctx
                    .end_pointer_sequence(index_var)
                    .is_some_and(|pointer_sequence| {
                        pointer_sequence == sequence_name && ctx.min_length_guard(sequence_name) > 0
                    })
        }
        Expr::NumberLiteral(num) => {
            let Number::Int(value) = &num.value else {
                return false;
            };
            let Some(index_value) = value.as_i64() else {
                return false;
            };
            let Ok(index_value) = usize::try_from(index_value) else {
                return false;
            };
            ctx.min_length_guard(sequence_name) > index_value
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use crate::{lower_module, HirModule, LoweringError, LoweringResult};
    use sifr_python_parser::parse_module;
    use sifr_type_system::Type;

    fn lower_source(source: &str) -> Result<HirModule, Vec<LoweringError>> {
        let parsed = parse_module(source).expect("parse failed");
        lower_module(parsed.suite()).map(|r| r.module)
    }

    fn lower_source_result(source: &str) -> Result<LoweringResult, Vec<LoweringError>> {
        let parsed = parse_module(source).expect("parse failed");
        lower_module(parsed.suite())
    }

    #[test]
    fn test_guarded_string_index_in_while_reveals_str() {
        let result = lower_source_result(
            "def main():\n    text: str = \"aeiou\"\n    i: int = 0\n    while i < len(text):\n        reveal_type(text[i])\n        i = i + 1\n",
        )
        .expect("guarded while string index should lower");

        assert!(result
            .reveal_types
            .iter()
            .any(|diagnostic| diagnostic == "reveal_type: str"));
    }

    #[test]
    fn test_range_len_list_index_reveals_element_type() {
        let result = lower_source_result(
            "def main():\n    nums: list[int] = [1, 2, 3]\n    for i in range(len(nums)):\n        reveal_type(nums[i])\n",
        )
        .expect("range(len(list)) index should lower");

        assert!(result
            .reveal_types
            .iter()
            .any(|diagnostic| diagnostic == "reveal_type: int"));
    }

    #[test]
    fn test_early_return_non_empty_guard_reveals_element_type() {
        let result = lower_source_result(
            "def head(nums: list[int]) -> int:\n    if len(nums) == 0:\n        return 0\n    reveal_type(nums[0])\n    return nums[0]\n",
        )
        .expect("early-return non-empty guard should lower");

        assert!(result
            .reveal_types
            .iter()
            .any(|diagnostic| diagnostic == "reveal_type: int"));
    }

    #[test]
    fn test_early_return_non_empty_guard_let_uses_narrowed_index_type() {
        let module = lower_source(
            "def head(values: list[int]) -> int:\n    if len(values) == 0:\n        return 0\n    first: int = values[0]\n    return first\n",
        )
        .expect("post-guard let should lower");

        let crate::HirStmt::Let { value, .. } = &module.functions[0].body[1] else {
            panic!("expected post-guard let statement");
        };
        assert_eq!(value.ty(), &Type::Int);
    }

    #[test]
    fn test_unguarded_list_index_stays_optional() {
        let result = lower_source(
            "def main():\n    nums: list[int] = [1, 2, 3]\n    first: int = nums[0]\n",
        );

        assert!(
            result.is_err(),
            "unguarded list index should remain optional"
        );
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|error| {
            error
                .message
                .contains("type mismatch: expected 'int', got 'int | None'")
        }));
    }

    #[test]
    fn test_two_pointer_while_reveals_element_type_after_single_step_updates() {
        let result = lower_source_result(
            "def main():\n    height: list[int] = [0, 1, 0, 2]\n    l: int = 0\n    r: int = len(height) - 1\n    while l < r:\n        l += 1\n        reveal_type(height[l])\n        r -= 1\n        reveal_type(height[r])\n",
        )
        .expect("two-pointer while index should lower");

        let reveal_count = result
            .reveal_types
            .iter()
            .filter(|diagnostic| diagnostic.as_str() == "reveal_type: int")
            .count();
        assert_eq!(reveal_count, 2);
    }

    #[test]
    fn test_two_pointer_while_with_pointer_jump_stays_optional() {
        let result = lower_source(
            "def main():\n    height: list[int] = [0, 1, 0, 2]\n    l: int = 0\n    r: int = len(height) - 1\n    while l < r:\n        l += 2\n        current: int = height[l]\n",
        );

        assert!(
            result.is_err(),
            "unsupported pointer jumps should remain optional"
        );
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|error| {
            error
                .message
                .contains("type mismatch: expected 'int', got 'int | None'")
        }));
    }

    #[test]
    fn test_non_empty_zero_and_end_pointers_reveal_element_type() {
        let result = lower_source_result(
            "def main(values: list[int]) -> int:\n    if not values:\n        return 0\n    l: int = 0\n    r: int = len(values) - 1\n    reveal_type(values[l])\n    reveal_type(values[r])\n    return values[l] + values[r]\n",
        )
        .expect("non-empty zero/end pointers should lower");

        let reveal_count = result
            .reveal_types
            .iter()
            .filter(|diagnostic| diagnostic.as_str() == "reveal_type: int")
            .count();
        assert_eq!(reveal_count, 2);
    }
}
