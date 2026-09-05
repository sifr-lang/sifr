use std::collections::{HashMap, HashSet};

use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

pub(crate) fn rewrite_slice_parameter_apis(file: &mut syn::File) {
    rewrite_item_scope(&mut file.items);
    let plans = collect_project_shared_slice_params(std::slice::from_ref(file));
    rewrite_project_shared_slice_calls(file, &plans);
}

pub(crate) fn collect_project_shared_slice_params(
    files: &[syn::File],
) -> HashMap<String, Vec<(usize, bool)>> {
    let mut plans = HashMap::<String, Vec<(usize, bool)>>::new();
    let mut ambiguous = HashSet::new();
    for file in files {
        let mut collector = SharedSliceSignatureCollector::default();
        collector.visit_file(file);
        for (key, indices) in collector.plans {
            plans
                .entry(key.clone())
                .and_modify(|known| {
                    if *known != indices {
                        ambiguous.insert(key.clone());
                    }
                })
                .or_insert(indices);
        }
    }
    plans.retain(|key, indices| !indices.is_empty() && !ambiguous.contains(key));
    plans
}

pub(crate) fn rewrite_project_shared_slice_calls(
    file: &mut syn::File,
    plans: &HashMap<String, Vec<(usize, bool)>>,
) {
    SharedSliceCallRewriter {
        plans,
        descend_modules: true,
        modules: Vec::new(),
        slice_bindings: HashSet::new(),
    }
    .visit_file_mut(file);
}

#[derive(Default)]
struct SharedSliceSignatureCollector {
    plans: HashMap<String, Vec<(usize, bool)>>,
    modules: Vec<String>,
    owner: Option<String>,
}

impl Visit<'_> for SharedSliceSignatureCollector {
    fn visit_item_mod(&mut self, module: &syn::ItemMod) {
        let Some((_, items)) = &module.content else {
            return;
        };
        self.modules.push(module.ident.to_string());
        for item in items {
            self.visit_item(item);
        }
        self.modules.pop();
    }

    fn visit_item_impl(&mut self, implementation: &syn::ItemImpl) {
        let previous = self.owner.replace(type_owner_name(&implementation.self_ty));
        visit::visit_item_impl(self, implementation);
        self.owner = previous;
    }

    fn visit_item_fn(&mut self, function: &syn::ItemFn) {
        self.collect_signature(&function.sig);
        visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &syn::ImplItemFn) {
        self.collect_signature(&function.sig);
        visit::visit_impl_item_fn(self, function);
    }
}

impl SharedSliceSignatureCollector {
    fn collect_signature(&mut self, signature: &syn::Signature) {
        let key = scoped_signature_key(&self.modules, self.owner.as_deref(), signature);
        let indices = slice_parameter_indices(signature);
        self.plans
            .entry(key)
            .and_modify(|known| {
                if *known != indices {
                    known.clear();
                }
            })
            .or_insert(indices);
    }
}

pub(crate) fn rewrite_shared_slice_calls(items: &mut [syn::Item]) {
    for item in items.iter_mut() {
        if let syn::Item::Mod(module) = item
            && let Some((_, nested)) = &mut module.content
        {
            rewrite_shared_slice_calls(nested);
        }
    }
    rewrite_calls_in_item_scope(items);
}

