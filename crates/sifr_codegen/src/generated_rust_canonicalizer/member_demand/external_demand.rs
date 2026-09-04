use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

pub(super) fn enum_aliases_for_scope(
    items: &[syn::Item],
    module_name: &str,
    variants: &HashMap<String, HashSet<String>>,
) -> HashMap<String, String> {
    let owners = variants.keys().cloned().collect();
    aliases_for_scope(items, module_name, &owners)
}

pub(super) fn variant_demand(
    items: &[syn::Item],
    module_index: usize,
    module_name: &str,
    variants: &HashMap<String, HashSet<String>>,
    aliases: &HashMap<String, String>,
) -> HashSet<(String, String)> {
    if variants.is_empty() {
        return HashSet::new();
    }
    let mut collector = ExternalVariantDemandCollector {
        module_name,
        variants,
        aliases,
        demanded: HashSet::new(),
    };
    let owners = variants.keys().cloned().collect::<HashSet<_>>();
    for (index, item) in items.iter().enumerate() {
        if index == module_index || matches!(item, syn::Item::Use(_)) {
            continue;
        }
        if let syn::Item::Mod(module) = item {
            let mut sibling_aliases = module
                .content
                .as_ref()
                .map(|(_, nested)| aliases_for_nested_scope(nested, module_name, &owners))
                .unwrap_or_default();
            if let Some((_, nested)) = &module.content {
                sibling_aliases.extend(parent_aliases_for_nested_scope(nested, aliases));
            }
            let mut sibling_collector = ExternalVariantDemandCollector {
                module_name,
                variants,
                aliases: &sibling_aliases,
                demanded: HashSet::new(),
            };
            sibling_collector.visit_item(item);
            collector.demanded.extend(sibling_collector.demanded);
        } else {
            collector.visit_item(item);
        }
    }
    collector.demanded
}

pub(super) fn trait_method_demand(
    items: &[syn::Item],
    module_index: usize,
    module_name: &str,
    trait_methods: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    if trait_methods.is_empty() {
        return HashSet::new();
    }
    let owners = trait_methods.keys().cloned().collect::<HashSet<_>>();
    let root_aliases = aliases_for_scope(items, module_name, &owners);
    let mut demanded = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        if index == module_index || matches!(item, syn::Item::Use(_)) {
            continue;
        }
        let aliases = if let syn::Item::Mod(module) = item {
            let mut aliases = module
                .content
                .as_ref()
                .map(|(_, nested)| aliases_for_nested_scope(nested, module_name, &owners))
                .unwrap_or_default();
            if let Some((_, nested)) = &module.content {
                aliases.extend(parent_aliases_for_nested_scope(nested, &root_aliases));
            }
            aliases
        } else {
            root_aliases.clone()
        };
        let mut collector = ExternalTraitMethodDemandCollector {
            module_name,
            trait_methods,
            aliases: &aliases,
            demanded: HashSet::new(),
        };
        collector.visit_item(item);
        demanded.extend(collector.demanded);
    }
    demanded
}

fn parent_aliases_for_nested_scope(
    items: &[syn::Item],
    parent_aliases: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for item in items {
        match item {
            syn::Item::Use(use_) => {
                collect_parent_aliases(&use_.tree, false, parent_aliases, &mut aliases);
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    aliases.extend(parent_aliases_for_nested_scope(nested, parent_aliases));
                }
            }
            _ => {}
        }
    }
    aliases
}

fn collect_parent_aliases(
    tree: &syn::UseTree,
    inside_parent: bool,
    parent_aliases: &HashMap<String, String>,
    aliases: &mut HashMap<String, String>,
) {
    match tree {
        syn::UseTree::Path(path) => collect_parent_aliases(
            &path.tree,
            inside_parent || path.ident == "crate" || path.ident == "super",
            parent_aliases,
            aliases,
        ),
        syn::UseTree::Name(name) if inside_parent => {
            let name = name.ident.to_string();
            if let Some(owner) = parent_aliases.get(&name) {
                aliases.insert(name, owner.clone());
            }
        }
        syn::UseTree::Rename(rename) if inside_parent => {
            if let Some(owner) = parent_aliases.get(&rename.ident.to_string()) {
                aliases.insert(rename.rename.to_string(), owner.clone());
            }
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_parent_aliases(item, inside_parent, parent_aliases, aliases);
            }
        }
        syn::UseTree::Glob(_) if inside_parent => {
            aliases.extend(parent_aliases.clone());
        }
        syn::UseTree::Name(_) | syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
    }
}

