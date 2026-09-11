//! Resolve receiver ownership and declared fields without erasing references.
//! Unknown imports/types never become standard containers merely by basename.

use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

#[derive(Clone, Default)]
pub(super) enum Value {
    Option,
    Map,
    Sequence,
    Nominal(String),
    Reference(Box<Value>),
    #[default]
    Unknown,
}

impl Value {
    pub(super) fn dereferenced(&self) -> &Self {
        match self {
            Self::Reference(value) => value.dereferenced(),
            value => value,
        }
    }
}

enum Definition {
    Struct(syn::Fields),
    Alias(syn::Type),
    Other,
    Trait,
    Module,
}

#[derive(Default)]
pub(super) struct Types {
    definitions: HashMap<String, Definition>,
    imports: HashMap<String, (String, Vec<String>, bool)>,
    globs: HashMap<String, Vec<(Vec<String>, bool)>>,
    generic_names: HashSet<String>,
    ambiguous_bindings: HashSet<String>,
    external_shadows: HashSet<String>,
    opaque_definitions: bool,
    pub(super) ambiguous_methods: HashSet<String>,
}

pub(super) fn qualify(scope: &str, name: &str) -> String {
    if scope.is_empty() {
        name.to_owned()
    } else {
        format!("{scope}::{name}")
    }
}

const METHODS: &[&str] = &["as_ref", "as_mut", "clone", "get", "get_mut"];

fn standard_value(name: &str, path: &syn::Path) -> Value {
    let Some(last) = path.segments.last() else {
        return Value::Unknown;
    };
    let arguments = match &last.arguments {
        syn::PathArguments::None => 0,
        syn::PathArguments::AngleBracketed(arguments)
            if arguments
                .args
                .iter()
                .all(|a| matches!(a, syn::GenericArgument::Type(_))) =>
        {
            arguments.args.len()
        }
        _ => return Value::Unknown,
    };
    match (name, arguments) {
        ("Option" | "::std::option::Option" | "::core::option::Option", 1) => Value::Option,
        ("::std::collections::HashMap" | "::std::collections::BTreeMap", 2 | 3) => Value::Map,
        ("Vec" | "::std::vec::Vec", 1 | 2) => Value::Sequence,
        ("String" | "str" | "::std::string::String", 0) => Value::Sequence,
        _ => Value::Unknown,
    }
}

