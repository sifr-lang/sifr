//! Resolve the dispatch declaration for every typed method call before HIR publication.

use super::LowerCtx;
use crate::hir_nodes::{HirExpr, HirModule, HirStmt};
use ruff_text_size::TextRange;
use sifr_ir::{
    CallableIdentity, MethodAuthority, visit_hir_function_exprs_mut, visit_hir_stmts_exprs_mut,
};
use sifr_type_system::{FunctionType, Type};
use std::collections::HashSet;

pub(super) struct AuthorityViolation {
    pub message: String,
    pub range: TextRange,
}

pub(super) fn classify_module(
    module: &mut HirModule,
    ctx: &LowerCtx<'_>,
) -> Vec<AuthorityViolation> {
    let mut violations = Vec::new();
    let mut classify_expr = |expr: &mut HirExpr| {
        let HirExpr::MethodCall {
            object,
            method,
            args,
            ty,
            source,
            authority,
            ..
        } = expr
        else {
            return;
        };
        if !matches!(authority, MethodAuthority::Unclassified) {
            return;
        }
        if let Some(resolved) = classify_call(object.ty(), method, args, ty, ctx) {
            *authority = resolved;
        } else {
            violations.push(AuthorityViolation {
                message: format!(
                    "method '{}' is unsupported on resolved receiver type '{}'",
                    method,
                    object.ty().display_name()
                ),
                range: source.as_ref().map_or_default(|source| source.call_range),
            });
        }
    };
    for function in &mut module.functions {
        visit_hir_function_exprs_mut(function, &mut classify_expr);
    }
    for class in &mut module.classes {
        for method in class
            .methods
            .iter_mut()
            .chain(class.operator_impls.iter_mut().map(|(_, method)| method))
        {
            visit_hir_function_exprs_mut(method, &mut classify_expr);
        }
        for (_, default) in &mut class.field_defaults {
            visit_stored_expr(default, &mut classify_expr);
        }
    }
    for (_, _, value) in &mut module.constants {
        visit_stored_expr(value, &mut classify_expr);
    }
    violations
}

fn visit_stored_expr(expr: &mut HirExpr, visit: &mut impl FnMut(&mut HirExpr)) {
    let mut statement = [HirStmt::Expr {
        expr: std::mem::replace(expr, HirExpr::NoneLiteral),
    }];
    visit_hir_stmts_exprs_mut(&mut statement, visit);
    if let HirStmt::Expr { expr: updated } = &mut statement[0] {
        *expr = std::mem::replace(updated, HirExpr::NoneLiteral);
    }
}

