use proc_macro2::{TokenStream, TokenTree};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use syn::visit::{self, Visit};

use super::impl_self_type_name;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct MethodKey {
    owner: String,
    name: String,
}

pub(super) fn demanded_inherent_method_names(file: &syn::File) -> BTreeSet<MethodKey> {
    let metadata = TypeMetadata::collect(&file.items);
    let mut methods = BTreeSet::new();
    let mut roots = MethodReferences::default();
    let mut dependencies = BTreeMap::<MethodKey, MethodReferences>::new();
    collect_method_demands_from_items(
        &file.items,
        &metadata,
        &mut methods,
        &mut roots,
        &mut dependencies,
    );

    let mut demanded = resolve_method_references(&roots, &methods);
    let mut worklist = demanded.iter().cloned().collect::<Vec<_>>();
    while let Some(method) = worklist.pop() {
        let Some(references) = dependencies.get(&method) else {
            continue;
        };
        for dependency in resolve_method_references(references, &methods) {
            if demanded.insert(dependency.clone()) {
                worklist.push(dependency);
            }
        }
    }
    demanded
}

fn collect_method_demands_from_items(
    items: &[syn::Item],
    metadata: &TypeMetadata,
    methods: &mut BTreeSet<MethodKey>,
    roots: &mut MethodReferences,
    dependencies: &mut BTreeMap<MethodKey, MethodReferences>,
) {
    for item in items {
        match item {
            syn::Item::Impl(item_impl) if item_impl.trait_.is_none() => {
                let Some(owner) = impl_self_type_name(item_impl.self_ty.as_ref()) else {
                    continue;
                };
                for impl_item in &item_impl.items {
                    let syn::ImplItem::Fn(method) = impl_item else {
                        continue;
                    };
                    let key = MethodKey {
                        owner: owner.clone(),
                        name: method.sig.ident.to_string(),
                    };
                    methods.insert(key.clone());
                    dependencies.insert(
                        key,
                        CalledMethodCollector::collect_function(
                            metadata,
                            Some(owner.clone()),
                            &method.sig,
                            &method.block,
                        ),
                    );
                }
            }
            syn::Item::Impl(item_impl) => {
                let owner = impl_self_type_name(item_impl.self_ty.as_ref());
                for impl_item in &item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        roots.extend(CalledMethodCollector::collect_function(
                            metadata,
                            owner.clone(),
                            &method.sig,
                            &method.block,
                        ));
                    }
                }
            }
            syn::Item::Fn(function) => roots.extend(CalledMethodCollector::collect_function(
                metadata,
                None,
                &function.sig,
                &function.block,
            )),
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    collect_method_demands_from_items(
                        nested,
                        metadata,
                        methods,
                        roots,
                        dependencies,
                    );
                }
            }
            _ => {
                let mut collector = CalledMethodCollector::new(metadata, None);
                collector.visit_item(item);
                roots.extend(collector.references);
            }
        }
    }
}

fn resolve_method_references(
    references: &MethodReferences,
    methods: &BTreeSet<MethodKey>,
) -> BTreeSet<MethodKey> {
    methods
        .iter()
        .filter(|key| {
            references.method_names.contains(&key.name)
                || references
                    .associated
                    .contains(&(key.owner.clone(), key.name.clone()))
        })
        .cloned()
        .collect()
}

#[derive(Default)]
struct MethodReferences {
    method_names: BTreeSet<String>,
    associated: BTreeSet<(String, String)>,
}

impl MethodReferences {
    fn extend(&mut self, other: Self) {
        self.method_names.extend(other.method_names);
        self.associated.extend(other.associated);
    }
}

#[derive(Default)]
struct TypeMetadata {
    fields: HashMap<(String, String), String>,
    function_returns: HashMap<String, String>,
    method_returns: HashMap<(String, String), String>,
}

impl TypeMetadata {
    fn collect(items: &[syn::Item]) -> Self {
        let mut metadata = Self::default();
        metadata.collect_items(items);
        metadata
    }

    fn collect_items(&mut self, items: &[syn::Item]) {
        for item in items {
            match item {
                syn::Item::Struct(item_struct) => {
                    let owner = item_struct.ident.to_string();
                    for field in &item_struct.fields {
                        if let (Some(name), Some(field_owner)) =
                            (&field.ident, type_owner(&field.ty))
                        {
                            self.fields
                                .insert((owner.clone(), name.to_string()), field_owner);
                        }
                    }
                }
                syn::Item::Fn(function) => {
                    if let Some(owner) = return_type_owner(&function.sig.output) {
                        self.function_returns
                            .insert(function.sig.ident.to_string(), owner);
                    }
                }
                syn::Item::Impl(item_impl) => {
                    let Some(owner) = impl_self_type_name(item_impl.self_ty.as_ref()) else {
                        continue;
                    };
                    for impl_item in &item_impl.items {
                        if let syn::ImplItem::Fn(method) = impl_item
                            && let Some(return_owner) = return_type_owner(&method.sig.output)
                        {
                            self.method_returns.insert(
                                (owner.clone(), method.sig.ident.to_string()),
                                return_owner,
                            );
                        }
                    }
                }
                syn::Item::Mod(module) => {
                    if let Some((_, nested)) = &module.content {
                        self.collect_items(nested);
                    }
                }
                _ => {}
            }
        }
    }
}

fn return_type_owner(output: &syn::ReturnType) -> Option<String> {
    let syn::ReturnType::Type(_, ty) = output else {
        return None;
    };
    type_owner(ty)
}

fn type_owner(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        syn::Type::Reference(reference) => type_owner(&reference.elem),
        syn::Type::Paren(paren) => type_owner(&paren.elem),
        syn::Type::Group(group) => type_owner(&group.elem),
        _ => None,
    }
}

