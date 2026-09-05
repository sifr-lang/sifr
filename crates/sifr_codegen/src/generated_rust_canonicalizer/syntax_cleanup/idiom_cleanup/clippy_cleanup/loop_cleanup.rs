pub(super) fn collect_boxed_iterable_fields(file: &syn::File) -> HashSet<String> {
    struct Collector {
        names: HashSet<String>,
    }

    impl Visit<'_> for Collector {
        fn visit_field(&mut self, field: &syn::Field) {
            if matches!(&field.ty, syn::Type::Path(path)
                if path.path.segments.last().is_some_and(|segment|
                    segment.ident == "Box"))
                && let Some(name) = &field.ident
            {
                self.names.insert(name.to_string());
            }
            visit::visit_field(self, field);
        }
    }

    let mut collector = Collector {
        names: HashSet::new(),
    };
    collector.visit_file(file);
    collector.names
}

pub(super) fn remove_unnecessary_owned_iteration(
    for_loop: &mut syn::ExprForLoop,
    borrowed_names: &HashSet<String>,
) {
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
        let mut bindings = HashSet::new();
        collect_owned_pattern_names(&for_loop.pat, &mut bindings);
        let requires_owned_value = bindings.iter().any(|binding| {
            let mut uses = BorrowOnlyLoopBindingUse {
                binding,
                owned_use: false,
            };
            uses.visit_block(&for_loop.body);
            uses.owned_use
        });
        if requires_owned_value {
            LoopBindingUseRewriter {
                owned: &bindings,
                borrowed: &HashSet::new(),
            }
            .visit_block_mut(&mut for_loop.body);
            remove_last_use_clones_with_owned(&mut for_loop.body.stmts, &bindings, false);
            return;
        }
        for_loop.expr = cloned.receiver.clone();
        LoopBindingUseRewriter {
            owned: &HashSet::new(),
            borrowed: &bindings,
        }
        .visit_block_mut(&mut for_loop.body);
        for binding in &bindings {
            BorrowedLoopComparisonRewriter {
                binding,
                borrowed_names,
            }
            .visit_block_mut(&mut for_loop.body);
        }
        return;
    }
    if let syn::Expr::Macro(vector) = for_loop.expr.as_ref()
        && vector.mac.path.is_ident("vec")
    {
        let elements = vector.mac.tokens.clone();
        *for_loop.expr = syn::parse_quote!([#elements]);
        return;
    }
    if let syn::Expr::MethodCall(iterated) = for_loop.expr.as_ref()
        && iterated.method == "iter"
        && iterated.args.is_empty()
    {
        let path_receiver = matches!(iterated.receiver.as_ref(), syn::Expr::Path(_));
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
                for_loop.attrs.push(
                    syn::parse_quote!(#[expect(clippy::explicit_iter_loop, reason = #reason)]),
                );
            }
        }
        let mut bindings = HashSet::new();
        collect_owned_pattern_names(&for_loop.pat, &mut bindings);
        let simple_binding = simple_pattern_name(&for_loop.pat);
        let shadowed = simple_binding.as_deref().is_some_and(|binding| {
            let mut shadow = BindingShadowCollector {
                binding,
                found: false,
            };
            shadow.visit_block(&for_loop.body);
            shadow.found
        });
        if !shadowed {
            LoopBindingUseRewriter {
                owned: &HashSet::new(),
                borrowed: &bindings,
            }
            .visit_block_mut(&mut for_loop.body);
            for binding in &bindings {
                BorrowedLoopComparisonRewriter {
                    binding,
                    borrowed_names,
                }
                .visit_block_mut(&mut for_loop.body);
            }
        }
        if let Some(binding) = simple_pattern_name(&for_loop.pat)
            && binding.starts_with('_')
        {
            let mut uses = IdentifierUseCounter::default();
            uses.visit_block(&for_loop.body);
            if uses.counts.get(&binding).copied().unwrap_or(0) == 0 {
                *for_loop.pat = syn::parse_quote!(_);
            }
        }
    }
}

pub(super) fn refresh_explicit_iteration_expectation(
    for_loop: &mut syn::ExprForLoop,
    boxed_fields: &HashSet<String>,
) {
    let syn::Expr::MethodCall(iterated) = for_loop.expr.as_ref() else {
        return;
    };
    if iterated.method != "iter" || !iterated.args.is_empty() {
        return;
    }
    let syn::Expr::Field(field) = iterated.receiver.as_ref() else {
        return;
    };
    let syn::Member::Named(name) = &field.member else {
        return;
    };
    if boxed_fields.contains(&name.to_string()) {
        return;
    }
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

fn borrow_comparison_operand(operand: &mut Box<syn::Expr>) {
    if matches!(operand.as_ref(), syn::Expr::Reference(_)) {
        return;
    }
    if let syn::Expr::Unary(unary) = operand.as_ref()
        && matches!(unary.op, syn::UnOp::Deref(_))
    {
        *operand = unary.expr.clone();
        return;
    }
    let value = operand.as_ref().clone();
    **operand = syn::parse_quote!(&#value);
}

struct BorrowedLoopComparisonRewriter<'binding> {
    binding: &'binding str,
    borrowed_names: &'binding HashSet<String>,
}

impl VisitMut for BorrowedLoopComparisonRewriter<'_> {
    fn visit_expr_binary_mut(&mut self, binary: &mut syn::ExprBinary) {
        visit_mut::visit_expr_binary_mut(self, binary);
        if !comparison_operator(&binary.op) {
            return;
        }
        if matches!(binary.left.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.binding))
            && !expression_root_name(&binary.right)
                .is_some_and(|name| self.borrowed_names.contains(&name))
        {
            borrow_comparison_operand(&mut binary.right);
        }
        if matches!(binary.right.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.binding))
            && !expression_root_name(&binary.left)
                .is_some_and(|name| self.borrowed_names.contains(&name))
        {
            borrow_comparison_operand(&mut binary.left);
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
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
    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        self.visit_expr_mut(&mut call.receiver);
        for argument in &mut call.args {
            if call.method == "clone_from" && matches!(argument, syn::Expr::Reference(_)) {
                continue;
            }
            self.visit_expr_mut(argument);
        }
    }

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
        if matches!(
            call.method.to_string().as_str(),
            "to_owned" | "to_string" | "to_vec"
        ) && call.args.is_empty()
            && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.owned.contains(&name.to_string()) || self.borrowed.contains(&name.to_string())))
        {
            call.method = syn::Ident::new("clone", call.method.span());
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}
