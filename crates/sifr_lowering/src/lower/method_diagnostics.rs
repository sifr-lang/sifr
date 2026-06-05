use super::expression_diagnostics;
use super::LowerCtx;
use ruff_text_size::TextRange;

pub(in crate::lower) fn method_count_range(
    actual: usize,
    max_allowed: usize,
    arg_ranges: &[TextRange],
    method_range: TextRange,
) -> TextRange {
    if actual > max_allowed {
        arg_ranges.get(max_allowed).copied().unwrap_or(method_range)
    } else {
        method_range
    }
}

pub(in crate::lower) fn reject_method_arg_count(
    ctx: &mut LowerCtx,
    message: String,
    range: TextRange,
) {
    expression_diagnostics::call_wrong_positional_count(ctx, message, range);
}

pub(in crate::lower) fn reject_exact_method_arg_count(
    ctx: &mut LowerCtx,
    method: &str,
    expected: usize,
    actual: usize,
    arg_ranges: &[TextRange],
    method_range: TextRange,
) {
    let suffix = if expected == 1 { "" } else { "s" };
    reject_method_arg_count(
        ctx,
        format!("{method}() takes exactly {expected} argument{suffix}, got {actual}"),
        method_count_range(actual, expected, arg_ranges, method_range),
    );
}

pub(in crate::lower) fn reject_max_method_arg_count(
    ctx: &mut LowerCtx,
    method: &str,
    max_allowed: usize,
    actual: usize,
    arg_ranges: &[TextRange],
    method_range: TextRange,
) {
    let suffix = if max_allowed == 1 { "" } else { "s" };
    reject_method_arg_count(
        ctx,
        format!("{method}() takes at most {max_allowed} argument{suffix}, got {actual}"),
        method_count_range(actual, max_allowed, arg_ranges, method_range),
    );
}

pub(in crate::lower) fn reject_no_method_args(
    ctx: &mut LowerCtx,
    method: &str,
    arg_ranges: &[TextRange],
    method_range: TextRange,
) {
    reject_method_arg_count(
        ctx,
        format!("{method}() takes no arguments"),
        method_count_range(arg_ranges.len(), 0, arg_ranges, method_range),
    );
}
