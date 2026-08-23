use super::LowerCtx;
use sifr_python_ast::{Expr, ExprBinOp, ExprSubscript, Number, Operator};
use sifr_type_system::{Type, remove_none_from_union};

pub(in crate::lower) fn guarded_sequence_index_result_type(
    sub: &ExprSubscript,
    object_ty: &Type,
    ctx: &LowerCtx,
) -> Option<Type> {
    if let Some(sequence_name) = sequence_guard_target_name(sub.value.as_ref()) {
        return match object_ty.resolve_alias() {
            Type::List(elem_ty) => {
                guarded_element_type(sequence_name.as_str(), elem_ty, &sub.slice, ctx)
            }
            Type::Dict(_, value_ty) => {
                if ctx.has_dict_key_guard(sequence_name.as_str(), &sub.slice) {
                    Some(*value_ty.clone())
                } else if ctx.has_subscript_guard(sequence_name.as_str(), &sub.slice) {
                    Some(remove_none_from_union(value_ty.as_ref()))
                } else {
                    None
                }
            }
            Type::Str => guarded_string_index_type(sequence_name.as_str(), &sub.slice, ctx),
            Type::Bytes => guarded_bytes_index_type(sequence_name.as_str(), &sub.slice, ctx),
            _ => None,
        };
    }

    let Expr::Subscript(outer_sub) = sub.value.as_ref() else {
        return None;
    };
    let Expr::Name(matrix_name) = outer_sub.value.as_ref() else {
        return None;
    };
    let (outer_anchor, outer_extra_len, inner_anchor, inner_extra_len) =
        ctx.matrix_sequence_fact(matrix_name.id.as_str())?;
    if !index_expr_is_safe_for_anchor(
        outer_sub.slice.as_ref(),
        &outer_anchor,
        outer_extra_len,
        ctx,
    ) {
        return None;
    }
    match object_ty.resolve_alias() {
        Type::List(elem_ty) => {
            if index_expr_is_safe_for_anchor(&sub.slice, &inner_anchor, inner_extra_len, ctx) {
                Some(*elem_ty.clone())
            } else {
                None
            }
        }
        Type::Str => None,
        _ => None,
    }
}

