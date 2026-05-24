use super::{
    class_specialization_payload_conflicts, domain_typed_sentinel_expr, lower_expr, make_union,
    match_diagnostics, name_diagnostics, numeric_domain_for_type, numeric_sentinel_kind,
    ownership_diagnostics, record_async_generator_advance_binding, record_const_integer_binding,
    record_len_alias_fact, record_sequence_pointer_fact, resolve_annotation_expr,
    sequence_shape_fact, statement_diagnostics, str, validate_fixed_width_initializer,
    DiagnosticCode, Expr, FixedWidthInitializerFit, HirExpr, HirPattern, HirStmt, LowerCtx,
    Pattern, Ranged, Singleton, StmtAnnAssign, StmtAssign, Type,
};
pub(in crate::lower) fn lower_pattern(
    pattern: &Pattern,
    subject_ty: &Type,
    ctx: &mut LowerCtx,
) -> Option<HirPattern> {
    match pattern {
        Pattern::MatchAs(pat_as) => {
            if pat_as.pattern.is_none() && pat_as.name.is_none() {
                // `case _:` — wildcard
                return Some(HirPattern::Wildcard);
            }
            if let Some(name) = &pat_as.name {
                let var_name = name.to_string();
                if let Some(inner_pat) = &pat_as.pattern {
                    // `case SomePattern as x:` — match inner pattern, bind to x
                    let inner = lower_pattern(inner_pat, subject_ty, ctx)?;
                    // For now, treat as capture with narrowed type
                    let narrowed_ty = pattern_narrowed_type(&inner, subject_ty, ctx);
                    let _ = inner; // inner pattern info embedded in capture
                    return Some(HirPattern::Capture {
                        name: var_name,
                        ty: narrowed_ty,
                    });
                }
                // `case x:` — capture pattern
                return Some(HirPattern::Capture {
                    name: var_name,
                    ty: subject_ty.clone(),
                });
            }
            if let Some(inner_pat) = &pat_as.pattern {
                return lower_pattern(inner_pat, subject_ty, ctx);
            }
            Some(HirPattern::Wildcard)
        }
        Pattern::MatchSingleton(singleton) => match &singleton.value {
            Singleton::None => Some(HirPattern::None),
            Singleton::True => Some(HirPattern::Literal {
                value: HirExpr::BoolLiteral(true),
            }),
            Singleton::False => Some(HirPattern::Literal {
                value: HirExpr::BoolLiteral(false),
            }),
        },
        Pattern::MatchValue(val_pat) => {
            // Could be a literal or an attribute access like Color.RED
            if let Expr::Attribute(attr) = val_pat.value.as_ref() {
                let obj_name = if let Expr::Name(n) = attr.value.as_ref() {
                    n.id.to_string()
                } else {
                    match_diagnostics::invalid_pattern_form(
                        ctx,
                        "complex attribute pattern is not supported",
                        attr.value.range(),
                    );
                    return None;
                };
                let attr_name = attr.attr.to_string();
                Some(HirPattern::Value {
                    path: vec![obj_name, attr_name],
                })
            } else {
                // Try to lower as a literal expression
                let expr = lower_expr(val_pat.value.as_ref(), ctx)?;
                let value = match validate_fixed_width_initializer(
                    ctx,
                    subject_ty,
                    &expr,
                    val_pat.value.range(),
                ) {
                    FixedWidthInitializerFit::Fits(value) => value,
                    FixedWidthInitializerFit::Rejected => return None,
                    FixedWidthInitializerFit::NotConst => expr,
                };
                Some(HirPattern::Literal { value })
            }
        }
        Pattern::MatchOr(or_pat) => {
            let mut patterns = Vec::new();
            for p in &or_pat.patterns {
                patterns.push(lower_pattern(p, subject_ty, ctx)?);
            }
            Some(HirPattern::Or { patterns })
        }
        Pattern::MatchClass(class_pat) => {
            let class_name = if let Expr::Name(n) = class_pat.cls.as_ref() {
                n.id.to_string()
            } else {
                match_diagnostics::invalid_pattern_form(
                    ctx,
                    "class pattern class name must be a simple name",
                    class_pat.cls.range(),
                );
                return None;
            };

            // Resolve the class type to get field types
            let class_ty = ctx.class_types.get(&class_name).cloned();

            let mut fields = Vec::new();
            for kw in &class_pat.arguments.keywords {
                let field_name = kw.attr.to_string();
                let field_ty = if let Some(Type::Class {
                    fields: class_fields,
                    ..
                }) = &class_ty
                {
                    let Some(field_ty) = class_fields
                        .iter()
                        .find(|(n, _)| n == &field_name)
                        .map(|(_, t)| t.clone())
                    else {
                        match_diagnostics::invalid_class_pattern_field(
                            ctx,
                            &class_name,
                            &field_name,
                            &class_fields
                                .iter()
                                .map(|(n, _)| n.as_str())
                                .collect::<Vec<_>>()
                                .join(", "),
                            kw.attr.range(),
                        );
                        return None;
                    };
                    field_ty
                } else {
                    Type::Any
                };
                let field_pattern = lower_pattern(&kw.pattern, &field_ty, ctx)?;
                fields.push((field_name, field_pattern));
            }

            Some(HirPattern::Class { class_name, fields })
        }
        Pattern::MatchSequence(seq_pat) => {
            if seq_pat.patterns.is_empty() {
                return Some(HirPattern::Tuple { elements: vec![] });
            }
            let elem_types: Vec<Type> = if let Type::Tuple(ref elems) = *subject_ty {
                elems.clone()
            } else {
                match_diagnostics::invalid_pattern_form(
                    ctx,
                    &format!(
                        "tuple pattern requires subject of tuple type, got '{}'",
                        subject_ty.display_name()
                    ),
                    seq_pat.range(),
                );
                return None;
            };
            if elem_types.len() != seq_pat.patterns.len() {
                match_diagnostics::invalid_pattern_form(
                    ctx,
                    &format!(
                        "tuple pattern expects {} element(s), subject has {}",
                        seq_pat.patterns.len(),
                        elem_types.len()
                    ),
                    seq_pat.range(),
                );
                return None;
            }
            let mut elements = Vec::new();
            for (i, pat) in seq_pat.patterns.iter().enumerate() {
                let elem_ty = elem_types[i].clone();
                if let Some(lowered) = lower_pattern(pat, &elem_ty, ctx) {
                    elements.push(lowered);
                } else {
                    return None;
                }
            }
            Some(HirPattern::Tuple { elements })
        }
        Pattern::MatchMapping(_) => {
            match_diagnostics::invalid_pattern_form(
                ctx,
                "mapping patterns are not yet supported",
                pattern.range(),
            );
            None
        }
        Pattern::MatchStar(_) => {
            match_diagnostics::invalid_pattern_form(
                ctx,
                "star patterns are not yet supported",
                pattern.range(),
            );
            None
        }
    }
}

