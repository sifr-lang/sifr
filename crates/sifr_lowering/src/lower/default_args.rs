use super::{LowerCtx, simple_expr::lower_expr_simple};
use crate::hir_nodes::HirExpr;
use ruff_text_size::Ranged;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{ParameterWithDefault, StmtFunctionDef};

use super::python_interop::{has_python_interop_decorator_syntax, is_python_omit};

pub(in crate::lower) fn collect_function_defaults(
    ctx: &mut LowerCtx,
    function_name: &str,
    func: &StmtFunctionDef,
) {
    let mut defaults = Vec::new();
    for (index, param) in func.parameters.args.iter().enumerate() {
        collect_param_default(ctx, &mut defaults, index, func, param);
    }
    let regular_count = func.parameters.args.len() + usize::from(func.parameters.vararg.is_some());
    for (index, param) in func.parameters.kwonlyargs.iter().enumerate() {
        collect_param_default(ctx, &mut defaults, regular_count + index, func, param);
    }
    if !defaults.is_empty() {
        ctx.function_defaults
            .insert(function_name.to_string(), defaults);
    }
}

fn collect_param_default(
    ctx: &mut LowerCtx,
    defaults: &mut Vec<(usize, HirExpr)>,
    index: usize,
    func: &StmtFunctionDef,
    param: &ParameterWithDefault,
) {
    if has_python_interop_decorator_syntax(&func.decorator_list)
        && param.default.as_deref().is_some_and(is_python_omit)
    {
        return;
    }
    if let Some(ref default_expr) = param.default {
        if let Some(hir_default) = lower_expr_simple(default_expr) {
            defaults.push((index, hir_default));
        } else {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_UNSUPPORTED_DEFAULT_ARGUMENT,
                format!(
                    "function '{}': unsupported default argument expression for parameter '{}'",
                    func.name, param.parameter.name
                ),
                default_expr.range(),
            );
        }
    }
}
