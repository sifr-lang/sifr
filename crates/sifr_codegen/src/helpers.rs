use super::{IsNoneUnionMatch, IsinstanceUnionMatch, ModuleFuncSignatures};
use sifr_hir::{HirExpr, HirFStringPart, HirFunction, HirModule, HirPattern, HirStmt};
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

fn walk_hir_expr<F>(expr: &HirExpr, on_expr: &mut F)
where
    F: FnMut(&HirExpr),
{
    on_expr(expr);
    match expr {
        HirExpr::BinOp { left, right, .. } => {
            walk_hir_expr(left, on_expr);
            walk_hir_expr(right, on_expr);
        }
        HirExpr::UnaryOp { operand, .. }
        | HirExpr::QuestionMark { expr: operand, .. }
        | HirExpr::OkWrap { value: operand, .. }
        | HirExpr::ErrWrap { value: operand, .. }
        | HirExpr::WalrusExpr { value: operand, .. }
        | HirExpr::FieldAccess {
            object: operand, ..
        } => {
            walk_hir_expr(operand, on_expr);
        }
        HirExpr::Compare {
            left, comparators, ..
        } => {
            walk_hir_expr(left, on_expr);
            for comparator in comparators {
                walk_hir_expr(comparator, on_expr);
            }
        }
        HirExpr::BoolOp { values, .. }
        | HirExpr::ListLiteral {
            elements: values, ..
        }
        | HirExpr::SetLiteral {
            elements: values, ..
        }
        | HirExpr::TupleLiteral {
            elements: values, ..
        } => {
            for value in values {
                walk_hir_expr(value, on_expr);
            }
        }
        HirExpr::Call { args, .. }
        | HirExpr::ConstructorCall { args, .. }
        | HirExpr::SuperCall { args, .. } => {
            for arg in args {
                walk_hir_expr(arg, on_expr);
            }
        }
        HirExpr::MethodCall { object, args, .. } => {
            walk_hir_expr(object, on_expr);
            for arg in args {
                walk_hir_expr(arg, on_expr);
            }
        }
        HirExpr::IfExpr {
            condition,
            then_expr,
            else_expr,
            ..
        } => {
            walk_hir_expr(condition, on_expr);
            walk_hir_expr(then_expr, on_expr);
            walk_hir_expr(else_expr, on_expr);
        }
        HirExpr::RangeLiteral {
            start, end, step, ..
        } => {
            walk_hir_expr(start, on_expr);
            walk_hir_expr(end, on_expr);
            if let Some(step) = step {
                walk_hir_expr(step, on_expr);
            }
        }
        HirExpr::DictLiteral { keys, values, .. } => {
            for key in keys {
                walk_hir_expr(key, on_expr);
            }
            for value in values {
                walk_hir_expr(value, on_expr);
            }
        }
        HirExpr::Index { object, index, .. } => {
            walk_hir_expr(object, on_expr);
            walk_hir_expr(index, on_expr);
        }
        HirExpr::ContainsOp {
            element,
            collection,
            ..
        } => {
            walk_hir_expr(element, on_expr);
            walk_hir_expr(collection, on_expr);
        }
        HirExpr::FString { parts, .. } => {
            for part in parts {
                if let HirFStringPart::Expr(expr) = part {
                    walk_hir_expr(expr, on_expr);
                }
            }
        }
        HirExpr::Slice {
            object,
            start,
            stop,
            step,
            ..
        } => {
            walk_hir_expr(object, on_expr);
            if let Some(start) = start {
                walk_hir_expr(start, on_expr);
            }
            if let Some(stop) = stop {
                walk_hir_expr(stop, on_expr);
            }
            if let Some(step) = step {
                walk_hir_expr(step, on_expr);
            }
        }
        HirExpr::Lambda { body, .. } => walk_hir_expr(body, on_expr),
        HirExpr::ListComp {
            expr, generators, ..
        }
        | HirExpr::SetComp {
            expr, generators, ..
        } => {
            walk_hir_expr(expr, on_expr);
            for (_, iter, filter) in generators {
                walk_hir_expr(iter, on_expr);
                if let Some(filter) = filter {
                    walk_hir_expr(filter, on_expr);
                }
            }
        }
        HirExpr::DictComp {
            key_expr,
            val_expr,
            generators,
            ..
        } => {
            walk_hir_expr(key_expr, on_expr);
            walk_hir_expr(val_expr, on_expr);
            for (_, iter, filter) in generators {
                walk_hir_expr(iter, on_expr);
                if let Some(filter) = filter {
                    walk_hir_expr(filter, on_expr);
                }
            }
        }
        HirExpr::GeneratorExpr {
            expr, iter, filter, ..
        } => {
            walk_hir_expr(expr, on_expr);
            walk_hir_expr(iter, on_expr);
            if let Some(filter) = filter {
                walk_hir_expr(filter, on_expr);
            }
        }
        HirExpr::Name { .. }
        | HirExpr::IntLiteral(_)
        | HirExpr::FloatLiteral(_)
        | HirExpr::StringLiteral(_)
        | HirExpr::BoolLiteral(_)
        | HirExpr::NoneLiteral
        | HirExpr::EnumVariant { .. } => {}
    }
}

