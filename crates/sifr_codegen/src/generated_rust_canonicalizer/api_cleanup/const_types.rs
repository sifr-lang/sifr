//! Resolve trivial standard containers without treating shadowing nominals as std.

use std::collections::HashSet;
use syn::visit::{self, Visit};

#[derive(Clone, Default)]
pub(super) struct DropTypes {
    shadowed: HashSet<String>,
}

impl DropTypes {
    fn shadow(&mut self, name: &syn::Ident) {
        if matches!(name.to_string().as_str(), "Option" | "Result") {
            self.shadowed.insert(name.to_string());
        }
    }
    pub(super) fn for_items(items: &[syn::Item]) -> Self {
        let mut types = Self::default();
        for item in items {
            types.visit_item(item);
        }
        types
    }

    pub(super) fn is_trivial(&self, ty: &syn::Type) -> bool {
        match ty {
            syn::Type::Array(array) => self.is_trivial(&array.elem),
            syn::Type::Group(group) => self.is_trivial(&group.elem),
            syn::Type::Paren(paren) => self.is_trivial(&paren.elem),
            syn::Type::Tuple(tuple) => tuple.elems.iter().all(|ty| self.is_trivial(ty)),
            syn::Type::Path(path) if path.qself.is_none() => {
                let names: Vec<_> = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                let Some(last) = path.path.segments.last() else {
                    return false;
                };
                let arity = match last.ident.to_string().as_str() {
                    "Option" => 1,
                    "Result" => 2,
                    _ => return super::super::member_demand::type_has_trivial_drop(ty),
                };
                let standard = match names.as_slice() {
                    [name] => path.path.leading_colon.is_none() && !self.shadowed.contains(name),
                    [root, module, name] => {
                        path.path.leading_colon.is_some()
                            && matches!(root.as_str(), "std" | "core")
                            && matches!(
                                (module.as_str(), name.as_str()),
                                ("option", "Option") | ("result", "Result")
                            )
                    }
                    _ => false,
                };
                if !standard {
                    return false;
                }
                let syn::PathArguments::AngleBracketed(arguments) = &last.arguments else {
                    return false;
                };
                arguments.args.len() == arity && arguments.args.iter().all(|argument| {
                    matches!(argument, syn::GenericArgument::Type(ty) if self.is_trivial(ty))
                })
            }
            _ => super::super::member_demand::type_has_trivial_drop(ty),
        }
    }
}

// A generated source unit may contain nested scopes and generic type
// bindings. Conservatively veto an unqualified container name anywhere it can
// be shadowed; absolute standard paths remain independently provable. This does
// not change the separate private-field discardability or nominal trait rules.
impl<'ast> Visit<'ast> for DropTypes {
    fn visit_item(&mut self, item: &'ast syn::Item) {
        let name = match item {
            syn::Item::Struct(item) => Some(&item.ident),
            syn::Item::Enum(item) => Some(&item.ident),
            syn::Item::Union(item) => Some(&item.ident),
            syn::Item::Type(item) => Some(&item.ident),
            syn::Item::Trait(item) => Some(&item.ident),
            _ => None,
        };
        if let Some(name) = name {
            self.shadow(name);
        }
        if matches!(item, syn::Item::Macro(_)) {
            self.shadowed
                .extend(["Option".to_string(), "Result".to_string()]);
        }
        visit::visit_item(self, item);
    }

    fn visit_type_param(&mut self, parameter: &'ast syn::TypeParam) {
        self.shadow(&parameter.ident);
        visit::visit_type_param(self, parameter);
    }

    fn visit_use_tree(&mut self, tree: &'ast syn::UseTree) {
        match tree {
            syn::UseTree::Name(name) => self.shadow(&name.ident),
            syn::UseTree::Rename(rename) => self.shadow(&rename.rename),
            syn::UseTree::Glob(_) => {
                self.shadowed
                    .extend(["Option".to_string(), "Result".to_string()]);
            }
            _ => {}
        }
        visit::visit_use_tree(self, tree);
    }
}
