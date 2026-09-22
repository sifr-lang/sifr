// Option/Result convenience rewrites require their actual standard contracts.
impl Rewriter<'_> {
    fn rewrite_standard_option_defaults(&self, expression: &mut syn::Expr) {
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        let Some(receiver) = self.ty(&call.receiver) else {
            return;
        };
        let receiver = unreference(&receiver);
        let is_option = self.standard_generic(receiver, "Option").is_some();
        let Some(payload) = self
            .standard_generic(receiver, "Option")
            .or_else(|| self.standard_generic(receiver, "Result"))
        else {
            return;
        };
        if is_option
            && !self.scalar_shadowed("None")
            && !self.bindings.contains_key("None")
            && call.method == "map_or"
            && call.args.len() == 2
            && matches!(&call.args[0], syn::Expr::Path(path) if path.path.is_ident("None"))
        {
            call.method = syn::Ident::new("and_then", call.method.span());
            call.args = std::iter::once(call.args[1].clone()).collect();
            return;
        }
        if matches!(call.method.to_string().as_str(), "map_or" | "map_or_else")
            && call.args.len() == 2
            && let syn::Expr::Closure(mapper) = &call.args[1]
            && mapper.asyncness.is_none()
            && mapper.inputs.len() == 1
            && let Some(syn::Pat::Ident(binding)) = mapper.inputs.first()
            && binding.by_ref.is_none()
            && binding.subpat.is_none()
            && matches!(mapper.body.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&binding.ident))
        {
            let method = if call.method == "map_or" {
                "unwrap_or"
            } else {
                "unwrap_or_else"
            };
            call.method = syn::Ident::new(method, call.method.span());
            call.args.pop();
        }
        if call.method == "unwrap_or"
            && call.args.len() == 1
            && self.standard_empty_constructor(&call.args[0], payload)
        {
            call.method = syn::Ident::new("unwrap_or_default", call.method.span());
            call.args.clear();
            return;
        }
        if matches!(call.method.to_string().as_str(), "unwrap_or" | "map_or")
            && !call.args.is_empty()
            && (self.pure_integer_default(&call.args[0])
                || self.literal_string_default(&call.args[0]))
            && !matches!(&call.args[0], syn::Expr::Call(default) if default.args.iter().all(|arg| matches!(arg, syn::Expr::Lit(_))))
        {
            let default = call.args[0].clone();
            call.args[0] = if is_option {
                syn::parse_quote!(|| #default)
            } else {
                syn::parse_quote!(|_| #default)
            };
            let method = if call.method == "map_or" {
                "map_or_else"
            } else {
                "unwrap_or_else"
            };
            call.method = syn::Ident::new(method, call.method.span());
        }
    }

    fn standard_empty_constructor(&self, expression: &syn::Expr, expected: &syn::Type) -> bool {
        let syn::Expr::Call(call) = expression else {
            return false;
        };
        if !call.args.is_empty() {
            return false;
        }
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return false;
        };
        if path.qself.is_some()
            || path.path.leading_colon.is_some()
            || path.path.segments.len() != 2
        {
            return false;
        }
        let name = path.path.segments[0].ident.to_string();
        matches!(
            path.path.segments[1].ident.to_string().as_str(),
            "new" | "default"
        ) && ((name == "String" && self.standard_named(expected, "String"))
            || (matches!(name.as_str(), "Vec" | "HashMap" | "HashSet")
                && self.standard_generic(expected, &name).is_some()))
    }

    fn literal_string_default(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::MethodCall(call) = expression else {
            return false;
        };
        !self.scalar_shadowed("str")
            && !self.scalar_shadowed("String")
            && self.clone_is_unambiguous()
            && matches!(call.method.to_string().as_str(), "to_string" | "to_owned")
            && call.args.is_empty()
            && matches!(call.receiver.as_ref(), syn::Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_)))
    }

    fn pure_integer_default(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::Call(call) = expression else {
            return false;
        };
        !self.scalar_shadowed("SifrInt")
            && matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.qself.is_none() && path.path.segments.len() == 2
                    && path.path.segments[0].ident == "SifrInt"
                    && matches!(path.path.segments[1].ident.to_string().as_str(), "from" | "from_i64"))
            && call.args.len() == 1
            && self.pure_integer_default_input(&call.args[0])
    }

    fn pure_integer_default_input(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Lit(lit) => matches!(lit.lit, syn::Lit::Int(_)),
            syn::Expr::Paren(paren) => self.pure_integer_default_input(&paren.expr),
            syn::Expr::MethodCall(call)
                if call.method == "len"
                    && call.args.is_empty()
                    && matches!(call.receiver.as_ref(), syn::Expr::Path(_)) =>
            {
                self.ty(&call.receiver).is_some_and(|ty| {
                    let ty = unreference(&ty);
                    matches!(ty, syn::Type::Slice(_) | syn::Type::Array(_))
                        || self.standard_named(ty, "String")
                        || self.standard_named(ty, "str")
                        || ["Vec", "HashMap", "HashSet"]
                            .iter()
                            .any(|name| self.standard_generic(ty, name).is_some())
                })
            }
            _ => false,
        }
    }
}
