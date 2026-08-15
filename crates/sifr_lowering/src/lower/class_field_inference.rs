use sifr_python_ast::{Expr, Operator, Stmt, UnaryOp};
use sifr_type_system::{make_union, Type};
use std::collections::HashMap;

use super::{simple_expr::lower_expr_simple, LowerCtx};

fn merge_inferred_types(existing: Type, inferred: Type) -> Type {
    if matches!(existing.resolve_alias(), Type::Any | Type::Unknown)
        && !matches!(inferred.resolve_alias(), Type::Any | Type::Unknown)
    {
        return inferred;
    }
    if matches!(inferred.resolve_alias(), Type::Any | Type::Unknown) {
        return existing;
    }
    if inferred.is_assignable_to(&existing) {
        return existing;
    }
    if existing.is_assignable_to(&inferred) {
        return inferred;
    }
    make_union(vec![existing, inferred])
}

fn upsert_inferred_field(fields: &mut Vec<(String, Type)>, field_name: String, inferred_ty: Type) {
    if let Some((_, existing_ty)) = fields.iter_mut().find(|(name, _)| name == &field_name) {
        *existing_ty = merge_inferred_types(existing_ty.clone(), inferred_ty);
    } else {
        fields.push((field_name, inferred_ty));
    }
}

fn field_target_name_if_self(target: &Expr) -> Option<String> {
    let Expr::Attribute(attr) = target else {
        return None;
    };
    let Expr::Name(base) = attr.value.as_ref() else {
        return None;
    };
    if base.id.as_str() == "self" {
        Some(attr.attr.to_string())
    } else {
        None
    }
}

fn infer_defaultdict_value_type(
    default_factory_expr: Option<&Expr>,
    local_bindings: &HashMap<String, Type>,
) -> Type {
    let Some(factory) = default_factory_expr else {
        return Type::Any;
    };
    match factory {
        Expr::Name(name) => match name.id.as_str() {
            "int" => Type::Int,
            "float" => Type::Float,
            "bool" => Type::Bool,
            "str" => Type::Str,
            "list" => Type::List(Box::new(Type::Any)),
            "set" => Type::Set(Box::new(Type::Any)),
            "dict" => Type::Dict(Box::new(Type::Any), Box::new(Type::Any)),
            other => local_bindings.get(other).cloned().unwrap_or(Type::Any),
        },
        _ => Type::Any,
    }
}

fn infer_constructor_call_type(
    call: &sifr_python_ast::ExprCall,
    local_bindings: &HashMap<String, Type>,
    ctx: &LowerCtx,
) -> Type {
    let Expr::Name(func_name) = call.func.as_ref() else {
        return Type::Any;
    };
    let name = func_name.id.as_str();
    if let Some(class_ty) = ctx.class_types.get(name) {
        return class_ty.clone();
    }
    match name {
        "range" => Type::Range,
        "list" => Type::List(Box::new(Type::Any)),
        "set" => Type::Set(Box::new(Type::Any)),
        "dict" => Type::Dict(Box::new(Type::Any), Box::new(Type::Any)),
        name if ctx.explicit_defaultdict_bindings.contains(name)
            && !local_bindings.contains_key(name) =>
        {
            let value_ty =
                infer_defaultdict_value_type(call.arguments.args.first(), local_bindings);
            Type::Dict(Box::new(Type::Any), Box::new(value_ty))
        }
        other => local_bindings.get(other).cloned().unwrap_or(Type::Any),
    }
}

fn infer_constructor_expr_type(
    expr: &Expr,
    local_bindings: &HashMap<String, Type>,
    ctx: &LowerCtx,
) -> Type {
    if let Some(simple_expr) = lower_expr_simple(expr) {
        return simple_expr.ty().clone();
    }
    match expr {
        Expr::Name(name) => local_bindings
            .get(name.id.as_str())
            .cloned()
            .or_else(|| ctx.class_types.get(name.id.as_str()).cloned())
            .unwrap_or(Type::Any),
        Expr::Call(call) => infer_constructor_call_type(call, local_bindings, ctx),
        Expr::UnaryOp(unary) if matches!(unary.op, UnaryOp::USub) => {
            infer_constructor_expr_type(&unary.operand, local_bindings, ctx)
        }
        Expr::List(list) => {
            let elem_ty = list
                .elts
                .iter()
                .map(|elt| infer_constructor_expr_type(elt, local_bindings, ctx))
                .reduce(merge_inferred_types)
                .unwrap_or(Type::Any);
            Type::List(Box::new(elem_ty))
        }
        Expr::Set(set) => {
            let elem_ty = set
                .elts
                .iter()
                .map(|elt| infer_constructor_expr_type(elt, local_bindings, ctx))
                .reduce(merge_inferred_types)
                .unwrap_or(Type::Any);
            Type::Set(Box::new(elem_ty))
        }
        Expr::Tuple(tuple) => Type::Tuple(
            tuple
                .elts
                .iter()
                .map(|elt| infer_constructor_expr_type(elt, local_bindings, ctx))
                .collect(),
        ),
        Expr::Dict(dict) => {
            let key_ty = dict
                .items
                .iter()
                .filter_map(|item| item.key.as_ref())
                .map(|key| infer_constructor_expr_type(key, local_bindings, ctx))
                .reduce(merge_inferred_types)
                .unwrap_or(Type::Any);
            let value_ty = dict
                .items
                .iter()
                .map(|item| infer_constructor_expr_type(&item.value, local_bindings, ctx))
                .reduce(merge_inferred_types)
                .unwrap_or(Type::Any);
            Type::Dict(Box::new(key_ty), Box::new(value_ty))
        }
        Expr::ListComp(comp) => {
            let mut nested_locals = local_bindings.clone();
            for generator in &comp.generators {
                let iter_ty = infer_constructor_expr_type(&generator.iter, &nested_locals, ctx);
                let elem_ty = iter_ty.iterable_element_type().unwrap_or(Type::Any);
                match &generator.target {
                    Expr::Name(name) => {
                        nested_locals.insert(name.id.to_string(), elem_ty);
                    }
                    Expr::Tuple(tuple) => {
                        let Type::Tuple(elem_types) = elem_ty else {
                            continue;
                        };
                        for (idx, target_elt) in tuple.elts.iter().enumerate() {
                            if let Expr::Name(name) = target_elt {
                                let target_ty = elem_types.get(idx).cloned().unwrap_or(Type::Any);
                                nested_locals.insert(name.id.to_string(), target_ty);
                            }
                        }
                    }
                    _ => {}
                }
            }
            let elt_ty = infer_constructor_expr_type(&comp.elt, &nested_locals, ctx);
            Type::List(Box::new(elt_ty))
        }
        Expr::BinOp(binop) if matches!(binop.op, Operator::Mult) => {
            let left_ty = infer_constructor_expr_type(&binop.left, local_bindings, ctx);
            let right_ty = infer_constructor_expr_type(&binop.right, local_bindings, ctx);
            if matches!(left_ty.resolve_alias(), Type::List(_)) {
                left_ty
            } else if matches!(right_ty.resolve_alias(), Type::List(_)) {
                right_ty
            } else {
                Type::Any
            }
        }
        _ => Type::Any,
    }
}

