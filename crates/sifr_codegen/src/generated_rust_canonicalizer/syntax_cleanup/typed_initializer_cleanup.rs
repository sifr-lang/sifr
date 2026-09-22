// Initializer deletion needs lexical type/effect proof, not a method spelling.
impl Rewriter<'_> {
    fn fold_proven_initializers(&mut self, block: &mut syn::Block) {
        let outer = self.bindings.clone();
        let mut proven = std::collections::HashSet::new();
        let mut rejected = std::collections::HashSet::new();
        for statement in &block.stmts {
            if let syn::Stmt::Local(local) = statement {
                let key = local.to_token_stream().to_string();
                if self.discardable_initializer(local) {
                    proven.insert(key);
                } else {
                    rejected.insert(key);
                }
                let ty = local.init.as_ref().and_then(|init| self.ty(&init.expr));
                self.bind(&local.pat, ty);
            }
        }
        self.bindings = outer;
        // Identical text in different shadowing positions is eligible only
        // when every occurrence has the same proof.
        proven.retain(|key| !rejected.contains(key));
        super::idiom_cleanup::fold_proven_initializers(
            &mut block.stmts,
            &super::identifier_collection::InitializerReferences {
                standard_vec: !self.scalar_shadowed("vec"),
            },
            |local| proven.contains(&local.to_token_stream().to_string()),
        );
    }

    fn discardable_initializer(&self, local: &syn::Local) -> bool {
        let Some(init) = &local.init else {
            return false;
        };
        if init.diverge.is_some() || !local.attrs.is_empty() {
            return false;
        }
        let expression = init.expr.as_ref();
        crate::discardability::syntax_expression_is_discardable_with_standard_proof(
            expression,
            |expression| self.standard_initializer_value(expression, &local.pat),
        )
    }

    fn standard_initializer_value(&self, expression: &syn::Expr, pattern: &syn::Pat) -> bool {
        let declared = match pattern {
            syn::Pat::Type(typed) => Some(typed.ty.as_ref().clone()),
            _ => self.ty(expression),
        };
        let Some(declared) = declared else {
            return false;
        };
        if self.standard_character_collection(expression, &declared) {
            return true;
        }
        if self.standard_named(&declared, "SifrInt") && self.eager_fallback_is_safe(expression) {
            return true;
        }
        if (self.standard_named(&declared, "String")
            || self.standard_generic(&declared, "Vec").is_some())
            && self.standard_empty_constructor(expression, &declared)
        {
            return true;
        }
        if self.standard_generic(&declared, "Option").is_some()
            && !self.scalar_shadowed("None")
            && !self.bindings.contains_key("None")
            && matches!(expression, syn::Expr::Path(path) if path.path.is_ident("None"))
        {
            return true;
        }
        self.standard_named(&declared, "bool")
            && matches!(expression, syn::Expr::Path(path) if path.path.get_ident().is_some())
            && self
                .ty(expression)
                .is_some_and(|ty| self.standard_named(&ty, "bool"))
    }
}

impl Rewriter<'_> {
    fn standard_character_collection(&self, expression: &syn::Expr, target: &syn::Type) -> bool {
        if !self
            .standard_generic(target, "Vec")
            .is_some_and(|ty| self.standard_named(ty, "char"))
        {
            return false;
        }
        let syn::Expr::MethodCall(collect) = expression else {
            return false;
        };
        if collect.method != "collect" || !collect.args.is_empty() {
            return false;
        }
        let syn::Expr::MethodCall(chars) = collect.receiver.as_ref() else {
            return false;
        };
        chars.method == "chars"
            && chars.args.is_empty()
            && matches!(chars.receiver.as_ref(), syn::Expr::Path(path) if path.path.get_ident().is_some())
            && self.ty(&chars.receiver).is_some_and(|ty| {
                self.standard_named(unreference(&ty), "String")
                    || self.standard_named(unreference(&ty), "str")
            })
    }

    fn record_discardable_assignment(&mut self, statement: &syn::Stmt) {
        let syn::Stmt::Expr(syn::Expr::Assign(assign), Some(_)) = statement else {
            return;
        };
        let syn::Expr::Path(path) = assign.left.as_ref() else {
            return;
        };
        let Some(name) = path.path.get_ident() else {
            return;
        };
        let proof = self
            .bindings
            .get(&name.to_string())
            .and_then(Option::as_ref)
            .is_some_and(|ty| {
                crate::discardability::syntax_expression_is_discardable_with_standard_proof(
                    &assign.right,
                    |value| self.standard_character_collection(value, ty),
                ) && (self.standard_named(ty, "bool")
                    || self
                        .standard_generic(ty, "Vec")
                        .is_some_and(|inner| self.standard_named(inner, "char")))
            });
        self.discardable_assignments
            .entry(statement.to_token_stream().to_string())
            .and_modify(|previous| *previous &= proof)
            .or_insert(proof);
    }

    fn remove_proven_dead_assignments(&self, block: &mut syn::Block) {
        super::dead_assignment_cleanup::remove_dead_generated_assignments(block, &|statement| {
            self.discardable_assignments
                .get(&statement.to_token_stream().to_string())
                == Some(&true)
        });
    }
}
