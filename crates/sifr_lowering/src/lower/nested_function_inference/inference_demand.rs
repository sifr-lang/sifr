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
mod tests {
    use super::*;
    use crate::lower::LowerCtx;
    use crate::lower::nested_function_inference::infer_nested_function_types;

    #[test]
    fn declarations_and_assignments_in_any_branch_require_inference() {
        for source in [
            "value = missing\n",
            "if True:\n    values = {}\n",
            "while True:\n    values = defaultdict(int)\n",
            "try:\n    pass\nfinally:\n    value = 1\n",
            "match value:\n    case 1:\n        value = 2\n",
            "def nested(arg):\n    return arg\n",
            "if True:\n    def nested(arg):\n        return arg\n",
        ] {
            let parsed = sifr_python_parser::parse_module(source).expect("valid syntax");
            assert!(block_needs_inference(parsed.suite()), "{source}");
        }
    }

    #[test]
    fn blocks_without_consumers_leave_diagnostics_and_hint_maps_empty() {
        for source in [
            "",
            "value: Missing = missing\n",
            "if True:\n    value: int = 1\nelse:\n    print(missing)\n",
            "for value in values:\n    print(value)\n",
            "try:\n    print(missing)\nfinally:\n    print(other)\n",
        ] {
            let parsed = sifr_python_parser::parse_module(source).expect("valid syntax");
            assert!(!block_needs_inference(parsed.suite()), "{source}");
            let mut ctx = LowerCtx::new();
            let result = infer_nested_function_types(parsed.suite(), &mut ctx);
            assert!(ctx.errors.is_empty());
            assert!(ctx.last_error_taint.is_none());
            assert!(result.function_types.is_empty());
            assert!(result.binding_hints.is_empty());
            assert!(result.exact_dict_write_hints.is_empty());
            assert!(result.function_captures.is_empty());
            assert!(result.function_mutated_captures.is_empty());
        }
    }
}
