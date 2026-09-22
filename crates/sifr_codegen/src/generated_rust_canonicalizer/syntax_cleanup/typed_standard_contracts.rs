include!("typed_option_defaults.rs");
// Resolve standard-type cleanup against declarations, imports, and opaque scopes.
// A local type with a standard spelling keeps its own methods and conversions.
impl Rewriter<'_> {
    fn standard_named(&self, ty: &syn::Type, name: &str) -> bool {
        !self.scalar_shadowed(name) && named(ty, name)
    }

    fn standard_generic<'a>(&self, ty: &'a syn::Type, name: &str) -> Option<&'a syn::Type> {
        if self.scalar_shadowed(name) {
            None
        } else {
            generic(ty, name)
        }
    }

    fn standard_result_error<'a>(&self, ty: &'a syn::Type) -> Option<&'a syn::Type> {
        self.standard_generic(ty, "Result")?;
        result_error(ty)
    }

    fn standard_vector_slice_coercion(&self, source: &syn::Type, target: &syn::Type) -> bool {
        !self.scalar_shadowed("Vec") && shared_vector_slice_coercion(source, target)
    }

    fn standard_callable_inputs(&self, ty: &syn::Type) -> Option<Vec<syn::Type>> {
        if self.scalar_shadowed("Box") {
            None
        } else {
            callable_inputs(ty)
        }
    }
}

impl Rewriter<'_> {
    fn rewrite_standard_lazy_fallback(&self, expression: &mut syn::Expr) {
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if call.method != "unwrap_or_else" || call.args.len() != 1 {
            return;
        }
        let Some(receiver) = self.ty(&call.receiver) else {
            return;
        };
        let arity = if self
            .standard_generic(unreference(&receiver), "Option")
            .is_some()
        {
            0
        } else if self
            .standard_generic(unreference(&receiver), "Result")
            .is_some()
        {
            1
        } else {
            return;
        };
        let Some(syn::Expr::Closure(closure)) = call.args.first() else {
            return;
        };
        if closure.asyncness.is_some() || closure.inputs.len() != arity
            || !closure.inputs.iter().all(|input| matches!(input, syn::Pat::Wild(_))
                || matches!(input, syn::Pat::Ident(binding) if binding.ident.to_string().starts_with('_')))
            || !self.eager_fallback_is_safe(&closure.body) {
            return;
        }
        let fallback = closure.body.as_ref().clone();
        call.method = syn::Ident::new("unwrap_or", call.method.span());
        call.args = std::iter::once(fallback).collect();
    }
}

impl Rewriter<'_> {
    fn eager_fallback_is_safe(&self, expression: &syn::Expr) -> bool {
        match expression {
            syn::Expr::Lit(_) => true,
            syn::Expr::Paren(paren) => self.eager_fallback_is_safe(&paren.expr),
            syn::Expr::Tuple(tuple) => tuple
                .elems
                .iter()
                .all(|value| self.eager_fallback_is_safe(value)),
            syn::Expr::Array(array) => array
                .elems
                .iter()
                .all(|value| self.eager_fallback_is_safe(value)),
            syn::Expr::Call(call) => {
                !self.scalar_shadowed("SifrInt")
                    && call.args.len() == 1
                    && matches!(call.func.as_ref(), syn::Expr::Path(path)
                    if path.qself.is_none() && path.path.leading_colon.is_none()
                        && path.path.segments.len() == 2
                        && path.path.segments[0].ident == "SifrInt"
                        && path.path.segments[1].ident == "from_i64")
                    && call
                        .args
                        .iter()
                        .all(|value| self.eager_fallback_is_safe(value))
            }
            syn::Expr::Path(path) => {
                path.qself.is_none()
                    && path.path.leading_colon.is_none()
                    && path.path.segments.len() == 2
                    && !self.scalar_shadowed(&path.path.segments[0].ident.to_string())
                    && matches!(
                        path.path.segments[0].ident.to_string().as_str(),
                        "usize"
                            | "isize"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "u128"
                            | "i8"
                            | "i16"
                            | "i32"
                            | "i64"
                            | "i128"
                    )
                    && path.path.segments[1].ident == "MAX"
            }
            _ => false,
        }
    }
}