fn rewrite_item_scope(items: &mut [syn::Item]) {
    let direct_plans = direct_slice_call_plans(items);
    for item in items.iter_mut() {
        match item {
            syn::Item::Fn(function) => {
                rewrite_slice_only_vec_parameters_with_calls(
                    &mut function.sig,
                    &function.block,
                    &direct_plans,
                );
            }
            syn::Item::Impl(implementation) => {
                for item in &mut implementation.items {
                    if let syn::ImplItem::Fn(method) = item {
                        rewrite_slice_only_vec_parameters_with_calls(
                            &mut method.sig,
                            &method.block,
                            &direct_plans,
                        );
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &mut module.content {
                    rewrite_item_scope(nested);
                }
            }
            _ => {}
        }
    }

    rewrite_calls_in_item_scope(items);
}

fn rewrite_calls_in_item_scope(items: &mut [syn::Item]) {
    let mut plans = HashMap::<String, Vec<(usize, bool)>>::new();
    let mut ambiguous = HashSet::new();
    for item in items.iter() {
        let syn::Item::Fn(function) = item else {
            continue;
        };
        let key = signature_key(&function.sig);
        let shared = slice_parameter_indices(&function.sig);
        if plans.insert(key.clone(), shared).is_some() {
            ambiguous.insert(key);
        }
    }
    plans.retain(|key, indices| !indices.is_empty() && !ambiguous.contains(key));
    if plans.is_empty() {
        return;
    }
    let mut rewriter = SharedSliceCallRewriter {
        plans: &plans,
        descend_modules: false,
        modules: Vec::new(),
        slice_bindings: HashSet::new(),
    };
    for item in items {
        rewriter.visit_item_mut(item);
    }
}

pub(crate) fn rewrite_slice_only_vec_parameters(signature: &mut syn::Signature, body: &syn::Block) {
    rewrite_slice_only_vec_parameters_with_calls(signature, body, &HashMap::new());
}

fn rewrite_slice_only_vec_parameters_with_calls(
    signature: &mut syn::Signature,
    body: &syn::Block,
    call_plans: &HashMap<String, Vec<(usize, bool)>>,
) {
    for input in &mut signature.inputs {
        let syn::FnArg::Typed(parameter) = input else {
            continue;
        };
        let syn::Pat::Ident(binding) = parameter.pat.as_ref() else {
            continue;
        };
        let (vector, originally_borrowed, originally_mutable) = match parameter.ty.as_ref() {
            syn::Type::Reference(reference) => {
                let syn::Type::Path(vector) = reference.elem.as_ref() else {
                    continue;
                };
                (vector, true, reference.mutability.is_some())
            }
            syn::Type::Path(vector) => (vector, false, false),
            _ => continue,
        };
        let Some(segment) = vector.path.segments.last() else {
            continue;
        };
        let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
            continue;
        };
        let Some(syn::GenericArgument::Type(element)) = arguments.args.first() else {
            continue;
        };
        if segment.ident != "Vec" {
            continue;
        }
        let mut use_ = SliceOnlyParameterUse {
            name: &binding.ident,
            valid: true,
            method_uses: 0,
            mutable_uses: 0,
            call_plans,
        };
        use_.visit_block(body);
        if use_.valid && use_.method_uses > 0 && (originally_borrowed || use_.mutable_uses == 0) {
            let element = element.clone();
            let mutable = originally_borrowed && originally_mutable && use_.mutable_uses > 0;
            parameter.ty = if mutable {
                Box::new(syn::parse_quote!(&mut [#element]))
            } else {
                Box::new(syn::parse_quote!(&[#element]))
            };
        }
    }
}

fn direct_slice_call_plans(items: &[syn::Item]) -> HashMap<String, Vec<(usize, bool)>> {
    let mut plans = HashMap::new();
    let mut ambiguous = HashSet::new();
    for item in items {
        let syn::Item::Fn(function) = item else {
            continue;
        };
        let mut signature = function.sig.clone();
        rewrite_slice_only_vec_parameters(&mut signature, &function.block);
        let parameters = signature
            .inputs
            .iter()
            .filter_map(|argument| match argument {
                syn::FnArg::Typed(parameter) => Some(parameter),
                syn::FnArg::Receiver(_) => None,
            })
            .enumerate()
            .filter_map(|(index, parameter)| {
                let syn::Type::Reference(reference) = parameter.ty.as_ref() else {
                    return None;
                };
                matches!(reference.elem.as_ref(), syn::Type::Slice(_))
                    .then_some((index, reference.mutability.is_some()))
            })
            .collect::<Vec<_>>();
        let key = signature_key(&signature);
        plans
            .entry(key.clone())
            .and_modify(|known| {
                if *known != parameters {
                    ambiguous.insert(key.clone());
                }
            })
            .or_insert(parameters);
    }
    plans.retain(|key, parameters| !parameters.is_empty() && !ambiguous.contains(key));
    plans
}

fn slice_parameter_indices(signature: &syn::Signature) -> Vec<(usize, bool)> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Typed(parameter) => Some(parameter),
            syn::FnArg::Receiver(_) => None,
        })
        .enumerate()
        .filter_map(|(index, parameter)| {
            let syn::Type::Reference(reference) = parameter.ty.as_ref() else {
                return None;
            };
            matches!(reference.elem.as_ref(), syn::Type::Slice(_))
                .then_some((index, reference.mutability.is_some()))
        })
        .collect()
}

fn signature_key(signature: &syn::Signature) -> String {
    let arguments = signature
        .inputs
        .iter()
        .filter(|argument| matches!(argument, syn::FnArg::Typed(_)))
        .count();
    format!("{}#{arguments}", signature.ident)
}

struct SharedSliceCallRewriter<'plans> {
    plans: &'plans HashMap<String, Vec<(usize, bool)>>,
    descend_modules: bool,
    modules: Vec<String>,
    slice_bindings: HashSet<String>,
}

impl VisitMut for SharedSliceCallRewriter<'_> {
    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        self.visit_function(&function.sig, &mut function.block);
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        self.visit_function(&function.sig, &mut function.block);
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        visit_mut::visit_expr_call_mut(self, call);
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return;
        };
        if path.qself.is_some() {
            return;
        }
        let Some(indices) =
            resolve_slice_call_plan(self.plans, &self.modules, &path.path, call.args.len())
        else {
            return;
        };
        for (index, mutable) in indices {
            let Some(argument) = call.args.get_mut(*index) else {
                continue;
            };
            if let syn::Expr::Reference(reference) = argument {
                if matches!(reference.expr.as_ref(), syn::Expr::Path(path)
                    if path.path.get_ident().is_some_and(|name| self.slice_bindings.contains(&name.to_string())))
                {
                    *argument = reference.expr.as_ref().clone();
                    continue;
                }
                if !mutable {
                    reference.mutability = None;
                }
                rewrite_temporary_vec_as_slice(reference);
            } else if matches!(argument, syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name| self.slice_bindings.contains(&name.to_string())))
            {
                // This lexical binding already has the receiving slice ABI.
            } else {
                let value = argument.clone();
                *argument = if *mutable {
                    syn::parse_quote!(&mut #value)
                } else {
                    syn::parse_quote!(&#value)
                };
            }
        }
    }

    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        if self.descend_modules {
            let Some((_, items)) = &mut module.content else {
                return;
            };
            self.modules.push(module.ident.to_string());
            for item in items {
                self.visit_item_mut(item);
            }
            self.modules.pop();
        }
    }

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        let Ok(mut arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        rust_macro.tokens = quote::quote!(#arguments);
    }
}

impl SharedSliceCallRewriter<'_> {
    fn visit_function(&mut self, signature: &syn::Signature, block: &mut syn::Block) {
        let previous = std::mem::take(&mut self.slice_bindings);
        self.slice_bindings = signature
            .inputs
            .iter()
            .filter_map(|input| {
                let syn::FnArg::Typed(parameter) = input else {
                    return None;
                };
                matches!(parameter.ty.as_ref(), syn::Type::Reference(reference)
                    if matches!(reference.elem.as_ref(), syn::Type::Slice(_))
                        || matches!(reference.elem.as_ref(), syn::Type::Path(path)
                            if path.path.segments.last().is_some_and(|segment| segment.ident == "Vec")))
                .then(|| super::simple_pattern_name(&parameter.pat))
                .flatten()
            })
            .collect();
        self.visit_block_mut(block);
        self.slice_bindings = previous;
    }
}

