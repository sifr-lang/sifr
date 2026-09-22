// Expression result facts keep arm-local bindings and diverging arms distinct.
impl Rewriter<'_> {
    fn nested_type_scope(&self) -> Self {
        Self {
            iteration_dispatch_closed: self.iteration_dispatch_closed,
            ambiguous_clone_scopes: self.ambiguous_clone_scopes,
            ambiguous_string_pattern_scopes: self.ambiguous_string_pattern_scopes,
            scalar_shadows: self.scalar_shadows,
            functions: self.functions,
            structures: self.structures,
            local_structures: self.local_structures.clone(),
            self_type: self.self_type.clone(),
            scope: self.scope.clone(),
            module_depth: self.module_depth,
            bindings: self.bindings.clone(),
            discardable_assignments: HashMap::new(),
            float_expectations: Default::default(),
        }
    }

    fn match_result_type(&self, expression: &syn::ExprMatch) -> Option<syn::Type> {
        fn returns(expression: &syn::Expr) -> bool {
            match expression {
                syn::Expr::Return(_) => true,
                syn::Expr::Block(block) => matches!(block.block.stmts.last(),
                    Some(syn::Stmt::Expr(tail, _)) if returns(tail)),
                syn::Expr::Paren(paren) => returns(&paren.expr),
                _ => false,
            }
        }
        let input = self.ty(&expression.expr);
        let mut output: Option<syn::Type> = None;
        for arm in &expression.arms {
            if returns(&arm.body) { continue; }
            let mut nested = self.nested_type_scope();
            nested.bind(&arm.pat, input.clone());
            let arm_type = nested.ty(&arm.body)?;
            if let Some(output) = &output {
                if !same_type(output, &arm_type) { return None; }
            } else {
                output = Some(arm_type);
            }
        }
        output
    }
}

fn resolve_signature_self(signature: &mut syn::Signature, owner: &syn::TypePath, scope: &[String]) {
    struct ResolveSelf(syn::Type);
    impl VisitMut for ResolveSelf {
        fn visit_type_mut(&mut self, ty: &mut syn::Type) {
            if matches!(ty, syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("Self")) {
                *ty = self.0.clone();
            } else {
                visit_mut::visit_type_mut(self, ty);
            }
        }
    }

    if owner.path.leading_colon.is_some() || owner.path.segments.len() != 1 { return; }
    let prefix = if scope.is_empty() { String::new() } else { format!("{}::", scope.join("::")) };
    let Ok(owner) = syn::parse_str::<syn::Type>(&format!("{prefix}{}", owner.to_token_stream())) else { return; };
    ResolveSelf(owner).visit_signature_mut(signature);
}
