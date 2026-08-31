use super::state_collection::{LocalFunctionState, ParamState};
use sifr_python_ast::visitor::{self, Visitor};
use sifr_python_ast::{Expr, Stmt};
use std::collections::{HashMap, HashSet};

pub(super) struct NestedFunctionCallEffects {
    pub(super) called_functions: Vec<String>,
}

pub(super) fn nested_function_call_effects(
    body: &[Stmt],
    states: &HashMap<String, LocalFunctionState<'_>>,
    _captures: &[(String, sifr_type_system::Type)],
    _params: &[ParamState],
) -> NestedFunctionCallEffects {
    let mut visitor = DirectNestedCallVisitor {
        nested_names: states.keys().map(String::as_str).collect(),
        called: HashSet::new(),
    };
    for stmt in body {
        visitor.visit_stmt(stmt);
    }
    let mut called_functions = visitor.called.into_iter().collect::<Vec<_>>();
    called_functions.sort();
    NestedFunctionCallEffects { called_functions }
}

struct DirectNestedCallVisitor<'a> {
    nested_names: HashSet<&'a str>,
    called: HashSet<String>,
}

impl<'ast> Visitor<'ast> for DirectNestedCallVisitor<'_> {
    fn visit_stmt(&mut self, stmt: &'ast Stmt) {
        if matches!(stmt, Stmt::FunctionDef(_)) {
            return;
        }
        visitor::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'ast Expr) {
        if let Expr::Call(call) = expr
            && let Expr::Name(function) = call.func.as_ref()
        {
            let function = function.id.as_str();
            if self.nested_names.contains(function) {
                self.called.insert(function.to_string());
            }
        }
        visitor::walk_expr(self, expr);
    }
}
