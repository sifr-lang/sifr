fn remove_shadowing_condition_clones(
    statements: &mut [syn::Stmt],
    owned_parameters: &HashSet<String>,
) {
    for index in 0..statements.len() {
        let suffix_uses = identifier_uses(&statements[index + 1..]);
        let syn::Stmt::Expr(syn::Expr::If(branch), _) = &mut statements[index] else {
            continue;
        };
        let else_uses = branch
            .else_branch
            .as_ref()
            .map(|(_, alternative)| expression_identifier_uses(alternative))
            .unwrap_or_default();
        let movable = owned_parameters
            .iter()
            .filter(|name| {
                !suffix_uses.contains_key(*name) && !else_uses.contains_key(*name)
            })
            .cloned()
            .collect();
        ShadowingConditionCloneRemover { movable }.visit_expr_mut(&mut branch.cond);
    }
}

fn identifier_uses(statements: &[syn::Stmt]) -> HashMap<String, usize> {
    let mut uses = IdentifierUseCounter::default();
    for statement in statements {
        uses.visit_stmt(statement);
    }
    uses.counts
}

fn expression_identifier_uses(expression: &syn::Expr) -> HashMap<String, usize> {
    let mut uses = IdentifierUseCounter::default();
    uses.visit_expr(expression);
    uses.counts
}

struct ShadowingConditionCloneRemover {
    movable: HashSet<String>,
}

impl VisitMut for ShadowingConditionCloneRemover {
    fn visit_expr_let_mut(&mut self, let_: &mut syn::ExprLet) {
        visit_mut::visit_expr_let_mut(self, let_);
        let syn::Expr::MethodCall(clone) = let_.expr.as_ref() else {
            return;
        };
        let Some(name) = expression_root_name(&clone.receiver) else {
            return;
        };
        if clone.method == "clone"
            && clone.args.is_empty()
            && self.movable.contains(&name)
            && pattern_contains_name(&let_.pat, &name)
        {
            let_.expr = clone.receiver.clone();
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}