fn guarded_element_type(
    sequence_name: &str,
    elem_ty: &Type,
    index_expr: &Expr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if ctx.has_subscript_guard(sequence_name, index_expr) {
        return Some(remove_none_from_union(elem_ty));
    }
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

fn guarded_bytes_index_type(
    sequence_name: &str,
    index_expr: &Expr,
    ctx: &LowerCtx,
) -> Option<Type> {
    if has_guarded_sequence_index(sequence_name, index_expr, ctx) {
        Some(Type::FixedInt(sifr_type_system::FixedIntType::U8))
    } else {
        None
    }
}

fn has_guarded_sequence_index(sequence_name: &str, index_expr: &Expr, ctx: &LowerCtx) -> bool {
    if ctx.has_subscript_guard(sequence_name, index_expr) {
        return true;
    }
    if index_expr_is_safe_for_anchor(index_expr, sequence_name, 0, ctx) {
        return true;
    }
    if let Some((anchor_sequence, extra_len)) = ctx.sized_sequence_fact(sequence_name) {
        return index_expr_is_safe_for_anchor(index_expr, &anchor_sequence, extra_len, ctx);
    }
    if let Some((outer_anchor, outer_extra_len, _, _)) = ctx.matrix_sequence_fact(sequence_name) {
        return index_expr_is_safe_for_anchor(index_expr, &outer_anchor, outer_extra_len, ctx);
    }
    false
}

fn index_expr_is_safe_for_anchor(
    index_expr: &Expr,
    anchor_sequence: &str,
    extra_len: usize,
    ctx: &LowerCtx,
) -> bool {
    match index_expr {
        Expr::Name(index_name) => {
            let index_var = index_name.id.as_str();
            ctx.has_index_var_guard(anchor_sequence, index_var)
                || (ctx.is_zero_based_pointer(index_var)
                    && ctx.min_length_guard(anchor_sequence) > 0)
                || ctx
                    .end_pointer_sequence(index_var)
                    .is_some_and(|pointer_sequence| {
                        pointer_sequence == anchor_sequence
                            && ctx.min_length_guard(anchor_sequence) > 0
                    })
        }
        Expr::BinOp(ExprBinOp {
            left, op, right, ..
        }) => {
            let Expr::Name(index_name) = left.as_ref() else {
                return false;
            };
            let Some(offset) = literal_usize(right.as_ref()) else {
                return false;
            };
            match op {
                Operator::Add => ctx.has_index_var_offset_guard(
                    anchor_sequence,
                    index_name.id.as_str(),
                    offset.saturating_sub(extra_len),
                ),
                Operator::Sub => false,
                _ => false,
            }
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
            if index_value < extra_len {
                true
            } else {
                ctx.min_length_guard(anchor_sequence) > index_value - extra_len
            }
        }
        _ => false,
    }
}

fn literal_usize(expr: &Expr) -> Option<usize> {
    let Expr::NumberLiteral(num) = expr else {
        return None;
    };
    let Number::Int(value) = &num.value else {
        return None;
    };
    value.as_i64().and_then(|value| usize::try_from(value).ok())
}

fn sequence_guard_target_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Name(name) => Some(name.id.to_string()),
        Expr::Attribute(attr) => {
            let base = sequence_guard_target_name(attr.value.as_ref())?;
            Some(format!("{base}.{}", attr.attr))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::{HirDiagnostic, HirModule, LoweringResult, lower_module};
    use sifr_python_parser::parse_module;
    use sifr_type_system::Type;

    fn lower_source(source: &str) -> Result<HirModule, Vec<HirDiagnostic>> {
        let parsed = parse_module(source).expect("parse failed");
        lower_module(parsed.suite()).map(|r| r.module)
    }

    fn lower_source_result(source: &str) -> Result<LoweringResult, Vec<HirDiagnostic>> {
        let parsed = parse_module(source).expect("parse failed");
        lower_module(parsed.suite())
    }

    #[test]
    fn test_guarded_string_index_in_while_reveals_str() {
        let result = lower_source_result(
            "def main():\n    text: str = \"aeiou\"\n    i: int = 0\n    while i < len(text):\n        reveal_type(text[i])\n        i = i + 1\n",
        )
        .expect("guarded while string index should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "str")
        );
    }

    #[test]
    fn test_tuple_unpack_len_alias_while_string_index_reveals_str() {
        let result = lower_source_result(
            "def main():\n    text: str = \"aeiou\"\n    i, n = 0, len(text)\n    while i < n:\n        reveal_type(text[i])\n        i += 1\n",
        )
        .expect("tuple-unpacked len alias should narrow while-loop string index");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "str")
        );
    }

    #[test]
    fn test_range_len_list_index_reveals_element_type() {
        let result = lower_source_result(
            "def main():\n    nums: list[int] = [1, 2, 3]\n    for i in range(len(nums)):\n        reveal_type(nums[i])\n",
        )
        .expect("range(len(list)) index should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_range_len_alias_list_index_reveals_element_type() {
        let result = lower_source_result(
            "def main():\n    nums: list[int] = [1, 2, 3]\n    n: int = len(nums)\n    for i in range(n):\n        reveal_type(nums[i])\n",
        )
        .expect("range(len-alias) index should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_reverse_range_len_alias_list_index_reveals_element_type() {
        let result = lower_source_result(
            "def main():\n    nums: list[int] = [1, 2, 3]\n    n: int = len(nums)\n    for i in range(n - 1, -1, -1):\n        reveal_type(nums[i])\n",
        )
        .expect("reverse range(len-alias) index should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_while_end_pointer_len_alias_reveals_element_type() {
        let result = lower_source_result(
            "def main():\n    nums: list[int] = [1, 2, 3]\n    n: int = len(nums)\n    i: int = n - 1\n    while i >= 0:\n        reveal_type(nums[i])\n        i -= 1\n",
        )
        .expect("while end-pointer len-alias index should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_append_growth_shape_allows_index_under_alias_guard() {
        let result = lower_source_result(
            "def main():\n    nums: list[int] = [1, 2, 3]\n    n: int = len(nums)\n    result: list[int] = []\n    for i in range(n):\n        result.append(1)\n    i: int = n - 1\n    while i >= 0:\n        reveal_type(result[i])\n        i -= 1\n",
        )
        .expect("append-growth sized list should narrow under alias-backed index guard");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_early_return_non_empty_guard_reveals_element_type() {
        let result = lower_source_result(
            "def head(nums: list[int]) -> int:\n    if len(nums) == 0:\n        return 0\n    reveal_type(nums[0])\n    return nums[0]\n",
        )
        .expect("early-return non-empty guard should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
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
    fn test_early_return_method_len_guard_narrows_index_type() {
        let result = lower_source_result(
            "def pick(values: list[int], i: int) -> int:\n    if i >= values.len():\n        return 0\n    reveal_type(values[i])\n    return values[i]\n",
        )
        .expect("post-return method len guard should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_early_return_len_lt_guard_narrows_fixed_index_type() {
        let result = lower_source_result(
            "def pick(values: list[int]) -> int:\n    if len(values) < 2:\n        return 0\n    reveal_type(values[1])\n    return values[1]\n",
        )
        .expect("post-return len(values) < 2 guard should narrow values[1]");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_early_return_len_lte_guard_narrows_fixed_index_type() {
        let result = lower_source_result(
            "def pick(values: list[int]) -> int:\n    if len(values) <= 1:\n        return 0\n    reveal_type(values[1])\n    return values[1]\n",
        )
        .expect("post-return len(values) <= 1 guard should narrow values[1]");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_early_return_or_guard_narrows_index_type() {
        let result = lower_source_result(
            "def pick(values: list[int], i: int, limit: int) -> int:\n    if i >= len(values) or limit < 0:\n        return 0\n    reveal_type(values[i])\n    return values[i]\n",
        )
        .expect("post-return or guard should narrow the guarded index");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
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
                .contains("type mismatch: expected 'int', got 'None | int'")
        }));
    }

    #[test]
    fn test_subscript_none_guard_after_early_return_narrows_repeated_read() {
        let result = lower_source(
            "def pick(children: list[int | None], i: int) -> int:\n    if children[i] is None:\n        return 0\n    value: int = children[i]\n    return value\n",
        );
        assert!(
            result.is_ok(),
            "post-return `if seq[i] is None` guard should narrow repeated subscript reads"
        );
    }

    #[test]
    fn test_subscript_is_not_none_guard_narrows_true_branch_read() {
        let result = lower_source(
            "def pick(children: list[int | None], i: int) -> int:\n    if children[i] is not None:\n        return children[i]\n    return 0\n",
        );
        assert!(
            result.is_ok(),
            "`if seq[i] is not None` should narrow repeated subscript reads in the true branch"
        );
    }

    #[test]
    fn test_dict_index_narrows_after_in_membership_guard() {
        let result = lower_source(
            "def main():\n    table: dict[int, int] = {1: 10}\n    key: int = 1\n    if key in table:\n        value: int = table[key]\n",
        );
        assert!(
            result.is_ok(),
            "dict index should narrow to value type when guarded by `key in dict`"
        );
    }

    #[test]
    fn test_dict_string_literal_index_narrows_after_membership_guard() {
        let result = lower_source(
            "def main():\n    table: dict[str, list[int]] = {}\n    table[\"a\"] = [1, 2]\n    if \"a\" in table:\n        table[\"a\"].append(3)\n        table[\"a\"].pop()\n",
        );
        assert!(
            result.is_ok(),
            "string-literal dict indexes should narrow under the matching membership guard"
        );
    }

    #[test]
    fn test_dict_index_narrows_after_keys_membership_guard_with_expression_key() {
        let result = lower_source(
            "def main():\n    table: dict[int, int] = {1: 10}\n    base: int = 0\n    if base + 1 in table.keys():\n        value: int = table[base + 1]\n",
        );
        assert!(
            result.is_ok(),
            "dict index should narrow for matching expression key under `key in dict.keys()` guard"
        );
    }

    #[test]
    fn test_dict_index_narrows_after_tuple_membership_guard() {
        let result = lower_source(
            "def main():\n    table: dict[tuple[int, bool], int] = {(1, True): 10}\n    i: int = 1\n    buying: bool = True\n    if (i, buying) in table:\n        value: int = table[(i, buying)]\n",
        );
        assert!(
            result.is_ok(),
            "dict index should narrow for tuple key guards used by memoization caches"
        );
    }

    #[test]
    fn test_dict_index_narrows_after_not_in_early_return_guard() {
        let result = lower_source(
            "def pick(table: dict[int, int], key: int) -> int:\n    if key not in table:\n        return 0\n    return table[key]\n",
        );
        assert!(
            result.is_ok(),
            "post-guard dict index should narrow after `if key not in dict: return`"
        );
    }

    #[test]
    fn test_dict_subscript_assignment_establishes_key_presence_for_following_read() {
        let result = lower_source(
            "def pick(i: int, buying: bool) -> int:\n    cache = {}\n    cache[(i, buying)] = 7\n    return cache[(i, buying)]\n",
        );
        assert!(
            result.is_ok(),
            "dict subscript assignment should establish key presence for a dominated read"
        );
    }

    #[test]
    fn test_dict_key_presence_survives_exhaustive_if_branch_merge() {
        let result = lower_source(
            "def pick(flag: bool, i: int, buying: bool) -> int:\n    cache = {}\n    if flag:\n        cache[(i, buying)] = 1\n    else:\n        cache[(i, buying)] = 2\n    return cache[(i, buying)]\n",
        );
        assert!(
            result.is_ok(),
            "key-presence guards established in every if branch should survive the merge"
        );
    }

    #[test]
    fn test_index_narrows_after_equal_len_early_return() {
        let result = lower_source(
            "def pick(values: list[int], i: int) -> int:\n    if i == len(values):\n        return 0\n    return values[i]\n",
        );
        assert!(
            result.is_ok(),
            "post-return `i == len(values)` should permit guarded index typing"
        );
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
            .filter(|diagnostic| diagnostic.revealed_type == "int")
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
                .contains("type mismatch: expected 'int', got 'None | int'")
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
            .filter(|diagnostic| diagnostic.revealed_type == "int")
            .count();
        assert_eq!(reveal_count, 2);
    }

    #[test]
    fn test_attribute_non_empty_guard_reveals_head_index_type() {
        let result = lower_source_result(
            "class Box:\n    values: list[int]\n\n    def __init__(self):\n        self.values = [1, 2]\n\n    def head(self) -> int:\n        if not self.values:\n            return 0\n        reveal_type(self.values[0])\n        return self.values[0]\n",
        )
        .expect("non-empty attribute guard should narrow head index");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_sliding_window_left_pointer_reveals_element_type_before_single_step_increment() {
        let result = lower_source_result(
            "def main(text: str, k: int) -> int:\n    l: int = 0\n    total: int = 0\n    for r in range(len(text)):\n        if (r - l + 1) > k:\n            reveal_type(text[l])\n            l += 1\n        if text[r] == \"a\":\n            total += 1\n    return total\n",
        )
        .expect("sliding-window left pointer should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "str")
        );
    }

    #[test]
    fn test_tuple_unpack_sliding_window_left_pointer_reveals_element_type() {
        let result = lower_source_result(
            "def main(text: str, k: int) -> int:\n    l, total = 0, 0\n    vowels = \"aeiou\"\n    for r in range(len(text)):\n        if (r - l + 1) > k:\n            reveal_type(text[l])\n            if text[l] in vowels:\n                total -= 1\n            l += 1\n        if text[r] in vowels:\n            total += 1\n    return total\n",
        )
        .expect("tuple-unpacked sliding-window left pointer should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "str")
        );
    }

    #[test]
    fn test_sliding_window_left_pointer_stays_optional_after_incremented_branch_merges() {
        let result = lower_source(
            "def main(text: str, k: int) -> str:\n    l: int = 0\n    for r in range(len(text)):\n        if (r - l + 1) > k:\n            l += 1\n        current: str = text[l]\n    return \"\"\n",
        );

        assert!(
            result.is_err(),
            "post-branch reads after a potential left-pointer increment should remain optional"
        );
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|error| {
            error
                .message
                .contains("type mismatch: expected 'str', got 'None | str'")
        }));
    }

    #[test]
    fn test_reverse_range_suffix_recurrence_reveals_int() {
        let result = lower_source_result(
            "def main(text: str) -> list[int]:\n    suffix = [0 for i in range(len(text) + 1)]\n    for i in range(len(text) - 1, -1, -1):\n        reveal_type(suffix[i + 1])\n        suffix[i] = suffix[i + 1] + 1\n    return suffix\n",
        )
        .expect("reverse range recurrence should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_matrix_recurrence_offsets_reveal_int() {
        let result = lower_source_result(
            "def main(text1: str, text2: str) -> int:\n    dp = [[0 for j in range(len(text2) + 1)] for i in range(len(text1) + 1)]\n    for i in range(len(text1) - 1, -1, -1):\n        for j in range(len(text2) - 1, -1, -1):\n            reveal_type(dp[i + 1][j + 1])\n            dp[i][j] = dp[i + 1][j + 1] + 1\n    return dp[0][0]\n",
        )
        .expect("matrix recurrence offsets should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_matrix_singleton_repeat_rows_allow_nested_fixed_index_reads() {
        let result = lower_source(
            "def main(s: str, p: str) -> bool:\n    cache = [[False] * (len(p) + 1) for i in range(len(s) + 1)]\n    return cache[0][0]\n",
        );
        assert!(
            result.is_ok(),
            "matrix rows built from singleton-list repetition should retain inner length anchors"
        );
    }

    #[test]
    fn test_reverse_range_suffix_plus_two_offset_reveals_int() {
        let result = lower_source_result(
            "def main(text: str) -> list[int]:\n    suffix = [0 for i in range(len(text) + 2)]\n    for i in range(len(text) - 1, -1, -1):\n        reveal_type(suffix[i + 2])\n        suffix[i] = suffix[i + 2] + 1\n    return suffix\n",
        )
        .expect("reverse range +2 recurrence should lower");

        assert!(
            result
                .reveal_types
                .iter()
                .any(|diagnostic| diagnostic.revealed_type == "int")
        );
    }

    #[test]
    fn test_subtractive_recurrence_offset_stays_optional() {
        let result = lower_source(
            "def main(limit: str, shift: int) -> list[int]:\n    suffix = [0 for i in range(len(limit) + 1)]\n    for i in range(len(limit) - 1, -1, -1):\n        value: int = suffix[i - shift]\n    return suffix\n",
        );

        assert!(
            result.is_err(),
            "subtractive offsets without lower-bound proof should remain optional"
        );
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|error| {
            error
                .message
                .contains("type mismatch: expected 'int', got 'None | int'")
        }));
    }
}