fn classify_call(
    receiver: &Type,
    method: &str,
    args: &[HirExpr],
    return_ty: &Type,
    ctx: &LowerCtx<'_>,
) -> Option<MethodAuthority> {
    let mut receiver = receiver.resolve_alias();
    let mut type_vars = HashSet::new();
    while let Type::TypeVar(name) = receiver {
        if !type_vars.insert(name) {
            return None;
        }
        receiver = ctx.class_types.get(name)?.resolve_alias();
    }
    if method == "clone" && receiver.supports_derived_clone() {
        return Some(builtin(receiver, method, args, return_ty));
    }
    match receiver {
        Type::Class {
            identity,
            type_args,
            name,
            methods,
            fields,
            ..
        } => {
            if is_compiler_task_owner(receiver, method) {
                return Some(builtin(receiver, method, args, return_ty));
            }
            if receiver.is_python_object_contract()
                && super::expressions::python_raw_object_methods::is_raw_method(method)
            {
                return Some(builtin(receiver, method, args, return_ty));
            }
            let signature = methods
                .iter()
                .find(|(candidate, _)| candidate == method)
                .map(|(_, signature)| signature.clone())
                .or_else(|| {
                    fields
                        .iter()
                        .find(|(candidate, _)| candidate == method)
                        .and_then(|(_, field)| callable_field_signature(field))
                })?;
            let origin_name = ctx
                .class_method_origins
                .get(&format!("{name}.{method}"))
                .map_or(name.as_str(), String::as_str);
            let defining_identity = if origin_name == name {
                identity.clone().unwrap_or_else(|| name.clone())
            } else {
                ctx.class_types
                    .get(origin_name)
                    .and_then(|ty| match ty.resolve_alias() {
                        Type::Class { identity, .. } => identity.clone(),
                        _ => None,
                    })
                    .unwrap_or_else(|| origin_name.to_string())
            };
            let declaration = nominal_declaration(
                &defining_identity,
                method,
                type_args,
                &signature,
                ctx.current_module_name.as_deref(),
            );
            if let Some(binding) =
                super::attached_api_surfaces::binding_for_owner(ctx, name, receiver, method)
            {
                Some(MethodAuthority::RustAdapted {
                    declaration: CallableIdentity {
                        module: binding.declaration.module.clone(),
                        owner: None,
                        symbol: binding.declaration.function.clone(),
                        generic_arguments: type_args.iter().map(Type::display_name).collect(),
                        signature: format!("{:?}", binding.declaration.function_type),
                    },
                })
            } else if ctx.rust_opaque_classes.contains(origin_name)
                || ctx.rust_structural_classes.contains(origin_name)
            {
                Some(MethodAuthority::RustAdapted { declaration })
            } else if origin_name != name {
                Some(MethodAuthority::InheritedNominal { declaration })
            } else if declaration.module != ctx.current_module_name.as_deref().unwrap_or_default() {
                Some(MethodAuthority::Imported { declaration })
            } else {
                Some(MethodAuthority::LocalNominal { declaration })
            }
        }
        Type::Protocol {
            identity,
            name,
            methods,
        } => {
            let signature = methods
                .iter()
                .find(|(candidate, _)| candidate == method)
                .map(|(_, signature)| signature)?;
            let declaration = nominal_declaration(
                identity.as_deref().unwrap_or(name),
                method,
                &[],
                signature,
                ctx.current_module_name.as_deref(),
            );
            Some(MethodAuthority::Protocol { declaration })
        }
        Type::Any | Type::Unknown | Type::Never | Type::Function(_) | Type::AsyncFunction(_) => {
            None
        }
        Type::Callable(_, _, _) | Type::AsyncCallable(_, _, _) if method == "__call__" => {
            Some(builtin(receiver, method, args, return_ty))
        }
        Type::Task(_, _) | Type::BlockingTask(_, _)
            if matches!(
                method,
                "join" | "cancel" | "cancel_and_join" | "__sifr_timeout"
            ) =>
        {
            Some(builtin(receiver, method, args, return_ty))
        }
        Type::JoinSet(_, _)
            if matches!(
                method,
                "__sifr_add_task"
                    | "__sifr_add_blocking_task"
                    | "__sifr_spawn_blocking"
                    | "__sifr_spawn_cpu"
                    | "__sifr_join_all"
                    | "__sifr_cancel_all"
            ) =>
        {
            Some(builtin(receiver, method, args, return_ty))
        }
        Type::Newtype { inner, .. } => {
            if method == "value" {
                Some(builtin(receiver, method, args, return_ty))
            } else {
                classify_call(inner, method, args, return_ty, ctx)
            }
        }
        Type::Enum { identity, name, .. } => {
            if matches!(method, "name" | "value") {
                return Some(builtin(receiver, method, args, return_ty));
            }
            let signature = ctx.functions.get(&format!("{name}.{method}"))?;
            let declaration = nominal_declaration(
                identity.as_deref().unwrap_or(name),
                method,
                &[],
                signature,
                ctx.current_module_name.as_deref(),
            );
            if declaration.module == ctx.current_module_name.as_deref().unwrap_or_default() {
                Some(MethodAuthority::LocalNominal { declaration })
            } else {
                Some(MethodAuthority::Imported { declaration })
            }
        }
        Type::List(_)
        | Type::Dict(_, _)
        | Type::Set(_)
        | Type::Str
        | Type::Bytes
        | Type::FixedInt(_)
        | Type::Tuple(_)
        | Type::PythonBuffer(_)
        | Type::PythonArrow(_)
        | Type::PythonDlpackTensor(_)
        | Type::PythonDlpackStream
        | Type::AsyncGenerator(_, _)
        | Type::StructuralRecord(_)
        | Type::Decimal
        | Type::BigDecimal => Some(builtin(receiver, method, args, return_ty)),
        _ => None,
    }
}

fn callable_field_signature(field: &Type) -> Option<FunctionType> {
    match field.resolve_alias() {
        Type::Callable(params, conventions, return_type)
        | Type::AsyncCallable(params, conventions, return_type) => Some(FunctionType {
            receiver: None,
            params: params
                .iter()
                .zip(conventions)
                .enumerate()
                .map(|(index, (ty, convention))| (format!("arg{index}"), ty.clone(), *convention))
                .collect(),
            return_type: return_type.clone(),
        }),
        _ => None,
    }
}

fn nominal_declaration(
    identity: &str,
    method: &str,
    type_args: &[Type],
    signature: &FunctionType,
    current_module: Option<&str>,
) -> CallableIdentity {
    let (module, owner) = identity.rsplit_once('.').map_or_else(
        || {
            (
                current_module.unwrap_or_default().to_string(),
                identity.to_string(),
            )
        },
        |(module, owner)| (module.to_string(), owner.to_string()),
    );
    CallableIdentity {
        module,
        owner: Some(owner),
        symbol: method.to_string(),
        generic_arguments: type_args.iter().map(Type::display_name).collect(),
        signature: format!("{signature:?}"),
    }
}

fn builtin(receiver: &Type, method: &str, args: &[HirExpr], return_ty: &Type) -> MethodAuthority {
    MethodAuthority::BuiltinIntrinsic {
        declaration: CallableIdentity {
            module: "sifr.builtin".to_string(),
            owner: Some(receiver.display_name()),
            symbol: method.to_string(),
            generic_arguments: Vec::new(),
            signature: format!(
                "({}) -> {}",
                args.iter()
                    .map(|arg| arg.ty().display_name())
                    .collect::<Vec<_>>()
                    .join(", "),
                return_ty.display_name()
            ),
        },
    }
}

fn is_compiler_task_owner(receiver: &Type, method: &str) -> bool {
    let is_owner = receiver == &super::task_owner_scope_state::task_scope_type()
        || receiver == &super::task_owner_scope_state::task_group_type();
    is_owner
        && matches!(
            method,
            "__sifr_spawn_infallible"
                | "__sifr_spawn_infallible_with_context"
                | "__sifr_spawn_result"
                | "__sifr_spawn_result_with_context"
                | "__sifr_scope_spawn_blocking_infallible"
                | "__sifr_scope_spawn_blocking_result"
                | "__sifr_scope_spawn_cpu_infallible"
                | "__sifr_scope_spawn_cpu_result"
                | "__sifr_scope_spawn_process"
        )
}
