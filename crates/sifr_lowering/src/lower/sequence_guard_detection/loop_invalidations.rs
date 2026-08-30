use super::sequence_guard_target_name;
use sifr_python_ast::visitor::{self, Visitor};
use sifr_python_ast::{Expr, Stmt};
use std::collections::BTreeSet;

pub(in crate::lower) fn loop_invalidated_sequence_targets(stmts: &[Stmt]) -> BTreeSet<String> {
    let mut visitor = LoopInvalidatedSequenceTargetVisitor::default();
    for stmt in stmts {
        visitor.visit_stmt(stmt);
    }
    visitor.targets
}

#[derive(Default)]
struct LoopInvalidatedSequenceTargetVisitor {
    targets: BTreeSet<String>,
}

impl LoopInvalidatedSequenceTargetVisitor {
    fn collect_rebound_target(&mut self, target: &Expr) {
        if matches!(target, Expr::Name(_) | Expr::Attribute(_)) {
            if let Some(target) = sequence_guard_target_name(target) {
                self.targets.insert(target);
            }
        }
    }

    fn collect_deleted_target(&mut self, target: &Expr) {
        let Expr::Subscript(subscript) = target else {
            return;
        };
        if let Some(target) = sequence_guard_target_name(subscript.value.as_ref()) {
            self.targets.insert(target);
        }
    }
}

impl<'ast> Visitor<'ast> for LoopInvalidatedSequenceTargetVisitor {
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        match stmt {
            Stmt::Assign(assign) => {
                for target in &assign.targets {
                    self.collect_rebound_target(target);
                }
            }
            Stmt::AnnAssign(assign) => self.collect_rebound_target(assign.target.as_ref()),
            Stmt::AugAssign(assign) => self.collect_rebound_target(assign.target.as_ref()),
            Stmt::Delete(delete) => {
                for target in &delete.targets {
                    self.collect_deleted_target(target);
                }
            }
            Stmt::FunctionDef(_) | Stmt::ClassDef(_) => return,
            _ => {}
        }
        visitor::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        if let Expr::Call(call) = expr {
            if let Expr::Attribute(attribute) = call.func.as_ref() {
                if matches!(
                    attribute.attr.as_str(),
                    "clear" | "pop" | "popleft" | "remove"
                ) {
                    if let Some(target) = sequence_guard_target_name(attribute.value.as_ref()) {
                        self.targets.insert(target);
                    }
                }
            }
        }
        visitor::walk_expr(self, expr);
    }
}
