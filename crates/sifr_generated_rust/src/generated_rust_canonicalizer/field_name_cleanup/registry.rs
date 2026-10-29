use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Default)]
pub(super) enum Ty {
    Named(String, Vec<Ty>),
    Tuple(Vec<Ty>),
    #[default]
    Unknown,
}

impl Ty {
    pub(super) fn owner(&self) -> Option<&str> {
        match self {
            Self::Named(name, _) => Some(name),
            _ => None,
        }
    }
    pub(super) fn inner(&self) -> Self {
        self.argument(0)
    }
    pub(super) fn argument(&self, index: usize) -> Self {
        match self {
            Self::Named(_, args) => args.get(index).cloned().unwrap_or_default(),
            _ => Self::Unknown,
        }
    }
}

#[derive(Default)]
pub(super) struct Registry {
    pub(super) fields: BTreeMap<String, BTreeMap<String, String>>,
    field_types: BTreeMap<(String, String), (String, syn::Type)>,
    functions: BTreeMap<String, (String, syn::ReturnType)>,
    aliases: BTreeMap<String, (String, Vec<String>)>,
    globs: BTreeMap<String, Vec<Vec<String>>>,
    definitions: BTreeSet<String>,
    type_aliases: BTreeMap<String, (String, syn::Type)>,
    parameters: BTreeMap<String, Vec<String>>,
}

pub(super) fn qualify(module: &str, name: &str) -> String {
    if module.is_empty() {
        name.to_string()
    } else {
        format!("{module}::{name}")
    }
}

impl Registry {
    pub(super) fn collect(files: &BTreeMap<String, syn::File>) -> Self {
        let mut registry = Self::default();
        for module in files.keys() {
            let mut path = String::new();
            for component in module.split("::").filter(|part| !part.is_empty()) {
                path = qualify(&path, component);
                registry.definitions.insert(path.clone());
            }
        }
        for (module, file) in files {
            registry.collect_items(module, &file.items);
        }
        for (module, file) in files {
            registry.collect_impls(module, &file.items);
        }
        registry
    }

    fn collect_items(&mut self, module: &str, items: &[syn::Item]) {
        for item in items {
            match item {
                syn::Item::Struct(item) => {
                    self.add_fields(module, &item.ident.to_string(), &item.fields);
                    self.parameters.insert(
                        qualify(module, &item.ident.to_string()),
                        item.generics
                            .type_params()
                            .map(|param| param.ident.to_string())
                            .collect(),
                    );
                }
                syn::Item::Enum(item) => {
                    let owner = qualify(module, &item.ident.to_string());
                    self.definitions.insert(owner.clone());
                    for variant in &item.variants {
                        self.add_fields(&owner, &variant.ident.to_string(), &variant.fields);
                        let variant_owner = qualify(&owner, &variant.ident.to_string());
                        for ((field_owner, _), (scope, _)) in &mut self.field_types {
                            if field_owner == &variant_owner {
                                *scope = module.to_string();
                            }
                        }
                    }
                }
                syn::Item::Mod(item) => {
                    let child = qualify(module, &item.ident.to_string());
                    self.definitions.insert(child.clone());
                    if let Some((_, items)) = &item.content {
                        self.collect_items(&child, items);
                    }
                }
                syn::Item::Use(item) => self.collect_use(module, &item.tree, Vec::new()),
                syn::Item::Type(item) => {
                    let name = qualify(module, &item.ident.to_string());
                    self.definitions.insert(name.clone());
                    self.type_aliases
                        .insert(name, (module.to_string(), (*item.ty).clone()));
                }
                syn::Item::Fn(item) => {
                    let name = qualify(module, &item.sig.ident.to_string());
                    self.definitions.insert(name.clone());
                    self.functions
                        .insert(name, (module.to_string(), item.sig.output.clone()));
                }
                _ => {}
            }
        }
    }

    fn collect_impls(&mut self, module: &str, items: &[syn::Item]) {
        for item in items {
            if let syn::Item::Mod(item) = item
                && let Some((_, items)) = &item.content
            {
                self.collect_impls(&qualify(module, &item.ident.to_string()), items);
            }
            if let syn::Item::Impl(item) = item {
                let owner = self.ty(module, &item.self_ty, None);
                if let Some(owner) = owner.owner() {
                    for member in &item.items {
                        if let syn::ImplItem::Fn(method) = member {
                            self.functions.insert(
                                qualify(owner, &method.sig.ident.to_string()),
                                (module.to_string(), method.sig.output.clone()),
                            );
                        }
                    }
                }
            }
        }
    }