fn aliases_for_scope(
    items: &[syn::Item],
    module_name: &str,
    owners: &HashSet<String>,
) -> HashMap<String, String> {
    let mut aliases = HashMap::new();
    for item in items {
        if let syn::Item::Use(use_) = item {
            collect_module_aliases(&use_.tree, module_name, false, owners, &mut aliases);
        }
    }
    aliases
}

fn aliases_for_nested_scope(
    items: &[syn::Item],
    module_name: &str,
    owners: &HashSet<String>,
) -> HashMap<String, String> {
    let mut aliases = aliases_for_scope(items, module_name, owners);
    for item in items {
        if let syn::Item::Mod(module) = item
            && let Some((_, nested)) = &module.content
        {
            aliases.extend(aliases_for_nested_scope(nested, module_name, owners));
        }
    }
    aliases
}

fn collect_module_aliases(
    tree: &syn::UseTree,
    module_name: &str,
    inside_module: bool,
    owners: &HashSet<String>,
    aliases: &mut HashMap<String, String>,
) {
    match tree {
        syn::UseTree::Path(path) => collect_module_aliases(
            &path.tree,
            module_name,
            inside_module || path.ident == module_name,
            owners,
            aliases,
        ),
        syn::UseTree::Name(name) if inside_module && owners.contains(&name.ident.to_string()) => {
            aliases.insert(name.ident.to_string(), name.ident.to_string());
        }
        syn::UseTree::Rename(rename)
            if inside_module && owners.contains(&rename.ident.to_string()) =>
        {
            aliases.insert(rename.rename.to_string(), rename.ident.to_string());
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_module_aliases(item, module_name, inside_module, owners, aliases);
            }
        }
        syn::UseTree::Glob(_) if inside_module => {
            aliases.extend(owners.iter().map(|owner| (owner.clone(), owner.clone())));
        }
        syn::UseTree::Name(_) | syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
    }
}

struct ExternalVariantDemandCollector<'definitions> {
    module_name: &'definitions str,
    variants: &'definitions HashMap<String, HashSet<String>>,
    aliases: &'definitions HashMap<String, String>,
    demanded: HashSet<(String, String)>,
}

impl ExternalVariantDemandCollector<'_> {
    fn collect_path(&mut self, path: &syn::Path) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        for triple in segments.windows(3) {
            if triple[0] == self.module_name
                && self
                    .variants
                    .get(&triple[1])
                    .is_some_and(|variants| variants.contains(&triple[2]))
            {
                self.demanded.insert((triple[1].clone(), triple[2].clone()));
            }
        }
        for pair in segments.windows(2) {
            let Some(owner) = self.aliases.get(&pair[0]) else {
                continue;
            };
            if self
                .variants
                .get(owner)
                .is_some_and(|variants| variants.contains(&pair[1]))
            {
                self.demanded.insert((owner.clone(), pair[1].clone()));
            }
        }
    }
}

impl<'ast> Visit<'ast> for ExternalVariantDemandCollector<'_> {
    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        self.collect_path(&expression.path);
        visit::visit_expr_path(self, expression);
    }

    fn visit_pat(&mut self, _pattern: &'ast syn::Pat) {}

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }
}

struct ExternalTraitMethodDemandCollector<'definitions> {
    module_name: &'definitions str,
    trait_methods: &'definitions HashMap<String, HashSet<String>>,
    aliases: &'definitions HashMap<String, String>,
    demanded: HashSet<String>,
}

impl ExternalTraitMethodDemandCollector<'_> {
    fn trait_owner(&self, path: &syn::Path) -> Option<String> {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        if let Some(owner) = segments.last().and_then(|name| self.aliases.get(name)) {
            return Some(owner.clone());
        }
        segments.windows(2).find_map(|pair| {
            (pair[0] == self.module_name && self.trait_methods.contains_key(&pair[1]))
                .then(|| pair[1].clone())
        })
    }
}

impl<'ast> Visit<'ast> for ExternalTraitMethodDemandCollector<'_> {
    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        if let Some((trait_path, _)) = &implementation.trait_
            && let Some(owner) = self.trait_owner(trait_path)
            && let Some(methods) = self.trait_methods.get(&owner)
        {
            self.demanded
                .extend(implementation.items.iter().filter_map(|item| {
                    let syn::ImplItem::Fn(method) = item else {
                        return None;
                    };
                    let name = method.sig.ident.to_string();
                    methods.contains(&name).then_some(name)
                }));
        }
        visit::visit_item_impl(self, implementation);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        let method = call.method.to_string();
        if self.aliases.values().any(|owner| {
            self.trait_methods
                .get(owner)
                .is_some_and(|methods| methods.contains(&method))
        }) {
            self.demanded.insert(method);
        }
        visit::visit_expr_method_call(self, call);
    }
}
