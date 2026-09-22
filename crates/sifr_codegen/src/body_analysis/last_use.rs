//! Last-use moves, including branch, loop, and capture liveness.
use super::{
    BodyAnalysis, HashMap, HashSet, HirExpr, HirStmt, expr_key, register_pattern_definitions,
    register_stmt_definitions, stmt_key, subtract_counts, traversal, walk_direct_stmt_exprs,
};

impl BodyAnalysis {
    pub(super) fn mark_last_uses(
        &mut self,
        stmts: &[HirStmt],
        defined: &mut HashSet<String>,
        outer_live: &HashSet<String>,
        borrowed: &HashSet<String>,
        moves: &mut HashSet<usize>,
    ) {
        let mut remaining = self
            .summary(stmts)
            .map(|summary| summary.referenced.clone())
            .unwrap_or_default();
        let mut live_nested_captures = HashSet::new();
        for stmt in stmts {
            let Some(stmt_summary) = self.statements.get(&stmt_key(stmt)).cloned() else {
                continue;
            };
            let stmt_referenced = stmt_summary.referenced;
            subtract_counts(&mut remaining, &stmt_referenced);
            let mut occurrences = HashMap::<String, Vec<(usize, bool, bool)>>::new();
            walk_direct_stmt_exprs(stmt, &mut |expr| {
                traversal::walk_expr(expr, &mut |candidate| {
                    if let HirExpr::Name { name, .. } = candidate {
                        occurrences.entry(name.clone()).or_default().push((
                            expr_key(candidate),
                            candidate.ty().contains_affine_resource(),
                            crate::helpers::is_copy_type_for_codegen(candidate.ty()),
                        ));
                    }
                });
            });
            let mut statement_has_last_use = false;
            for (name, expressions) in occurrences {
                let [(expr, contains_affine_resource, is_copy)] = expressions.as_slice() else {
                    continue;
                };
                if defined.contains(&name)
                    && stmt_referenced.get(&name).copied() == Some(expressions.len())
                    && !matches!(stmt, HirStmt::While { .. })
                    && !borrowed.contains(&name)
                    && remaining.get(&name).copied().unwrap_or(0) == 0
                    && !outer_live.contains(&name)
                    && !live_nested_captures.contains(&name)
                    && !contains_affine_resource
                    && !is_copy
                {
                    moves.insert(*expr);
                    statement_has_last_use = true;
                }
            }
            if statement_has_last_use {
                self.last_use_statements.insert(stmt_key(stmt));
            }
            // Construction may consume a source at its last use. Only later
            // statements must account for borrows retained by the new value.
            if let Some(captures) = self.nested_captures.get(&stmt_key(stmt)) {
                live_nested_captures.extend(captures.iter().cloned());
            }
            let mut child_outer_live = outer_live.clone();
            child_outer_live.extend(live_nested_captures.iter().cloned());
            self.mark_child_last_uses(
                stmt,
                defined,
                &child_outer_live,
                &remaining,
                borrowed,
                moves,
            );
            register_stmt_definitions(stmt, defined);
        }
    }

    fn mark_child_last_uses(
        &mut self,
        stmt: &HirStmt,
        defined: &HashSet<String>,
        outer_live: &HashSet<String>,
        remaining: &HashMap<String, usize>,
        borrowed: &HashSet<String>,
        moves: &mut HashSet<usize>,
    ) {
        let mut live_after = outer_live.clone();
        live_after.extend(remaining.keys().cloned());
        let scan = |analysis: &mut Self,
                    body: &[HirStmt],
                    seed: &HashSet<String>,
                    live: &HashSet<String>,
                    moves: &mut HashSet<usize>| {
            let mut body_defined = seed.clone();
            analysis.mark_last_uses(body, &mut body_defined, live, borrowed, moves);
        };
        match stmt {
            HirStmt::If {
                then_body,
                elif_clauses,
                else_body,
                ..
            } => {
                scan(self, then_body, defined, &live_after, moves);
                for (_, body) in elif_clauses {
                    scan(self, body, defined, &live_after, moves);
                }
                if let Some(body) = else_body {
                    scan(self, body, defined, &live_after, moves);
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
                let mut loop_live = live_after.clone();
                loop_live.extend(defined.iter().cloned());
                let mut body_defined = defined.clone();
                if let HirStmt::For { target, .. } | HirStmt::AsyncFor { target, .. } = stmt {
                    body_defined.insert(target.clone());
                }
                scan(self, body, &body_defined, &loop_live, moves);
                if let Some(body) = else_body {
                    scan(self, body, defined, &live_after, moves);
                }
            }
            HirStmt::TryExcept { body, handlers, .. } => {
                let mut body_live = live_after.clone();
                for handler in handlers {
                    if let Some(summary) = self.summary(&handler.body) {
                        body_live.extend(summary.referenced.keys().cloned());
                    }
                }
                scan(self, body, defined, &body_live, moves);
                for handler in handlers {
                    let mut handler_defined = defined.clone();
                    if let Some(name) = &handler.name {
                        handler_defined.insert(name.clone());
                    }
                    scan(self, &handler.body, &handler_defined, &live_after, moves);
                }
            }
            HirStmt::TryFinally { body, finalbody } => {
                let mut body_live = live_after.clone();
                if let Some(summary) = self.summary(finalbody) {
                    body_live.extend(summary.referenced.keys().cloned());
                }
                scan(self, body, defined, &body_live, moves);
                scan(self, finalbody, defined, &live_after, moves);
            }
            HirStmt::With { items, body } => {
                let mut body_defined = defined.clone();
                body_defined.extend(items.iter().map(|item| item.target.clone()));
                scan(self, body, &body_defined, &live_after, moves);
            }
            HirStmt::AsyncWith { target, body, .. } => {
                let mut body_defined = defined.clone();
                body_defined.extend(target.iter().cloned());
                scan(self, body, &body_defined, &live_after, moves);
            }
            HirStmt::Match { arms, .. } => {
                for arm in arms {
                    let mut arm_defined = defined.clone();
                    register_pattern_definitions(&arm.pattern, &mut arm_defined);
                    scan(self, &arm.body, &arm_defined, &live_after, moves);
                }
            }
            _ => {}
        }
    }
}
