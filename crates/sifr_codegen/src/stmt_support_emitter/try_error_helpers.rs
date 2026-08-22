use super::{HirExceptHandler, HirStmt, RustExpr, Type};

pub(crate) fn successful_try_bindings(
    body: &[HirStmt],
    handlers: &[HirExceptHandler],
    following_stmts: Option<&[HirStmt]>,
) -> Vec<(String, Type)> {
    successful_try_binding_candidates(body, handlers)
        .into_iter()
        .filter(|(name, _)| {
            following_stmts.is_none_or(|following| {
                crate::hir_analysis::queries::stmts_require_var_value_at_entry_including_nested_functions(
                    following, name,
                )
            })
        })
        .collect()
}

pub(crate) fn declaration_only_try_bindings(
    body: &[HirStmt],
    handlers: &[HirExceptHandler],
    following_stmts: Option<&[HirStmt]>,
) -> Vec<(String, Type)> {
    let Some(following) = following_stmts else {
        return Vec::new();
    };
    successful_try_binding_candidates(body, handlers)
        .into_iter()
        .filter(|(name, _)| {
            crate::hir_analysis::queries::stmts_reference_var_including_nested_functions(
                following, name,
            ) && !crate::hir_analysis::queries::stmts_require_var_value_at_entry_including_nested_functions(
                following, name,
            )
        })
        .collect()
}

fn successful_try_binding_candidates(
    body: &[HirStmt],
    handlers: &[HirExceptHandler],
) -> Vec<(String, Type)> {
    if crate::hir_analysis::queries::block_control_flow_effect(body).always_exits()
        || handlers.iter().any(|handler| {
            !crate::hir_analysis::queries::block_control_flow_effect(&handler.body).always_exits()
        })
    {
        return Vec::new();
    }

    let mut names = std::collections::HashSet::new();
    body.iter()
        .filter_map(|stmt| {
            let HirStmt::Let { name, ty, .. } = stmt else {
                return None;
            };
            (name != "_" && names.insert(name.clone())).then(|| (name.clone(), ty.clone()))
        })
        .collect()
}

pub(crate) fn io_error_kind_for_handler(error_type: &str) -> Option<&'static str> {
    sifr_type_system::io_error_kind(error_type)
}

pub(crate) fn select_try_error_type(handlers: &[HirExceptHandler]) -> String {
    if handlers.iter().any(|handler| {
        let Some(error_type) = handler.error_type.as_deref() else {
            return false;
        };
        error_type == "IOError" || io_error_kind_for_handler(error_type).is_some()
    }) {
        return "IOError".to_string();
    }

    handlers
        .first()
        .and_then(|handler| handler.error_resolved_type.as_ref())
        .map(|ty| crate::render_type(&crate::sifr_type_to_rust_type(ty)))
        .unwrap_or_else(|| "Error".to_string())
}

pub(crate) fn first_try_error_type_in_stmts(stmts: &[HirStmt]) -> Option<String> {
    for stmt in stmts {
        if let Some(error_type) = first_try_error_type_in_stmt(stmt) {
            return Some(error_type);
        }
    }
    None
}

fn first_try_error_type_in_stmt(stmt: &HirStmt) -> Option<String> {
    match stmt {
        HirStmt::TryExcept {
            body,
            handlers,
            body_error_types,
        } => body_error_types
            .first()
            .map(|error_ty| crate::render_type(&crate::sifr_type_to_rust_type(error_ty)))
            .or_else(|| {
                first_try_error_type_in_stmts(body).or_else(|| {
                    handlers
                        .iter()
                        .find_map(|handler| first_try_error_type_in_stmts(&handler.body))
                })
            }),
        HirStmt::TryFinally { body, finalbody } => {
            first_try_error_type_in_stmts(body).or_else(|| first_try_error_type_in_stmts(finalbody))
        }
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => first_try_error_type_in_stmts(then_body)
            .or_else(|| {
                elif_clauses
                    .iter()
                    .find_map(|(_, body)| first_try_error_type_in_stmts(body))
            })
            .or_else(|| else_body.as_deref().and_then(first_try_error_type_in_stmts)),
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        } => first_try_error_type_in_stmts(body)
            .or_else(|| else_body.as_deref().and_then(first_try_error_type_in_stmts)),
        HirStmt::With { body, .. }
        | HirStmt::AsyncWith { body, .. }
        | HirStmt::AsyncFor { body, .. } => first_try_error_type_in_stmts(body),
        HirStmt::NestedFunction { func, .. } => first_try_error_type_in_stmts(&func.body),
        HirStmt::Match { arms, .. } => arms
            .iter()
            .find_map(|arm| first_try_error_type_in_stmts(&arm.body)),
        _ => None,
    }
}

pub(crate) fn can_construct_error_from_message_for_ir(ty_name: &str) -> bool {
    matches!(
        ty_name,
        "Error"
            | "ValueError"
            | "TypeError"
            | "NameError"
            | "ParseError"
            | "OverflowError"
            | "ZeroDivisionError"
            | "LookupError"
            | "IndexError"
            | "KeyError"
            | "RuntimeError"
            | "AssertionError"
            | "ImportError"
            | "IOError"
            | "RegexError"
            | "JsonIntegerRangeError"
            | "JsonLimitError"
            | "HashlibError"
            | "DecimalConversionError"
            | "TimeoutError"
            | "ScopeFailure"
            | "TaskCancelled"
            | "SecondaryError"
    )
}

pub(crate) enum HandlerMatchCondition {
    Unsupported,
    Always,
    Expr(RustExpr),
}
