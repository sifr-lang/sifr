use std::collections::{HashMap, HashSet};

use syn::visit::{self, Visit};

pub(super) fn remove_unused_generated_support_imports(items: &mut [syn::Item]) {
    let trait_names = generated_support_trait_names(items);
    remove_unused_generated_support_imports_with_traits(items, &trait_names);
}

fn remove_unused_generated_support_imports_with_traits(
    items: &mut [syn::Item],
    trait_methods: &HashMap<String, HashSet<String>>,
) {
    remove_unused_imported_functions(items, trait_methods);
    for item in items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        let Some((_, nested)) = &mut module.content else {
            continue;
        };
        if is_generated_support_module(&module.ident) {
            remove_unused_root_bindings(nested);
        } else {
            remove_unused_generated_support_imports_with_traits(nested, trait_methods);
        }
    }
}

fn generated_support_trait_names(items: &[syn::Item]) -> HashMap<String, HashSet<String>> {
    let mut traits = HashMap::new();
    for item in items {
        if let syn::Item::Mod(module) = item
            && is_generated_support_module(&module.ident)
            && let Some((_, nested)) = &module.content
        {
            for item in nested {
                let syn::Item::Trait(item_trait) = item else {
                    continue;
                };
                traits.insert(
                    item_trait.ident.to_string(),
                    item_trait
                        .items
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
    traits
}

fn remove_unused_root_bindings(items: &mut [syn::Item]) {
    let mut referenced = NonImportIdentifierCollector::default();
    for item in items.iter() {
        if !matches!(item, syn::Item::Use(_)) {
            referenced.visit_item(item);
        }
    }
    for item in items {
        let syn::Item::Use(import) = item else {
            continue;
        };
        let syn::UseTree::Path(root) = &mut import.tree else {
            continue;
        };
        if root.ident != "crate" {
            continue;
        }
        let syn::UseTree::Group(group) = root.tree.as_mut() else {
            continue;
        };
        retain_referenced_names(group, &referenced.names, false);
    }
}

fn remove_unused_imported_functions(
    items: &mut [syn::Item],
    trait_methods: &HashMap<String, HashSet<String>>,
) {
    let mut referenced = NonImportPathCollector {
        inherent_methods: collect_inherent_methods(items),
        ..NonImportPathCollector::default()
    };
    let mut sibling_bindings = HashSet::new();
    for item in items.iter() {
        if !matches!(item, syn::Item::Use(_)) {
            referenced.visit_item(item);
        } else if let syn::Item::Use(import) = item
            && !is_generated_support_import(import)
        {
            collect_use_binding_names(&import.tree, &mut sibling_bindings);
        }
    }
    for item in items {
        let syn::Item::Use(import) = item else {
            continue;
        };
        let syn::UseTree::Path(crate_root) = &mut import.tree else {
            continue;
        };
        let syn::UseTree::Path(support) = crate_root.tree.as_mut() else {
            continue;
        };
        if crate_root.ident != "crate" || !is_generated_support_module(&support.ident) {
            continue;
        }
        let syn::UseTree::Group(group) = support.tree.as_mut() else {
            continue;
        };
        retain_referenced_names_without_duplicates(
            group,
            &referenced,
            &sibling_bindings,
            trait_methods,
        );
    }
}

fn collect_inherent_methods(items: &[syn::Item]) -> HashSet<(String, String)> {
    let mut methods = HashSet::new();
    for item in items {
        match item {
            syn::Item::Impl(impl_) if impl_.trait_.is_none() => {
                let syn::Type::Path(owner) = impl_.self_ty.as_ref() else {
                    continue;
                };
                let Some(owner) = owner.path.segments.last() else {
                    continue;
                };
                methods.extend(impl_.items.iter().filter_map(|item| match item {
                    syn::ImplItem::Fn(method) => {
                        Some((owner.ident.to_string(), method.sig.ident.to_string()))
                    }
                    _ => None,
                }));
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    methods.extend(collect_inherent_methods(nested));
                }
            }
            _ => {}
        }
    }
    methods
}

fn is_generated_support_import(import: &syn::ItemUse) -> bool {
    matches!(&import.tree,
        syn::UseTree::Path(crate_root)
            if crate_root.ident == "crate"
                && matches!(crate_root.tree.as_ref(), syn::UseTree::Path(support)
                    if is_generated_support_module(&support.ident)))
}

fn is_generated_support_module(identifier: &proc_macro2::Ident) -> bool {
    matches!(
        identifier.to_string().as_str(),
        "__sifr_generated_support" | "sifr_generated_generated_support"
    )
}

fn collect_use_binding_names(tree: &syn::UseTree, names: &mut HashSet<String>) {
    match tree {
        syn::UseTree::Name(name) => {
            names.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename) => {
            names.insert(rename.rename.to_string());
        }
        syn::UseTree::Path(path) => collect_use_binding_names(&path.tree, names),
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_use_binding_names(item, names);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

fn retain_referenced_names(
    group: &mut syn::UseGroup,
    referenced: &HashSet<String>,
    retain_possible_traits: bool,
) {
    group.items = group
        .items
        .iter()
        .filter(|tree| {
            let name = match tree {
                syn::UseTree::Name(name) => Some(name.ident.to_string()),
                syn::UseTree::Rename(rename) => Some(rename.rename.to_string()),
                _ => None,
            };
            name.is_none_or(|name| {
                referenced.contains(&name)
                    || (retain_possible_traits
                        && name.chars().next().is_some_and(char::is_uppercase))
            })
        })
        .cloned()
        .collect();
}

fn retain_referenced_names_without_duplicates(
    group: &mut syn::UseGroup,
    referenced: &NonImportPathCollector,
    sibling_bindings: &HashSet<String>,
    trait_methods: &HashMap<String, HashSet<String>>,
) {
    group.items = group
        .items
        .iter()
        .filter(|tree| {
            let name = match tree {
                syn::UseTree::Name(name) => Some(name.ident.to_string()),
                syn::UseTree::Rename(rename) => Some(rename.rename.to_string()),
                _ => None,
            };
            name.is_none_or(|name| {
                !sibling_bindings.contains(&name)
                    && (referenced.names.contains(&name)
                        || trait_methods
                            .get(&name)
                            .is_some_and(|methods| !methods.is_disjoint(&referenced.method_names)))
            })
        })
        .cloned()
        .collect();
}

#[derive(Default)]
struct NonImportPathCollector {
    names: HashSet<String>,
    method_names: HashSet<String>,
    inherent_methods: HashSet<(String, String)>,
    scopes: Vec<HashSet<String>>,
    type_scopes: Vec<HashMap<String, String>>,
}

impl Visit<'_> for NonImportPathCollector {
    fn visit_item_mod(&mut self, _module: &syn::ItemMod) {}

    fn visit_path(&mut self, path: &syn::Path) {
        if path.leading_colon.is_none()
            && let Some(first) = path.segments.first()
            && !self.is_bound(&first.ident.to_string())
        {
            self.names.insert(first.ident.to_string());
        }
        visit::visit_path(self, path);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        let method = call.method.to_string();
        let owner = match call.receiver.as_ref() {
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.binding_type(&name.to_string())),
            _ => None,
        };
        if owner.is_none_or(|owner| {
            !self
                .inherent_methods
                .contains(&(owner.to_string(), method.clone()))
        }) {
            self.method_names.insert(method);
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_item_fn(&mut self, function: &syn::ItemFn) {
        self.visit_signature(&function.sig);
        self.push_function_inputs(&function.sig);
        self.visit_block(&function.block);
        self.scopes.pop();
        self.type_scopes.pop();
    }

    fn visit_impl_item_fn(&mut self, function: &syn::ImplItemFn) {
        self.visit_signature(&function.sig);
        self.push_function_inputs(&function.sig);
        self.visit_block(&function.block);
        self.scopes.pop();
        self.type_scopes.pop();
    }

    fn visit_block(&mut self, block: &syn::Block) {
        self.scopes.push(HashSet::new());
        self.type_scopes.push(HashMap::new());
        for statement in &block.stmts {
            if let syn::Stmt::Local(local) = statement {
                self.visit_pat(&local.pat);
                if let Some(init) = &local.init {
                    self.visit_expr(&init.expr);
                    if let Some((_, diverge)) = &init.diverge {
                        self.visit_expr(diverge);
                    }
                }
                if let Some(scope) = self.scopes.last_mut() {
                    collect_pattern_names(&local.pat, scope);
                }
                if let syn::Pat::Type(typed) = &local.pat
                    && let syn::Pat::Ident(binding) = typed.pat.as_ref()
                    && let Some(owner) = type_name(&typed.ty)
                    && let Some(types) = self.type_scopes.last_mut()
                {
                    types.insert(binding.ident.to_string(), owner);
                }
            } else {
                self.visit_stmt(statement);
            }
        }
        self.scopes.pop();
        self.type_scopes.pop();
    }

    fn visit_expr_closure(&mut self, closure: &syn::ExprClosure) {
        let mut scope = HashSet::new();
        for input in &closure.inputs {
            self.visit_pat(input);
            collect_pattern_names(input, &mut scope);
        }
        self.scopes.push(scope);
        self.type_scopes.push(HashMap::new());
        self.visit_expr(&closure.body);
        self.scopes.pop();
        self.type_scopes.pop();
    }

    fn visit_expr_for_loop(&mut self, for_loop: &syn::ExprForLoop) {
        self.visit_expr(&for_loop.expr);
        self.visit_pat(&for_loop.pat);
        let mut scope = HashSet::new();
        collect_pattern_names(&for_loop.pat, &mut scope);
        self.scopes.push(scope);
        self.type_scopes.push(HashMap::new());
        self.visit_block(&for_loop.body);
        self.scopes.pop();
        self.type_scopes.pop();
    }

    fn visit_arm(&mut self, arm: &syn::Arm) {
        let mut scope = HashSet::new();
        collect_pattern_names(&arm.pat, &mut scope);
        self.scopes.push(scope);
        self.type_scopes.push(HashMap::new());
        self.visit_expr(&arm.body);
        self.scopes.pop();
        self.type_scopes.pop();
    }

    fn visit_item_use(&mut self, _item: &syn::ItemUse) {}

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }
}

impl NonImportPathCollector {
    fn is_bound(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|scope| scope.contains(name))
    }

    fn push_function_inputs(&mut self, signature: &syn::Signature) {
        let mut scope = HashSet::new();
        let mut types = HashMap::new();
        for input in &signature.inputs {
            match input {
                syn::FnArg::Receiver(_) => {
                    scope.insert("self".to_string());
                }
                syn::FnArg::Typed(parameter) => {
                    collect_pattern_names(&parameter.pat, &mut scope);
                    if let syn::Pat::Ident(binding) = parameter.pat.as_ref()
                        && let Some(owner) = type_name(&parameter.ty)
                    {
                        types.insert(binding.ident.to_string(), owner);
                    }
                }
            }
        }
        self.scopes.push(scope);
        self.type_scopes.push(types);
    }

    fn binding_type(&self, name: &str) -> Option<&str> {
        self.type_scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).map(String::as_str))
    }
}

fn type_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        syn::Type::Reference(reference) => type_name(&reference.elem),
        syn::Type::Paren(paren) => type_name(&paren.elem),
        _ => None,
    }
}

