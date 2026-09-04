fn rewrite_residual_typed_calls(body: &mut syn::Block, sifr_ints: &HashSet<String>) {
    CheckedReadIndexBorrowRewriter { sifr_ints }.visit_block_mut(body);
    RedundantLiteralIdentityConversionRewriter.visit_block_mut(body);
    let mut string_maps = StringMapCollector::default();
    string_maps.visit_block(body);
    StringMapLiteralLookupRewriter {
        maps: &string_maps.names,
    }
    .visit_block_mut(body);
    RepeatedCharReplaceRewriter.visit_block_mut(body);
}

struct CheckedReadIndexBorrowRewriter<'names> {
    sifr_ints: &'names HashSet<String>,
}

impl VisitMut for CheckedReadIndexBorrowRewriter<'_> {
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        visit_mut::visit_local_mut(self, local);
        if !simple_pattern_name(&local.pat)
            .is_some_and(|name| name.starts_with("sifr_generated_checked_read_index"))
        {
            return;
        }
        let Some(init) = &mut local.init else {
            return;
        };
        let syn::Expr::MethodCall(clone) = init.expr.as_ref() else {
            return;
        };
        if clone.method == "clone"
            && clone.args.is_empty()
            && expression_root_name(&clone.receiver)
                .is_some_and(|name| self.sifr_ints.contains(&name))
        {
            let source = clone.receiver.as_ref();
            init.expr = Box::new(syn::parse_quote!(&#source));
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

struct RedundantLiteralIdentityConversionRewriter;

impl VisitMut for RedundantLiteralIdentityConversionRewriter {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(conversion) = expression else {
            return;
        };
        if !matches!(conversion.method.to_string().as_str(), "to_string" | "to_owned")
            || !conversion.args.is_empty()
            || !matches!(conversion.receiver.as_ref(), syn::Expr::Call(call)
                if matches!(call.func.as_ref(), syn::Expr::Path(path)
                    if path.path.is_ident("identity"))
                    && matches!(call.args.first(), Some(syn::Expr::Reference(reference))
                        if matches!(reference.expr.as_ref(), syn::Expr::MethodCall(inner)
                            if matches!(inner.method.to_string().as_str(), "to_string" | "to_owned")
                                && matches!(inner.receiver.as_ref(), syn::Expr::Lit(literal)
                                    if matches!(literal.lit, syn::Lit::Str(_))))))
        {
            return;
        }
        *expression = conversion.receiver.as_ref().clone();
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        rewrite_macro_expressions(self, rust_macro);
    }
}

#[derive(Default)]
struct StringMapCollector {
    names: HashSet<String>,
}

impl Visit<'_> for StringMapCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && type_is_string_keyed_map(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.names.insert(name);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn type_is_string_keyed_map(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    matches!(segment.arguments, syn::PathArguments::AngleBracketed(ref arguments)
        if segment.ident == "HashMap"
            && matches!(arguments.args.first(), Some(syn::GenericArgument::Type(key))
                if type_is_owned_string(key)))
}

struct StringMapLiteralLookupRewriter<'names> {
    maps: &'names HashSet<String>,
}

impl VisitMut for StringMapLiteralLookupRewriter<'_> {
    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if !matches!(call.method.to_string().as_str(), "contains_key" | "get" | "get_mut" | "remove")
            || !expression_root_name(&call.receiver)
                .is_some_and(|name| self.maps.contains(&name))
        {
            return;
        }
        let Some(syn::Expr::Reference(reference)) = call.args.first() else {
            return;
        };
        let syn::Expr::MethodCall(conversion) = reference.expr.as_ref() else {
            return;
        };
        if matches!(conversion.method.to_string().as_str(), "to_string" | "to_owned")
            && conversion.args.is_empty()
            && matches!(conversion.receiver.as_ref(), syn::Expr::Lit(literal)
                if matches!(literal.lit, syn::Lit::Str(_)))
        {
            call.args[0] = conversion.receiver.as_ref().clone();
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        rewrite_macro_expressions(self, rust_macro);
    }
}

struct RepeatedCharReplaceRewriter;

impl VisitMut for RepeatedCharReplaceRewriter {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let mut characters = Vec::new();
        let Some((base, replacement)) = repeated_char_replacement(expression, &mut characters)
        else {
            return;
        };
        if characters.len() < 2
            || characters
                .iter()
                .any(|character| replacement.value().contains(character.value()))
        {
            return;
        }
        *expression = syn::parse_quote!((#base).replace([#(#characters),*], #replacement));
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn repeated_char_replacement(
    expression: &syn::Expr,
    characters: &mut Vec<syn::LitChar>,
) -> Option<(syn::Expr, syn::LitStr)> {
    let syn::Expr::MethodCall(call) = expression else {
        return None;
    };
    let [syn::Expr::Lit(pattern), syn::Expr::Lit(replacement)] = call.args.iter().collect::<Vec<_>>().as_slice() else {
        return None;
    };
    let (syn::Lit::Char(character), syn::Lit::Str(replacement)) = (&pattern.lit, &replacement.lit) else {
        return None;
    };
    if call.method != "replace" {
        return None;
    }
    characters.push(character.clone());
    if let Some((base, inner_replacement)) = repeated_char_replacement(&call.receiver, characters)
        && inner_replacement.value() == replacement.value()
    {
        return Some((base, replacement.clone()));
    }
    Some((call.receiver.as_ref().clone(), replacement.clone()))
}

fn rewrite_macro_expressions(rewriter: &mut impl VisitMut, rust_macro: &mut syn::Macro) {
    let Ok(mut arguments) = rust_macro.parse_body_with(
        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
    ) else {
        return;
    };
    for argument in &mut arguments {
        rewriter.visit_expr_mut(argument);
    }
    rust_macro.tokens = arguments.to_token_stream();
}
