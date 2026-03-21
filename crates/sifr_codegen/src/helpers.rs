use super::{IsNoneUnionMatch, IsinstanceUnionMatch, ModuleFuncSignatures};
use crate::hir_analysis::{
    queries,
    traversal::{self, TraversalConfig},
};
use sifr_hir::{HirExpr, HirFunction, HirModule, HirStmt};
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

fn is_reusable_place_expr(expr: &HirExpr) -> bool {
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

fn iteration_element_ownership(source_ty: &Type) -> Option<OwnershipKind> {
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

fn infer_source_access_mode(source_expr: &HirExpr, source_ty: &Type) -> SourceAccessMode {
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

fn infer_yield_mode(
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
    let _ = element_type_hint;
    // Keep planner decisions conservative: if element ownership cannot be inferred
    // from source iteration metadata, leave it unknown (`None`) and let lowering
    // default to borrowing behavior.
    let element_ownership = inferred_element_ownership;
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
        ParamConvention::own()
    } else {
        ParamConvention::borrow()
    }
}

pub(super) fn is_option_type(ty: &Type) -> bool {
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
pub(super) fn detect_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty } = expr {
        if is_option_type(ty) {
            return Some(name.clone());
        }
    }
    None
}

/// Detect negated truthiness on an Option variable: `if not x:`.
pub(super) fn detect_not_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::UnaryOp { op, operand, .. } = expr {
        if op == "not" {
            return detect_option_truthiness(operand);
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

/// Detect compound `not a or not b` where both names are Option values.
pub(super) fn detect_or_not_option_truthiness_vars(expr: &HirExpr) -> Option<Vec<String>> {
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

/// Detect `isinstance(x, type)` where x is a non-Option union type.
/// Returns (`var_name`, `variant_name`, `enum_name`, `other_variants`: Vec<(`variant_name`, type)>).
pub(super) fn detect_isinstance_union(expr: &HirExpr) -> Option<IsinstanceUnionMatch> {
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
    queries::block_control_flow_effect(stmts).always_exits()
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
        Type::Int | Type::Bool | Type::Str | Type::None | Type::BigInt | Type::Decimal => true,
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
    traversal::walk_stmts(
        stmts,
        TraversalConfig::INCLUDE_NESTED_FUNCTIONS,
        &mut on_stmt,
        &mut on_expr,
    );
    found.get()
}

/// Check if an expression contains a mutating method call on a self field (e.g., self.items.append(...)).
pub(super) fn expr_contains_self_field_mutation(expr: &HirExpr) -> bool {
    let mut found = false;
    traversal::walk_expr(expr, &mut |candidate| {
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

pub(super) fn type_references_any_class(
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
pub(super) fn stmts_reference_var(stmts: &[HirStmt], var_name: &str) -> bool {
    queries::stmts_reference_var(stmts, var_name)
}

/// Check if an expression references a variable name.
pub(super) fn expr_references_var(expr: &HirExpr, var_name: &str) -> bool {
    queries::expr_references_var(expr, var_name)
}

/// Check if a function body contains any yield statements (making it a generator).
pub(super) fn body_contains_return_stmt(stmts: &[HirStmt]) -> bool {
    queries::body_contains_return(stmts)
}

/// Check if a try body contains a return statement with a non-unit value.
/// Used to determine if the try closure needs to return T instead of ().
pub(super) fn try_body_has_value_return(stmts: &[HirStmt]) -> bool {
    queries::try_body_has_value_return(stmts)
}

pub(super) fn body_contains_yield_inner(stmts: &[HirStmt]) -> bool {
    queries::body_contains_yield(stmts)
}

/// Check if a type needs .`clone()` when accessed from &self (non-Copy types).
pub(super) fn needs_clone_for_type(ty: &Type) -> bool {
    ty.ownership() == sifr_type_system::OwnershipKind::Move
}

/// Mutating methods that require the receiver variable to be `mut`.
pub(super) const MUTATING_METHODS: &[&str] = queries::MUTATING_METHODS;

/// Collect the set of variable names that are mutated in a function body.
/// A variable is mutated if it appears in:
/// - `HirStmt::Assign` (reassignment)
/// - `HirStmt::AugAssign` (augmented assignment like +=)
/// - `HirStmt::Expr` containing a `MethodCall` on the variable with a mutating method
/// - `HirStmt::Delete` on the variable
pub(super) fn collect_mutated_vars(stmts: &[HirStmt]) -> HashSet<String> {
    queries::collect_mutated_vars(stmts, None)
}

pub(super) fn collect_mutated_vars_with_sigs(
    stmts: &[HirStmt],
    func_signatures: &ModuleFuncSignatures,
) -> HashSet<String> {
    queries::collect_mutated_vars(stmts, Some(func_signatures))
}

pub(super) fn collect_reassigned_vars(stmts: &[HirStmt]) -> HashSet<String> {
    queries::collect_reassigned_vars(stmts)
}

/// Collect all variable names and their types referenced in a list of statements.
pub(super) fn collect_referenced_vars_with_types(stmts: &[HirStmt]) -> Vec<(String, Type)> {
    queries::collect_referenced_vars_with_types(stmts)
}

pub(super) fn collect_typed_refs_in_expr(expr: &HirExpr, refs: &mut HashMap<String, Type>) {
    queries::collect_typed_refs_in_expr(expr, refs);
}

/// Collect all variable names defined (let-bound) in a list of statements.
/// Does NOT recurse into nested functions.
pub(super) fn collect_locally_defined_vars(stmts: &[HirStmt]) -> HashSet<String> {
    queries::collect_locally_defined_vars(stmts)
}

/// Check if a function body contains calls to a specific function name.
pub(super) fn body_calls_function(stmts: &[HirStmt], func_name: &str) -> bool {
    queries::body_calls_function(stmts, func_name)
}

pub(super) fn expr_calls_function(expr: &HirExpr, func_name: &str) -> bool {
    queries::expr_calls_function(expr, func_name)
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
    fn classify_value_category_marks_names_and_fields_as_places() {
        let name_expr = HirExpr::Name {
            name: "xs".to_string(),
            ty: Type::List(Box::new(Type::Int)),
        };
        let field_expr = HirExpr::FieldAccess {
            object: Box::new(HirExpr::Name {
                name: "self".to_string(),
                ty: Type::Class {
                    name: "C".to_string(),
                    fields: vec![("items".to_string(), Type::List(Box::new(Type::Int)))],
                    methods: vec![],
                    parent_class: None,
                },
            }),
            field: "items".to_string(),
            ty: Type::List(Box::new(Type::Int)),
        };
        let temp_expr = HirExpr::ListLiteral {
            elements: vec![HirExpr::IntLiteral(1)],
            ty: Type::List(Box::new(Type::Int)),
        };

        assert_eq!(classify_value_category(&name_expr), ValueCategory::Place);
        assert_eq!(classify_value_category(&field_expr), ValueCategory::Place);
        assert_eq!(
            classify_value_category(&temp_expr),
            ValueCategory::Temporary
        );
    }

    #[test]
    fn classify_value_category_treats_copy_tuple_literal_of_places_as_place() {
        let tuple_expr = HirExpr::TupleLiteral {
            elements: vec![
                HirExpr::Name {
                    name: "a".to_string(),
                    ty: Type::Int,
                },
                HirExpr::Name {
                    name: "b".to_string(),
                    ty: Type::Bool,
                },
            ],
            ty: Type::Tuple(vec![Type::Int, Type::Bool]),
        };

        assert_eq!(classify_value_category(&tuple_expr), ValueCategory::Place);
    }

    #[test]
    fn classify_value_category_treats_move_tuple_literal_as_temporary() {
        let tuple_expr = HirExpr::TupleLiteral {
            elements: vec![
                HirExpr::Name {
                    name: "a".to_string(),
                    ty: Type::Int,
                },
                HirExpr::Name {
                    name: "b".to_string(),
                    ty: Type::Str,
                },
            ],
            ty: Type::Tuple(vec![Type::Int, Type::Str]),
        };

        assert_eq!(
            classify_value_category(&tuple_expr),
            ValueCategory::Temporary
        );
    }

    #[test]
    fn iterator_plan_preserves_named_copy_element_collection() {
        let source = HirExpr::Name {
            name: "xs".to_string(),
            ty: Type::List(Box::new(Type::Int)),
        };
        let plan = plan_iterator_ownership(&source);

        assert_eq!(plan.value_category, ValueCategory::Place);
        assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
        assert_eq!(plan.yield_mode, YieldMode::Copy);
        assert_eq!(plan.element_ownership, Some(OwnershipKind::Copy));
    }

    #[test]
    fn iterator_plan_clones_named_move_element_collection() {
        let source = HirExpr::Name {
            name: "strings".to_string(),
            ty: Type::List(Box::new(Type::Str)),
        };
        let plan = plan_iterator_ownership(&source);

        assert_eq!(plan.value_category, ValueCategory::Place);
        assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
        assert_eq!(plan.yield_mode, YieldMode::Clone);
        assert_eq!(plan.element_ownership, Some(OwnershipKind::Move));
    }

    #[test]
    fn iterator_plan_consumes_temporary_collection() {
        let source = HirExpr::ListLiteral {
            elements: vec![HirExpr::IntLiteral(1), HirExpr::IntLiteral(2)],
            ty: Type::List(Box::new(Type::Int)),
        };
        let plan = plan_iterator_ownership(&source);

        assert_eq!(plan.value_category, ValueCategory::Temporary);
        assert_eq!(plan.source_access_mode, SourceAccessMode::Consume);
        assert_eq!(plan.yield_mode, YieldMode::Move);
        assert_eq!(plan.element_ownership, Some(OwnershipKind::Copy));
    }

    #[test]
    fn iterator_plan_defaults_to_borrow_for_conservative_unknown_elements() {
        let source = HirExpr::Name {
            name: "unknown".to_string(),
            ty: Type::Class {
                name: "Unknown".to_string(),
                fields: vec![],
                methods: vec![],
                parent_class: None,
            },
        };
        let plan = plan_iterator_ownership(&source);

        assert_eq!(plan.value_category, ValueCategory::Place);
        assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
        assert_eq!(plan.yield_mode, YieldMode::Borrow);
        assert_eq!(plan.element_ownership, None);
    }

    #[test]
    fn option_projection_method_prefers_copy_for_copy_types() {
        assert_eq!(
            option_projection_method_for_owned_type(&Type::Int),
            "copied"
        );
        assert_eq!(
            option_projection_method_for_owned_type(&Type::Str),
            "cloned"
        );
    }

    #[test]
    fn iterator_plan_copy_hint_does_not_force_unknown_source_to_copy() {
        let source = HirExpr::Name {
            name: "x".to_string(),
            ty: Type::Any,
        };
        let plan = plan_iterator_ownership_with_element_hint(&source, Some(&Type::Int));

        assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
        assert_eq!(plan.yield_mode, YieldMode::Borrow);
        assert_eq!(plan.element_ownership, None);
    }

    #[test]
    fn iterator_plan_preserved_list_any_uses_borrow_not_clone() {
        let source = HirExpr::Name {
            name: "items".to_string(),
            ty: Type::List(Box::new(Type::Any)),
        };
        let plan = plan_iterator_ownership(&source);

        assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
        assert_eq!(plan.yield_mode, YieldMode::Borrow);
        assert_eq!(plan.element_ownership, None);
    }

    #[test]
    fn iterator_plan_typevar_hint_stays_conservative() {
        let source = HirExpr::Name {
            name: "xs".to_string(),
            ty: Type::TypeVar("T".to_string()),
        };
        let plan = plan_iterator_ownership_with_element_hint(&source, Some(&Type::Int));

        assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
        assert_eq!(plan.yield_mode, YieldMode::Borrow);
        assert_eq!(plan.element_ownership, None);
    }

    #[test]
    fn iterator_plan_list_typevar_uses_clone_yield() {
        let source = HirExpr::Name {
            name: "xs".to_string(),
            ty: Type::List(Box::new(Type::TypeVar("T".to_string()))),
        };
        let plan = plan_iterator_ownership(&source);

        assert_eq!(plan.source_access_mode, SourceAccessMode::Preserve);
        assert_eq!(plan.yield_mode, YieldMode::Clone);
        assert_eq!(plan.element_ownership, Some(OwnershipKind::Move));
    }

    #[test]
    fn iterator_plan_copies_tuple_of_copy_elements() {
        let source = HirExpr::Name {
            name: "pairs".to_string(),
            ty: Type::List(Box::new(Type::Tuple(vec![Type::Int, Type::Int]))),
        };
        let plan = plan_iterator_ownership(&source);

        assert_eq!(plan.yield_mode, YieldMode::Copy);
        assert_eq!(plan.element_ownership, Some(OwnershipKind::Copy));
    }

    #[test]
    fn iterator_plan_consumes_range_without_clone_contract() {
        let source = HirExpr::Name {
            name: "r".to_string(),
            ty: Type::Range,
        };
        let plan = plan_iterator_ownership(&source);

        assert_eq!(plan.source_access_mode, SourceAccessMode::Consume);
        assert_eq!(plan.yield_mode, YieldMode::Move);
        assert_eq!(plan.element_ownership, Some(OwnershipKind::Copy));
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
                convention: ParamConvention::own(),
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
    fn body_contains_field_assign_detects_delegated_self_field_class_mutation() {
        let writer_ty = Type::Class {
            name: "writer".to_string(),
            fields: vec![],
            methods: vec![],
            parent_class: None,
        };
        let holder_ty = Type::Class {
            name: "DictWriter".to_string(),
            fields: vec![("_writer".to_string(), writer_ty.clone())],
            methods: vec![],
            parent_class: None,
        };
        let stmts = vec![HirStmt::Expr {
            expr: HirExpr::MethodCall {
                object: Box::new(HirExpr::FieldAccess {
                    object: Box::new(HirExpr::Name {
                        name: "self".to_string(),
                        ty: holder_ty,
                    }),
                    field: "_writer".to_string(),
                    ty: writer_ty,
                }),
                method: "writerow".to_string(),
                args: vec![],
                ty: Type::None,
            },
        }];

        assert!(body_contains_field_assign_codegen(&stmts));
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
