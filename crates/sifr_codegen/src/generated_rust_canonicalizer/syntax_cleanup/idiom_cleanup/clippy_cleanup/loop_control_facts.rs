#[derive(Default)]
struct LoopControlCollector {
    found: bool,
    nested_loops: usize,
}

impl Visit<'_> for LoopControlCollector {
    fn visit_expr_break(&mut self, expression: &syn::ExprBreak) {
        self.found |= self.nested_loops == 0 || expression.label.is_some();
        visit::visit_expr_break(self, expression);
    }

    fn visit_expr_continue(&mut self, expression: &syn::ExprContinue) {
        self.found |= self.nested_loops == 0 || expression.label.is_some();
    }

    fn visit_expr_for_loop(&mut self, expression: &syn::ExprForLoop) {
        self.visit_expr(&expression.expr);
        self.nested_loops += 1;
        self.visit_block(&expression.body);
        self.nested_loops -= 1;
    }

    fn visit_expr_while(&mut self, expression: &syn::ExprWhile) {
        self.visit_expr(&expression.cond);
        self.nested_loops += 1;
        self.visit_block(&expression.body);
        self.nested_loops -= 1;
    }

    fn visit_expr_loop(&mut self, expression: &syn::ExprLoop) {
        self.nested_loops += 1;
        self.visit_block(&expression.body);
        self.nested_loops -= 1;
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
    fn visit_expr_closure(&mut self, _expression: &syn::ExprClosure) {}
}