struct CalledMethodCollector<'metadata> {
    references: MethodReferences,
    metadata: &'metadata TypeMetadata,
    self_owner: Option<String>,
    variable_types: HashMap<String, String>,
}

impl<'metadata> CalledMethodCollector<'metadata> {
    fn new(metadata: &'metadata TypeMetadata, self_owner: Option<String>) -> Self {
        Self {
            references: MethodReferences::default(),
            metadata,
            self_owner,
            variable_types: HashMap::new(),
        }
    }

    fn collect_function(
        metadata: &'metadata TypeMetadata,
        self_owner: Option<String>,
        signature: &syn::Signature,
        body: &syn::Block,
    ) -> MethodReferences {
        let mut collector = Self::new(metadata, self_owner);
        for input in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = input
                && let syn::Pat::Ident(binding) = parameter.pat.as_ref()
                && let Some(owner) = type_owner(&parameter.ty)
            {
                collector
                    .variable_types
                    .insert(binding.ident.to_string(), owner);
            }
        }
        collector.visit_block(body);
        collector.references
    }

    fn receiver_owner(&self, expression: &syn::Expr) -> Option<String> {
        match expression {
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
                let name = path.path.segments.first()?.ident.to_string();
                if name == "self" {
                    self.self_owner.clone()
                } else {
                    self.variable_types.get(&name).cloned()
                }
            }
            syn::Expr::Field(field) => {
                let owner = self.receiver_owner(&field.base)?;
                let member = match &field.member {
                    syn::Member::Named(name) => name.to_string(),
                    syn::Member::Unnamed(_) => return None,
                };
                self.metadata.fields.get(&(owner, member)).cloned()
            }
            syn::Expr::Reference(reference) => self.receiver_owner(&reference.expr),
            syn::Expr::Paren(paren) => self.receiver_owner(&paren.expr),
            syn::Expr::Group(group) => self.receiver_owner(&group.expr),
            syn::Expr::Call(call) => {
                let syn::Expr::Path(path) = call.func.as_ref() else {
                    return None;
                };
                let segments = path.path.segments.iter().collect::<Vec<_>>();
                if segments.len() >= 2 && segments.last()?.ident == "new" {
                    return Some(segments[segments.len() - 2].ident.to_string());
                }
                self.metadata
                    .function_returns
                    .get(&segments.last()?.ident.to_string())
                    .cloned()
            }
            syn::Expr::MethodCall(call) => {
                let owner = self.receiver_owner(&call.receiver)?;
                self.metadata
                    .method_returns
                    .get(&(owner, call.method.to_string()))
                    .cloned()
            }
            _ => None,
        }
    }

    fn collect_macro_tokens(&mut self, tokens: TokenStream) {
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        for (index, token) in tokens.iter().enumerate() {
            if let TokenTree::Group(group) = token {
                self.collect_macro_tokens(group.stream());
            }
            let TokenTree::Ident(identifier) = token else {
                continue;
            };
            if index > 0
                && matches!(&tokens[index - 1], TokenTree::Punct(punct) if punct.as_char() == '.')
            {
                self.references.method_names.insert(identifier.to_string());
            }
            if index > 2
                && matches!(&tokens[index - 1], TokenTree::Punct(punct) if punct.as_char() == ':')
                && matches!(&tokens[index - 2], TokenTree::Punct(punct) if punct.as_char() == ':')
                && let TokenTree::Ident(owner) = &tokens[index - 3]
            {
                self.references
                    .associated
                    .insert((owner.to_string(), identifier.to_string()));
            }
        }
    }
}

impl<'ast> Visit<'ast> for CalledMethodCollector<'_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        visit::visit_local(self, local);
        let syn::Pat::Type(typed) = &local.pat else {
            return;
        };
        if let syn::Pat::Ident(binding) = typed.pat.as_ref()
            && let Some(owner) = type_owner(&typed.ty)
        {
            self.variable_types.insert(binding.ident.to_string(), owner);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if let Some(owner) = self.receiver_owner(&call.receiver) {
            self.references
                .associated
                .insert((owner, call.method.to_string()));
        } else {
            self.references.method_names.insert(call.method.to_string());
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        let segments = path.path.segments.iter().collect::<Vec<_>>();
        if let Some(pair) = segments.windows(2).last() {
            let owner = if pair[0].ident == "Self" {
                self.self_owner
                    .clone()
                    .unwrap_or_else(|| pair[0].ident.to_string())
            } else {
                pair[0].ident.to_string()
            };
            self.references
                .associated
                .insert((owner, pair[1].ident.to_string()));
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, macro_: &'ast syn::Macro) {
        visit::visit_macro(self, macro_);
        self.collect_macro_tokens(macro_.tokens.clone());
    }
}

pub(super) fn prune_inherent_methods(items: &mut Vec<syn::Item>, demanded: &BTreeSet<MethodKey>) {
    for item in items.iter_mut() {
        match item {
            syn::Item::Impl(item_impl) if item_impl.trait_.is_none() => {
                let owner = impl_self_type_name(item_impl.self_ty.as_ref());
                item_impl.items.retain(|impl_item| {
                    !matches!(impl_item, syn::ImplItem::Fn(method) if owner.as_ref().is_some_and(|owner| !demanded.contains(&MethodKey { owner: owner.clone(), name: method.sig.ident.to_string() })))
                });
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &mut module.content {
                    prune_inherent_methods(nested, demanded);
                }
            }
            _ => {}
        }
    }
    items.retain(|item| {
        !matches!(item, syn::Item::Impl(item_impl) if item_impl.trait_.is_none() && item_impl.items.is_empty())
    });
}
