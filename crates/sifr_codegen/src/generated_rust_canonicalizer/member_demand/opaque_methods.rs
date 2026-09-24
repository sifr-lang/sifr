//! Demand for compiler-owned opaque extension methods spans nominal/support modules.
use super::macro_arguments::MacroArguments;
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

pub(super) fn prune(file: &mut syn::File) {
    let mut definitions = Definitions::default();
    definitions.visit_file(file);
    if definitions.methods.is_empty() {
        return;
    }
    let mut demand = Demand {
        definitions: &definitions.methods,
        standard_operator_shadowed: definitions.standard_operator_shadowed,
        methods: HashSet::new(),
    };
    loop {
        let before = demand.methods.len();
        demand.visit_file(file);
        if demand.methods.len() == before {
            break;
        }
    }
    Cleanup {
        definitions: &definitions.methods,
        methods: &demand.methods,
    }
    .visit_file_mut(file);
}

#[derive(Default)]
struct Definitions {
    methods: HashMap<String, HashSet<String>>,
    standard_operator_shadowed: bool,
}

pub(super) fn is_opaque_extension_trait(name: &str) -> bool {
    (name.starts_with("__SifrOpaque") || name.starts_with("SifrGeneratedOpaque"))
        && name.ends_with("Methods")
}

impl<'ast> Visit<'ast> for Definitions {
    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        let name = item.rename.as_ref().map_or(&item.ident, |(_, name)| name);
        self.standard_operator_shadowed |= matches!(name.to_string().as_str(), "std" | "core");
    }
    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        fn shadows(tree: &syn::UseTree) -> bool {
            match tree {
                syn::UseTree::Path(path) => shadows(&path.tree),
                syn::UseTree::Name(name) => {
                    matches!(name.ident.to_string().as_str(), "std" | "core")
                }
                syn::UseTree::Rename(name) => {
                    matches!(name.rename.to_string().as_str(), "std" | "core")
                }
                syn::UseTree::Group(group) => group.items.iter().any(shadows),
                syn::UseTree::Glob(_) => true,
            }
        }
        self.standard_operator_shadowed |= shadows(&item.tree);
    }
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        let name = item.ident.to_string();
        if is_opaque_extension_trait(&name) {
            self.methods.insert(
                name,
                item.items
                    .iter()
                    .filter_map(|item| match item {
                        syn::TraitItem::Fn(method) => Some(method.sig.ident.to_string()),
                        _ => None,
                    })
                    .collect(),
            );
        }
    }
}

struct Demand<'a> {
    definitions: &'a HashMap<String, HashSet<String>>,
    standard_operator_shadowed: bool,
    methods: HashSet<String>,
}

impl Demand<'_> {
    fn collect_macro_methods(&mut self, tokens: proc_macro2::TokenStream) {
        for token in tokens {
            match token {
                proc_macro2::TokenTree::Ident(identifier) => {
                    let name = identifier.to_string();
                    if self
                        .definitions
                        .values()
                        .any(|methods| methods.contains(&name))
                    {
                        self.methods.insert(name);
                    }
                }
                proc_macro2::TokenTree::Group(group) => self.collect_macro_methods(group.stream()),
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for Demand<'_> {
    fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
        if self.definitions.contains_key(&item.ident.to_string()) {
            for member in &item.items {
                if let syn::TraitItem::Fn(method) = member
                    && self.methods.contains(&method.sig.ident.to_string())
                {
                    self.visit_trait_item_fn(method);
                }
            }
        } else {
            visit::visit_item_trait(self, item);
        }
    }

    fn visit_item_impl(&mut self, item: &'ast syn::ItemImpl) {
        if implementation_owner(item).is_some_and(|name| self.definitions.contains_key(&name)) {
            for member in &item.items {
                if let syn::ImplItem::Fn(method) = member {
                    if self.methods.contains(&method.sig.ident.to_string()) {
                        self.visit_impl_item_fn(method);
                    }
                } else {
                    self.visit_impl_item(member);
                }
            }
        } else {
            visit::visit_item_impl(self, item);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let method = call.method.to_string();
        if self
            .definitions
            .values()
            .any(|methods| methods.contains(&method))
        {
            self.methods.insert(method);
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if let Some(arguments) = MacroArguments::parse(rust_macro) {
            arguments.visit(self);
        } else {
            // Custom macro syntax still contains opaque method identifiers.
            // Unknown syntax must not make a callable extension method disappear.
            self.collect_macro_methods(rust_macro.tokens.clone());
        }
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        let parts = expression
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        // An exact standard operator trait is not a receiver-type UFCS call
        // into one of the generated opaque extension traits.
        let standard_operator = !self.standard_operator_shadowed
            && expression.path.leading_colon.is_some()
            && parts.len() == 4
            && matches!(parts[0].as_str(), "std" | "core")
            && parts[1] == "ops"
            && matches!(
                parts[2].as_str(),
                "Add"
                    | "Sub"
                    | "Mul"
                    | "Div"
                    | "Rem"
                    | "Neg"
                    | "Not"
                    | "BitAnd"
                    | "BitOr"
                    | "BitXor"
                    | "Shl"
                    | "Shr"
                    | "AddAssign"
                    | "SubAssign"
                    | "MulAssign"
                    | "DivAssign"
                    | "RemAssign"
                    | "BitAndAssign"
                    | "BitOrAssign"
                    | "BitXorAssign"
                    | "ShlAssign"
                    | "ShrAssign"
            );
        // UFCS may name the receiver type rather than the extension trait.
        if !standard_operator
            && (parts.len() >= 2 || expression.qself.is_some())
            && let Some(method) = parts.last()
            && self
                .definitions
                .values()
                .any(|methods| methods.contains(method))
        {
            self.methods.insert(method.clone());
        }
        visit::visit_expr_path(self, expression);
    }
}

struct Cleanup<'a> {
    definitions: &'a HashMap<String, HashSet<String>>,
    methods: &'a HashSet<String>,
}

impl VisitMut for Cleanup<'_> {
    fn visit_item_trait_mut(&mut self, item: &mut syn::ItemTrait) {
        if self.definitions.contains_key(&item.ident.to_string()) {
            item.items.retain(|member| {
                !matches!(member, syn::TraitItem::Fn(method)
                if !self.methods.contains(&method.sig.ident.to_string()))
            });
        }
        visit_mut::visit_item_trait_mut(self, item);
    }

    fn visit_item_impl_mut(&mut self, item: &mut syn::ItemImpl) {
        if implementation_owner(item).is_some_and(|name| self.definitions.contains_key(&name)) {
            item.items.retain(|member| {
                !matches!(member, syn::ImplItem::Fn(method)
                if !self.methods.contains(&method.sig.ident.to_string()))
            });
        }
        visit_mut::visit_item_impl_mut(self, item);
    }
}

fn implementation_owner(item: &syn::ItemImpl) -> Option<String> {
    item.trait_
        .as_ref()?
        .0
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

#[cfg(test)]
mod tests {
    #[test]
    fn shadowed_standard_namespace_keeps_opaque_ufcs_demand() {
        let mut file = syn::parse_file(
            r#"
            extern crate unknown as std;
            trait SifrGeneratedOpaqueExampleMethods { fn sub(&self); }
            fn run() { ::std::ops::Sub::sub(); }
        "#,
        )
        .expect("test syntax");
        super::prune(&mut file);
        assert!(quote::quote!(#file).to_string().contains("fn sub"));
    }
}
