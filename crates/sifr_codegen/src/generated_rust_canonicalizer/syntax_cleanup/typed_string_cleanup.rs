impl Rewriter<'_> {
    fn rewrite_typed_string_clones(&self, expression: &mut syn::Expr) {
        let syn::Expr::MethodCall(outer) = expression else {
            return;
        };
        if !outer.args.is_empty() {
            return;
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
        ) && !self.scalar_shadows.contains("String")
            && self
                .ty(&outer.receiver)
                .is_some_and(|ty| named(&ty, "String"))
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
                (named(base, "String") && !self.scalar_shadows.contains("String"))
                    || (outer.method == "to_string"
                        && named(base, "SifrInt")
                        && !self.scalar_shadows.contains("SifrInt"))
            });
            if known {
                outer.receiver = inner.receiver.clone();
            }
        }
    }
}