    fn add_fields(&mut self, module: &str, name: &str, fields: &syn::Fields) {
        let owner = qualify(module, name);
        self.definitions.insert(owner.clone());
        self.fields
            .insert(owner.clone(), super::field_names(fields));
        for (index, field) in fields.iter().enumerate() {
            let name = field
                .ident
                .as_ref()
                .map_or_else(|| index.to_string(), ToString::to_string);
            self.field_types.insert(
                (owner.clone(), name),
                (module.to_string(), field.ty.clone()),
            );
        }
    }

    fn collect_use(&mut self, module: &str, tree: &syn::UseTree, mut prefix: Vec<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.collect_use(module, &path.tree, prefix);
            }
            syn::UseTree::Name(name) => {
                let binding = if name.ident == "self" {
                    prefix.last().cloned().unwrap_or_default()
                } else {
                    prefix.push(name.ident.to_string());
                    name.ident.to_string()
                };
                self.aliases
                    .insert(qualify(module, &binding), (module.to_string(), prefix));
            }
            syn::UseTree::Rename(name) => {
                if name.ident != "self" {
                    prefix.push(name.ident.to_string());
                }
                self.aliases.insert(
                    qualify(module, &name.rename.to_string()),
                    (module.to_string(), prefix),
                );
            }
            syn::UseTree::Glob(_) => {
                self.globs
                    .entry(module.to_string())
                    .or_default()
                    .push(prefix);
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.collect_use(module, tree, prefix.clone());
                }
            }
        }
    }

    fn resolve(
        &self,
        module: &str,
        parts: &[String],
        depth: usize,
        visited: &mut BTreeSet<(String, Vec<String>)>,
    ) -> Option<String> {
        if depth > 32 || parts.is_empty() || !visited.insert((module.to_string(), parts.to_vec())) {
            return None;
        }
        let (scope, tail) = match parts[0].as_str() {
            "crate" => ("", &parts[1..]),
            "self" => (module, &parts[1..]),
            "super" => {
                let parent = module.rsplit_once("::").map_or("", |(parent, _)| parent);
                return self.resolve(parent, &parts[1..], depth + 1, visited);
            }
            _ => (module, parts),
        };
        if tail.is_empty() {
            return Some(scope.to_string());
        }
        let local = qualify(scope, &tail[0]);
        if let Some((alias_scope, path)) = self.aliases.get(&local) {
            let mut expanded = path.clone();
            expanded.extend_from_slice(&tail[1..]);
            return self.resolve(alias_scope, &expanded, depth + 1, visited);
        }
        let candidate = qualify(scope, &tail.join("::"));
        if self.definitions.contains(&candidate) || self.functions.contains_key(&candidate) {
            return Some(candidate);
        }
        // Resolve re-exports at intermediate module boundaries too.
        if tail.len() > 1 && self.definitions.contains(&local) {
            return self.resolve(&local, &tail[1..], depth + 1, visited);
        }
        let mut matches = BTreeSet::new();
        if let Some(globs) = self.globs.get(scope) {
            for glob in globs {
                let mut expanded = glob.clone();
                expanded.extend_from_slice(tail);
                if let Some(found) = self.resolve(scope, &expanded, depth + 1, visited) {
                    matches.insert(found);
                }
            }
        }
        if matches.len() == 1 {
            return matches.into_iter().next();
        }
        None
    }

    pub(super) fn path(&self, module: &str, path: &syn::Path, owner: Option<&str>) -> String {
        let mut parts = path
            .segments
            .iter()
            .map(|part| part.ident.to_string())
            .collect::<Vec<_>>();
        if parts.first().is_some_and(|part| part == "Self") {
            if let Some(owner) = owner {
                parts[0] = owner.to_string();
                return parts.join("::");
            }
        }
        // Leading :: names external crates; never bind them to a local nominal.
        if path.leading_colon.is_some() {
            return format!("::{}", parts.join("::"));
        }
        self.resolve(module, &parts, 0, &mut BTreeSet::new())
            .unwrap_or_else(|| parts.join("::"))
    }

    pub(super) fn ty(&self, module: &str, ty: &syn::Type, owner: Option<&str>) -> Ty {
        self.ty_at_depth(module, ty, owner, 0)
    }

    fn ty_at_depth(&self, module: &str, ty: &syn::Type, owner: Option<&str>, depth: usize) -> Ty {
        if depth > 32 {
            return Ty::Unknown;
        }
        match ty {
            syn::Type::Reference(ty) => self.ty_at_depth(module, &ty.elem, owner, depth + 1),
            syn::Type::Paren(ty) => self.ty_at_depth(module, &ty.elem, owner, depth + 1),
            syn::Type::Group(ty) => self.ty_at_depth(module, &ty.elem, owner, depth + 1),
            syn::Type::Slice(ty) => Ty::Named(
                "Slice".to_string(),
                vec![self.ty_at_depth(module, &ty.elem, owner, depth + 1)],
            ),
            syn::Type::Array(ty) => Ty::Named(
                "Array".to_string(),
                vec![self.ty_at_depth(module, &ty.elem, owner, depth + 1)],
            ),
            syn::Type::Tuple(ty) => Ty::Tuple(
                ty.elems
                    .iter()
                    .map(|ty| self.ty_at_depth(module, ty, owner, depth + 1))
                    .collect(),
            ),
            syn::Type::Path(ty) => {
                let name = self.path(module, &ty.path, owner);
                if let Some((scope, ty)) = self.type_aliases.get(&name) {
                    return self.ty_at_depth(scope, ty, owner, depth + 1);
                }
                let args = ty
                    .path
                    .segments
                    .last()
                    .and_then(|segment| match &segment.arguments {
                        syn::PathArguments::AngleBracketed(args) => Some(
                            args.args
                                .iter()
                                .filter_map(|arg| {
                                    if let syn::GenericArgument::Type(ty) = arg {
                                        Some(self.ty_at_depth(module, ty, owner, depth + 1))
                                    } else {
                                        None
                                    }
                                })
                                .collect(),
                        ),
                        _ => None,
                    })
                    .unwrap_or_default();
                Ty::Named(name, args)
            }
            _ => Ty::Unknown,
        }
    }

    pub(super) fn field_type(&self, owner: &Ty, member: &syn::Member) -> Ty {
        let name = match member {
            syn::Member::Named(name) => name.to_string(),
            syn::Member::Unnamed(index) => {
                if let Ty::Tuple(types) = owner {
                    return types.get(index.index as usize).cloned().unwrap_or_default();
                }
                index.index.to_string()
            }
        };
        let Ty::Named(owner, arguments) = owner else {
            return Ty::Unknown;
        };
        self.field_types
            .get(&(owner.clone(), name))
            .map_or(Ty::Unknown, |(module, ty)| {
                let substitutions = self
                    .parameters
                    .get(owner)
                    .into_iter()
                    .flatten()
                    .cloned()
                    .zip(arguments.iter().cloned())
                    .collect();
                self.instantiated_type(module, ty, owner, &substitutions)
            })
    }

    fn instantiated_type(
        &self,
        module: &str,
        ty: &syn::Type,
        owner: &str,
        substitutions: &BTreeMap<String, Ty>,
    ) -> Ty {
        match ty {
            syn::Type::Reference(ty) => {
                self.instantiated_type(module, &ty.elem, owner, substitutions)
            }
            syn::Type::Path(path) => {
                if let Some(name) = path.path.get_ident()
                    && let Some(ty) = substitutions.get(&name.to_string())
                {
                    return ty.clone();
                }
                let mut result = self.ty(module, ty, Some(owner));
                if let Ty::Named(_, arguments) = &mut result
                    && let Some(segment) = path.path.segments.last()
                    && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
                {
                    *arguments = args
                        .args
                        .iter()
                        .filter_map(|arg| {
                            if let syn::GenericArgument::Type(ty) = arg {
                                Some(self.instantiated_type(module, ty, owner, substitutions))
                            } else {
                                None
                            }
                        })
                        .collect();
                }
                result
            }
            _ => self.ty(module, ty, Some(owner)),
        }
    }

    pub(super) fn return_type(&self, function: &str, owner: Option<&str>) -> Ty {
        self.functions
            .get(function)
            .map_or(Ty::Unknown, |(module, result)| match result {
                syn::ReturnType::Type(_, ty) => self.ty(module, ty, owner),
                syn::ReturnType::Default => Ty::Unknown,
            })
    }
}
