use super::classes::collect_literal_coverage;
use super::expressions::lower_expr;
use super::match_diagnostics;
use super::statements::{bind_pattern_vars, lower_pattern, lower_stmts};
use super::LowerCtx;
use crate::hir_nodes::{HirMatchArm, HirPattern, HirStmt};
use ruff_text_size::Ranged;
use sifr_python_ast::StmtMatch;
use sifr_type_system::{FunctionType, Type};

pub(in crate::lower) fn lower_match(
    match_stmt: &StmtMatch,
    func_type: &FunctionType,
    ctx: &mut LowerCtx,
) -> Option<HirStmt> {
    let subject = lower_expr(&match_stmt.subject, ctx)?;
    let subject_ty = subject.ty().clone();

    let mut arms = Vec::new();
    for case in &match_stmt.cases {
        let arm = ctx.with_pushed_scope(|ctx| {
            let pattern = lower_pattern(&case.pattern, &subject_ty, ctx)?;

            bind_pattern_vars(&pattern, ctx);

            let guard = if let Some(ref g) = case.guard {
                let guard_expr = lower_expr(g, ctx)?;
                let guard_ty = guard_expr.ty();
                if *guard_ty != Type::Bool && *guard_ty != Type::Any {
                    match_diagnostics::guard_not_bool(ctx, &guard_ty.display_name(), g.range());
                }
                Some(guard_expr)
            } else {
                None
            };

            let body = lower_stmts(&case.body, func_type, ctx);
            Some(HirMatchArm {
                pattern,
                guard,
                body,
            })
        })?;

        arms.push(arm);
    }

    let has_wildcard = arms
        .iter()
        .any(|arm| matches!(arm.pattern, HirPattern::Wildcard));
    let has_capture_without_guard = arms
        .iter()
        .any(|arm| matches!(arm.pattern, HirPattern::Capture { .. }) && arm.guard.is_none());

    if !has_wildcard && !has_capture_without_guard {
        report_union_exhaustiveness(&subject_ty, &arms, match_stmt, ctx);
        report_enum_exhaustiveness(&subject_ty, &arms, match_stmt, ctx);
        report_literal_exhaustiveness(&subject_ty, &arms, match_stmt, ctx);
    }

    Some(HirStmt::Match {
        subject,
        subject_ty,
        arms,
    })
}

fn report_union_exhaustiveness(
    subject_ty: &Type,
    arms: &[HirMatchArm],
    match_stmt: &StmtMatch,
    ctx: &mut LowerCtx,
) {
    let Type::Union(members) = subject_ty else {
        return;
    };

    let mut covered_none = false;
    let mut covered_classes: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut covered_types: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut covered_literal_strs: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut covered_literal_ints: std::collections::HashSet<i64> = std::collections::HashSet::new();
    let mut covered_literal_bools: std::collections::HashSet<bool> =
        std::collections::HashSet::new();

    for arm in arms {
        match &arm.pattern {
            HirPattern::None => {
                covered_none = true;
            }
            HirPattern::Class { class_name, .. } => {
                covered_classes.insert(class_name.clone());
            }
            HirPattern::Capture { ty, .. } if arm.guard.is_none() => {
                covered_types.insert(ty.display_name());
            }
            HirPattern::Literal { .. } => {
                collect_literal_coverage(
                    &arm.pattern,
                    &mut covered_literal_strs,
                    &mut covered_literal_ints,
                    &mut covered_literal_bools,
                );
            }
            HirPattern::Or { patterns } => {
                collect_or_pattern_coverage(
                    patterns,
                    &mut covered_none,
                    &mut covered_classes,
                    &mut covered_literal_strs,
                    &mut covered_literal_ints,
                    &mut covered_literal_bools,
                );
            }
            _ => {}
        }
    }

    let mut uncovered: Vec<String> = Vec::new();
    for member in members {
        match member {
            Type::None => {
                if !covered_none {
                    uncovered.push("None".to_string());
                }
            }
            Type::Class { name, .. } => {
                if !covered_classes.contains(name) && !covered_types.contains(name) {
                    uncovered.push(name.clone());
                }
            }
            Type::Int => {
                if !covered_types.contains("int") && !covered_classes.contains("int") {
                    uncovered.push("int".to_string());
                }
            }
            Type::Str => {
                if !covered_types.contains("str") && !covered_classes.contains("str") {
                    uncovered.push("str".to_string());
                }
            }
            Type::Float => {
                if !covered_types.contains("float") && !covered_classes.contains("float") {
                    uncovered.push("float".to_string());
                }
            }
            Type::Bool => {
                if !covered_types.contains("bool") && !covered_classes.contains("bool") {
                    uncovered.push("bool".to_string());
                }
            }
            Type::LiteralStr(s) => {
                if !covered_literal_strs.contains(s) {
                    uncovered.push(format!("\"{s}\""));
                }
            }
            Type::LiteralInt(n) => {
                if !covered_literal_ints.contains(n) {
                    uncovered.push(n.to_string());
                }
            }
            Type::LiteralBool(b) => {
                if !covered_literal_bools.contains(b) {
                    uncovered.push(b.to_string());
                }
            }
            _ => {}
        }
    }

    if !uncovered.is_empty() {
        match_diagnostics::non_exhaustive_union(
            ctx,
            &subject_ty.display_name(),
            &uncovered.join(", "),
            match_stmt.subject.range(),
        );
    }
}

