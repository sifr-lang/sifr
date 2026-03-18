use crate::hir_nodes::HirStmt;
use sifr_type_system::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum LazyGeneratorShapeError {
    MissingTopLevelWhileLoop,
    YieldBeforeLoop,
    TrailingStatementsAfterLoop,
    MissingYieldSite,
    MultipleTopLevelYields,
    MultipleConditionalYieldSites,
    MixedTopLevelAndConditionalYield,
    NestedYieldNotSupported,
}

impl LazyGeneratorShapeError {
    pub(super) fn message(self, function_name: &str) -> String {
        let detail = match self {
            Self::MissingTopLevelWhileLoop => "must lower through a single top-level while loop",
            Self::YieldBeforeLoop => "cannot yield before the generator while loop begins",
            Self::TrailingStatementsAfterLoop => {
                "cannot have trailing statements after the generator while loop"
            }
            Self::MissingYieldSite => {
                "must contain exactly one supported yield site inside the while loop"
            }
            Self::MultipleTopLevelYields => {
                "currently supports exactly one top-level yield statement in the while loop"
            }
            Self::MultipleConditionalYieldSites => {
                "currently supports exactly one if-guarded yield site in the while loop"
            }
            Self::MixedTopLevelAndConditionalYield => {
                "cannot mix top-level yield and if-guarded yield forms in the same while loop"
            }
            Self::NestedYieldNotSupported => {
                "only direct top-level yield or a direct if-guarded yield is currently supported"
            }
        };
        format!(
            "unsupported lazy generator shape for '{}': {}",
            function_name, detail
        )
    }
}

/// Collect all return types from a list of HIR statements (recursively).
pub(super) fn collect_return_types(stmts: &[HirStmt]) -> Vec<Type> {
    crate::cfg::flow_facts(stmts)
        .reachable_return_types()
        .to_vec()
}

/// Collect all yielded expression types from a list of HIR statements (recursively).
pub(super) fn collect_yield_types(stmts: &[HirStmt]) -> Vec<Type> {
    fn walk(stmts: &[HirStmt], out: &mut Vec<Type>) {
        for stmt in stmts {
            match stmt {
                HirStmt::Yield { value } => out.push(value.ty().clone()),
                HirStmt::If {
                    then_body,
                    elif_clauses,
                    else_body,
                    ..
                } => {
                    walk(then_body, out);
                    for (_, body) in elif_clauses {
                        walk(body, out);
                    }
                    if let Some(else_body) = else_body {
                        walk(else_body, out);
                    }
                }
                HirStmt::While {
                    body, else_body, ..
                }
                | HirStmt::For {
                    body, else_body, ..
                } => {
                    walk(body, out);
                    if let Some(else_body) = else_body {
                        walk(else_body, out);
                    }
                }
                HirStmt::TryExcept { body, handlers, .. } => {
                    walk(body, out);
                    for handler in handlers {
                        walk(&handler.body, out);
                    }
                }
                HirStmt::With { body, .. } => walk(body, out),
                HirStmt::Match { arms, .. } => {
                    for arm in arms {
                        walk(&arm.body, out);
                    }
                }
                HirStmt::NestedFunction { .. }
                | HirStmt::Let { .. }
                | HirStmt::Assign { .. }
                | HirStmt::AugAssign { .. }
                | HirStmt::TupleUnpack { .. }
                | HirStmt::StarUnpack { .. }
                | HirStmt::Assert { .. }
                | HirStmt::FieldAssign { .. }
                | HirStmt::SubscriptAssign { .. }
                | HirStmt::NestedSubscriptAssign { .. }
                | HirStmt::SubscriptAugAssign { .. }
                | HirStmt::AttributeAugAssign { .. }
                | HirStmt::AttributeSubscriptAssign { .. }
                | HirStmt::Delete { .. }
                | HirStmt::Expr { .. }
                | HirStmt::Return { .. }
                | HirStmt::Break
                | HirStmt::Continue
                | HirStmt::Pass
                | HirStmt::Raise { .. } => {}
            }
        }
    }

    let mut types = Vec::new();
    walk(stmts, &mut types);
    types
}

fn stmt_contains_yield(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Yield { .. } => true,
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            body_contains_yield(then_body)
                || elif_clauses
                    .iter()
                    .any(|(_, body)| body_contains_yield(body))
                || else_body
                    .as_ref()
                    .is_some_and(|body| body_contains_yield(body))
        }
        HirStmt::While {
            body, else_body, ..
        }
        | HirStmt::For {
            body, else_body, ..
        } => {
            body_contains_yield(body)
                || else_body
                    .as_ref()
                    .is_some_and(|body| body_contains_yield(body))
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            body_contains_yield(body)
                || handlers
                    .iter()
                    .any(|handler| body_contains_yield(&handler.body))
        }
        HirStmt::With { body, .. } => body_contains_yield(body),
        HirStmt::Match { arms, .. } => arms.iter().any(|arm| body_contains_yield(&arm.body)),
        HirStmt::NestedFunction { .. }
        | HirStmt::Let { .. }
        | HirStmt::Assign { .. }
        | HirStmt::AugAssign { .. }
        | HirStmt::TupleUnpack { .. }
        | HirStmt::StarUnpack { .. }
        | HirStmt::Assert { .. }
        | HirStmt::FieldAssign { .. }
        | HirStmt::SubscriptAssign { .. }
        | HirStmt::NestedSubscriptAssign { .. }
        | HirStmt::SubscriptAugAssign { .. }
        | HirStmt::AttributeAugAssign { .. }
        | HirStmt::AttributeSubscriptAssign { .. }
        | HirStmt::Delete { .. }
        | HirStmt::Expr { .. }
        | HirStmt::Return { .. }
        | HirStmt::Break
        | HirStmt::Continue
        | HirStmt::Pass
        | HirStmt::Raise { .. } => false,
    }
}

