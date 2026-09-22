impl Rewriter<'_> {
    fn rewrite_typed_string_clones(&self, expression: &mut syn::Expr) {
        self.rewrite_single_character_pattern(expression);
        if !self.clone_is_unambiguous() || self.scalar_shadowed("String") { return; }
        let mut owned = std::collections::HashSet::new();
        let mut borrowed = std::collections::HashSet::new();
        for (name, ty) in &self.bindings {
            match ty {
                Some(ty) if self.standard_named(ty, "String") => { owned.insert(name.clone()); }
                Some(ty) if matches!(ty, syn::Type::Reference(_)) && self.standard_named(unreference(ty), "str") => { borrowed.insert(name.clone()); }
                _ => {}
            }
        }
        if super::idiom_cleanup::rewrite_known_identity_concat(expression, &owned, &borrowed,
            |capacity| self.known_string_capacity(capacity)) { return; }
        let syn::Expr::MethodCall(outer) = expression else {
            return;
        };
        if outer.method == "map_or_else" && outer.args.len() == 2 && !self.scalar_shadowed("Option")
            && self.ty(&outer.receiver).is_some_and(|ty| self.standard_generic(&ty, "Option").is_some_and(|inner| self.standard_named(inner, "String")))
            && let Some(syn::Expr::Closure(mapper)) = outer.args.last_mut()
            && mapper.inputs.len() == 1
            && let Some(syn::Pat::Ident(binding)) = mapper.inputs.first()
            && binding.by_ref.is_none()
            && let syn::Expr::MethodCall(conversion) = mapper.body.as_ref()
            && matches!(conversion.method.to_string().as_str(), "clone" | "to_owned" | "to_string")
            && conversion.args.is_empty()
            && matches!(conversion.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&binding.ident)) {
            mapper.body = conversion.receiver.clone();
        }
        if !outer.args.is_empty() { return; }
        if matches!(outer.method.to_string().as_str(), "to_string" | "to_owned")
            && self.ty(&outer.receiver).is_some_and(|ty| self.standard_named(unreference(&ty), "String")) {
            outer.method = syn::Ident::new("clone", outer.method.span());
        }
        let syn::Expr::MethodCall(inner) = outer.receiver.as_ref() else {
            return;
        };
        if !inner.args.is_empty() {
            return;
        }
        if matches!(
            outer.method.to_string().as_str(),
            "clone" | "to_owned" | "to_string"
        ) && matches!(
            inner.method.to_string().as_str(),
            "clone" | "to_owned" | "to_string"
        ) && !self.scalar_shadowed("String")
            && self
                .ty(&outer.receiver)
                .is_some_and(|ty| self.standard_named(&ty, "String"))
        {
            *expression = outer.receiver.as_ref().clone();
            return;
        }
        if (outer.method == "to_string" && inner.method == "clone")
            || (outer.method == "as_str"
                && matches!(inner.method.to_string().as_str(), "clone" | "to_string"))
        {
            let known = self.ty(&inner.receiver).is_some_and(|ty| {
                let base = unreference(&ty);
                (self.standard_named(base, "String") && !self.scalar_shadowed("String"))
                    || (outer.method == "to_string"
                        && self.standard_named(base, "SifrInt")
                        && !self.scalar_shadowed("SifrInt"))
            });
            if known {
                outer.receiver = inner.receiver.clone();
            }
        }
    }
}

impl Rewriter<'_> {
    fn known_string_capacity(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Lit(literal) => matches!(literal.lit, syn::Lit::Int(_)),
            syn::Expr::Paren(paren) => self.known_string_capacity(&paren.expr),
            syn::Expr::MethodCall(call) if call.method == "len" && call.args.is_empty() => {
                matches!(call.receiver.as_ref(), syn::Expr::Path(_) | syn::Expr::Lit(_))
                    && self.ty(&call.receiver).is_some_and(|ty| self.standard_named(unreference(&ty), "String") || self.standard_named(unreference(&ty), "str"))
            }
            syn::Expr::MethodCall(call) if call.method == "saturating_add" && call.args.len() == 1 => {
                self.known_string_capacity(&call.receiver) && self.known_string_capacity(&call.args[0])
            }
            _ => false,
        }
    }

    fn repair_borrowed_str_clone(&self, expr: &mut syn::Expr, expected: &syn::Type) {
        if self.standard_named(expected, "String") && !self.scalar_shadowed("String") && self.clone_is_unambiguous()
            && let syn::Expr::MethodCall(call) = expr
            && call.method == "clone" && call.args.is_empty()
            && self.ty(&call.receiver).is_some_and(|ty| self.standard_named(unreference(&ty), "str")) {
            call.method = syn::Ident::new("to_string", call.method.span());
        }
    }
}

impl Rewriter<'_> {
    fn rewrite_borrowed_empty_string(&self, expression: &mut syn::Expr, target: &syn::Type) -> bool {
        if !self.standard_named(target, "str") { return false; }
        let value = match expression {
            syn::Expr::Reference(reference) if reference.mutability.is_none() => reference.expr.as_ref(),
            syn::Expr::Call(_) => &*expression,
            _ => return false,
        };
        if !self.standard_empty_constructor(value, &syn::parse_quote!(String)) { return false; }
        *expression = syn::parse_quote!("");
        true
    }
}

impl Rewriter<'_> {
fn rewrite_single_character_pattern(&self, expression: &mut syn::Expr) -> bool {
    let syn::Expr::MethodCall(call) = expression else {
        return false;
    };
    if self.ambiguous_string_pattern_scopes.contains(&self.scope[..self.module_depth].join("::"))
        || !self.ty(&call.receiver).is_some_and(|ty| self.standard_named(unreference(&ty), "String")
            || self.standard_named(unreference(&ty), "str")) {
        return false;
    }
    if !matches!(
        call.method.to_string().as_str(),
        "contains"
            | "ends_with"
            | "find"
            | "rfind"
            | "split"
            | "split_inclusive"
            | "split_terminator"
            | "starts_with"
            | "strip_prefix"
            | "strip_suffix"
            | "trim_end_matches"
            | "trim_matches"
            | "trim_start_matches"
    ) || call.args.len() != 1
    {
        return false;
    }
    let Some(syn::Expr::Lit(literal)) = call.args.first_mut() else {
        return false;
    };
    let syn::Lit::Str(text) = &literal.lit else {
        return false;
    };
    let value = text.value();
    let mut characters = value.chars();
    let Some(character) = characters.next() else {
        return false;
    };
    if characters.next().is_some() {
        return false;
    }
    literal.lit = syn::Lit::Char(syn::LitChar::new(character, text.span()));
    true
}

}