fn bind_simple_target_type(
    target: &Expr,
    value_ty: &Type,
    local_bindings: &mut HashMap<String, Type>,
) {
    if let Expr::Name(name) = target {
        local_bindings.insert(name.id.to_string(), value_ty.clone());
    }
}

pub(in crate::lower) fn collect_constructor_self_field_assignments(
    stmts: &[Stmt],
    local_bindings: &mut HashMap<String, Type>,
    fields: &mut Vec<(String, Type)>,
    ctx: &LowerCtx,
) {
    for stmt in stmts {
        match stmt {
            Stmt::Assign(assign) => {
                let value_ty = infer_constructor_expr_type(&assign.value, local_bindings, ctx);
                for target in &assign.targets {
                    bind_simple_target_type(target, &value_ty, local_bindings);
                    if let Some(field_name) = field_target_name_if_self(target) {
                        upsert_inferred_field(fields, field_name, value_ty.clone());
                    }
                }
            }
            Stmt::AnnAssign(ann) => {
                let inferred_ty = if let Some(value) = ann.value.as_ref() {
                    infer_constructor_expr_type(value, local_bindings, ctx)
                } else {
                    Type::Any
                };
                bind_simple_target_type(&ann.target, &inferred_ty, local_bindings);
                if let Some(field_name) = field_target_name_if_self(&ann.target) {
                    upsert_inferred_field(fields, field_name, inferred_ty);
                }
            }
            Stmt::If(if_stmt) => {
                let mut then_locals = local_bindings.clone();
                collect_constructor_self_field_assignments(
                    &if_stmt.body,
                    &mut then_locals,
                    fields,
                    ctx,
                );
                for clause in &if_stmt.elif_else_clauses {
                    let mut clause_locals = local_bindings.clone();
                    collect_constructor_self_field_assignments(
                        &clause.body,
                        &mut clause_locals,
                        fields,
                        ctx,
                    );
                }
            }
            Stmt::For(for_stmt) => {
                let mut body_locals = local_bindings.clone();
                let iter_ty = infer_constructor_expr_type(&for_stmt.iter, &body_locals, ctx);
                let target_ty = iter_ty.iterable_element_type().unwrap_or(Type::Any);
                bind_simple_target_type(&for_stmt.target, &target_ty, &mut body_locals);
                collect_constructor_self_field_assignments(
                    &for_stmt.body,
                    &mut body_locals,
                    fields,
                    ctx,
                );
                let mut else_locals = local_bindings.clone();
                collect_constructor_self_field_assignments(
                    &for_stmt.orelse,
                    &mut else_locals,
                    fields,
                    ctx,
                );
            }
            Stmt::While(while_stmt) => {
                let mut body_locals = local_bindings.clone();
                collect_constructor_self_field_assignments(
                    &while_stmt.body,
                    &mut body_locals,
                    fields,
                    ctx,
                );
                let mut else_locals = local_bindings.clone();
                collect_constructor_self_field_assignments(
                    &while_stmt.orelse,
                    &mut else_locals,
                    fields,
                    ctx,
                );
            }
            Stmt::With(with_stmt) => {
                let mut body_locals = local_bindings.clone();
                collect_constructor_self_field_assignments(
                    &with_stmt.body,
                    &mut body_locals,
                    fields,
                    ctx,
                );
            }
            _ => {}
        }
    }
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
    fn inferred_field_union_uses_declaration_identity_not_local_spelling() {
        let alpha = imported_class("Zeta", "pkg.Alpha");
        let beta = imported_class("Beta", "pkg.Beta");

        assert_eq!(
            merge_inferred_types(beta.clone(), alpha.clone()),
            sifr_type_system::make_union(vec![alpha, beta])
        );
    }
}
