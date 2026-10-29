//! Identify consumers before constructing a speculative binding environment.
use sifr_python_ast::visitor::{self, Visitor};
use sifr_python_ast::{Expr, Stmt};

pub(super) fn block_needs_inference(stmts: &[Stmt]) -> bool {
    struct Demand(bool);
    impl<'a> Visitor<'a> for Demand {
        // Expressions (including lambdas/comprehensions) contain no statement
        // assignments or function declarations and cannot consume these hints.
        fn visit_expr(&mut self, _expr: &'a Expr) {}

        fn visit_stmt(&mut self, stmt: &'a Stmt) {
            if self.0 {
                return;
            }
            if matches!(stmt, Stmt::Assign(_) | Stmt::FunctionDef(_)) {
                self.0 = true;
            } else {
                visitor::walk_stmt(self, stmt);
            }
        }
    }
    let mut demand = Demand(false);
    for stmt in stmts {
        demand.visit_stmt(stmt);
        if demand.0 {
            break;
        }
    }
    demand.0
}

#[cfg(test)]
#[path = "../../../tests/unit/lower/nested_function_inference/inference_demand/tests.rs"]
mod tests;
