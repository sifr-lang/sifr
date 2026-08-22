use super::LowerCtx;
use ruff_text_size::TextRange;
use sifr_type_system::{FunctionType, Type};
use std::collections::HashSet;

pub(super) fn record_or_reject_unmatched_try_errors(
    try_error_types: &HashSet<Type>,
    covered_types: &HashSet<Type>,
    has_catch_all: bool,
    enclosing_try_accepts_errors: bool,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
    range: TextRange,
) {
    if has_catch_all || try_error_types.is_empty() {
        return;
    }

    let unmatched = try_error_types
        .iter()
        .filter(|error_ty| {
            !covered_types
                .iter()
                .any(|covered| error_ty.is_assignable_to(covered))
        })
        .cloned()
        .collect::<Vec<_>>();
    if unmatched.is_empty() {
        return;
    }

    if enclosing_try_accepts_errors {
        ctx.try_block_error_types.extend(unmatched);
        return;
    }

    let return_error_type = match func_type.return_type.resolve_alias() {
        Type::Result(_, error_type) => Some(error_type.as_ref()),
        _ => None,
    };
    let mut rejected = unmatched
        .into_iter()
        .filter(|error_ty| {
            !return_error_type.is_some_and(|target| error_ty.is_assignable_to(target))
        })
        .map(|error_ty| error_ty.display_name())
        .collect::<Vec<_>>();
    if rejected.is_empty() {
        return;
    }

    rejected.sort();
    super::result_diagnostics::uncovered_try_errors(ctx, &rejected.join(", "), range);
}
