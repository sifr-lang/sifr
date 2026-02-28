use super::{IsNoneUnionMatch, IsinstanceUnionMatch, ModuleFuncSignatures, RustEmitter};
use sifr_hir::{HirExpr, HirFStringPart, HirModule, HirStmt};
use sifr_type_system::{ParamConvention, Type};
use std::collections::{HashMap, HashSet};

/// Check if a type can be auto-formatted with `{}` (implements Display).
/// Used to determine if auto-generated Display impl is safe for a class field.
pub(super) fn is_auto_display_type(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Float | Type::Bool | Type::Str | Type::None => true,
        Type::LiteralInt(_) | Type::LiteralBool(_) | Type::LiteralStr(_) => true,
        Type::Class { .. } => true, // Classes get auto-Display too
        Type::Newtype { .. } => true,
        // Union types map to Option<T> or Rust enum — neither implements Display
        _ => false,
    }
}

/// Returns the default parameter convention for a type.
/// Copy types (int, float, bool) are passed by value (Own).
/// Move types (str, list, dict, class, etc.) are passed by reference (Borrow).
pub(super) fn default_param_convention(ty: &Type) -> ParamConvention {
    if ty.ownership() == sifr_type_system::OwnershipKind::Copy {
        ParamConvention::Own
    } else {
        ParamConvention::Borrow
    }
}

pub(super) fn is_option_type(ty: &Type) -> bool {
    if let Type::Union(members) = ty {
        let non_none: Vec<&Type> = members
            .iter()
            .filter(|m| !matches!(m, Type::None))
            .collect();
        let has_none = members.iter().any(|m| matches!(m, Type::None));
        has_none && non_none.len() == 1
    } else {
        false
    }
}

