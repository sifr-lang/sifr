// Infer callback input contracts only from consistent, already typed arguments.
// Unsupported coercions and unresolved type variables retain the original API.
impl Rewriter<'_> {
    fn instantiate_call_signature(&self, signature: &syn::Signature, call: &syn::ExprCall) -> syn::Signature {
        struct Substitute<'a>(&'a HashMap<String, syn::Type>);
        impl VisitMut for Substitute<'_> {
            fn visit_type_mut(&mut self, ty: &mut syn::Type) {
                if let syn::Type::Path(path) = ty
                    && path.qself.is_none()
                    && let Some(name) = path.path.get_ident()
                    && let Some(actual) = self.0.get(&name.to_string()) {
                    *ty = actual.clone();
                    return;
                }
                visit_mut::visit_type_mut(self, ty);
            }
        }

        let variables: std::collections::HashSet<_> = signature.generics.type_params()
            .map(|parameter| parameter.ident.to_string()).collect();
        let mut bindings = HashMap::new();
        let mut conflicts = std::collections::HashSet::new();
        for (input, argument) in signature.inputs.iter().zip(&call.args) {
            if let syn::FnArg::Typed(input) = input
                && let Some(actual) = self.ty(argument) {
                self.infer_call_type(&input.ty, &actual, &variables, &mut bindings, &mut conflicts);
            }
        }
        bindings.retain(|name, _| !conflicts.contains(name));
        let mut instantiated = signature.clone();
        Substitute(&bindings).visit_signature_mut(&mut instantiated);
        instantiated
    }

    fn infer_call_type(&self, expected: &syn::Type, actual: &syn::Type,
        variables: &std::collections::HashSet<String>, bindings: &mut HashMap<String, syn::Type>,
        conflicts: &mut std::collections::HashSet<String>) {
        if let syn::Type::Path(path) = expected
            && path.qself.is_none()
            && let Some(name) = path.path.get_ident().map(ToString::to_string)
            && variables.contains(&name) {
            if bindings.get(&name).is_some_and(|prior| !same_type(prior, actual)) {
                conflicts.insert(name);
            } else {
                bindings.insert(name, actual.clone());
            }
            return;
        }
        match (expected, actual) {
            (syn::Type::Reference(expected), syn::Type::Reference(actual))
                if expected.mutability.is_some() == actual.mutability.is_some() => {
                self.infer_call_type(&expected.elem, &actual.elem, variables, bindings, conflicts);
            }
            (syn::Type::Slice(expected), actual) => {
                let element = match actual {
                    syn::Type::Slice(slice) => Some(slice.elem.as_ref()),
                    syn::Type::Array(array) => Some(array.elem.as_ref()),
                    actual => self.standard_generic(actual, "Vec"),
                };
                if let Some(actual) = element {
                    self.infer_call_type(&expected.elem, actual, variables, bindings, conflicts);
                }
            }
            (syn::Type::Tuple(expected), syn::Type::Tuple(actual)) if expected.elems.len() == actual.elems.len() => {
                for (expected, actual) in expected.elems.iter().zip(&actual.elems) {
                    self.infer_call_type(expected, actual, variables, bindings, conflicts);
                }
            }
            _ => {}
        }
    }
}