pub(in crate::lower) fn pattern_narrowed_type(
    pattern: &HirPattern,
    subject_ty: &Type,
    ctx: &LowerCtx,
) -> Type {
    match pattern {
        HirPattern::None => Type::None,
        HirPattern::Class { class_name, .. } => {
            // Look up the class type
            if let Some(class_ty) = ctx.class_types.get(class_name) {
                class_ty.clone()
            } else {
                subject_ty.clone()
            }
        }
        _ => subject_ty.clone(),
    }
}

pub(in crate::lower) fn bind_pattern_vars(pattern: &HirPattern, ctx: &mut LowerCtx) {
    match pattern {
        HirPattern::Capture { name, ty } => {
            ctx.scope.define(name.clone(), ty.clone());
        }
        HirPattern::Class { fields, .. } => {
            for (_, field_pat) in fields {
                bind_pattern_vars(field_pat, ctx);
            }
        }
        HirPattern::Or { patterns } => {
            // Bind from first pattern (all OR alternatives should bind same names)
            if let Some(first) = patterns.first() {
                bind_pattern_vars(first, ctx);
            }
        }
        HirPattern::Tuple { elements } => {
            for elem in elements {
                bind_pattern_vars(elem, ctx);
            }
        }
        _ => {}
    }
}

pub(super) fn seed_binding_after_failed_initializer(
    ctx: &mut LowerCtx,
    name: &str,
    ty: Type,
    is_explicit_local: bool,
    error_taint: crate::scope::ErrorTaint,
) {
    ctx.scope
        .define_poisoned_local(name.to_string(), ty, is_explicit_local, error_taint);
    ctx.empty_dict_specializations.remove(name);
    ctx.pending_container_specialization_patches.remove(name);
    ctx.clear_numeric_sentinel_var(name);
    ctx.clear_sequence_shape_fact(name);
}