/// Detect truthiness check on an Option variable: `if x:` where x has type T | None.
pub(super) fn detect_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty } = expr {
        if is_option_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

/// Detect `x is not None` pattern in a Compare expression. Returns the variable name.
pub(super) fn detect_is_not_none_var(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = expr
    {
        if ops.len() == 1
            && (ops[0] == "is not" || ops[0] == "!=")
            && matches!(comparators[0], HirExpr::NoneLiteral)
        {
            if let HirExpr::Name { name, ty } = left.as_ref() {
                // Only match for Option types (2-member unions with None)
                if is_option_type(ty) {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

/// Detect compound `a is not None and b is not None` pattern.
/// Returns list of variable names that are checked for not-None.
pub(super) fn detect_and_not_none_vars(expr: &HirExpr) -> Option<Vec<String>> {
    if let HirExpr::BoolOp { op, values, .. } = expr {
        if op == "and" {
            let mut vars = Vec::new();
            for val in values {
                if let Some(var_name) = detect_is_not_none_var(val) {
                    vars.push(var_name);
                }
            }
            if vars.len() >= 2 {
                return Some(vars);
            }
        }
    }
    None
}

/// Detect `isinstance(x, type)` where x is a non-Option union type.
/// Returns (`var_name`, `variant_name`, `enum_name`, `other_variants`: Vec<(`variant_name`, type)>).
pub(super) fn detect_isinstance_union(expr: &HirExpr) -> Option<IsinstanceUnionMatch> {
    if let HirExpr::Call { func, args, .. } = expr {
        if func == "isinstance" && args.len() == 2 {
            if let HirExpr::Name { name, ty } = &args[0] {
                if let Type::Union(members) = ty {
                    if !is_option_type(ty) {
                        // The second arg is a StringLiteral with the type name
                        if let HirExpr::StringLiteral(type_name) = &args[1] {
                            let target_ty = match type_name.as_str() {
                                "int" => Type::Int,
                                "str" => Type::Str,
                                "float" => Type::Float,
                                "bool" => Type::Bool,
                                other => {
                                    // Check if it's a class type in the union members
                                    if let Some(class_ty) = members.iter().find(
                                        |m| matches!(m, Type::Class { name, .. } if name == other),
                                    ) {
                                        class_ty.clone()
                                    } else {
                                        return None;
                                    }
                                }
                            };
                            // Check that this type is a member of the union
                            if members.contains(&target_ty) {
                                let variant = target_ty.union_variant_name();
                                let enum_name = ty.union_enum_name();
                                // Collect other variants for else branch destructuring
                                let other_variants: Vec<(String, Type)> = members
                                    .iter()
                                    .filter(|m| *m != &target_ty)
                                    .map(|m| (m.union_variant_name(), m.clone()))
                                    .collect();
                                return Some((name.clone(), variant, enum_name, other_variants));
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

/// Find the matching union variant name for an argument type.
pub(super) fn find_union_variant(members: &[Type], arg_ty: &Type) -> Option<String> {
    for member in members {
        if arg_ty.is_assignable_to(member) {
            return Some(member.union_variant_name());
        }
    }
    None
}

/// Detect `x is None` pattern in a Compare expression. Returns the variable name.
/// Check if a block of HIR statements always exits (return, break, continue).
/// Used for early-return narrowing in codegen.
pub(super) fn codegen_body_always_exits(stmts: &[HirStmt]) -> bool {
    if let Some(last) = stmts.last() {
        matches!(last, HirStmt::Return { .. })
    } else {
        false
    }
}

/// Detect `x is None` pattern. Returns the variable name.
/// Only matches when the variable type is an Option (T | None with exactly 2 members).
pub(super) fn detect_is_none_var(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = expr
    {
        if ops.len() == 1
            && (ops[0] == "is" || ops[0] == "==")
            && matches!(comparators[0], HirExpr::NoneLiteral)
        {
            if let HirExpr::Name { name, ty } = left.as_ref() {
                // Only match for Option types (2-member unions with None)
                if is_option_type(ty) {
                    return Some(name.clone());
                }
            }
        }
    }
    None
}

/// Detect `x is None` pattern for 3+ member unions containing None.
/// Returns (`var_name`, `enum_name`, `non_none_variants`).
pub(super) fn detect_is_none_union_var(expr: &HirExpr) -> Option<IsNoneUnionMatch> {
    if let HirExpr::Compare {
        left,
        ops,
        comparators,
        ..
    } = expr
    {
        if ops.len() == 1 && ops[0] == "is" && matches!(comparators[0], HirExpr::NoneLiteral) {
            if let HirExpr::Name { name, ty } = left.as_ref() {
                if let Type::Union(members) = ty {
                    let has_none = members.iter().any(|m| matches!(m, Type::None));
                    let non_none: Vec<&Type> = members
                        .iter()
                        .filter(|m| !matches!(m, Type::None))
                        .collect();
                    // Only match for 3+ member unions (not simple Option)
                    if has_none && non_none.len() >= 2 {
                        let enum_name = ty.union_enum_name();
                        let non_none_variants: Vec<(String, Type)> = non_none
                            .iter()
                            .map(|t| (t.union_variant_name(), (*t).clone()))
                            .collect();
                        return Some((name.clone(), enum_name, non_none_variants));
                    }
                }
            }
        }
    }
    None
}

pub(super) fn is_hashable_type_codegen(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Bool | Type::Str | Type::None | Type::BigInt => true,
        Type::Float => false,
        _ => false,
    }
}

/// Check if a module uses the `bigint` type anywhere.
pub(super) fn module_uses_bigint(module: &HirModule) -> bool {
    fn type_has_bigint(ty: &Type) -> bool {
        match ty {
            Type::BigInt => true,
            Type::List(t) | Type::Set(t) => type_has_bigint(t),
            Type::Dict(k, v) => type_has_bigint(k) || type_has_bigint(v),
            Type::Tuple(ts) | Type::Union(ts) => ts.iter().any(type_has_bigint),
            Type::Result(ok, err) => type_has_bigint(ok) || type_has_bigint(err),
            _ => false,
        }
    }
    fn expr_has_bigint(expr: &HirExpr) -> bool {
        type_has_bigint(expr.ty())
    }
    fn stmts_have_bigint(stmts: &[HirStmt]) -> bool {
        stmts.iter().any(stmt_has_bigint)
    }
    fn stmt_has_bigint(stmt: &HirStmt) -> bool {
        match stmt {
            HirStmt::Let { ty, value, .. } => type_has_bigint(ty) || expr_has_bigint(value),
            HirStmt::Return { value } => value.as_ref().map(expr_has_bigint).unwrap_or(false),
            HirStmt::Expr { expr } => expr_has_bigint(expr),
            HirStmt::If {
                condition,
                then_body,
                else_body,
                elif_clauses,
                ..
            } => {
                expr_has_bigint(condition)
                    || stmts_have_bigint(then_body)
                    || else_body
                        .as_ref()
                        .map(|b| stmts_have_bigint(b))
                        .unwrap_or(false)
                    || elif_clauses.iter().any(|(_, b)| stmts_have_bigint(b))
            }
            HirStmt::While { body, .. } => stmts_have_bigint(body),
            HirStmt::For { body, .. } => stmts_have_bigint(body),
            _ => false,
        }
    }
    for func in &module.functions {
        if type_has_bigint(&func.return_type) {
            return true;
        }
        if func.params.iter().any(|p| type_has_bigint(&p.ty)) {
            return true;
        }
        if stmts_have_bigint(&func.body) {
            return true;
        }
    }
    for class in &module.classes {
        if class.fields.iter().any(|(_, t)| type_has_bigint(t)) {
            return true;
        }
        for method in &class.methods {
            if type_has_bigint(&method.return_type) {
                return true;
            }
            if method.params.iter().any(|p| type_has_bigint(&p.ty)) {
                return true;
            }
            if stmts_have_bigint(&method.body) {
                return true;
            }
        }
    }
    false
}

/// Collect all parts of a chained string concatenation (`a + b + c`).
/// Recursively flattens nested `BinOp::Add` on strings into a flat list of expressions.
pub(super) fn collect_string_concat_parts<'a>(expr: &'a HirExpr, parts: &mut Vec<&'a HirExpr>) {
    if let HirExpr::BinOp {
        left,
        op,
        right,
        ty,
    } = expr
    {
        if op == "+" && *ty == Type::Str {
            collect_string_concat_parts(left, parts);
            collect_string_concat_parts(right, parts);
            return;
        }
    }
    parts.push(expr);
}

/// Check if a method body contains any field assignments or attribute augmented assignments (self.field = ... or self.field += ...).
pub(super) fn body_contains_field_assign_codegen(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(|s| match s {
        HirStmt::FieldAssign { .. }
        | HirStmt::AttributeAugAssign { .. }
        | HirStmt::AttributeSubscriptAssign { .. } => true,
        HirStmt::Expr { expr } => expr_contains_self_field_mutation(expr),
        HirStmt::Return { value: Some(expr) } => expr_contains_self_field_mutation(expr),
        HirStmt::Let { value, .. } => expr_contains_self_field_mutation(value),
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body,
            ..
        } => {
            body_contains_field_assign_codegen(then_body)
                || elif_clauses
                    .iter()
                    .any(|(_, body)| body_contains_field_assign_codegen(body))
                || else_body
                    .as_ref()
                    .is_some_and(|b| body_contains_field_assign_codegen(b))
        }
        HirStmt::While { body, .. } | HirStmt::For { body, .. } => {
            body_contains_field_assign_codegen(body)
        }
        _ => false,
    })
}

/// Check if an expression contains a mutating method call on a self field (e.g., self.items.append(...)).
pub(super) fn expr_contains_self_field_mutation(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::MethodCall { object, method, .. } => {
            // Check if calling a mutating method on self.field
            let is_self_field = matches!(object.as_ref(), HirExpr::FieldAccess { object: inner, .. }
                if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
            if is_self_field && MUTATING_METHODS.contains(&method.as_str()) {
                return true;
            }
            // Recurse into the object
            expr_contains_self_field_mutation(object)
        }
        _ => false,
    }
}

/// Check if a type contains a specific type variable name.
pub(super) fn type_contains_typevar(ty: &Type, tv_name: &str) -> bool {
    match ty {
        Type::TypeVar(name) => name == tv_name,
        Type::List(inner) => type_contains_typevar(inner, tv_name),
        Type::Set(inner) => type_contains_typevar(inner, tv_name),
        Type::Dict(key, val) => {
            type_contains_typevar(key, tv_name) || type_contains_typevar(val, tv_name)
        }
        Type::Tuple(elems) => elems.iter().any(|e| type_contains_typevar(e, tv_name)),
        Type::Union(members) => members.iter().any(|m| type_contains_typevar(m, tv_name)),
        Type::Result(ok, err) => {
            type_contains_typevar(ok, tv_name) || type_contains_typevar(err, tv_name)
        }
        Type::Class {
            fields, methods, ..
        } => {
            fields
                .iter()
                .any(|(_, t)| type_contains_typevar(t, tv_name))
                || methods.iter().any(|(_, ft)| {
                    ft.params
                        .iter()
                        .any(|(_, t, _)| type_contains_typevar(t, tv_name))
                        || type_contains_typevar(&ft.return_type, tv_name)
                })
        }
        _ => false,
    }
}

/// Check if a type references a specific class name (directly or via union/option).
pub(super) fn type_references_class(ty: &Type, class_name: &str) -> bool {
    match ty {
        Type::Class { name, .. } => name == class_name,
        Type::Union(members) => members.iter().any(|m| type_references_class(m, class_name)),
        Type::List(inner) => type_references_class(inner, class_name),
        Type::Dict(key, val) => {
            type_references_class(key, class_name) || type_references_class(val, class_name)
        }
        Type::Tuple(elems) => elems.iter().any(|e| type_references_class(e, class_name)),
        Type::Result(ok, err) => {
            type_references_class(ok, class_name) || type_references_class(err, class_name)
        }
        _ => false,
    }
}

/// Generate the Rust type string for a recursive field.
/// For `ClassName | None` -> `Option<Box<ClassName>>`
/// For `ClassName` directly -> `Box<ClassName>`
pub(super) fn recursive_field_rust_type(ty: &Type, class_name: &str) -> String {
    match ty {
        Type::Union(members) => {
            let non_none: Vec<&Type> = members
                .iter()
                .filter(|m| !matches!(m, Type::None))
                .collect();
            let has_none = members.iter().any(|m| matches!(m, Type::None));
            if has_none && non_none.len() == 1 {
                // T | None where T references the class -> Option<Box<T>>
                if type_references_class(non_none[0], class_name) {
                    format!("Option<Box<{}>>", non_none[0].rust_type())
                } else {
                    ty.rust_type()
                }
            } else {
                // General union with recursive member - wrap the whole thing in Box
                format!("Box<{}>", ty.rust_type())
            }
        }
        Type::Class { name, .. } if name == class_name => {
            format!("Box<{name}>")
        }
        _ => format!("Box<{}>", ty.rust_type()),
    }
}

/// Check if a variable name is referenced anywhere in a list of statements.
pub(super) fn stmts_reference_var(stmts: &[HirStmt], var_name: &str) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Expr { expr } => {
                if expr_references_var(expr, var_name) {
                    return true;
                }
            }
            HirStmt::Return { value: Some(expr) } => {
                if expr_references_var(expr, var_name) {
                    return true;
                }
            }
            HirStmt::Return { value: None } => {}
            HirStmt::Yield { value } => {
                if expr_references_var(value, var_name) {
                    return true;
                }
            }
            HirStmt::Let { value, .. } => {
                if expr_references_var(value, var_name) {
                    return true;
                }
            }
            HirStmt::Assign { value, .. } => {
                if expr_references_var(value, var_name) {
                    return true;
                }
            }
            HirStmt::FieldAssign { value, .. } => {
                if expr_references_var(value, var_name) {
                    return true;
                }
            }
            HirStmt::SubscriptAssign { index, value, .. } => {
                if expr_references_var(index, var_name) {
                    return true;
                }
                if expr_references_var(value, var_name) {
                    return true;
                }
            }
            HirStmt::AttributeAugAssign { value, .. } => {
                if expr_references_var(value, var_name) {
                    return true;
                }
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                if expr_references_var(condition, var_name) {
                    return true;
                }
                if stmts_reference_var(then_body, var_name) {
                    return true;
                }
                for (cond, body) in elif_clauses {
                    if expr_references_var(cond, var_name) {
                        return true;
                    }
                    if stmts_reference_var(body, var_name) {
                        return true;
                    }
                }
                if let Some(eb) = else_body {
                    if stmts_reference_var(eb, var_name) {
                        return true;
                    }
                }
            }
            HirStmt::While {
                condition, body, ..
            } => {
                if expr_references_var(condition, var_name) {
                    return true;
                }
                if stmts_reference_var(body, var_name) {
                    return true;
                }
            }
            HirStmt::For { iter, body, .. } => {
                if expr_references_var(iter, var_name) {
                    return true;
                }
                if stmts_reference_var(body, var_name) {
                    return true;
                }
            }
            HirStmt::With { items, body, .. } => {
                for (_, value, _) in items {
                    if expr_references_var(value, var_name) {
                        return true;
                    }
                }
                if stmts_reference_var(body, var_name) {
                    return true;
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                if stmts_reference_var(body, var_name) {
                    return true;
                }
                for handler in handlers {
                    if stmts_reference_var(&handler.body, var_name) {
                        return true;
                    }
                }
            }
            HirStmt::Raise { value } => {
                if expr_references_var(value, var_name) {
                    return true;
                }
            }
            HirStmt::AugAssign { value, .. } => {
                if expr_references_var(value, var_name) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Check if an expression references a variable name.
pub(super) fn expr_references_var(expr: &HirExpr, var_name: &str) -> bool {
    match expr {
        HirExpr::Name { name, .. } => name == var_name,
        HirExpr::BinOp { left, right, .. } => {
            expr_references_var(left, var_name) || expr_references_var(right, var_name)
        }
        HirExpr::BoolOp { values, .. } => values.iter().any(|v| expr_references_var(v, var_name)),
        HirExpr::UnaryOp { operand, .. } => expr_references_var(operand, var_name),
        HirExpr::Call { args, .. } => args.iter().any(|a| expr_references_var(a, var_name)),
        HirExpr::MethodCall { object, args, .. } => {
            expr_references_var(object, var_name)
                || args.iter().any(|a| expr_references_var(a, var_name))
        }
        HirExpr::FieldAccess { object, .. } => expr_references_var(object, var_name),
        HirExpr::Index { object, index, .. } => {
            expr_references_var(object, var_name) || expr_references_var(index, var_name)
        }
        HirExpr::ListLiteral { elements, .. } => {
            elements.iter().any(|e| expr_references_var(e, var_name))
        }
        HirExpr::SetLiteral { elements, .. } => {
            elements.iter().any(|e| expr_references_var(e, var_name))
        }
        HirExpr::TupleLiteral { elements, .. } => {
            elements.iter().any(|e| expr_references_var(e, var_name))
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            expr_references_var(left, var_name)
                || comparators.iter().any(|c| expr_references_var(c, var_name))
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_references_var(condition, var_name)
                || expr_references_var(then_expr, var_name)
                || expr_references_var(else_expr, var_name)
        }
        HirExpr::Lambda { body, .. } => expr_references_var(body, var_name),
        HirExpr::ListComp {
            expr: e,
            generators,
            ..
        } => {
            expr_references_var(e, var_name)
                || generators.iter().any(|(_, iter, filter)| {
                    expr_references_var(iter, var_name)
                        || filter
                            .as_ref()
                            .is_some_and(|f| expr_references_var(f, var_name))
                })
        }
        HirExpr::QuestionMark { expr, .. } => expr_references_var(expr, var_name),
        HirExpr::OkWrap { value, .. } => expr_references_var(value, var_name),
        HirExpr::ErrWrap { value, .. } => expr_references_var(value, var_name),
        HirExpr::DictLiteral { keys, values, .. } => keys
            .iter()
            .chain(values.iter())
            .any(|e| expr_references_var(e, var_name)),
        _ => false,
    }
}

/// Check if a function body contains any yield statements (making it a generator).
/// Check if a try body contains a return statement with a non-unit value.
/// Used to determine if the try closure needs to return T instead of ().
pub(super) fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Return { value: Some(val) } => {
                // A return with a non-None value
                if !matches!(val, HirExpr::NoneLiteral) {
                    return true;
                }
            }
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                if try_body_has_value_return(then_body) {
                    return true;
                }
                for (_, body) in elif_clauses {
                    if try_body_has_value_return(body) {
                        return true;
                    }
                }
                if let Some(eb) = else_body {
                    if try_body_has_value_return(eb) {
                        return true;
                    }
                }
            }
            HirStmt::While { body, .. } => {
                if try_body_has_value_return(body) {
                    return true;
                }
            }
            HirStmt::For { body, .. } => {
                if try_body_has_value_return(body) {
                    return true;
                }
            }
            HirStmt::With { body, .. } => {
                if try_body_has_value_return(body) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

pub(super) fn body_contains_yield_inner(stmts: &[HirStmt]) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Yield { .. } => return true,
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                if body_contains_yield_inner(then_body) {
                    return true;
                }
                for (_, body) in elif_clauses {
                    if body_contains_yield_inner(body) {
                        return true;
                    }
                }
                if let Some(eb) = else_body {
                    if body_contains_yield_inner(eb) {
                        return true;
                    }
                }
            }
            HirStmt::While {
                body, else_body, ..
            } => {
                if body_contains_yield_inner(body) {
                    return true;
                }
                if let Some(eb) = else_body {
                    if body_contains_yield_inner(eb) {
                        return true;
                    }
                }
            }
            HirStmt::For {
                body, else_body, ..
            } => {
                if body_contains_yield_inner(body) {
                    return true;
                }
                if let Some(eb) = else_body {
                    if body_contains_yield_inner(eb) {
                        return true;
                    }
                }
            }
            HirStmt::With { body, .. } => {
                if body_contains_yield_inner(body) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Check if a type needs .`clone()` when accessed from &self (non-Copy types).
pub(super) fn needs_clone_for_type(ty: &Type) -> bool {
    match ty {
        Type::Int | Type::Float | Type::Bool | Type::None => false,
        Type::LiteralInt(_) | Type::LiteralBool(_) => false,
        Type::Str | Type::LiteralStr(_) => true, // String is not Copy
        Type::List(_) | Type::Dict(_, _) => true,
        Type::Tuple(_) => true, // tuples of non-Copy are non-Copy
        Type::Class { .. } => true,
        Type::Newtype { .. } => true,
        Type::TypeVar(_) => true, // Generic type params have T: Clone bound, so .clone() is safe
        Type::BigInt => true,     // num_bigint::BigInt is not Copy
        _ => false,
    }
}

/// Mutating methods that require the receiver variable to be `mut`.
pub(super) const MUTATING_METHODS: &[&str] = &[
    "append",
    "appendleft",
    "extend",
    "insert",
    "clear",
    "reverse",
    "sort",
    "pop",
    "popleft",
    "remove",
    "push_str",
    "update",
    "add",
    "discard",
];

/// Collect the set of variable names that are mutated in a function body.
/// A variable is mutated if it appears in:
/// - `HirStmt::Assign` (reassignment)
/// - `HirStmt::AugAssign` (augmented assignment like +=)
/// - `HirStmt::Expr` containing a `MethodCall` on the variable with a mutating method
/// - `HirStmt::Delete` on the variable
pub(super) fn collect_mutated_vars(stmts: &[HirStmt]) -> HashSet<String> {
    let mut mutated = HashSet::new();
    collect_mutated_vars_inner(stmts, &mut mutated, None);
    mutated
}

pub(super) fn collect_mutated_vars_with_sigs(
    stmts: &[HirStmt],
    func_signatures: &ModuleFuncSignatures,
) -> HashSet<String> {
    let mut mutated = HashSet::new();
    collect_mutated_vars_inner(stmts, &mut mutated, Some(func_signatures));
    mutated
}

pub(super) fn collect_mutated_vars_inner(
    stmts: &[HirStmt],
    mutated: &mut HashSet<String>,
    func_signatures: Option<&ModuleFuncSignatures>,
) {
    for stmt in stmts {
        match stmt {
            HirStmt::Assign { name, .. } => {
                mutated.insert(name.clone());
            }
            HirStmt::AugAssign { name, .. } => {
                mutated.insert(name.clone());
            }
            HirStmt::Expr { expr } => {
                collect_mutated_vars_in_expr(expr, mutated, func_signatures);
            }
            HirStmt::Let { value, .. } => {
                // Scan the value expression for mutating method calls
                collect_mutated_vars_in_expr(value, mutated, func_signatures);
            }
            HirStmt::Return { value: Some(expr) } => {
                collect_mutated_vars_in_expr(expr, mutated, func_signatures);
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                collect_mutated_vars_in_expr(condition, mutated, func_signatures);
                collect_mutated_vars_inner(then_body, mutated, func_signatures);
                for (cond, body) in elif_clauses {
                    collect_mutated_vars_in_expr(cond, mutated, func_signatures);
                    collect_mutated_vars_inner(body, mutated, func_signatures);
                }
                if let Some(body) = else_body {
                    collect_mutated_vars_inner(body, mutated, func_signatures);
                }
            }
            HirStmt::While {
                condition,
                body,
                else_body,
            } => {
                collect_mutated_vars_in_expr(condition, mutated, func_signatures);
                collect_mutated_vars_inner(body, mutated, func_signatures);
                if let Some(eb) = else_body {
                    collect_mutated_vars_inner(eb, mutated, func_signatures);
                }
            }
            HirStmt::For {
                body, else_body, ..
            } => {
                collect_mutated_vars_inner(body, mutated, func_signatures);
                if let Some(eb) = else_body {
                    collect_mutated_vars_inner(eb, mutated, func_signatures);
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                collect_mutated_vars_inner(body, mutated, func_signatures);
                for handler in handlers {
                    collect_mutated_vars_inner(&handler.body, mutated, func_signatures);
                }
            }
            HirStmt::SubscriptAssign { object, .. } => {
                mutated.insert(object.clone());
            }
            HirStmt::NestedSubscriptAssign { object, .. } => {
                mutated.insert(object.clone());
            }
            HirStmt::SubscriptAugAssign { object, .. } => {
                mutated.insert(object.clone());
            }
            HirStmt::AttributeAugAssign { object, .. } => {
                mutated.insert(object.clone());
            }
            HirStmt::Delete {
                object: HirExpr::Name { name, .. },
                ..
            } => {
                mutated.insert(name.clone());
            }
            HirStmt::Yield { value } => {
                collect_mutated_vars_in_expr(value, mutated, func_signatures);
            }
            HirStmt::With { items, body, .. } => {
                for (_, value, _) in items {
                    collect_mutated_vars_in_expr(value, mutated, func_signatures);
                }
                collect_mutated_vars_inner(body, mutated, func_signatures);
            }
            _ => {}
        }
    }
}

pub(super) fn collect_mutated_vars_in_expr(
    expr: &HirExpr,
    mutated: &mut HashSet<String>,
    func_signatures: Option<&ModuleFuncSignatures>,
) {
    match expr {
        HirExpr::MethodCall {
            object,
            method,
            args,
            ..
        } => {
            if MUTATING_METHODS.contains(&method.as_str()) {
                if let HirExpr::Name { name, .. } = object.as_ref() {
                    mutated.insert(name.clone());
                }
            }
            // Class method calls may mutate the object (conservative)
            if matches!(object.ty(), Type::Class { .. }) {
                if let HirExpr::Name { name, .. } = object.as_ref() {
                    mutated.insert(name.clone());
                }
            }
            // Recurse into sub-expressions
            collect_mutated_vars_in_expr(object, mutated, func_signatures);
            for arg in args {
                collect_mutated_vars_in_expr(arg, mutated, func_signatures);
            }
        }
        HirExpr::Call { func, args, .. } => {
            // Mark variables passed to MutBorrow params as mutated (need `let mut` in Rust)
            if let Some(sigs) = func_signatures {
                if let Some((param_convs, _)) = sigs.get(func) {
                    for (i, arg) in args.iter().enumerate() {
                        if let Some((_, ParamConvention::MutBorrow)) = param_convs.get(i) {
                            if let HirExpr::Name { name, .. } = arg {
                                mutated.insert(name.clone());
                            }
                        }
                    }
                }
            }
            for arg in args {
                collect_mutated_vars_in_expr(arg, mutated, func_signatures);
            }
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_mutated_vars_in_expr(left, mutated, func_signatures);
            collect_mutated_vars_in_expr(right, mutated, func_signatures);
        }
        HirExpr::UnaryOp { operand, .. } => {
            collect_mutated_vars_in_expr(operand, mutated, func_signatures);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_mutated_vars_in_expr(left, mutated, func_signatures);
            for c in comparators {
                collect_mutated_vars_in_expr(c, mutated, func_signatures);
            }
        }
        HirExpr::BoolOp { values, .. } => {
            for v in values {
                collect_mutated_vars_in_expr(v, mutated, func_signatures);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_mutated_vars_in_expr(condition, mutated, func_signatures);
            collect_mutated_vars_in_expr(then_expr, mutated, func_signatures);
            collect_mutated_vars_in_expr(else_expr, mutated, func_signatures);
        }
        HirExpr::Index { object, index, .. } => {
            collect_mutated_vars_in_expr(object, mutated, func_signatures);
            collect_mutated_vars_in_expr(index, mutated, func_signatures);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(e) = part {
                    collect_mutated_vars_in_expr(e, mutated, func_signatures);
                }
            }
        }
        _ => {}
    }
}

/// Collect all variable names and their types referenced in a list of statements.
pub(super) fn collect_referenced_vars_with_types(stmts: &[HirStmt]) -> Vec<(String, Type)> {
    let mut refs: HashMap<String, Type> = HashMap::new();
    collect_referenced_vars_with_types_inner(stmts, &mut refs);
    refs.into_iter().collect()
}

pub(super) fn collect_referenced_vars_with_types_inner(
    stmts: &[HirStmt],
    refs: &mut HashMap<String, Type>,
) {
    for stmt in stmts {
        match stmt {
            HirStmt::Let { value, .. } => {
                collect_typed_refs_in_expr(value, refs);
            }
            HirStmt::Assign { value, .. } => {
                collect_typed_refs_in_expr(value, refs);
            }
            HirStmt::AugAssign { value, .. } => {
                collect_typed_refs_in_expr(value, refs);
            }
            HirStmt::Return { value: Some(expr) } => {
                collect_typed_refs_in_expr(expr, refs);
            }
            HirStmt::Expr { expr } => {
                collect_typed_refs_in_expr(expr, refs);
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                collect_typed_refs_in_expr(condition, refs);
                collect_referenced_vars_with_types_inner(then_body, refs);
                for (cond, body) in elif_clauses {
                    collect_typed_refs_in_expr(cond, refs);
                    collect_referenced_vars_with_types_inner(body, refs);
                }
                if let Some(body) = else_body {
                    collect_referenced_vars_with_types_inner(body, refs);
                }
            }
            HirStmt::While {
                condition, body, ..
            } => {
                collect_typed_refs_in_expr(condition, refs);
                collect_referenced_vars_with_types_inner(body, refs);
            }
            HirStmt::For { iter, body, .. } => {
                collect_typed_refs_in_expr(iter, refs);
                collect_referenced_vars_with_types_inner(body, refs);
            }
            HirStmt::FieldAssign { value, .. } => {
                collect_typed_refs_in_expr(value, refs);
            }
            HirStmt::SubscriptAssign { index, value, .. } => {
                collect_typed_refs_in_expr(index, refs);
                collect_typed_refs_in_expr(value, refs);
            }
            _ => {}
        }
    }
}

pub(super) fn collect_typed_refs_in_expr(expr: &HirExpr, refs: &mut HashMap<String, Type>) {
    match expr {
        HirExpr::Name { name, ty } => {
            refs.entry(name.clone()).or_insert_with(|| ty.clone());
        }
        HirExpr::BinOp { left, right, .. } => {
            collect_typed_refs_in_expr(left, refs);
            collect_typed_refs_in_expr(right, refs);
        }
        HirExpr::BoolOp { values, .. } => {
            for v in values {
                collect_typed_refs_in_expr(v, refs);
            }
        }
        HirExpr::UnaryOp { operand, .. } => {
            collect_typed_refs_in_expr(operand, refs);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            collect_typed_refs_in_expr(left, refs);
            for c in comparators {
                collect_typed_refs_in_expr(c, refs);
            }
        }
        HirExpr::Call { args, .. } => {
            for a in args {
                collect_typed_refs_in_expr(a, refs);
            }
        }
        HirExpr::MethodCall { object, args, .. } => {
            collect_typed_refs_in_expr(object, refs);
            for a in args {
                collect_typed_refs_in_expr(a, refs);
            }
        }
        HirExpr::Index { object, index, .. } => {
            collect_typed_refs_in_expr(object, refs);
            collect_typed_refs_in_expr(index, refs);
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            collect_typed_refs_in_expr(condition, refs);
            collect_typed_refs_in_expr(then_expr, refs);
            collect_typed_refs_in_expr(else_expr, refs);
        }
        HirExpr::ListLiteral { elements, .. }
        | HirExpr::TupleLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. } => {
            for e in elements {
                collect_typed_refs_in_expr(e, refs);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for k in keys {
                collect_typed_refs_in_expr(k, refs);
            }
            for v in values {
                collect_typed_refs_in_expr(v, refs);
            }
        }
        HirExpr::Lambda { body, .. } => {
            collect_typed_refs_in_expr(body, refs);
        }
        _ => {}
    }
}

/// Collect all variable names defined (let-bound) in a list of statements.
/// Does NOT recurse into nested functions.
pub(super) fn collect_locally_defined_vars(stmts: &[HirStmt]) -> HashSet<String> {
    let mut defined = HashSet::new();
    for stmt in stmts {
        match stmt {
            HirStmt::Let { name, .. } => {
                defined.insert(name.clone());
            }
            HirStmt::For { target, body, .. } => {
                defined.insert(target.clone());
                // Also collect from body
                defined.extend(collect_locally_defined_vars(body));
            }
            HirStmt::TupleUnpack { targets, .. } => {
                for (name, _) in targets {
                    defined.insert(name.clone());
                }
            }
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                defined.extend(collect_locally_defined_vars(then_body));
                for (_, body) in elif_clauses {
                    defined.extend(collect_locally_defined_vars(body));
                }
                if let Some(body) = else_body {
                    defined.extend(collect_locally_defined_vars(body));
                }
            }
            HirStmt::While { body, .. } => {
                defined.extend(collect_locally_defined_vars(body));
            }
            HirStmt::NestedFunction { func } => {
                // The nested function name itself is defined
                defined.insert(func.name.clone());
            }
            _ => {}
        }
    }
    defined
}

/// Check if a function body contains calls to a specific function name.
pub(super) fn body_calls_function(stmts: &[HirStmt], func_name: &str) -> bool {
    for stmt in stmts {
        match stmt {
            HirStmt::Let { value, .. } => {
                if expr_calls_function(value, func_name) {
                    return true;
                }
            }
            HirStmt::Assign { value, .. } => {
                if expr_calls_function(value, func_name) {
                    return true;
                }
            }
            HirStmt::AugAssign { value, .. } => {
                if expr_calls_function(value, func_name) {
                    return true;
                }
            }
            HirStmt::Return { value: Some(expr) } => {
                if expr_calls_function(expr, func_name) {
                    return true;
                }
            }
            HirStmt::Expr { expr } => {
                if expr_calls_function(expr, func_name) {
                    return true;
                }
            }
            HirStmt::If {
                condition,
                then_body,
                elif_clauses,
                else_body,
            } => {
                if expr_calls_function(condition, func_name) {
                    return true;
                }
                if body_calls_function(then_body, func_name) {
                    return true;
                }
                for (cond, body) in elif_clauses {
                    if expr_calls_function(cond, func_name) {
                        return true;
                    }
                    if body_calls_function(body, func_name) {
                        return true;
                    }
                }
                if let Some(body) = else_body {
                    if body_calls_function(body, func_name) {
                        return true;
                    }
                }
            }
            HirStmt::While {
                condition, body, ..
            } => {
                if expr_calls_function(condition, func_name) {
                    return true;
                }
                if body_calls_function(body, func_name) {
                    return true;
                }
            }
            HirStmt::For { body, .. } => {
                if body_calls_function(body, func_name) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

pub(super) fn expr_calls_function(expr: &HirExpr, func_name: &str) -> bool {
    match expr {
        HirExpr::Call { func, args, .. } => {
            if func == func_name {
                return true;
            }
            args.iter().any(|a| expr_calls_function(a, func_name))
        }
        HirExpr::BinOp { left, right, .. } => {
            expr_calls_function(left, func_name) || expr_calls_function(right, func_name)
        }
        HirExpr::BoolOp { values, .. } => values.iter().any(|v| expr_calls_function(v, func_name)),
        HirExpr::UnaryOp { operand, .. } => expr_calls_function(operand, func_name),
        HirExpr::Compare {
            left, comparators, ..
        } => {
            expr_calls_function(left, func_name)
                || comparators
                    .iter()
                    .any(|c| expr_calls_function(c, func_name))
        }
        HirExpr::MethodCall { object, args, .. } => {
            expr_calls_function(object, func_name)
                || args.iter().any(|a| expr_calls_function(a, func_name))
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            expr_calls_function(condition, func_name)
                || expr_calls_function(then_expr, func_name)
                || expr_calls_function(else_expr, func_name)
        }
        HirExpr::Index { object, index, .. } => {
            expr_calls_function(object, func_name) || expr_calls_function(index, func_name)
        }
        HirExpr::ListLiteral { elements, .. }
        | HirExpr::TupleLiteral { elements, .. }
        | HirExpr::SetLiteral { elements, .. } => {
            elements.iter().any(|e| expr_calls_function(e, func_name))
        }
        HirExpr::Lambda { body, .. } => expr_calls_function(body, func_name),
        _ => false,
    }
}

impl RustEmitter {
    /// Emit a `BigInt` expression, cloning if it's a variable name (to avoid move).
    pub(super) fn emit_expr_with_bigint_clone(&mut self, expr: &HirExpr) {
        match expr {
            HirExpr::Name { .. } | HirExpr::FieldAccess { .. } => {
                self.emit_expr(expr);
                self.write(".clone()");
            }
            _ => self.emit_expr(expr),
        }
    }
}
