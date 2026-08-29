use super::{IsinstanceUnionMatch, ModuleFuncSignatures};
use crate::RustExpr;
use crate::hir_analysis::{queries, traversal};
use sifr_ir::{HirExpr, HirStmt};
use sifr_type_system::{OwnershipKind, ParamConvention, ReceiverConvention, Type};
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
            is_copy_type_for_codegen(ty) && elements.iter().all(is_reusable_place_expr)
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
                Some(if is_copy_type_for_codegen(&metadata.element_type) {
                    OwnershipKind::Copy
                } else {
                    OwnershipKind::Move
                })
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
    if is_copy_type_for_codegen(resolved) {
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
                Some(if is_copy_type_for_codegen(resolved_hint) {
                    OwnershipKind::Copy
                } else {
                    OwnershipKind::Move
                })
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
    let resolved = crate::resolve_alias_type_for_plain_call(ty);
    match resolved {
        Type::Int | Type::LiteralInt(_) | Type::Range => false,
        Type::Tuple(elements) | Type::Union(elements) | Type::Intersection(elements) => {
            elements.iter().all(is_copy_type_for_codegen)
        }
        Type::Newtype { inner, .. } => is_copy_type_for_codegen(inner),
        Type::Alias { body, .. } => is_copy_type_for_codegen(body),
        _ => resolved.ownership() == OwnershipKind::Copy,
    }
}

/// Returns whether Sifr gives a value implicit-copy semantics while the canonical Rust
/// representation requires an explicit clone.
pub(crate) fn is_logically_copy_rust_move_type(ty: &Type) -> bool {
    ty.ownership() == OwnershipKind::Copy && !is_copy_type_for_codegen(ty)
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
    ty.optional_member_type().is_some()
}

fn option_wrapper_depth(ty: &Type) -> usize {
    let mut current = ty.clone();
    let mut depth = 0;
    while let Some(payload) = current.optional_member_type() {
        depth += 1;
        current = payload;
    }
    depth
}

/// Adapt a representation-significant optional wrapper to its assignable target.
pub(crate) fn flatten_option_value_for_target(
    target: &Type,
    source: &Type,
    mut value: RustExpr,
) -> RustExpr {
    let target_depth = option_wrapper_depth(target);
    let source_depth = option_wrapper_depth(source);
    if target_depth == 0 && source_depth == 1 {
        let Some(source_payload) = source.optional_member_type() else {
            return value;
        };
        let canonical_source_payload =
            match crate::resolve_alias_type_for_plain_call(&source_payload) {
                Type::Union(members) => sifr_type_system::make_union(members.clone()),
                other => other.clone(),
            };
        let canonical_target = match crate::resolve_alias_type_for_plain_call(target) {
            Type::Union(members) => sifr_type_system::make_union(members.clone()),
            other => other.clone(),
        };
        let Type::Union(target_members) = &canonical_target else {
            return value;
        };
        if canonical_source_payload != canonical_target
            || !target_members
                .iter()
                .any(|member| matches!(member.resolve_alias(), Type::None))
        {
            return value;
        }
        return RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(value))),
            method: "unwrap_or".to_string(),
            args: vec![RustExpr::FnCall {
                func: Box::new(RustExpr::Path(vec![
                    canonical_target.union_enum_name(),
                    Type::None.union_variant_name(),
                ])),
                args: vec![RustExpr::Literal(crate::RustLiteral::Unit)],
            }],
        };
    }
    if target_depth == 0 || source_depth <= target_depth || !source.is_assignable_to(target) {
        return value;
    }
    for _ in target_depth..source_depth {
        value = RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(value))),
            method: "flatten".to_string(),
            args: Vec::new(),
        };
    }
    value
}

/// Whether assignment needs the emitter's recursive union conversion authority.
pub(crate) fn requires_union_representation_transition(target: &Type, source: &Type) -> bool {
    let (target, source) = (
        crate::resolve_alias_type_for_plain_call(target),
        crate::resolve_alias_type_for_plain_call(source),
    );
    target != source && matches!(target, Type::Union(_)) && !is_option_type(target)
}

/// Normalize nested absence produced by a safe collection operation.
pub(crate) fn normalize_safe_option_result(payload: &Type, value: RustExpr) -> RustExpr {
    let canonical_payload = match crate::resolve_alias_type_for_plain_call(payload) {
        Type::Union(members) => sifr_type_system::make_union(members.clone()),
        _ => return value,
    };
    if is_option_type(&canonical_payload) {
        return RustExpr::MethodCall {
            receiver: Box::new(RustExpr::Paren(Box::new(value))),
            method: "flatten".to_string(),
            args: Vec::new(),
        };
    }
    value
}

