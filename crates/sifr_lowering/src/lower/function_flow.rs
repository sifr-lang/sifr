use crate::hir_nodes::HirStmt;
use sifr_type_system::{make_union, Type};

/// Collect all return types from a list of HIR statements (recursively).
pub(in crate::lower) fn collect_return_types(stmts: &[HirStmt]) -> Vec<Type> {
    crate::cfg::flow_facts(stmts)
        .map(|facts| facts.reachable_return_types().to_vec())
        .unwrap_or_default()
}

/// Collect all yielded expression types from a list of HIR statements (recursively).
pub(in crate::lower) fn collect_yield_types(stmts: &[HirStmt]) -> Vec<Type> {
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
                }
                | HirStmt::AsyncFor {
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
                HirStmt::TryFinally { body, finalbody } => {
                    walk(body, out);
                    walk(finalbody, out);
                }
                HirStmt::With { body, .. } | HirStmt::AsyncWith { body, .. } => walk(body, out),
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
                | HirStmt::NestedFieldAssign { .. }
                | HirStmt::SubscriptAssign { .. }
                | HirStmt::NestedSubscriptAssign { .. }
                | HirStmt::AttributeNestedSubscriptAssign { .. }
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

pub(in crate::lower) fn collapse_types(types: Vec<Type>, empty_type: Type) -> Type {
    if types.is_empty() {
        return empty_type;
    }
    make_union(types)
}

pub(in crate::lower) fn infer_function_return_type(
    function_name: &str,
    is_async: bool,
    declared_return_type: &Type,
    has_explicit_return_annotation: bool,
    body: &[HirStmt],
    mut report_error: impl FnMut(String),
) -> Type {
    let yielded_types = collect_yield_types(body);
    if !yielded_types.is_empty() {
        let yielded_type = normalize_generator_yield_type(collapse_types(yielded_types, Type::Any));
        if is_async {
            if let Type::Union(members) = yielded_type.resolve_alias() {
                if members.len() > 1 {
                    report_error(format!(
                        "async generator '{function_name}' has inconsistent yield types '{}'; yielded values must converge to one async generator element type",
                        yielded_type.display_name()
                    ));
                }
            }
            let inferred_generator =
                Type::AsyncGenerator(Box::new(yielded_type.clone()), Box::new(Type::Never));
            if has_explicit_return_annotation {
                match declared_return_type.resolve_alias() {
                    Type::AsyncGenerator(elem_ty, err_ty) => {
                        if !yielded_type.is_assignable_to(elem_ty.as_ref()) {
                            report_error(format!(
                                "async generator '{}' yields '{}', which is not assignable to declared async generator element type '{}'",
                                function_name,
                                yielded_type.display_name(),
                                elem_ty.display_name()
                            ));
                        }
                        Type::AsyncGenerator(elem_ty.clone(), err_ty.clone())
                    }
                    declared => {
                        report_error(format!(
                            "async generator function '{}' must declare return type 'AsyncGenerator[T, E]', got '{}'",
                            function_name,
                            declared.display_name()
                        ));
                        inferred_generator
                    }
                }
            } else {
                inferred_generator
            }
        } else {
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

fn normalize_generator_yield_type(yielded_type: Type) -> Type {
    if let Some(member) = yielded_type.optional_member_type() {
        return member;
    }
    let Type::Union(members) = yielded_type else {
        return yielded_type;
    };
    if members.is_empty() {
        return Type::Union(members);
    }
    sifr_type_system::make_union(members)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn imported_class(local_name: &str, identity: &str) -> Type {
        Type::Class {
            identity: Some(identity.to_string()),
            type_args: Vec::new(),
            name: local_name.to_string(),
            fields: Vec::new(),
            methods: Vec::new(),
            parent_class: None,
        }
    }

    #[test]
    fn inferred_return_union_uses_declaration_identity_not_local_spelling() {
        let alpha = imported_class("Zeta", "pkg.Alpha");
        let beta = imported_class("Beta", "pkg.Beta");

        assert_eq!(
            collapse_types(vec![beta.clone(), alpha.clone()], Type::Any),
            sifr_type_system::make_union(vec![alpha, beta])
        );
    }
}