impl Types {
    pub(super) fn collect(file: &syn::File) -> Self {
        struct Boundaries<'a>(&'a mut Types);
        impl<'ast> Visit<'ast> for Boundaries<'_> {
            fn visit_type_param(&mut self, parameter: &'ast syn::TypeParam) {
                self.0.generic_names.insert(parameter.ident.to_string());
                visit::visit_type_param(self, parameter);
            }
            fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
                self.0.external_shadows.insert(
                    item.rename
                        .as_ref()
                        .map_or(&item.ident, |(_, name)| name)
                        .to_string(),
                );
            }
            fn visit_item_trait(&mut self, item: &'ast syn::ItemTrait) {
                for member in &item.items {
                    if let syn::TraitItem::Fn(method) = member {
                        self.0
                            .ambiguous_methods
                            .insert(method.sig.ident.to_string());
                    }
                }
                visit::visit_item_trait(self, item);
            }
            fn visit_item_macro(&mut self, _: &'ast syn::ItemMacro) {
                self.0.opaque_definitions = true;
                self.0
                    .ambiguous_methods
                    .extend(METHODS.iter().map(|s| (*s).to_owned()));
            }
        }
        let mut types = Self::default();
        types.items("", &file.items);
        Boundaries(&mut types).visit_file(file);
        let imports_closed = types.imports.values().all(|(scope, path, absolute)| {
            types
                .resolve(scope, path, *absolute, 0)
                .is_some_and(|name| types.closed_import(&name))
        });
        let globs_closed = types.globs.iter().all(|(scope, imports)| {
            imports.iter().all(|(path, absolute)| {
                types
                    .resolve(scope, path, *absolute, 0)
                    .is_some_and(|name| {
                        matches!(types.definitions.get(&name), Some(Definition::Module))
                            || name.starts_with("::std::")
                            || name.starts_with("::core::")
                    })
            })
        });
        if !imports_closed || !globs_closed {
            types
                .ambiguous_methods
                .extend(METHODS.iter().map(|s| (*s).to_owned()));
        }
        types
    }

    fn items(&mut self, scope: &str, items: &[syn::Item]) {
        for item in items {
            let definition = match item {
                syn::Item::Struct(item) => Some((
                    item.ident.to_string(),
                    Definition::Struct(item.fields.clone()),
                )),
                syn::Item::Enum(item) => Some((item.ident.to_string(), Definition::Other)),
                syn::Item::Union(item) => Some((item.ident.to_string(), Definition::Other)),
                syn::Item::Fn(item) => Some((item.sig.ident.to_string(), Definition::Other)),
                syn::Item::Const(item) => Some((item.ident.to_string(), Definition::Other)),
                syn::Item::Static(item) => Some((item.ident.to_string(), Definition::Other)),
                syn::Item::Type(item) => Some((
                    item.ident.to_string(),
                    Definition::Alias((*item.ty).clone()),
                )),
                syn::Item::Trait(item) => Some((item.ident.to_string(), Definition::Trait)),
                syn::Item::Mod(item) => {
                    let child = qualify(scope, &item.ident.to_string());
                    if let Some((_, items)) = &item.content {
                        self.items(&child, items);
                        Some((item.ident.to_string(), Definition::Module))
                    } else {
                        None
                    }
                }
                syn::Item::Use(item) => {
                    self.import(scope, &item.tree, Vec::new(), item.leading_colon.is_some());
                    None
                }
                _ => None,
            };
            if let Some((name, definition)) = definition {
                let name = qualify(scope, &name);
                if self.definitions.insert(name.clone(), definition).is_some() {
                    self.ambiguous_bindings.insert(name);
                }
            }
        }
    }

    fn import(
        &mut self,
        scope: &str,
        tree: &syn::UseTree,
        mut prefix: Vec<String>,
        absolute: bool,
    ) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.import(scope, &path.tree, prefix, absolute);
            }
            syn::UseTree::Name(name) => {
                let binding = if name.ident == "self" {
                    prefix.last().cloned().unwrap_or_default()
                } else {
                    prefix.push(name.ident.to_string());
                    name.ident.to_string()
                };
                let name = qualify(scope, &binding);
                if self
                    .imports
                    .insert(name.clone(), (scope.to_owned(), prefix, absolute))
                    .is_some()
                {
                    self.ambiguous_bindings.insert(name);
                }
            }
            syn::UseTree::Rename(name) => {
                if name.ident != "self" {
                    prefix.push(name.ident.to_string());
                }
                let name = qualify(scope, &name.rename.to_string());
                if self
                    .imports
                    .insert(name.clone(), (scope.to_owned(), prefix, absolute))
                    .is_some()
                {
                    self.ambiguous_bindings.insert(name);
                }
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.import(scope, tree, prefix.clone(), absolute);
                }
            }
            syn::UseTree::Glob(_) => self
                .globs
                .entry(scope.to_owned())
                .or_default()
                .push((prefix, absolute)),
        }
    }

    fn resolve(
        &self,
        scope: &str,
        parts: &[String],
        absolute: bool,
        depth: usize,
    ) -> Option<String> {
        self.resolve_inner(scope, parts, absolute, depth, &mut HashSet::new())
    }

    fn resolve_inner(
        &self,
        scope: &str,
        parts: &[String],
        absolute: bool,
        depth: usize,
        visited: &mut HashSet<(String, Vec<String>, bool)>,
    ) -> Option<String> {
        if depth > 32
            || parts.is_empty()
            || visited.len() >= 256
            || !visited.insert((scope.to_owned(), parts.to_vec(), absolute))
        {
            return None;
        }
        if absolute {
            if self.external_shadows.contains(&parts[0]) {
                return None;
            }
            return Some(format!("::{}", parts.join("::")));
        }
        match parts[0].as_str() {
            "crate" => return self.resolve_inner("", &parts[1..], false, depth + 1, visited),
            "self" => return self.resolve_inner(scope, &parts[1..], false, depth + 1, visited),
            "super" => {
                return self.resolve_inner(
                    scope.rsplit_once("::").map_or("", |p| p.0),
                    &parts[1..],
                    false,
                    depth + 1,
                    visited,
                );
            }
            _ => {}
        }
        let local = qualify(scope, &parts[0]);
        if self.ambiguous_bindings.contains(&local) {
            return None;
        }
        if let Some((from, path, absolute)) = self.imports.get(&local) {
            let mut expanded = path.clone();
            expanded.extend_from_slice(&parts[1..]);
            return self.resolve_inner(from, &expanded, *absolute, depth + 1, visited);
        }
        let full = qualify(scope, &parts.join("::"));
        if self.ambiguous_bindings.contains(&full) {
            return None;
        }
        if self.definitions.contains_key(&full) {
            return Some(full);
        }
        if self.definitions.contains_key(&local) && parts.len() > 1 {
            return self.resolve_inner(&local, &parts[1..], false, depth + 1, visited);
        }
        let mut found = HashSet::new();
        for (path, absolute) in self.globs.get(scope).into_iter().flatten() {
            let mut expanded = path.clone();
            expanded.extend_from_slice(parts);
            if let Some(name) = self.resolve_inner(scope, &expanded, *absolute, depth + 1, visited)
            {
                found.insert(name);
            }
        }
        if found.len() == 1 {
            return found.into_iter().next();
        }
        if found.is_empty()
            && parts.len() > 1
            && matches!(parts[0].as_str(), "std" | "core" | "sifr_runtime")
            && !self.external_shadows.contains(&parts[0])
        {
            return Some(format!("::{}", parts.join("::")));
        }
        None
    }

    fn closed_import(&self, name: &str) -> bool {
        matches!(self.definitions.get(name), Some(Definition::Struct(_) | Definition::Other | Definition::Alias(_) | Definition::Module))
            || name.starts_with("::std::") || name.starts_with("::core::")
            // This is the compiler-owned exact runtime nominal, not a basename
            // heuristic or authorization for arbitrary external extension traits.
            || name == "::sifr_runtime::SifrInt"
    }

    pub(super) fn ty(&self, scope: &str, ty: &syn::Type, owner: Option<&str>) -> Value {
        self.ty_at(scope, ty, owner, 0)
    }

    fn ty_at(&self, scope: &str, ty: &syn::Type, owner: Option<&str>, depth: usize) -> Value {
        if depth > 32 {
            return Value::Unknown;
        }
        match ty {
            syn::Type::Reference(reference) => Value::Reference(Box::new(self.ty_at(
                scope,
                &reference.elem,
                owner,
                depth + 1,
            ))),
            syn::Type::Paren(paren) => self.ty_at(scope, &paren.elem, owner, depth + 1),
            syn::Type::Group(group) => self.ty_at(scope, &group.elem, owner, depth + 1),
            syn::Type::Slice(_) | syn::Type::Array(_) => Value::Sequence,
            syn::Type::Path(path) if path.qself.is_none() => {
                if self.opaque_definitions {
                    return Value::Unknown;
                }
                let parts: Vec<_> = path
                    .path
                    .segments
                    .iter()
                    .map(|s| s.ident.to_string())
                    .collect();
                if parts
                    .first()
                    .is_some_and(|part| self.generic_names.contains(part))
                {
                    return Value::Unknown;
                }
                if path.path.is_ident("Self") {
                    return owner.map_or(Value::Unknown, |s| Value::Nominal(s.to_owned()));
                }
                let resolved = self.resolve(scope, &parts, path.path.leading_colon.is_some(), 0);
                if let Some(name) = resolved {
                    return match self.definitions.get(&name) {
                        Some(Definition::Struct(_)) => Value::Nominal(name),
                        Some(Definition::Alias(ty)) => self.ty_at(
                            name.rsplit_once("::").map_or("", |p| p.0),
                            ty,
                            owner,
                            depth + 1,
                        ),
                        Some(_) => Value::Unknown,
                        None => standard_value(&name, &path.path),
                    };
                }
                // Only actual prelude types have unqualified standard identity.
                // Unknown globs may shadow them and cannot establish a fact.
                if parts.first().is_some_and(|name| {
                    self.imports.contains_key(&qualify(scope, name))
                        || self.definitions.contains_key(&qualify(scope, name))
                }) {
                    return Value::Unknown;
                }
                if self.globs.get(scope).is_some_and(|globs| !globs.is_empty()) {
                    return Value::Unknown;
                }
                match parts.as_slice() {
                    [name] => standard_value(name, &path.path),
                    _ => Value::Unknown,
                }
            }
            _ => Value::Unknown,
        }
    }

    pub(super) fn field(&self, receiver: &Value, member: &syn::Member) -> Value {
        let Value::Nominal(owner) = receiver.dereferenced() else {
            return Value::Unknown;
        };
        let Some(Definition::Struct(fields)) = self.definitions.get(owner) else {
            return Value::Unknown;
        };
        let field = match member {
            syn::Member::Named(name) => fields
                .iter()
                .find(|field| field.ident.as_ref() == Some(name)),
            syn::Member::Unnamed(index) => fields.iter().nth(index.index as usize),
        };
        field.map_or(Value::Unknown, |field| {
            self.ty(
                owner.rsplit_once("::").map_or("", |p| p.0),
                &field.ty,
                Some(owner),
            )
        })
    }
}