/// Detect truthiness check on an Option variable: `if x:` where x has type T | None.
pub(crate) fn detect_option_truthiness(expr: &HirExpr) -> Option<String> {
    if let HirExpr::Name { name, ty, .. } = expr {
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
            if let HirExpr::Name { name, ty, .. } = left.as_ref() {
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
            if let HirExpr::Name { name, ty, .. } = &args[0] {
                let resolved_ty = crate::resolve_alias_type_for_plain_call(ty);
                if let Type::Union(raw_members) = resolved_ty {
                    if !is_option_type(resolved_ty) {
                        let Type::Union(members) =
                            sifr_type_system::make_union(raw_members.clone())
                        else {
                            return None;
                        };
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
                                members
                                    .iter()
                                    .find(
                                        |m| matches!(m, Type::Class { name, .. } if name == other),
                                    )
                                    .cloned()?
                            }
                        };
                        // Check that this type is a member of the union
                        if members.contains(&target_ty) {
                            let variant = target_ty.union_variant_name();
                            let enum_name = Type::Union(members.clone()).union_enum_name();
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
    find_union_member(members, arg_ty).map(Type::union_variant_name)
}

pub(crate) fn find_union_member<'a>(members: &'a [Type], arg_ty: &Type) -> Option<&'a Type> {
    let exact_variant = arg_ty.union_variant_name();
    members
        .iter()
        .find(|member| member.union_variant_name() == exact_variant)
        .or_else(|| {
            members
                .iter()
                .find(|member| arg_ty.is_assignable_to(member))
        })
}

pub(crate) fn wrap_union_member_expr(
    union_ty: &Type,
    member_ty: &Type,
    lowered: RustExpr,
) -> Option<RustExpr> {
    let Type::Union(members) = crate::resolve_alias_type_for_plain_call(union_ty) else {
        return None;
    };
    if is_option_type(union_ty)
        || matches!(
            crate::resolve_alias_type_for_plain_call(member_ty),
            Type::Union(_)
        )
    {
        return None;
    }
    let variant = find_union_variant(members, member_ty)?;
    let payload = if matches!(
        crate::resolve_alias_type_for_plain_call(member_ty),
        Type::None
    ) {
        RustExpr::Literal(crate::RustLiteral::Unit)
    } else {
        lowered
    };
    Some(RustExpr::FnCall {
        func: Box::new(RustExpr::Path(vec![union_ty.union_enum_name(), variant])),
        args: vec![payload],
    })
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
            if let HirExpr::Name { name, ty, .. } = left.as_ref() {
                if is_option_type(ty) {
                    return Some(name.clone());
                }
            }
        }
    }
    None
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
        HirExpr::MethodCall {
            object,
            receiver_convention,
            ..
        } => {
            let is_self_field = matches!(object.as_ref(), HirExpr::FieldAccess { .. })
                && field_access_root_name(object) == Some("self");
            is_self_field && *receiver_convention == Some(ReceiverConvention::MutableBorrow)
        }
        _ => false,
    }
}

fn field_access_root_name(expr: &HirExpr) -> Option<&str> {
    match expr {
        HirExpr::Name { name, .. } => Some(name),
        HirExpr::FieldAccess { object, .. } => field_access_root_name(object),
        _ => None,
    }
}

/// Check if a type contains a specific type variable name.
pub(crate) fn type_contains_typevar(ty: &Type, tv_name: &str) -> bool {
    match ty {
        Type::TypeVar(name) => name == tv_name,
        Type::List(inner) | Type::PythonBuffer(inner) | Type::PythonDlpackTensor(inner) => {
            type_contains_typevar(inner, tv_name)
        }
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
    match ty.resolve_alias() {
        Type::Class { name, .. } => name == class_name,
        Type::Union(members) => members.iter().any(|m| type_references_class(m, class_name)),
        Type::List(inner)
        | Type::Set(inner)
        | Type::Iterable(inner)
        | Type::Iterator(inner)
        | Type::Awaitable(inner)
        | Type::Failure(inner)
        | Type::TimeoutResult(inner)
        | Type::Newtype { inner, .. } => type_references_class(inner, class_name),
        Type::Dict(key, val) => {
            type_references_class(key, class_name) || type_references_class(val, class_name)
        }
        Type::Tuple(elems) => elems.iter().any(|e| type_references_class(e, class_name)),
        Type::Result(ok, err)
        | Type::Task(ok, err)
        | Type::TaskResult(ok, err)
        | Type::Coroutine(ok, err)
        | Type::Select2(ok, err)
        | Type::BlockingTask(ok, err)
        | Type::JoinSet(ok, err)
        | Type::AsyncIterator(ok, err)
        | Type::AsyncGenerator(ok, err) => {
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
    !is_copy_type_for_codegen(ty)
}

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
