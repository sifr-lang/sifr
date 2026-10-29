fn condition_binds_name(expression: &syn::Expr, name: &str) -> bool {
    condition_binding_names(expression).contains(name)
}

fn condition_binding_names(expression: &syn::Expr) -> HashSet<String> {
    match expression {
        syn::Expr::Let(let_) => super::identifier_names_in_pattern(&let_.pat),
        syn::Expr::Binary(binary) if matches!(binary.op, syn::BinOp::And(_)) => {
            let mut names = condition_binding_names(&binary.left);
            names.extend(condition_binding_names(&binary.right));
            names
        }
        syn::Expr::Group(group) => condition_binding_names(&group.expr),
        syn::Expr::Paren(paren) => condition_binding_names(&paren.expr),
        _ => HashSet::new(),
    }
}
