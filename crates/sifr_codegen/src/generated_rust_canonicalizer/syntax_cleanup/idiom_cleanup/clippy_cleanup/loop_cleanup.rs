pub(super) fn remove_unnecessary_owned_iteration(for_loop: &mut syn::ExprForLoop) {
    let mut owned = HashSet::new();
    collect_owned_pattern_names(&for_loop.pat, &mut owned);
    remove_clones_from_return_expressions(&mut for_loop.body, &owned);
    if let syn::Expr::MethodCall(into_iter) = for_loop.expr.as_ref()
        && into_iter.method == "into_iter"
        && into_iter.args.is_empty()
    {
        let receiver = into_iter.receiver.clone();
        rewrite_materialized_loop_binding_uses(for_loop, &receiver);
        if matches!(receiver.as_ref(), syn::Expr::MethodCall(collected)
            if collected.method == "collect"
                && collected.args.is_empty()
                && matches!(collected.receiver.as_ref(), syn::Expr::MethodCall(mapped)
                    if mapped.method == "map"))
        {
            let reason = syn::LitStr::new(
                "language necessity: this generated Rust materializes cloned mapping entries before typed loop mutation; owner Item 12; remove when ownership lowering proves direct iteration safe",
                proc_macro2::Span::call_site(),
            );
            for_loop
                .attrs
                .push(syn::parse_quote!(#[expect(clippy::needless_collect, reason = #reason)]));
        }
        for_loop.expr = receiver;
        return;
    }
    if let syn::Expr::MethodCall(mapped) = for_loop.expr.as_mut()
        && mapped.method == "map"
        && mapped.args.len() == 1
        && let syn::Expr::MethodCall(cloned) = mapped.receiver.as_ref()
        && cloned.method == "cloned"
        && cloned.args.is_empty()
        && matches!(cloned.receiver.as_ref(), syn::Expr::MethodCall(iterated)
            if iterated.method == "iter" && iterated.args.is_empty())
        && let Some(syn::Expr::Closure(mapper)) = mapped.args.first_mut()
        && let Some(binding) = mapper.inputs.first().and_then(simple_pattern_name)
        && mapper.inputs.len() == 1
    {
        let mut uses = BorrowOnlyLoopBindingUse {
            binding: &binding,
            owned_use: false,
        };
        uses.visit_expr(&mapper.body);
        if !uses.owned_use {
            mapped.receiver = cloned.receiver.clone();
            LoopBindingUseRewriter {
                owned: &HashSet::new(),
                borrowed: &HashSet::from([binding]),
            }
            .visit_expr_mut(&mut mapper.body);
            return;
        }
    }
    if let syn::Expr::MethodCall(cloned) = for_loop.expr.as_ref()
        && cloned.method == "cloned"
        && cloned.args.is_empty()
        && matches!(cloned.receiver.as_ref(), syn::Expr::MethodCall(iterated)
            if iterated.method == "iter" && iterated.args.is_empty())
    {
        if let Some(binding) = simple_pattern_name(&for_loop.pat) {
            let mut uses = BorrowOnlyLoopBindingUse {
                binding: &binding,
                owned_use: false,
            };
            uses.visit_block(&for_loop.body);
            if uses.owned_use {
                return;
            }
            for_loop.expr = cloned.receiver.clone();
            BorrowedLoopComparisonRewriter { binding: &binding }
                .visit_block_mut(&mut for_loop.body);
            return;
        }
        let mut bindings = HashSet::new();
        collect_owned_pattern_names(&for_loop.pat, &mut bindings);
        if bindings.iter().any(|binding| {
            let mut uses = BorrowOnlyLoopBindingUse {
                binding,
                owned_use: false,
            };
            uses.visit_block(&for_loop.body);
            uses.owned_use
        }) {
            return;
        }
        for_loop.expr = cloned.receiver.clone();
        LoopBindingUseRewriter {
            owned: &HashSet::new(),
            borrowed: &bindings,
        }
        .visit_block_mut(&mut for_loop.body);
        for binding in &bindings {
            BorrowedLoopComparisonRewriter { binding }.visit_block_mut(&mut for_loop.body);
        }
        return;
    }
    let Some(binding) = simple_pattern_name(&for_loop.pat) else {
        return;
    };
    if let syn::Expr::MethodCall(iterated) = for_loop.expr.as_ref()
        && iterated.method == "iter"
        && iterated.args.is_empty()
    {
        let path_receiver = matches!(iterated.receiver.as_ref(), syn::Expr::Path(_));
        let direct_field_receiver = matches!(iterated.receiver.as_ref(), syn::Expr::Field(field)
            if matches!(&field.member, syn::Member::Named(name)
                if matches!(name.to_string().as_str(), "data" | "fallbacks")));
        if direct_field_receiver {
            let receiver = iterated.receiver.as_ref().clone();
            for_loop.expr = Box::new(syn::parse_quote!(&#receiver));
        }
        if path_receiver {
            let reason = syn::LitStr::new(
                "language necessity: generated Rust borrows this typed Sifr iteration source; owner Item 12; remove when direct IntoIterator preserves the same source lifetime",
                proc_macro2::Span::call_site(),
            );
            if !for_loop.attrs.iter().any(|attribute| {
                attribute.path().is_ident("expect")
                    && attribute
                        .meta
                        .to_token_stream()
                        .to_string()
                        .contains("explicit_iter_loop")
            }) {
                for_loop
                    .attrs
                    .push(syn::parse_quote!(#[expect(clippy::explicit_iter_loop, reason = #reason)]));
            }
        }
        let mut shadow = BindingShadowCollector {
            binding: &binding,
            found: false,
        };
        shadow.visit_block(&for_loop.body);
        if !shadow.found {
            LoopBindingUseRewriter {
                owned: &HashSet::new(),
                borrowed: &HashSet::from([binding.clone()]),
            }
            .visit_block_mut(&mut for_loop.body);
        }
        if binding.starts_with('_') {
            let mut uses = IdentifierUseCounter::default();
            uses.visit_block(&for_loop.body);
            if uses.counts.get(&binding).copied().unwrap_or(0) == 0 {
                for_loop.pat = Box::new(syn::parse_quote!(_));
            }
        }
    }
}

fn rewrite_materialized_loop_binding_uses(
    for_loop: &mut syn::ExprForLoop,
    materialized: &syn::Expr,
) {
    let syn::Pat::Tuple(pattern) = for_loop.pat.as_ref() else {
        return;
    };
    let syn::Expr::MethodCall(collected) = materialized else {
        return;
    };
    if collected.method != "collect" {
        return;
    }
    let syn::Expr::MethodCall(mapped) = collected.receiver.as_ref() else {
        return;
    };
    if mapped.method != "map" || mapped.args.len() != 1 {
        return;
    }
    let Some(syn::Expr::Closure(mapper)) = mapped.args.first() else {
        return;
    };
    let syn::Expr::Tuple(values) = mapper.body.as_ref() else {
        return;
    };
    if pattern.elems.len() != values.elems.len() {
        return;
    }
    let mut owned = HashSet::new();
    let mut borrowed = HashSet::new();
    for (binding, value) in pattern.elems.iter().zip(&values.elems) {
        let Some(name) = simple_pattern_name(binding) else {
            continue;
        };
        if matches!(value, syn::Expr::MethodCall(call)
            if matches!(call.method.to_string().as_str(), "clone" | "to_owned" | "to_string")
                && call.args.is_empty())
        {
            owned.insert(name);
        } else if matches!(value, syn::Expr::Field(_)) {
            borrowed.insert(name);
        }
    }
    LoopBindingUseRewriter {
        owned: &owned,
        borrowed: &borrowed,
    }
    .visit_block_mut(&mut for_loop.body);
    remove_last_use_clones_with_owned(&mut for_loop.body.stmts, &owned, false);
    remove_clones_from_return_expressions(&mut for_loop.body, &owned);
    if block_ends_control_flow(&for_loop.body) {
        let mut counts = IdentifierUseCounter::default();
        counts.visit_block(&for_loop.body);
        LastUseCloneRemover {
            movable: &owned,
            remaining: counts.counts,
        }
        .visit_block_mut(&mut for_loop.body);
    }
}

struct LoopBindingUseRewriter<'names> {
    owned: &'names HashSet<String>,
    borrowed: &'names HashSet<String>,
}

impl VisitMut for LoopBindingUseRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if let syn::Expr::Reference(reference) = expression
            && reference.mutability.is_none()
            && matches!(reference.expr.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.borrowed.contains(&name.to_string())))
        {
            *expression = reference.expr.as_ref().clone();
            return;
        }
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if matches!(call.method.to_string().as_str(), "to_owned" | "to_string")
            && call.args.is_empty()
            && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.owned.contains(&name.to_string())))
        {
            call.method = syn::Ident::new("clone", call.method.span());
        } else if matches!(call.method.to_string().as_str(), "to_owned" | "to_string")
            && call.args.is_empty()
            && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.borrowed.contains(&name.to_string())))
        {
            call.method = syn::Ident::new("clone", call.method.span());
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}