pub(super) fn failed_initializer_taint(
    ctx: &mut LowerCtx,
    name: &str,
    range: ruff_text_size::TextRange,
    error_count_before_initializer: usize,
) -> Option<crate::scope::ErrorTaint> {
    let taint = ctx.error_taint_since(error_count_before_initializer);
    if taint.is_none() {
        ctx.error_with_code_at(
            DiagnosticCode::INTERNAL_COMPILER_PANIC,
            format!(
                "internal compiler error: failed initializer for '{name}' did not emit a diagnostic"
            ),
            range,
        );
    }
    taint
}

pub(super) fn invalidate_rebound_binding_facts(ctx: &mut LowerCtx, name: &str) {
    ctx.scope.clear_narrowing(name);
    ctx.scope.clear_const_integer_value(name);
    ctx.clear_sequence_guards_for_binding(name);
    ctx.clear_proven_nonzero_integer_binding(name);
}

pub(in crate::lower) fn lower_ann_assign(
    ann: &StmtAnnAssign,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let name = if let Expr::Name(n) = ann.target.as_ref() {
        n.id.to_string()
    } else {
        statement_diagnostics::invalid_assignment_target(
            ctx,
            "annotated assignment target must be a simple name",
            ann.target.range(),
        );
        return None;
    };
    let declared_type = resolve_annotation_expr(&ann.annotation, ctx);

    let (value, initializer_range) = if let Some(val) = &ann.value {
        let initializer_range = val.range();
        let error_count_before_initializer = ctx.error_count();
        let mut expr = if let Some(kind) = numeric_sentinel_kind(val) {
            if let Some(domain) = numeric_domain_for_type(&declared_type) {
                domain_typed_sentinel_expr(kind, domain)
            } else if let Some(expr) = lower_expr(val, ctx) {
                expr
            } else {
                let error_taint = failed_initializer_taint(
                    ctx,
                    &name,
                    initializer_range,
                    error_count_before_initializer,
                )?;
                seed_binding_after_failed_initializer(
                    ctx,
                    &name,
                    declared_type.clone(),
                    true,
                    error_taint,
                );
                return None;
            }
        } else if let Some(expr) = lower_expr(val, ctx) {
            expr
        } else {
            let error_taint = failed_initializer_taint(
                ctx,
                &name,
                initializer_range,
                error_count_before_initializer,
            )?;
            seed_binding_after_failed_initializer(
                ctx,
                &name,
                declared_type.clone(),
                true,
                error_taint,
            );
            return None;
        };
        let expr_ty = expr.ty().clone();
        // Inside try blocks, auto-unwrap Result[T, E] when declared type is T
        if ctx.in_try_block {
            if let Type::Result(ref ok_ty, ref err_ty) = expr_ty {
                if ok_ty.as_ref().is_assignable_to(&declared_type) {
                    // Track the error type for exhaustiveness checking
                    if let Type::Class { name, .. } = err_ty.as_ref() {
                        ctx.try_block_error_types.insert(name.clone());
                    }
                    expr = HirExpr::QuestionMark {
                        expr: Box::new(expr),
                        ty: declared_type.clone(),
                    };
                }
            }
        }
        // Type check: value must be assignable to declared type
        let final_ty = expr.ty().clone();
        // int literals are assignable to bigint (coercion: 42 -> BigInt::from(42))
        let is_int_to_bigint = final_ty == Type::Int && declared_type == Type::BigInt;
        let fixed_width_fit =
            validate_fixed_width_initializer(ctx, &declared_type, &expr, initializer_range);
        let fixed_width_not_const = matches!(fixed_width_fit, FixedWidthInitializerFit::NotConst);
        if let FixedWidthInitializerFit::Fits(folded_expr) = fixed_width_fit {
            expr = folded_expr;
        }
        let class_specialization_conflict =
            class_specialization_payload_conflicts(&final_ty, &declared_type);
        if !is_int_to_bigint
            && ((fixed_width_not_const && !final_ty.is_assignable_to(&declared_type))
                || class_specialization_conflict)
        {
            ctx.error_with_code_at(
                DiagnosticCode::TYPE_MISMATCH,
                format!(
                    "type mismatch: expected '{}', got '{}'",
                    declared_type.display_name(),
                    final_ty.display_name()
                ),
                initializer_range,
            );
        }
        (expr, initializer_range)
    } else {
        name_diagnostics::uninitialized_variable(ctx, &name, ann.target.range());
        return None;
    };

    // Track move: if RHS is a variable name with Move ownership, mark it as moved.
    // Also check escape analysis: storing a borrowed parameter into a local variable
    // would allow it to outlive the borrow, which is not allowed.
    if let HirExpr::Name {
        name: ref src_name,
        ref ty,
    } = value
    {
        if ty.ownership() == sifr_type_system::OwnershipKind::Move {
            // Escape analysis: cannot store a borrowed parameter into a new binding
            if ctx.borrowed_params.contains(src_name.as_str()) {
                ownership_diagnostics::borrowed_parameter_store_escape(
                    ctx,
                    src_name,
                    initializer_range,
                );
            } else {
                ctx.scope.mark_moved(src_name);
            }
        }
    }

    ctx.empty_dict_specializations.remove(&name);
    ctx.pending_container_specialization_patches.remove(&name);
    ctx.scope
        .define_explicit_local(name.clone(), declared_type.clone());
    if matches!(value.ty(), Type::Int | Type::LiteralInt(_))
        && value.ty().is_assignable_to(&declared_type)
    {
        record_const_integer_binding(ctx, &name, &value);
    }
    if let Some(kind) = ann
        .value
        .as_ref()
        .and_then(|value| numeric_sentinel_kind(value))
    {
        ctx.record_numeric_sentinel_initializer(name.clone(), kind);
        if let Some(domain) = numeric_domain_for_type(&declared_type) {
            ctx.resolve_numeric_sentinel_domain(&name, domain);
        }
    } else {
        ctx.clear_numeric_sentinel_var(&name);
    }
    if let Some(fact) = ann
        .value
        .as_ref()
        .and_then(|value| sequence_shape_fact(&name, value))
    {
        ctx.record_sequence_shape_fact(fact);
    } else {
        ctx.clear_sequence_shape_fact(&name);
    }
    let initializer = ann.value.as_ref()?;
    record_len_alias_fact(ctx, &name, initializer);
    record_sequence_pointer_fact(ctx, &name, initializer);
    record_async_generator_advance_binding(ctx, &name, &value);
    Some(HirStmt::Let {
        name,
        ty: declared_type,
        value,
        is_mutable: true,
    })
}
/// Handle chained assignment: x = y = z = 0
/// Expands into: z = 0; y = z; x = y (right-to-left, last target gets the value first)
pub(in crate::lower) fn lower_chained_assign(
    assign: &StmtAssign,
    ctx: &mut LowerCtx,
) -> Vec<HirStmt> {
    let mut result = Vec::new();
    // Lower the value expression once
    let Some(value) = lower_expr(&assign.value, ctx) else {
        return result;
    };
    let val_ty = value.ty().clone();

    // Process targets in reverse order (rightmost gets the value first)
    let targets: Vec<_> = assign.targets.iter().collect();
    for (i, target) in targets.iter().rev().enumerate() {
        if let Expr::Name(n) = target {
            let name = n.id.to_string();
            if i == 0 {
                // First (rightmost) target gets the actual value
                let existing = ctx.scope.lookup(&name);
                if existing.is_some() {
                    // Reassignment
                    invalidate_rebound_binding_facts(ctx, &name);
                    ctx.empty_dict_specializations.remove(&name);
                    ctx.pending_container_specialization_patches.remove(&name);
                    result.push(HirStmt::Assign {
                        name: name.clone(),
                        value: value.clone(),
                    });
                } else {
                    // New variable
                    ctx.scope.define(name.clone(), val_ty.clone());
                    ctx.empty_dict_specializations.remove(&name);
                    ctx.pending_container_specialization_patches.remove(&name);
                    result.push(HirStmt::Let {
                        name: name.clone(),
                        ty: val_ty.clone(),
                        value: value.clone(),
                        is_mutable: true,
                    });
                }
            } else {
                // Subsequent targets get a reference to the previous target
                let prev_target = match targets.get(targets.len() - i) {
                    Some(Expr::Name(prev_n)) => prev_n.id.to_string(),
                    _ => continue,
                };
                let name_expr = HirExpr::Name {
                    name: prev_target.clone(),
                    ty: val_ty.clone(),
                };
                let existing = ctx.scope.lookup(&name);
                if existing.is_some() {
                    invalidate_rebound_binding_facts(ctx, &name);
                    ctx.empty_dict_specializations.remove(&name);
                    ctx.pending_container_specialization_patches.remove(&name);
                    result.push(HirStmt::Assign {
                        name: name.clone(),
                        value: name_expr,
                    });
                } else {
                    ctx.scope.define(name.clone(), val_ty.clone());
                    ctx.empty_dict_specializations.remove(&name);
                    ctx.pending_container_specialization_patches.remove(&name);
                    result.push(HirStmt::Let {
                        name: name.clone(),
                        ty: val_ty.clone(),
                        value: name_expr,
                        is_mutable: true,
                    });
                }
            }
        } else {
            statement_diagnostics::invalid_assignment_target(
                ctx,
                "chained assignment targets must be simple names",
                target.range(),
            );
        }
    }

    result
}