fn scoped_signature_key(
    modules: &[String],
    owner: Option<&str>,
    signature: &syn::Signature,
) -> String {
    let arguments = signature
        .inputs
        .iter()
        .filter(|input| matches!(input, syn::FnArg::Typed(_)))
        .count();
    let mut path = modules.to_vec();
    if let Some(owner) = owner {
        path.push(owner.to_string());
    }
    path.push(signature.ident.to_string());
    format!("{}#{arguments}", path.join("::"))
}

fn resolve_slice_call_plan<'plans>(
    plans: &'plans HashMap<String, Vec<(usize, bool)>>,
    modules: &[String],
    path: &syn::Path,
    arguments: usize,
) -> Option<&'plans Vec<(usize, bool)>> {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let mut candidates = Vec::new();
    if segments.len() == 1 {
        let mut local = modules.to_vec();
        local.push(segments[0].clone());
        candidates.push(format!("{}#{arguments}", local.join("::")));
    } else {
        let mut qualified = segments;
        while matches!(
            qualified.first().map(String::as_str),
            Some("crate" | "self")
        ) {
            qualified.remove(0);
        }
        candidates.push(format!("{}#{arguments}", qualified.join("::")));
        let mut local = modules.to_vec();
        local.extend(qualified);
        candidates.push(format!("{}#{arguments}", local.join("::")));
    }
    candidates.sort();
    candidates.dedup();
    let mut matching = candidates.into_iter().filter_map(|key| plans.get(&key));
    if let Some(plan) = matching.next()
        && matching.next().is_none()
    {
        return Some(plan);
    }
    if path.segments.len() < 2 {
        return None;
    }
    let suffix = format!(
        "::{}::{}#{arguments}",
        path.segments[path.segments.len() - 2].ident,
        path.segments[path.segments.len() - 1].ident
    );
    let mut matching = plans
        .iter()
        .filter_map(|(key, plan)| key.ends_with(&suffix).then_some(plan));
    let plan = matching.next()?;
    matching.next().is_none().then_some(plan)
}