fn collect_pattern_names(pattern: &syn::Pat, names: &mut HashSet<String>) {
    match pattern {
        syn::Pat::Ident(binding) => {
            names.insert(binding.ident.to_string());
            if let Some((_, subpattern)) = &binding.subpat {
                collect_pattern_names(subpattern, names);
            }
        }
        syn::Pat::Or(or) => {
            for case in &or.cases {
                collect_pattern_names(case, names);
            }
        }
        syn::Pat::Paren(paren) => collect_pattern_names(&paren.pat, names),
        syn::Pat::Reference(reference) => collect_pattern_names(&reference.pat, names),
        syn::Pat::Slice(slice) => {
            for element in &slice.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::Struct(struct_) => {
            for field in &struct_.fields {
                collect_pattern_names(&field.pat, names);
            }
        }
        syn::Pat::Tuple(tuple) => {
            for element in &tuple.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::TupleStruct(tuple) => {
            for element in &tuple.elems {
                collect_pattern_names(element, names);
            }
        }
        syn::Pat::Type(typed) => collect_pattern_names(&typed.pat, names),
        _ => {}
    }
}

#[derive(Default)]
struct NonImportIdentifierCollector {
    names: HashSet<String>,
}

impl Visit<'_> for NonImportIdentifierCollector {
    fn visit_ident(&mut self, identifier: &proc_macro2::Ident) {
        self.names.insert(identifier.to_string());
    }

    fn visit_item_use(&mut self, _item: &syn::ItemUse) {}
}