fn collect_or_pattern_coverage(
    patterns: &[HirPattern],
    covered_none: &mut bool,
    covered_classes: &mut std::collections::HashSet<String>,
    covered_literal_strs: &mut std::collections::HashSet<String>,
    covered_literal_ints: &mut std::collections::HashSet<i64>,
    covered_literal_bools: &mut std::collections::HashSet<bool>,
) {
    for pattern in patterns {
        match pattern {
            HirPattern::None => {
                *covered_none = true;
            }
            HirPattern::Class { class_name, .. } => {
                covered_classes.insert(class_name.clone());
            }
            HirPattern::Literal { .. } => {
                collect_literal_coverage(
                    pattern,
                    covered_literal_strs,
                    covered_literal_ints,
                    covered_literal_bools,
                );
            }
            _ => {}
        }
    }
}

fn report_enum_exhaustiveness(
    subject_ty: &Type,
    arms: &[HirMatchArm],
    match_stmt: &StmtMatch,
    ctx: &mut LowerCtx,
) {
    let Type::Enum { name, variants } = subject_ty else {
        return;
    };

    let mut covered_variants: std::collections::HashSet<String> = std::collections::HashSet::new();
    for arm in arms {
        if let HirPattern::Value { path } = &arm.pattern {
            if path.len() == 2 {
                covered_variants.insert(path[1].clone());
            }
        }
        if let HirPattern::Or { patterns } = &arm.pattern {
            for pattern in patterns {
                if let HirPattern::Value { path } = pattern {
                    if path.len() == 2 {
                        covered_variants.insert(path[1].clone());
                    }
                }
            }
        }
    }

    let uncovered: Vec<&String> = variants
        .iter()
        .map(|(variant, _)| variant)
        .filter(|variant| !covered_variants.contains(*variant))
        .collect();
    if uncovered.is_empty() {
        return;
    }

    match_diagnostics::non_exhaustive_enum(
        ctx,
        name,
        &uncovered
            .iter()
            .map(|variant| variant.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        match_stmt.subject.range(),
    );
}

fn report_literal_exhaustiveness(
    subject_ty: &Type,
    arms: &[HirMatchArm],
    match_stmt: &StmtMatch,
    ctx: &mut LowerCtx,
) {
    if matches!(subject_ty, Type::Union(_) | Type::Enum { .. }) {
        return;
    }

    let all_literal_or_guarded = arms.iter().all(|arm| {
        matches!(
            arm.pattern,
            HirPattern::Literal { .. } | HirPattern::Or { .. }
        ) || arm.guard.is_some()
    });
    if all_literal_or_guarded {
        match_diagnostics::non_exhaustive_literal(
            ctx,
            &subject_ty.display_name(),
            match_stmt.subject.range(),
        );
    }
}