fn body_contains_yield(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(stmt_contains_yield)
}

fn count_direct_yields(stmts: &[HirStmt]) -> usize {
    stmts
        .iter()
        .filter(|stmt| matches!(stmt, HirStmt::Yield { .. }))
        .count()
}

fn has_nested_yield_beyond_direct(stmts: &[HirStmt]) -> bool {
    stmts
        .iter()
        .any(|stmt| !matches!(stmt, HirStmt::Yield { .. }) && stmt_contains_yield(stmt))
}

pub(super) fn validate_lazy_generator_shape(
    stmts: &[HirStmt],
) -> Result<(), LazyGeneratorShapeError> {
    let mut while_body: Option<&[HirStmt]> = None;
    for stmt in stmts {
        if while_body.is_none() {
            match stmt {
                HirStmt::While { body, .. } => while_body = Some(body.as_slice()),
                _ if stmt_contains_yield(stmt) => {
                    return Err(LazyGeneratorShapeError::YieldBeforeLoop)
                }
                _ => {}
            }
            continue;
        }

        return Err(LazyGeneratorShapeError::TrailingStatementsAfterLoop);
    }

    let Some(while_body) = while_body else {
        return Err(LazyGeneratorShapeError::MissingTopLevelWhileLoop);
    };

    let top_level_yields = count_direct_yields(while_body);
    let mut conditional_yield_sites = 0usize;
    let mut has_unsupported_nested_yield = false;

    for stmt in while_body {
        match stmt {
            HirStmt::Yield { .. } => {}
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                let then_has_yield = body_contains_yield(then_body);
                let elif_has_yield = elif_clauses
                    .iter()
                    .any(|(_, body)| body_contains_yield(body));
                let else_has_yield = else_body
                    .as_ref()
                    .is_some_and(|body| body_contains_yield(body));

                if !then_has_yield && !elif_has_yield && !else_has_yield {
                    continue;
                }

                let then_direct_yields = count_direct_yields(then_body);
                let then_has_nested = has_nested_yield_beyond_direct(then_body);
                let supported_then_branch = then_direct_yields == 1
                    && !then_has_nested
                    && !elif_has_yield
                    && !else_has_yield;
                if supported_then_branch {
                    conditional_yield_sites += 1;
                } else {
                    has_unsupported_nested_yield = true;
                }
            }
            _ => {
                if stmt_contains_yield(stmt) {
                    has_unsupported_nested_yield = true;
                }
            }
        }
    }

    if top_level_yields == 0 && conditional_yield_sites == 0 {
        return Err(LazyGeneratorShapeError::MissingYieldSite);
    }
    if top_level_yields > 1 {
        return Err(LazyGeneratorShapeError::MultipleTopLevelYields);
    }
    if conditional_yield_sites > 1 {
        return Err(LazyGeneratorShapeError::MultipleConditionalYieldSites);
    }
    if top_level_yields == 1 && conditional_yield_sites == 1 {
        return Err(LazyGeneratorShapeError::MixedTopLevelAndConditionalYield);
    }
    if has_unsupported_nested_yield {
        return Err(LazyGeneratorShapeError::NestedYieldNotSupported);
    }

    Ok(())
}

pub(super) fn collapse_types(types: Vec<Type>, empty_type: Type) -> Type {
    if types.is_empty() {
        return empty_type;
    }
    if types.len() == 1 {
        return types.into_iter().next().unwrap_or(Type::Any);
    }
    let mut members = types;
    members.sort_by_key(Type::display_name);
    members.dedup();
    if members.len() == 1 {
        members.into_iter().next().unwrap_or(Type::Any)
    } else {
        Type::Union(members)
    }
}

pub(super) fn infer_function_return_type(
    function_name: &str,
    declared_return_type: &Type,
    has_explicit_return_annotation: bool,
    body: &[HirStmt],
    mut report_error: impl FnMut(String),
) -> Type {
    let yielded_types = collect_yield_types(body);
    if !yielded_types.is_empty() {
        if let Err(shape_error) = validate_lazy_generator_shape(body) {
            report_error(shape_error.message(function_name));
        }
        let yielded_type = collapse_types(yielded_types, Type::Any);
        let inferred_iterator = Type::Iterator(Box::new(yielded_type.clone()));
        if has_explicit_return_annotation {
            match declared_return_type.resolve_alias() {
                Type::Iterator(elem_ty) => {
                    if !yielded_type.is_assignable_to(elem_ty.as_ref()) {
                        report_error(format!(
                            "generator '{}' yields '{}', which is not assignable to declared iterator element type '{}'",
                            function_name,
                            yielded_type.display_name(),
                            elem_ty.display_name()
                        ));
                    }
                    Type::Iterator(elem_ty.clone())
                }
                declared => {
                    report_error(format!(
                        "generator function '{}' must declare return type 'Iterator[T]', got '{}'",
                        function_name,
                        declared.display_name()
                    ));
                    inferred_iterator
                }
            }
        } else {
            inferred_iterator
        }
    } else if *declared_return_type == Type::Any && !has_explicit_return_annotation {
        let return_types = collect_return_types(body);
        if return_types.is_empty() {
            Type::None
        } else {
            collapse_types(return_types, Type::Any)
        }
    } else {
        declared_return_type.clone()
    }
}