fn type_owner_name(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(path) => path.path.segments.last().map_or_else(
            || quote::quote!(#ty).to_string(),
            |segment| segment.ident.to_string(),
        ),
        _ => quote::quote!(#ty).to_string(),
    }
}

fn rewrite_temporary_vec_as_slice(reference: &mut syn::ExprReference) {
    let syn::Expr::Macro(vector) = reference.expr.as_ref() else {
        return;
    };
    if !vector.mac.path.is_ident("vec") {
        return;
    }
    let elements = vector.mac.tokens.clone();
    *reference.expr = syn::parse_quote!([#elements]);
}

struct SliceOnlyParameterUse<'name> {
    name: &'name proc_macro2::Ident,
    valid: bool,
    method_uses: usize,
    mutable_uses: usize,
    call_plans: &'name HashMap<String, Vec<(usize, bool)>>,
}

impl Visit<'_> for SliceOnlyParameterUse<'_> {
    fn visit_expr_closure(&mut self, closure: &syn::ExprClosure) {
        if closure.capture.is_some() && expression_mentions_ident(&closure.body, self.name) {
            self.valid = false;
            return;
        }
        visit::visit_expr_closure(self, closure);
    }

    fn visit_expr_call(&mut self, call: &syn::ExprCall) {
        let syn::Expr::Path(path) = call.func.as_ref() else {
            visit::visit_expr_call(self, call);
            return;
        };
        let Some(function) = path.path.get_ident() else {
            visit::visit_expr_call(self, call);
            return;
        };
        let key = format!("{}#{}", function, call.args.len());
        let Some(parameters) = self.call_plans.get(&key) else {
            visit::visit_expr_call(self, call);
            return;
        };
        for (index, argument) in call.args.iter().enumerate() {
            if matches!(argument, syn::Expr::Path(path) if path.path.is_ident(self.name)) {
                let Some((_, mutable)) = parameters.iter().find(|(planned, _)| *planned == index)
                else {
                    self.valid = false;
                    continue;
                };
                self.method_uses += 1;
                if *mutable {
                    self.mutable_uses += 1;
                }
            } else {
                self.visit_expr(argument);
            }
        }
    }

    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        if matches!(reference.expr.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
        {
            self.method_uses += 1;
            if reference.mutability.is_some() {
                self.mutable_uses += 1;
            }
            return;
        }
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(self.name))
        {
            let method = call.method.to_string();
            if matches!(
                method.as_str(),
                "fill"
                    | "get_mut"
                    | "reverse"
                    | "rotate_left"
                    | "rotate_right"
                    | "sort"
                    | "sort_by"
                    | "sort_by_key"
                    | "sort_unstable"
                    | "sort_unstable_by"
                    | "sort_unstable_by_key"
                    | "swap"
            ) {
                self.method_uses += 1;
                self.mutable_uses += 1;
                for argument in &call.args {
                    self.visit_expr(argument);
                }
            } else if matches!(
                method.as_str(),
                "binary_search"
                    | "contains"
                    | "first"
                    | "get"
                    | "is_empty"
                    | "iter"
                    | "last"
                    | "len"
                    | "starts_with"
                    | "ends_with"
            ) {
                self.method_uses += 1;
                for argument in &call.args {
                    self.visit_expr(argument);
                }
            } else {
                self.valid = false;
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.valid = false;
        }
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn expression_mentions_ident(expression: &syn::Expr, name: &proc_macro2::Ident) -> bool {
    struct Finder<'name> {
        name: &'name proc_macro2::Ident,
        found: bool,
    }

    impl Visit<'_> for Finder<'_> {
        fn visit_expr_path(&mut self, path: &syn::ExprPath) {
            if path.path.is_ident(self.name) {
                self.found = true;
            }
        }

        fn visit_item(&mut self, _item: &syn::Item) {}
    }

    let mut finder = Finder { name, found: false };
    finder.visit_expr(expression);
    finder.found
}
