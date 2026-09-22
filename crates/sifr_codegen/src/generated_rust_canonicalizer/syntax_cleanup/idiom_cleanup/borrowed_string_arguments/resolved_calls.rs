// Only resolved concrete &str contracts authorize changing an owned parameter.
struct StringCallContext<'a> {
    inputs: &'a HashMap<String, Vec<bool>>,
    modules: &'a [String],
    scope: Vec<String>,
    shadowed: HashSet<String>,
    local_import: bool,
}

impl<'a> StringCallContext<'a> {
    fn new(inputs: &'a HashMap<String, Vec<bool>>, modules: &'a [String],
           signature: &syn::Signature, block: &syn::Block) -> Self {
        #[derive(Default)]
        struct Bindings { names: HashSet<String>, local_import: bool }
        impl Visit<'_> for Bindings {
            fn visit_pat_ident(&mut self, pattern: &syn::PatIdent) {
                self.names.insert(pattern.ident.to_string());
                visit::visit_pat_ident(self, pattern);
            }
            fn visit_item(&mut self, item: &syn::Item) {
                if matches!(item, syn::Item::Use(_)) { self.local_import = true; }
            }
        }
        let mut bindings = Bindings::default();
        for input in &signature.inputs {
            if let syn::FnArg::Typed(input) = input { bindings.visit_pat(&input.pat); }
        }
        bindings.visit_block(block);
        let mut scope = modules.to_vec();
        scope.push(signature.ident.to_string());
        Self { inputs, modules, scope, shadowed: bindings.names, local_import: bindings.local_import }
    }

    fn inputs(&self, call: &syn::ExprCall) -> Option<&Vec<bool>> {
        if self.local_import { return None; }
        let syn::Expr::Path(path) = call.func.as_ref() else { return None };
        if path.qself.is_some() || path.path.leading_colon.is_some() { return None; }
        let parts: Vec<_> = path.path.segments.iter().map(|part| part.ident.to_string()).collect();
        if parts.first().is_none_or(|first| self.shadowed.contains(first)) { return None; }
        if parts.len() > 1 {
            let qualified = super::super::scoped_imports::qualified_path(self.modules, &parts)?;
            return self.inputs.get(&qualified.join("::"));
        }
        for depth in (self.modules.len()..=self.scope.len()).rev() {
            let mut key = self.scope[..depth].to_vec();
            key.extend(parts.iter().cloned());
            if let Some(inputs) = self.inputs.get(&key.join("::")) { return Some(inputs); }
        }
        None
    }
}
