use super::{IsNoneUnionMatch, IsinstanceUnionMatch, ModuleFuncSignatures};
use crate::hir_analysis::{
    queries,
    traversal::{self, TraversalConfig},
};
use sifr_ir::{HirExpr, HirFunction, HirModule, HirStmt};
use sifr_type_system::{OwnershipKind, ParamConvention, Type};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueCategory {
    Place,
    Temporary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceAccessMode {
    Preserve,
    Consume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum YieldMode {
    Copy,
    Clone,
    Move,
    Borrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct IteratorOwnershipPlan {
    pub value_category: ValueCategory,
    pub source_access_mode: SourceAccessMode,
    pub yield_mode: YieldMode,
    pub element_ownership: Option<OwnershipKind>,
}

pub(crate) fn is_reusable_place_expr(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::Name { .. } => true,
        HirExpr::FieldAccess { object, .. } => is_reusable_place_expr(object),
        HirExpr::Index { object, index, .. } => {
            is_reusable_place_expr(object)
                && matches!(
                    index.as_ref(),
                    HirExpr::IntLiteral(_)
                        | HirExpr::StringLiteral(_)
                        | HirExpr::BoolLiteral(_)
                        | HirExpr::Name { .. }
                )
        }
        HirExpr::TupleLiteral { elements, ty } => {
            ty.resolve_alias().ownership() == OwnershipKind::Copy
                && elements.iter().all(is_reusable_place_expr)
        }
        _ => false,
    }
}

pub(crate) fn classify_value_category(expr: &HirExpr) -> ValueCategory {
    if is_reusable_place_expr(expr) {
        ValueCategory::Place
    } else {
        ValueCategory::Temporary
    }
}

pub(crate) fn iteration_element_ownership(source_ty: &Type) -> Option<OwnershipKind> {
    /// Returns `true` for element types where ownership cannot be inferred
    /// soundly from iteration metadata.
    ///
    /// `TypeVar` is handled by `Type::ownership()` (which resolves to `Move`),
    /// while this helper only marks dynamic/unknown shapes as conservative:
    /// `Any`, `Unknown`, and unions/intersections that contain them.
    fn is_conservative_element_type(ty: &Type) -> bool {
        match ty.resolve_alias() {
            Type::Any | Type::Unknown => true,
            Type::Union(members) | Type::Intersection(members) => {
                members.iter().any(is_conservative_element_type)
            }
            _ => false,
        }
    }

    source_ty
        .resolve_alias()
        .iteration_metadata()
        .and_then(|metadata| {
            if is_conservative_element_type(&metadata.element_type) {
                None
            } else {
                Some(metadata.element_type.ownership())
            }
        })
}

pub(crate) fn infer_source_access_mode(
    source_expr: &HirExpr,
    source_ty: &Type,
) -> SourceAccessMode {
    let resolved = source_ty.resolve_alias();
    if matches!(resolved, Type::Iterator(_)) {
        return SourceAccessMode::Consume;
    }
    if resolved.ownership() == OwnershipKind::Copy {
        // Copy-typed sources (for example `range`) can be consumed directly
        // without mutating the original binding.
        return SourceAccessMode::Consume;
    }
    match classify_value_category(source_expr) {
        ValueCategory::Place => SourceAccessMode::Preserve,
        ValueCategory::Temporary => SourceAccessMode::Consume,
    }
}

pub(crate) fn infer_yield_mode(
    source_ty: &Type,
    source_access_mode: SourceAccessMode,
    element_ownership: Option<OwnershipKind>,
) -> YieldMode {
    let resolved = source_ty.resolve_alias();
    if matches!(resolved, Type::Iterator(_)) {
        return YieldMode::Move;
    }
    match source_access_mode {
        SourceAccessMode::Consume => YieldMode::Move,
        SourceAccessMode::Preserve => match element_ownership {
            Some(OwnershipKind::Copy) => YieldMode::Copy,
            Some(OwnershipKind::Move) => YieldMode::Clone,
            None => {
                if matches!(resolved, Type::Str) {
                    YieldMode::Move
                } else {
                    YieldMode::Borrow
                }
            }
        },
    }
}

pub(crate) fn plan_iterator_ownership(source_expr: &HirExpr) -> IteratorOwnershipPlan {
    plan_iterator_ownership_with_element_hint(source_expr, None)
}

pub(crate) fn plan_iterator_ownership_with_element_hint(
    source_expr: &HirExpr,
    element_type_hint: Option<&Type>,
) -> IteratorOwnershipPlan {
    let source_ty = crate::resolve_alias_type_for_plain_call(source_expr.ty());
    let inferred_element_ownership = iteration_element_ownership(source_ty);
    // Keep planner decisions conservative, but allow explicit target-element hints
    // for concrete container sources when iteration metadata is unknown.
    let hint_ownership = element_type_hint.and_then(|hint| match source_ty.resolve_alias() {
        Type::List(_) | Type::Set(_) | Type::Tuple(_) | Type::Dict(_, _) | Type::Iterable(_) => {
            let resolved_hint = hint.resolve_alias();
            if matches!(resolved_hint, Type::Any | Type::Unknown | Type::TypeVar(_)) {
                None
            } else {
                Some(resolved_hint.ownership())
            }
        }
        _ => None,
    });
    let element_ownership = inferred_element_ownership.or(hint_ownership);
    let source_access_mode = infer_source_access_mode(source_expr, source_ty);
    IteratorOwnershipPlan {
        value_category: classify_value_category(source_expr),
        source_access_mode,
        yield_mode: infer_yield_mode(source_ty, source_access_mode, element_ownership),
        element_ownership,
    }
}

pub(crate) fn is_copy_type_for_codegen(ty: &Type) -> bool {
    crate::resolve_alias_type_for_plain_call(ty).ownership() == OwnershipKind::Copy
}

pub(crate) fn option_projection_method_for_owned_type(ty: &Type) -> &'static str {
    if is_copy_type_for_codegen(ty) {
        "copied"
    } else {
        "cloned"
    }
}

/// Check if a type can be auto-formatted with `{}` (implements Display).
/// Used to determine if auto-generated Display impl is safe for a class field.
pub(crate) fn is_auto_display_type(ty: &Type) -> bool {
    ty.supports_display_formatting()
}

/// Returns the default parameter convention for a type.
/// Copy types (int, float, bool) are passed by value (Own).
/// Move types (str, list, dict, class, etc.) are passed by reference (Borrow).
pub(crate) fn default_param_convention(ty: &Type) -> ParamConvention {
    if ty.ownership() == sifr_type_system::OwnershipKind::Copy {
        ParamConvention::own()
    } else {
        ParamConvention::borrow()
    }
}

pub(crate) fn is_option_type(ty: &Type) -> bool {
    let resolved = crate::resolve_alias_type_for_plain_call(ty);
    if let Type::Union(members) = resolved {
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
pub(crate) fn detect_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty } = expr {
        if is_option_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

/// Detect negated truthiness on an Option variable: `if not x:`.
pub(crate) fn detect_not_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::UnaryOp { op, operand, .. } = expr {
        if op == "not" {
            return detect_option_truthiness(operand);
        }
    }
    None
}

/// Detect `x is not None` pattern in a Compare expression. Returns the variable name.
pub(crate) fn detect_is_not_none_var(expr: &HirExpr) -> Option<String> {
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
pub(crate) fn detect_and_not_none_vars(expr: &HirExpr) -> Option<Vec<String>> {
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

/// Detect compound `not a or not b` where both names are Option values.
pub(crate) fn detect_or_not_option_truthiness_vars(expr: &HirExpr) -> Option<Vec<String>> {
    if let HirExpr::BoolOp { op, values, .. } = expr {
        if op == "or" {
            let vars: Vec<String> = values
                .iter()
                .filter_map(detect_not_option_truthiness)
                .collect();
            if vars.len() >= 2 {
                return Some(vars);
            }
        }
    }
    None
}

/// Detect compound `a is None or b is None` where both names are Option values.
pub(crate) fn detect_or_is_none_vars(expr: &HirExpr) -> Option<Vec<String>> {
    if let HirExpr::BoolOp { op, values, .. } = expr {
        if op == "or" {
            let vars: Vec<String> = values.iter().filter_map(detect_is_none_var).collect();
            if vars.len() >= 2 {
                return Some(vars);
            }
        }
    }
    None
}

/// Detect `isinstance(x, type)` where x is a non-Option union type.
/// Returns (`var_name`, `variant_name`, `enum_name`, `other_variants`: Vec<(`variant_name`, type)>).
pub(crate) fn detect_isinstance_union(expr: &HirExpr) -> Option<IsinstanceUnionMatch> {
    if let HirExpr::Call { func, args, .. } = expr {
        if func == "isinstance" && args.len() == 2 {
            if let HirExpr::Name { name, ty } = &args[0] {
                let resolved_ty = crate::resolve_alias_type_for_plain_call(ty);
                if let Type::Union(members) = resolved_ty {
                    if !is_option_type(resolved_ty) {
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
                            let enum_name = resolved_ty.union_enum_name();
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
pub(crate) fn find_union_variant(members: &[Type], arg_ty: &Type) -> Option<String> {
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
pub(crate) fn codegen_body_always_exits(stmts: &[HirStmt]) -> bool {
    stmts.iter().any(stmt_always_leaves_current_path)
}

fn stmt_always_leaves_current_path(stmt: &HirStmt) -> bool {
    match stmt {
        HirStmt::Return { .. } | HirStmt::Break | HirStmt::Continue | HirStmt::Raise { .. } => true,
        HirStmt::If {
            then_body,
            elif_clauses,
            else_body: Some(else_body),
            ..
        } => {
            codegen_body_always_exits(then_body)
                && elif_clauses
                    .iter()
                    .all(|(_, body)| codegen_body_always_exits(body))
                && codegen_body_always_exits(else_body)
        }
        _ => false,
    }
}

/// Detect `x is None` pattern. Returns the variable name.
/// Only matches when the variable type is an Option (T | None with exactly 2 members).
pub(crate) fn detect_is_none_var(expr: &HirExpr) -> Option<String> {
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
pub(crate) fn detect_is_none_union_var(expr: &HirExpr) -> Option<IsNoneUnionMatch> {
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

/// Check if a module uses the `bigint` type anywhere.
pub(crate) fn module_uses_bigint(module: &HirModule) -> bool {
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
            HirStmt::For { target_ty, .. } | HirStmt::AsyncFor { target_ty, .. } => {
                type_has_bigint(target_ty)
            }
            HirStmt::TupleUnpack { targets, .. } => {
                targets.iter().any(|target| type_has_bigint(&target.ty))
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
            HirStmt::AttributeNestedSubscriptAssign { field_ty, .. }
            | HirStmt::AttributeSubscriptAssign { field_ty, .. } => type_has_bigint(field_ty),
            HirStmt::NestedFieldAssign {
                nested_field_ty, ..
            } => type_has_bigint(nested_field_ty),
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
        traversal::walk_stmts(
            &func.body,
            TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
            &mut on_stmt,
            &mut on_expr,
        );
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
pub(crate) fn collect_string_concat_parts<'a>(expr: &'a HirExpr, parts: &mut Vec<&'a HirExpr>) {
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
pub(crate) fn body_contains_field_assign_codegen(stmts: &[HirStmt]) -> bool {
    let found = std::cell::Cell::new(false);
    let mut on_stmt = |stmt: &HirStmt| {
        if matches!(
            stmt,
            HirStmt::FieldAssign { .. }
                | HirStmt::NestedFieldAssign { .. }
                | HirStmt::AttributeAugAssign { .. }
                | HirStmt::AttributeNestedSubscriptAssign { .. }
                | HirStmt::AttributeSubscriptAssign { .. }
        ) {
            found.set(true);
        } else if let HirStmt::TupleUnpack { targets, .. } = stmt {
            if targets.iter().any(|target| {
                matches!(target.binding, sifr_ir::HirTupleTargetBinding::Field { .. })
            }) {
                found.set(true);
            }
        }
    };
    let mut on_expr = |expr: &HirExpr| {
        if !found.get() && is_self_field_mutating_method_call(expr) {
            found.set(true);
        }
    };
    traversal::walk_stmts(
        stmts,
        TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
        &mut on_stmt,
        &mut on_expr,
    );
    found.get()
}

/// Check if an expression contains a mutating method call on a self field (e.g., self.items.append(...)).
pub(crate) fn expr_contains_self_field_mutation(expr: &HirExpr) -> bool {
    let mut found = false;
    traversal::walk_expr(expr, &mut |candidate| {
        if !found && is_self_field_mutating_method_call(candidate) {
            found = true;
        }
    });
    found
}

pub(crate) fn is_self_field_mutating_method_call(expr: &HirExpr) -> bool {
    match expr {
        HirExpr::MethodCall { object, method, .. } => {
            let is_self_field = matches!(object.as_ref(), HirExpr::FieldAccess { object: inner, .. }
                if matches!(inner.as_ref(), HirExpr::Name { name, .. } if name == "self"));
            is_self_field
                && (MUTATING_METHODS.contains(&method.as_str())
                    || matches!(
                        crate::resolve_alias_type_for_plain_call(object.ty()),
                        Type::Class { .. }
                    ))
        }
        _ => false,
    }
}

/// Check if a type contains a specific type variable name.
pub(crate) fn type_contains_typevar(ty: &Type, tv_name: &str) -> bool {
    match ty {
        Type::TypeVar(name) => name == tv_name,
        Type::List(inner) | Type::PythonBuffer(inner) => type_contains_typevar(inner, tv_name),
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
pub(crate) fn type_references_class(ty: &Type, class_name: &str) -> bool {
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

pub(crate) fn type_references_any_class(
    ty: &Type,
    class_names: &std::collections::HashSet<String>,
) -> bool {
    match ty {
        Type::Class { name, .. } => class_names.contains(name),
        Type::Union(members) => members
            .iter()
            .any(|m| type_references_any_class(m, class_names)),
        Type::List(inner) => type_references_any_class(inner, class_names),
        Type::Dict(key, val) => {
            type_references_any_class(key, class_names)
                || type_references_any_class(val, class_names)
        }
        Type::Tuple(elems) => elems
            .iter()
            .any(|e| type_references_any_class(e, class_names)),
        Type::Result(ok, err) => {
            type_references_any_class(ok, class_names)
                || type_references_any_class(err, class_names)
        }
        Type::Alias { body, .. } => type_references_any_class(body, class_names),
        _ => false,
    }
}

/// Check if a variable name is referenced anywhere in a list of statements.
pub(crate) fn stmts_reference_var(stmts: &[HirStmt], var_name: &str) -> bool {
    queries::stmts_reference_var(stmts, var_name)
}

/// Check if an expression references a variable name.
pub(crate) fn expr_references_var(expr: &HirExpr, var_name: &str) -> bool {
    queries::expr_references_var(expr, var_name)
}

/// Check if a function body contains any yield statements (making it a generator).
pub(crate) fn body_contains_return_stmt(stmts: &[HirStmt]) -> bool {
    queries::body_contains_return(stmts)
}

/// Check if a try body contains a return statement with a non-unit value.
/// Used to determine if the try closure needs to return T instead of ().
pub(crate) fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    queries::try_body_has_value_return(stmts)
}

pub(crate) fn body_contains_yield_inner(stmts: &[HirStmt]) -> bool {
    queries::body_contains_yield(stmts)
}

/// Check if a type needs .`clone()` when accessed from &self (non-Copy types).
pub(crate) fn needs_clone_for_type(ty: &Type) -> bool {
    ty.ownership() == sifr_type_system::OwnershipKind::Move
}

/// Mutating methods that require the receiver variable to be `mut`.
pub(crate) const MUTATING_METHODS: &[&str] = queries::MUTATING_METHODS;

/// Collect the set of variable names that are mutated in a function body.
/// A variable is mutated if it appears in:
/// - `HirStmt::Assign` (reassignment)
/// - `HirStmt::AugAssign` (augmented assignment like +=)
/// - `HirStmt::Expr` containing a `MethodCall` on the variable with a mutating method
/// - `HirStmt::Delete` on the variable
pub(crate) fn collect_mutated_vars(stmts: &[HirStmt]) -> HashSet<String> {
    queries::collect_mutated_vars(stmts, None)
}

pub(crate) fn collect_mutated_vars_with_sigs(
    stmts: &[HirStmt],
    func_signatures: &ModuleFuncSignatures,
) -> HashSet<String> {
    queries::collect_mutated_vars(stmts, Some(func_signatures))
}

pub(crate) fn collect_reassigned_vars(stmts: &[HirStmt]) -> HashSet<String> {
    queries::collect_reassigned_vars(stmts)
}

/// Collect all variable names and their types referenced in a list of statements.
pub(crate) fn collect_referenced_vars_with_types(stmts: &[HirStmt]) -> Vec<(String, Type)> {
    queries::collect_referenced_vars_with_types(stmts)
}

pub(crate) fn collect_typed_refs_in_expr(expr: &HirExpr, refs: &mut HashMap<String, Type>) {
    queries::collect_typed_refs_in_expr(expr, refs);
}

/// Collect all variable names defined (let-bound) in a list of statements.
/// Does NOT recurse into nested functions.
pub(crate) fn collect_locally_defined_vars(stmts: &[HirStmt]) -> HashSet<String> {
    queries::collect_locally_defined_vars(stmts)
}

/// Check if a function body contains calls to a specific function name.
pub(crate) fn body_calls_function(stmts: &[HirStmt], func_name: &str) -> bool {
    queries::body_calls_function(stmts, func_name)
}

pub(crate) fn expr_calls_function(expr: &HirExpr, func_name: &str) -> bool {
    queries::expr_calls_function(expr, func_name)
}