fn walk_hir_pattern<F>(pattern: &HirPattern, on_expr: &mut F)
where
    F: FnMut(&HirExpr),
{
    match pattern {
        HirPattern::Literal { value } => walk_hir_expr(value, on_expr),
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => {
            for pattern in patterns {
                walk_hir_pattern(pattern, on_expr);
            }
        }
        HirPattern::Class { fields, .. } => {
            for (_, pattern) in fields {
                walk_hir_pattern(pattern, on_expr);
            }
        }
        HirPattern::Wildcard
        | HirPattern::Capture { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => {}
    }
}

fn walk_hir_stmt<FStmt, FExpr>(
    stmt: &HirStmt,
    descend_nested_functions: bool,
    on_stmt: &mut FStmt,
    on_expr: &mut FExpr,
) where
    FStmt: FnMut(&HirStmt),
    FExpr: FnMut(&HirExpr),
{
    on_stmt(stmt);
    match stmt {
        HirStmt::Let { value, .. }
        | HirStmt::Assign { value, .. }
        | HirStmt::AugAssign { value, .. }
        | HirStmt::AttributeAugAssign { value, .. }
        | HirStmt::FieldAssign { value, .. }
        | HirStmt::Raise { value }
        | HirStmt::Yield { value } => walk_hir_expr(value, on_expr),
        HirStmt::Return { value } => {
            if let Some(value) = value {
                walk_hir_expr(value, on_expr);
            }
        }
        HirStmt::Expr { expr } => walk_hir_expr(expr, on_expr),
        HirStmt::If {
            condition,
            then_body,
            elif_clauses,
            else_body,
        } => {
            walk_hir_expr(condition, on_expr);
            walk_hir_stmts(then_body, descend_nested_functions, on_stmt, on_expr);
            for (cond, body) in elif_clauses {
                walk_hir_expr(cond, on_expr);
                walk_hir_stmts(body, descend_nested_functions, on_stmt, on_expr);
            }
            if let Some(else_body) = else_body {
                walk_hir_stmts(else_body, descend_nested_functions, on_stmt, on_expr);
            }
        }
        HirStmt::While {
            condition,
            body,
            else_body,
        } => {
            walk_hir_expr(condition, on_expr);
            walk_hir_stmts(body, descend_nested_functions, on_stmt, on_expr);
            if let Some(else_body) = else_body {
                walk_hir_stmts(else_body, descend_nested_functions, on_stmt, on_expr);
            }
        }
        HirStmt::For {
            iter,
            body,
            else_body,
            ..
        } => {
            walk_hir_expr(iter, on_expr);
            walk_hir_stmts(body, descend_nested_functions, on_stmt, on_expr);
            if let Some(else_body) = else_body {
                walk_hir_stmts(else_body, descend_nested_functions, on_stmt, on_expr);
            }
        }
        HirStmt::TupleUnpack { value, .. } | HirStmt::StarUnpack { value, .. } => {
            walk_hir_expr(value, on_expr);
        }
        HirStmt::Assert { test, msg } => {
            walk_hir_expr(test, on_expr);
            if let Some(msg) = msg {
                walk_hir_expr(msg, on_expr);
            }
        }
        HirStmt::TryExcept { body, handlers, .. } => {
            // HIR TryExcept currently has no dedicated `else_body` field.
            // Any equivalent behavior is represented with explicit control flow
            // inside `body`/handler blocks and is traversed through these walks.
            walk_hir_stmts(body, descend_nested_functions, on_stmt, on_expr);
            for handler in handlers {
                walk_hir_stmts(&handler.body, descend_nested_functions, on_stmt, on_expr);
            }
        }
        HirStmt::SubscriptAssign { index, value, .. }
        | HirStmt::SubscriptAugAssign { index, value, .. }
        | HirStmt::AttributeSubscriptAssign { index, value, .. } => {
            walk_hir_expr(index, on_expr);
            walk_hir_expr(value, on_expr);
        }
        HirStmt::NestedSubscriptAssign {
            outer_index,
            inner_index,
            value,
            ..
        } => {
            walk_hir_expr(outer_index, on_expr);
            walk_hir_expr(inner_index, on_expr);
            walk_hir_expr(value, on_expr);
        }
        HirStmt::Delete { object, index } => {
            walk_hir_expr(object, on_expr);
            walk_hir_expr(index, on_expr);
        }
        HirStmt::With { items, body } => {
            for (_, expr, _) in items {
                walk_hir_expr(expr, on_expr);
            }
            walk_hir_stmts(body, descend_nested_functions, on_stmt, on_expr);
        }
        HirStmt::NestedFunction { func } => {
            if descend_nested_functions {
                walk_hir_stmts(&func.body, true, on_stmt, on_expr);
            }
        }
        HirStmt::Match { subject, arms, .. } => {
            walk_hir_expr(subject, on_expr);
            for arm in arms {
                walk_hir_pattern(&arm.pattern, on_expr);
                if let Some(guard) = &arm.guard {
                    walk_hir_expr(guard, on_expr);
                }
                walk_hir_stmts(&arm.body, descend_nested_functions, on_stmt, on_expr);
            }
        }
        HirStmt::Pass | HirStmt::Break | HirStmt::Continue => {}
    }
}

fn walk_hir_stmts<FStmt, FExpr>(
    stmts: &[HirStmt],
    descend_nested_functions: bool,
    on_stmt: &mut FStmt,
    on_expr: &mut FExpr,
) where
    FStmt: FnMut(&HirStmt),
    FExpr: FnMut(&HirExpr),
{
    for stmt in stmts {
        walk_hir_stmt(stmt, descend_nested_functions, on_stmt, on_expr);
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
                        let type_name = match &args[1] {
                            HirExpr::StringLiteral(type_name) => type_name.as_str(),
                            HirExpr::Name { name, .. } => name.as_str(),
                            _ => return None,
                        };
                        let target_ty = match type_name {
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
    fn stmt_type_has_bigint(stmt: &HirStmt) -> bool {
        match stmt {
            HirStmt::Let { ty, .. } => type_has_bigint(ty),
            HirStmt::For { target_ty, .. } => type_has_bigint(target_ty),
            HirStmt::TupleUnpack { targets, .. } => {
                targets.iter().any(|(_, ty)| type_has_bigint(ty))
            }
            HirStmt::StarUnpack {
                before,
                star,
                after,
                ..
            } => {
                before.iter().any(|(_, ty)| type_has_bigint(ty))
                    || type_has_bigint(&star.1)
                    || after.iter().any(|(_, ty)| type_has_bigint(ty))
            }
            HirStmt::SubscriptAssign { object_ty, .. }
            | HirStmt::NestedSubscriptAssign { object_ty, .. }
            | HirStmt::SubscriptAugAssign { object_ty, .. } => type_has_bigint(object_ty),
            HirStmt::AttributeSubscriptAssign { field_ty, .. } => type_has_bigint(field_ty),
            HirStmt::Match { subject_ty, .. } => type_has_bigint(subject_ty),
            HirStmt::NestedFunction { func } => {
                func.params.iter().any(|param| type_has_bigint(&param.ty))
                    || type_has_bigint(&func.return_type)
            }
            _ => false,
        }
    }
    fn function_uses_bigint(func: &HirFunction) -> bool {
        if type_has_bigint(&func.return_type) {
            return true;
        }
        if func.params.iter().any(|p| type_has_bigint(&p.ty)) {
            return true;
        }
        let found = std::cell::Cell::new(false);
        let mut on_stmt = |stmt: &HirStmt| {
            if !found.get() && stmt_type_has_bigint(stmt) {
                found.set(true);
            }
        };
        let mut on_expr = |expr: &HirExpr| {
            if !found.get() && type_has_bigint(expr.ty()) {
                found.set(true);
            }
        };
        walk_hir_stmts(&func.body, true, &mut on_stmt, &mut on_expr);
        found.get()
    }
    for func in &module.functions {
        if function_uses_bigint(func) {
            return true;
        }
    }
    for class in &module.classes {
        if class.fields.iter().any(|(_, t)| type_has_bigint(t)) {
            return true;
        }
        for method in &class.methods {
            if function_uses_bigint(method) {
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
    let found = std::cell::Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        if matches!(
            stmt,
            HirStmt::FieldAssign { .. }
                | HirStmt::AttributeAugAssign { .. }
                | HirStmt::AttributeSubscriptAssign { .. }
        ) {
            found.set(true);
        }
    };
    let mut on_expr = |expr: &HirExpr| {
        if !found.get() && is_self_field_mutating_method_call(expr) {
            found.set(true);
        }
    };
    walk_hir_stmts(stmts, true, &mut on_stmt, &mut on_expr);
    found.get()
}

/// Check if an expression contains a mutating method call on a self field (e.g., self.items.append(...)).
pub(super) fn expr_contains_self_field_mutation(expr: &HirExpr) -> bool {
    let mut found = false;
    walk_hir_expr(expr, &mut |candidate| {
        if !found && is_self_field_mutating_method_call(candidate) {
            found = true;
        }
    });
    found
}

fn is_self_field_mutating_method_call(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::MethodCall { object, method, .. } => {
            let is_self_field = matches!(object.as_ref(), HirExpr::FieldAccess { object: inner, .. }
                if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
            is_self_field && MUTATING_METHODS.contains(&method.as_str())
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
pub(super) fn body_contains_return_stmt(stmts: &[HirStmt]) -> bool {
    let found = std::cell::Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        if matches!(stmt, HirStmt::Return { .. }) {
            found.set(true);
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    walk_hir_stmts(stmts, false, &mut on_stmt, &mut on_expr);
    found.get()
}

/// Check if a try body contains a return statement with a non-unit value.
/// Used to determine if the try closure needs to return T instead of ().
pub(super) fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    let found = std::cell::Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        if let HirStmt::Return { value: Some(val) } = stmt {
            if !matches!(val, HirExpr::NoneLiteral) {
                found.set(true);
            }
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    walk_hir_stmts(stmts, false, &mut on_stmt, &mut on_expr);
    found.get()
}

pub(super) fn body_contains_yield_inner(stmts: &[HirStmt]) -> bool {
    let found = std::cell::Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        if matches!(stmt, HirStmt::Yield { .. }) {
            found.set(true);
        }
    };
    let mut on_expr = |_expr: &HirExpr| {};
    walk_hir_stmts(stmts, false, &mut on_stmt, &mut on_expr);
    found.get()
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
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if let HirExpr::Name { name, ty } = expr {
            refs.entry(name.clone()).or_insert_with(|| ty.clone());
        }
    };
    walk_hir_stmts(stmts, false, &mut on_stmt, &mut on_expr);
}

pub(super) fn collect_typed_refs_in_expr(expr: &HirExpr, refs: &mut HashMap<String, Type>) {
    walk_hir_expr(expr, &mut |node| {
        if let HirExpr::Name { name, ty } = node {
            refs.entry(name.clone()).or_insert_with(|| ty.clone());
        }
    });
}

/// Collect all variable names defined (let-bound) in a list of statements.
/// Does NOT recurse into nested functions.
pub(super) fn collect_locally_defined_vars(stmts: &[HirStmt]) -> HashSet<String> {
    let mut defined = HashSet::new();
    let mut on_stmt = |stmt: &HirStmt| match stmt {
        HirStmt::Let { name, .. } => {
            defined.insert(name.clone());
        }
        HirStmt::For { target, .. } => {
            defined.insert(target.clone());
        }
        HirStmt::TupleUnpack { targets, .. } => {
            for (name, _) in targets {
                defined.insert(name.clone());
            }
        }
        HirStmt::StarUnpack {
            before,
            star,
            after,
            ..
        } => {
            for (name, _) in before {
                defined.insert(name.clone());
            }
            defined.insert(star.0.clone());
            for (name, _) in after {
                defined.insert(name.clone());
            }
        }
        HirStmt::NestedFunction { func } => {
            defined.insert(func.name.clone());
        }
        HirStmt::Match { arms, .. } => {
            for arm in arms {
                collect_capture_pattern_names(&arm.pattern, &mut defined);
            }
        }
        _ => {}
    };
    let mut on_expr = |_expr: &HirExpr| {};
    walk_hir_stmts(stmts, false, &mut on_stmt, &mut on_expr);
    defined
}

fn collect_capture_pattern_names(pattern: &HirPattern, defined: &mut HashSet<String>) {
    match pattern {
        HirPattern::Capture { name, .. } => {
            defined.insert(name.clone());
        }
        HirPattern::Or { patterns } | HirPattern::Tuple { elements: patterns } => {
            for pattern in patterns {
                collect_capture_pattern_names(pattern, defined);
            }
        }
        HirPattern::Class { fields, .. } => {
            for (_, pattern) in fields {
                collect_capture_pattern_names(pattern, defined);
            }
        }
        HirPattern::Wildcard
        | HirPattern::Literal { .. }
        | HirPattern::None
        | HirPattern::Value { .. } => {}
    }
}

/// Check if a function body contains calls to a specific function name.
pub(super) fn body_calls_function(stmts: &[HirStmt], func_name: &str) -> bool {
    let mut found = false;
    let mut on_stmt = |_stmt: &HirStmt| {};
    let mut on_expr = |expr: &HirExpr| {
        if found {
            return;
        }
        if let HirExpr::Call { func, .. } = expr {
            if func == func_name {
                found = true;
            }
        }
    };
    walk_hir_stmts(stmts, false, &mut on_stmt, &mut on_expr);
    found
}

pub(super) fn expr_calls_function(expr: &HirExpr, func_name: &str) -> bool {
    let mut found = false;
    walk_hir_expr(expr, &mut |node| {
        if found {
            return;
        }
        if let HirExpr::Call { func, .. } = node {
            if func == func_name {
                found = true;
            }
        }
    });
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use sifr_hir::{HirExceptHandler, HirFunction, HirModule, HirParam, MethodKind};

    fn mk_function(name: &str, body: Vec<HirStmt>) -> HirFunction {
        HirFunction {
            name: name.to_string(),
            params: vec![],
            return_type: Type::None,
            body,
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        }
    }

    fn mk_module_with_main(body: Vec<HirStmt>) -> HirModule {
        HirModule {
            functions: vec![mk_function("main", body)],
            classes: vec![],
            imports: vec![],
            constants: vec![],
            generic_functions: HashMap::new(),
            type_param_bounds: HashMap::new(),
        }
    }

    #[test]
    fn body_calls_function_detects_calls_in_for_else() {
        let stmts = vec![HirStmt::For {
            target: "i".to_string(),
            target_ty: Type::Int,
            iter: HirExpr::ListLiteral {
                elements: vec![],
                ty: Type::List(Box::new(Type::Int)),
            },
            body: vec![HirStmt::Pass],
            else_body: Some(vec![HirStmt::Expr {
                expr: HirExpr::Call {
                    func: "rec".to_string(),
                    args: vec![HirExpr::IntLiteral(1)],
                    ty: Type::Int,
                },
            }]),
        }];

        assert!(body_calls_function(&stmts, "rec"));
    }

    #[test]
    fn body_calls_function_ignores_nested_function_scope() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![HirParam {
                name: "n".to_string(),
                ty: Type::Int,
                default: None,
                keyword_only: false,
                convention: ParamConvention::Own,
            }],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::Call {
                    func: "target".to_string(),
                    args: vec![HirExpr::Name {
                        name: "n".to_string(),
                        ty: Type::Int,
                    }],
                    ty: Type::Int,
                }),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };
        let stmts = vec![HirStmt::NestedFunction { func: nested }];

        assert!(!body_calls_function(&stmts, "target"));
    }

    #[test]
    fn collect_locally_defined_vars_includes_else_and_star_unpack() {
        let stmts = vec![
            HirStmt::For {
                target: "item".to_string(),
                target_ty: Type::Int,
                iter: HirExpr::ListLiteral {
                    elements: vec![],
                    ty: Type::List(Box::new(Type::Int)),
                },
                body: vec![HirStmt::Pass],
                else_body: Some(vec![HirStmt::Let {
                    name: "from_else".to_string(),
                    ty: Type::Int,
                    value: HirExpr::IntLiteral(1),
                    is_mutable: true,
                }]),
            },
            HirStmt::StarUnpack {
                before: vec![("first".to_string(), Type::Int)],
                star: ("rest".to_string(), Type::List(Box::new(Type::Int))),
                after: vec![("last".to_string(), Type::Int)],
                value: HirExpr::ListLiteral {
                    elements: vec![HirExpr::IntLiteral(1)],
                    ty: Type::List(Box::new(Type::Int)),
                },
            },
        ];

        let defined = collect_locally_defined_vars(&stmts);
        assert!(defined.contains("item"));
        assert!(defined.contains("from_else"));
        assert!(defined.contains("first"));
        assert!(defined.contains("rest"));
        assert!(defined.contains("last"));
    }

    #[test]
    fn body_contains_yield_detects_try_except_and_loop_else_paths() {
        let stmts = vec![HirStmt::TryExcept {
            body: vec![HirStmt::While {
                condition: HirExpr::BoolLiteral(false),
                body: vec![HirStmt::Pass],
                else_body: Some(vec![HirStmt::Yield {
                    value: HirExpr::IntLiteral(1),
                }]),
            }],
            handlers: vec![HirExceptHandler {
                error_type: Some("Error".to_string()),
                error_resolved_type: None,
                name: Some("e".to_string()),
                body: vec![HirStmt::Yield {
                    value: HirExpr::IntLiteral(2),
                }],
            }],
            body_error_types: vec!["Error".to_string()],
        }];

        assert!(body_contains_yield_inner(&stmts));
    }

    #[test]
    fn body_contains_return_detects_try_handlers_and_loop_else_paths() {
        let stmts = vec![HirStmt::TryExcept {
            body: vec![HirStmt::For {
                target: "i".to_string(),
                target_ty: Type::Int,
                iter: HirExpr::ListLiteral {
                    elements: vec![],
                    ty: Type::List(Box::new(Type::Int)),
                },
                body: vec![HirStmt::Pass],
                else_body: Some(vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(1)),
                }]),
            }],
            handlers: vec![HirExceptHandler {
                error_type: Some("Error".to_string()),
                error_resolved_type: None,
                name: Some("e".to_string()),
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(2)),
                }],
            }],
            body_error_types: vec!["Error".to_string()],
        }];

        assert!(body_contains_return_stmt(&stmts));
    }

    #[test]
    fn body_contains_return_ignores_nested_function_scope() {
        let nested = HirFunction {
            name: "inner".to_string(),
            params: vec![],
            return_type: Type::Int,
            body: vec![HirStmt::Return {
                value: Some(HirExpr::IntLiteral(1)),
            }],
            method_kind: MethodKind::Regular,
            decorators: vec![],
            type_params: vec![],
        };
        let stmts = vec![HirStmt::NestedFunction { func: nested }];

        assert!(!body_contains_return_stmt(&stmts));
    }

    #[test]
    fn try_body_has_value_return_detects_loop_else_and_try_handler_returns() {
        let stmts = vec![HirStmt::TryExcept {
            body: vec![HirStmt::For {
                target: "i".to_string(),
                target_ty: Type::Int,
                iter: HirExpr::ListLiteral {
                    elements: vec![],
                    ty: Type::List(Box::new(Type::Int)),
                },
                body: vec![HirStmt::Pass],
                else_body: Some(vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(9)),
                }]),
            }],
            handlers: vec![HirExceptHandler {
                error_type: Some("Error".to_string()),
                error_resolved_type: None,
                name: Some("e".to_string()),
                body: vec![HirStmt::Return {
                    value: Some(HirExpr::IntLiteral(7)),
                }],
            }],
            body_error_types: vec!["Error".to_string()],
        }];

        assert!(try_body_has_value_return(&stmts));
    }

    #[test]
    fn try_body_has_value_return_ignores_return_none() {
        let stmts = vec![HirStmt::Return {
            value: Some(HirExpr::NoneLiteral),
        }];
        assert!(!try_body_has_value_return(&stmts));
    }

    #[test]
    fn module_uses_bigint_detects_try_handler_branches() {
        let module = mk_module_with_main(vec![HirStmt::TryExcept {
            body: vec![HirStmt::Pass],
            handlers: vec![HirExceptHandler {
                error_type: Some("Error".to_string()),
                error_resolved_type: None,
                name: Some("e".to_string()),
                body: vec![HirStmt::Let {
                    name: "n".to_string(),
                    ty: Type::BigInt,
                    value: HirExpr::Call {
                        func: "bigint".to_string(),
                        args: vec![HirExpr::IntLiteral(3)],
                        ty: Type::BigInt,
                    },
                    is_mutable: true,
                }],
            }],
            body_error_types: vec!["Error".to_string()],
        }]);

        assert!(module_uses_bigint(&module));
    }

    #[test]
    fn module_uses_bigint_false_without_bigint() {
        let module = mk_module_with_main(vec![HirStmt::Let {
            name: "x".to_string(),
            ty: Type::Int,
            value: HirExpr::IntLiteral(1),
            is_mutable: true,
        }]);

        assert!(!module_uses_bigint(&module));
    }
}
