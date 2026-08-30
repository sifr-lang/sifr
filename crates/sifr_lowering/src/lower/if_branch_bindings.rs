use crate::hir_nodes::{HirExpr, HirStmt};
use ruff_text_size::TextRange;
use sifr_diagnostics::DiagnosticCode;
use sifr_python_ast::{Expr, Stmt, StmtIf};
use sifr_type_system::{Type, make_union};

use super::LowerCtx;

fn top_level_assigned_name(stmt: &Stmt) -> Option<String> {
    match stmt {
        Stmt::Assign(assign) if assign.targets.len() == 1 => {
            if let Expr::Name(name) = &assign.targets[0] {
                Some(name.id.to_string())
            } else {
                None
            }
        }
        Stmt::AnnAssign(ann) => {
            if let Expr::Name(name) = ann.target.as_ref() {
                Some(name.id.to_string())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn collect_top_level_assigned_names(stmts: &[Stmt]) -> std::collections::HashSet<String> {
    let mut names = std::collections::HashSet::new();
    for stmt in stmts {
        if let Some(name) = top_level_assigned_name(stmt) {
            names.insert(name);
        }
    }
    names
}

pub(in crate::lower) fn predeclare_exhaustive_if_assigned_names(
    if_stmt: &StmtIf,
    ctx: &mut LowerCtx,
) {
    let else_clause = if_stmt
        .elif_else_clauses
        .iter()
        .find(|clause| clause.test.is_none());
    let Some(else_clause) = else_clause else {
        return;
    };

    let mut branch_names = Vec::new();
    branch_names.push(collect_top_level_assigned_names(&if_stmt.body));
    for clause in &if_stmt.elif_else_clauses {
        if clause.test.is_some() {
            branch_names.push(collect_top_level_assigned_names(&clause.body));
        }
    }
    branch_names.push(collect_top_level_assigned_names(&else_clause.body));
    if branch_names.iter().any(std::collections::HashSet::is_empty) {
        return;
    }

    let mut iter = branch_names.into_iter();
    let Some(mut shared_names) = iter.next() else {
        return;
    };
    for names in iter {
        shared_names = shared_names.intersection(&names).cloned().collect();
        if shared_names.is_empty() {
            return;
        }
    }

    for name in shared_names {
        if ctx.scope.lookup(&name).is_none() {
            ctx.scope.define(name, Type::Unknown);
        }
    }
}

fn collect_top_level_let_bindings(stmts: &[HirStmt]) -> std::collections::HashMap<String, Type> {
    let mut bindings = std::collections::HashMap::new();
    for stmt in stmts {
        if let HirStmt::Let { name, ty, .. } = stmt {
            bindings.insert(name.clone(), ty.clone());
        }
    }
    bindings
}

fn merge_binding_types(types: impl Iterator<Item = Type>) -> Option<Type> {
    let mut merged: Vec<Type> = Vec::new();
    for ty in types {
        if !merged.iter().any(|existing| existing == &ty) {
            merged.push(ty);
        }
    }
    match merged.len() {
        0 => None,
        1 => merged.into_iter().next(),
        _ => Some(make_union(merged)),
    }
}

pub(in crate::lower) fn seed_exhaustive_if_bindings(
    ctx: &mut LowerCtx,
    then_body: &[HirStmt],
    elif_clauses: &[(HirExpr, Vec<HirStmt>)],
    else_body: Option<&Vec<HirStmt>>,
    range: TextRange,
) {
    let Some(else_body) = else_body else {
        return;
    };
    let then_bindings = collect_top_level_let_bindings(then_body);
    if then_bindings.is_empty() {
        return;
    }
    let mut branch_bindings = Vec::new();
    branch_bindings.push(then_bindings);
    for (_, body) in elif_clauses {
        branch_bindings.push(collect_top_level_let_bindings(body));
    }
    branch_bindings.push(collect_top_level_let_bindings(else_body));
    if branch_bindings
        .iter()
        .any(std::collections::HashMap::is_empty)
    {
        return;
    }

    let first_keys: Vec<String> = branch_bindings[0].keys().cloned().collect();
    for name in first_keys {
        if ctx
            .scope
            .lookup(&name)
            .is_some_and(|existing| !matches!(existing.ty.resolve_alias(), Type::Unknown))
        {
            continue;
        }
        let mut ty_candidates = Vec::new();
        let mut present_in_all = true;
        for branch in &branch_bindings {
            let Some(ty) = branch.get(&name) else {
                present_in_all = false;
                break;
            };
            ty_candidates.push(ty.clone());
        }
        if !present_in_all {
            continue;
        }
        if let Some(merged_ty) = merge_binding_types(ty_candidates.into_iter()) {
            if merged_ty.has_width_related_structural_records() {
                ctx.error_with_code_at(
                    DiagnosticCode::TYPE_MISMATCH,
                    format!(
                        "branches cannot infer '{name}' as a union of width-related record shapes; project to one shape or add a tag field"
                    ),
                    range,
                );
                continue;
            }
            ctx.scope.define(name, merged_ty);
        }
    }
}
