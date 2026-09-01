use syn::visit::{self, Visit};

pub(super) fn rewrite_discarded_result_matches(statements: &mut [syn::Stmt]) {
    for statement in statements {
        let syn::Stmt::Expr(syn::Expr::Match(match_), Some(_)) = statement else {
            continue;
        };
        if match_.arms.len() != 2 {
            continue;
        }
        let success = &match_.arms[0];
        let failure = &match_.arms[1];
        let Some(binding) = single_variant_binding(&success.pat, "Ok") else {
            continue;
        };
        let Some(failure_pattern) = single_variant_pattern(&failure.pat, "Err").cloned() else {
            continue;
        };
        if !matches!(success.body.as_ref(), syn::Expr::Path(path)
            if path.qself.is_none() && path.path.is_ident(&binding))
            || !crosses_closure_control_flow_boundary(&failure.body)
        {
            continue;
        }
        let matched = match_.expr.clone();
        let fallback = super::super::expression_into_block(failure.body.clone());
        if matches!(failure_pattern, syn::Pat::Wild(_)) {
            *statement = syn::parse_quote! {
                if (#matched).is_err() #fallback
            };
        } else {
            *statement = syn::parse_quote! {
                if let Err(#failure_pattern) = #matched #fallback
            };
        }
    }
}

pub(super) fn rewrite_result_identity_match(expression: &mut syn::Expr) {
    let syn::Expr::Match(match_) = expression else {
        return;
    };
    if match_.arms.len() != 2
        || match_
            .arms
            .iter()
            .any(|arm| matches!(arm.pat, syn::Pat::Guard(_)))
    {
        return;
    }
    let success = &match_.arms[0];
    let failure = &match_.arms[1];
    let Some(binding) = single_variant_binding(&success.pat, "Ok") else {
        return;
    };
    let Some(failure_pattern) = single_variant_pattern(&failure.pat, "Err") else {
        return;
    };
    if !matches!(success.body.as_ref(), syn::Expr::Path(path)
        if path.qself.is_none() && path.path.is_ident(&binding))
    {
        return;
    }
    if crosses_closure_control_flow_boundary(&failure.body) {
        if matches!(match_.expr.as_ref(), syn::Expr::Path(_))
            && !match_.attrs.iter().any(|attribute| {
                attribute.path().is_ident("expect")
                    && quote::ToTokens::to_token_stream(&attribute.meta)
                        .to_string()
                        .contains("single_match_else")
            })
        {
            match_.attrs.push(syn::parse_quote!(
                #[expect(
                    clippy::single_match_else,
                    reason = "the fallback returns through the enclosing Sifr control-flow carrier"
                )]
            ));
        }
        return;
    }
    let matched = match_.expr.clone();
    let fallback = failure.body.clone();
    *expression = syn::parse_quote! {
        (#matched).unwrap_or_else(|#failure_pattern| #fallback)
    };
}

fn crosses_closure_control_flow_boundary(expression: &syn::Expr) -> bool {
    #[derive(Default)]
    struct BoundaryUse {
        found: bool,
    }

    impl<'ast> Visit<'ast> for BoundaryUse {
        fn visit_expr(&mut self, expression: &'ast syn::Expr) {
            if matches!(
                expression,
                syn::Expr::Return(_)
                    | syn::Expr::Break(_)
                    | syn::Expr::Continue(_)
                    | syn::Expr::Try(_)
                    | syn::Expr::Await(_)
                    | syn::Expr::Yield(_)
            ) {
                self.found = true;
                return;
            }
            visit::visit_expr(self, expression);
        }
    }

    let mut use_ = BoundaryUse::default();
    use_.visit_expr(expression);
    use_.found
}

fn single_variant_binding(pattern: &syn::Pat, variant: &str) -> Option<syn::Ident> {
    let syn::Pat::Ident(binding) = single_variant_pattern(pattern, variant)? else {
        return None;
    };
    Some(binding.ident.clone())
}

fn single_variant_pattern<'a>(pattern: &'a syn::Pat, variant: &str) -> Option<&'a syn::Pat> {
    let syn::Pat::TupleStruct(tuple) = pattern else {
        return None;
    };
    if tuple
        .path
        .segments
        .last()
        .is_none_or(|segment| segment.ident != variant)
    {
        return None;
    }
    let mut elements = tuple.elems.iter();
    let element = elements.next()?;
    elements.next().is_none().then_some(element)
}