pub(super) fn resolve_field_type_from_type(object_ty: &Type, field_name: &str) -> Option<Type> {
    let resolved = object_ty.resolve_alias();
    if let Type::Class { fields, .. } = resolved {
        return fields
            .iter()
            .find(|(name, _)| name == field_name)
            .map(|(_, ty)| ty.clone());
    }
    if let Type::Union(members) = resolved {
        let mut field_members = Vec::new();
        let mut has_none = false;
        for member in members {
            match member.resolve_alias() {
                Type::None => {
                    has_none = true;
                }
                Type::Class { fields, .. } => {
                    let (_, member_field_ty) =
                        fields.iter().find(|(name, _)| name == field_name)?;
                    field_members.push(member_field_ty.clone());
                }
                _ => return None,
            }
        }
        if field_members.is_empty() {
            return None;
        }
        if has_none {
            field_members.push(Type::None);
        }
        return Some(make_union(field_members));
    }
    None
}

pub(in crate::lower) fn resolve_object_field_type(
    ctx: &LowerCtx,
    object_name: &str,
    field_name: &str,
) -> Type {
    ctx.scope
        .lookup(object_name)
        .and_then(|info| resolve_field_type_from_type(info.effective_type(), field_name))
        .unwrap_or(Type::Unknown)
}
