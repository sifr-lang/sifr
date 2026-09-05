pub(super) fn borrow_generated_index_clone(local: &mut syn::Local) {
    let Some(name) = simple_pattern_name(&local.pat) else {
        return;
    };
    if !name.starts_with("sifr_generated_index") && !name.starts_with("sifr_generated_string_index")
    {
        return;
    }
    let Some(initializer) = &mut local.init else {
        return;
    };
    let syn::Expr::MethodCall(clone) = initializer.expr.as_ref() else {
        return;
    };
    if clone.method == "clone" && clone.args.is_empty() {
        let receiver = clone.receiver.as_ref();
        initializer.expr = if let syn::Expr::Unary(dereference) = receiver
            && matches!(dereference.op, syn::UnOp::Deref(_))
        {
            dereference.expr.clone()
        } else {
            Box::new(syn::parse_quote!(&#receiver))
        };
    }
}

pub(super) fn remove_generated_checked_value_clone_borrow(expression: &mut syn::Expr) {
    let syn::Expr::Reference(reference) = expression else {
        return;
    };
    let syn::Expr::MethodCall(clone) = reference.expr.as_ref() else {
        return;
    };
    if clone.method != "clone" || !clone.args.is_empty() {
        return;
    }
    if matches!(clone.receiver.as_ref(), syn::Expr::Path(path)
        if path.path.get_ident().is_some_and(|name|
            name.to_string().starts_with("sifr_generated_checked_value")))
    {
        reference.expr = clone.receiver.clone();
    }
}
