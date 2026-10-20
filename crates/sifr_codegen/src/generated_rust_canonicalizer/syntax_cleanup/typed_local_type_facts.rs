#[derive(Clone, Default)]
struct LocalItemScope {
    types: HashMap<String, Option<syn::ItemStruct>>,
    values: std::collections::HashSet<String>,
}

// Rust block items are hoisted and shadow enclosing type declarations. Keep
// their field layout only within that block; local Clone/Drop behavior is opaque.
impl Rewriter<'_> {
    fn enter_local_type_scope(&mut self, block: &syn::Block) -> LocalItemScope {
        let outer = self.local_items.clone();
        for statement in &block.stmts {
            if let syn::Stmt::Macro(statement) = statement {
                let path = statement.mac.path.to_token_stream().to_string().replace(' ', "");
                let name = path.strip_prefix("::").unwrap_or(&path);
                let standard = name.strip_prefix("std::").unwrap_or(name);
                if !matches!(standard, "assert" | "assert_eq" | "assert_ne" | "debug_assert"
                    | "debug_assert_eq" | "debug_assert_ne" | "println" | "print"
                    | "eprintln" | "eprint" | "panic" | "unreachable" | "todo")
                    || !self.clone_is_unambiguous()
                {
                    self.local_items.types.insert("*".to_string(), None);
                }
            }
            if let syn::Stmt::Item(item) = statement {
                // Value items are hoisted too, but they do not shadow the type
                // namespace. In particular, `const Model: Other` is not Model.
                let value_name = match item {
                    syn::Item::Const(value) => Some(&value.ident),
                    syn::Item::Static(value) => Some(&value.ident),
                    syn::Item::Fn(value) => Some(&value.sig.ident),
                    syn::Item::Struct(value) if !matches!(value.fields, syn::Fields::Named(_)) => Some(&value.ident),
                    _ => None,
                };
                if let Some(name) = value_name {
                    self.local_items.values.insert(name.to_string());
                    // An enclosing binding's type no longer applies. Keep an
                    // existing shadow marker so callable lookup stays opaque.
                    if let Some(binding) = self.bindings.get_mut(&name.to_string()) {
                        *binding = None;
                    }
                }
                let (name, structure) = match item {
                    syn::Item::Struct(value) => (&value.ident, Some(value.clone())),
                    syn::Item::Enum(value) => (&value.ident, None),
                    syn::Item::Union(value) => (&value.ident, None),
                    syn::Item::Type(value) => (&value.ident, None),
                    syn::Item::Mod(value) => (&value.ident, None),
                    syn::Item::Macro(_) | syn::Item::Use(_) => {
                        self.local_items.types.insert("*".to_string(), None);
                        continue;
                    }
                    _ => continue,
                };
                self.local_items.types.insert(name.to_string(), structure);
            }
        }
        outer
    }

    fn local_structure_type(&self, expression: &syn::ExprStruct) -> Option<syn::Type> {
        if expression.qself.is_some() || self.local_items.types.contains_key("*") { return None; }
        let name = expression.path.get_ident()?;
        let structure = self.local_items.types.get(&name.to_string())?.as_ref()?;
        if !structure.generics.params.is_empty() { return None; }
        Some(syn::parse_quote!(#name))
    }
}

// Generic arguments do not change which lexical declaration owns a bare path.
fn local_type_name(path: &syn::Path) -> Option<String> {
    if path.leading_colon.is_some() || path.segments.len() != 1 { return None; }
    Some(path.segments.first()?.ident.to_string())
}

// A unit structure is also a value constructor. Resolve it from declarations;
// an unknown/shadowing binding must never inherit the outer constructor's type.
impl Rewriter<'_> {
    fn path_value_type(&self, expression: &syn::ExprPath) -> Option<syn::Type> {
        if expression.qself.is_some() { return None; }
        if let Some(name) = expression.path.get_ident()
            && let Some(binding) = self.bindings.get(&name.to_string()) {
            return binding.clone();
        }
        if expression.path.get_ident().is_some_and(|name|
            self.local_items.values.contains(&name.to_string())) { return None; }
        let path = if expression.path.is_ident("Self") {
            let syn::Type::Path(owner) = self.self_type.as_ref()? else { return None; };
            if owner.qself.is_some() { return None; }
            &owner.path
        } else {
            &expression.path
        };
        if path.leading_colon.is_some() || self.local_items.types.contains_key("*")
            || local_type_name(path).is_some_and(|name| self.local_items.types.contains_key(&name)) {
            return None;
        }
        let parts = path.segments.iter().map(|part| part.ident.to_string()).collect::<Vec<_>>();
        let key = super::scoped_imports::qualified_path(&self.scope[..self.module_depth], &parts)?;
        let structure = self.structures.get(&key.join("::"))?;
        if !matches!(structure.fields, syn::Fields::Unit) || !structure.generics.params.is_empty() {
            return None;
        }
        Some(syn::parse_quote!(#path))
    }
}
