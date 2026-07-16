use super::{nested_function_inference, LowerCtx, Stmt, Type};

pub(super) fn infer_unannotated_returns(stmts: &[Stmt], ctx: &mut LowerCtx) {
    // This pass only seeds mutually visible signatures. Normal body lowering remains the
    // diagnostic authority and has the precise reachability information needed to ignore dead
    // return expressions.
    let existing_error_count = ctx.errors.len();
    let inferred = nested_function_inference::infer_module_function_types(stmts, ctx);
    ctx.errors.truncate(existing_error_count);
    for stmt in stmts {
        let Stmt::FunctionDef(function) = stmt else {
            continue;
        };
        if function.returns.is_some() {
            continue;
        }
        let name = function.name.as_str();
        let Some(inferred_type) = inferred.function_types.get(name) else {
            continue;
        };
        if matches!(
            inferred_type.return_type.resolve_alias(),
            Type::Any | Type::Unknown
        ) {
            continue;
        }
        if let Some(function_type) = ctx.functions.get_mut(name) {
            function_type
                .return_type
                .clone_from(&inferred_type.return_type);
        }
    }
}
