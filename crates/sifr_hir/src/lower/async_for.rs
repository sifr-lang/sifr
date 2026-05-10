use super::expressions::lower_expr;
use super::statement_diagnostics;
use super::statements::lower_stmts;
use super::LowerCtx;
use crate::hir_nodes::HirStmt;
use ruff_text_size::{Ranged, TextRange};
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, StmtFor};
use sifr_type_system::{FunctionType, Type};

fn method_signature<'a>(
    methods: &'a [(String, FunctionType)],
    method_name: &str,
) -> Option<&'a FunctionType> {
    methods.iter().find_map(
        |(name, ft)| {
            if name == method_name {
                Some(ft)
            } else {
                None
            }
        },
    )
}

fn async_result_parts(ty: &Type) -> Option<(Type, Type)> {
    let Type::Coroutine(ok_ty, err_ty) = ty.resolve_alias() else {
        return None;
    };
    Some((ok_ty.as_ref().clone(), err_ty.as_ref().clone()))
}

fn option_value_type(ty: &Type) -> Option<Type> {
    let Type::Union(members) = ty.resolve_alias() else {
        return None;
    };
    let has_none = members
        .iter()
        .any(|member| matches!(member.resolve_alias(), Type::None));
    if !has_none {
        return None;
    }
    let non_none = members
        .iter()
        .filter(|member| !matches!(member.resolve_alias(), Type::None))
        .cloned()
        .collect::<Vec<_>>();
    if non_none.len() == 1 {
        non_none.into_iter().next()
    } else {
        None
    }
}

fn async_iterator_parts(ty: &Type) -> Option<(Type, Type)> {
    match ty.resolve_alias() {
        Type::AsyncIterator(item_ty, err_ty) | Type::AsyncGenerator(item_ty, err_ty) => {
            Some((item_ty.as_ref().clone(), err_ty.as_ref().clone()))
        }
        Type::Class { methods, .. } | Type::Protocol { methods, .. } => {
            let anext_ft = method_signature(methods, "anext")?;
            if !anext_ft.params.is_empty() {
                return None;
            }
            let (next_ty, err_ty) = async_result_parts(&anext_ft.return_type)?;
            let item_ty = option_value_type(&next_ty)?;
            Some((item_ty, err_ty))
        }
        _ => None,
    }
}

fn return_type_accepts_error(return_type: &Type, error_ty: &Type) -> bool {
    if matches!(error_ty.resolve_alias(), Type::Never) {
        return true;
    }
    let Type::Result(_, err) = return_type.resolve_alias() else {
        return false;
    };
    error_ty.is_assignable_to(err)
}

fn simple_for_target_name(target: &Expr, ctx: &mut LowerCtx) -> Option<(String, TextRange)> {
    if let Expr::Name(name) = target {
        Some((name.id.to_string(), target.range()))
    } else {
        statement_diagnostics::invalid_iteration(
            ctx,
            "async for target must be a simple name",
            target.range(),
        );
        None
    }
}

pub(super) fn lower_async_for(
    for_stmt: &StmtFor,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    if !ctx.current_function_is_async {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "async for is only valid inside async functions".to_string(),
            for_stmt.range(),
        );
        return None;
    }
    if !for_stmt.orelse.is_empty() {
        statement_diagnostics::unsupported_form(
            ctx,
            "async for else clauses are not supported in v1",
            for_stmt.range(),
        );
        return None;
    }

    let iter = lower_expr(&for_stmt.iter, ctx)?;
    let iter_ty = iter.ty().clone();
    let Some((target_ty, iter_error_ty)) = async_iterator_parts(&iter_ty) else {
        statement_diagnostics::invalid_iteration(
            ctx,
            &format!(
                "async for requires AsyncIterator[T, E] with anext() -> Result[Option[T], E], got '{}'",
                iter_ty.display_name()
            ),
            for_stmt.iter.range(),
        );
        return None;
    };
    if !return_type_accepts_error(&func_type.return_type, &iter_error_ty) {
        ctx.error_with_code_at(
            DiagnosticCode::TYPE_MISMATCH,
            "fallible async iterator requires the enclosing function to return a compatible Result error type".to_string(),
            for_stmt.iter.range(),
        );
        return None;
    }

    let (target, _) = simple_for_target_name(&for_stmt.target, ctx)?;
    ctx.scope.push();
    ctx.scope.define(target.clone(), target_ty.clone());
    ctx.loop_depth += 1;
    let body = lower_stmts(&for_stmt.body, func_type, ctx);
    ctx.loop_depth -= 1;
    ctx.scope.pop();

    Some(HirStmt::AsyncFor {
        target,
        target_ty,
        iter,
        iter_error_ty,
        body,
        else_body: None,
    })
}
