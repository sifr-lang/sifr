use std::collections::{HashMap, HashSet};

use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BorrowKind {
    Reference,
    TraitObject,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct BorrowPlan(Vec<(usize, BorrowKind)>);

pub(super) fn rewrite_borrow_only_value_parameters(file: &mut syn::File) {
    let shared_trait_methods = shared_trait_methods(file);
    let mut plans = collect_plans(file, &shared_trait_methods);
    super::scoped_imports::expand(file, &mut plans);
    SignatureRewriter {
        plans: &plans,
        modules: Vec::new(),
        owner: None,
    }
    .visit_file_mut(file);
    CallRewriter {
        plans: &plans,
        modules: Vec::new(),
        owner: None,
        binding_types: HashMap::new(),
    }
    .visit_file_mut(file);
}

fn shared_trait_methods(file: &syn::File) -> HashSet<(String, String)> {
    let mut collector = SharedTraitMethodCollector::default();
    collector.visit_file(file);
    collector.methods
}

#[derive(Default)]
struct SharedTraitMethodCollector {
    methods: HashSet<(String, String)>,
}

impl Visit<'_> for SharedTraitMethodCollector {
    fn visit_item_trait(&mut self, trait_: &syn::ItemTrait) {
        for item in &trait_.items {
            let syn::TraitItem::Fn(method) = item else {
                continue;
            };
            if method.sig.receiver().is_some_and(|receiver| {
                matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, None))
            }) {
                self.methods
                    .insert((trait_.ident.to_string(), method.sig.ident.to_string()));
            }
        }
        visit::visit_item_trait(self, trait_);
    }
}

fn collect_plans(
    file: &syn::File,
    shared_trait_methods: &HashSet<(String, String)>,
) -> HashMap<String, BorrowPlan> {
    let mut collector = PlanCollector {
        plans: HashMap::new(),
        ambiguous: HashSet::new(),
        modules: Vec::new(),
        owner: None,
        trait_implementation: false,
        shared_trait_methods,
    };
    collector.visit_file(file);
    collector
        .plans
        .retain(|key, plan| !plan.0.is_empty() && !collector.ambiguous.contains(key));
    collector.plans
}

struct PlanCollector<'facts> {
    plans: HashMap<String, BorrowPlan>,
    ambiguous: HashSet<String>,
    modules: Vec<String>,
    owner: Option<String>,
    trait_implementation: bool,
    shared_trait_methods: &'facts HashSet<(String, String)>,
}

impl Visit<'_> for PlanCollector<'_> {
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
        let previous_trait = std::mem::replace(
            &mut self.trait_implementation,
            implementation.trait_.is_some(),
        );
        visit::visit_item_impl(self, implementation);
        self.trait_implementation = previous_trait;
        self.owner = previous;
    }

    fn visit_item_fn(&mut self, function: &syn::ItemFn) {
        self.collect_signature(&function.sig, &function.block);
        visit::visit_item_fn(self, function);
    }

    fn visit_impl_item_fn(&mut self, function: &syn::ImplItemFn) {
        self.collect_signature(&function.sig, &function.block);
        visit::visit_impl_item_fn(self, function);
    }
}

impl PlanCollector<'_> {
    fn collect_signature(&mut self, signature: &syn::Signature, block: &syn::Block) {
        if self.trait_implementation {
            return;
        }
        let mut parameters = Vec::new();
        for (index, parameter) in signature
            .inputs
            .iter()
            .filter_map(|input| {
                let syn::FnArg::Typed(parameter) = input else {
                    return None;
                };
                Some(parameter)
            })
            .enumerate()
        {
            let Some(name) = simple_pattern_name(&parameter.pat) else {
                continue;
            };
            let kind = if type_is_self(&parameter.ty) || type_is_owned_option(&parameter.ty) {
                let mut uses = BorrowOnlySelfUses {
                    name: &name,
                    seen: false,
                    unsupported: false,
                };
                uses.visit_block(block);
                (uses.seen && !uses.unsupported).then_some(BorrowKind::Reference)
            } else if let Some(trait_name) = boxed_trait_name(&parameter.ty) {
                let mut uses = BorrowOnlyTraitObjectUses {
                    name: &name,
                    trait_name,
                    methods: self.shared_trait_methods,
                    seen: false,
                    unsupported: false,
                };
                uses.visit_block(block);
                (uses.seen && !uses.unsupported).then_some(BorrowKind::TraitObject)
            } else {
                None
            };
            if let Some(kind) = kind {
                parameters.push((index, kind));
            }
        }
        let key = scoped_key(&self.modules, self.owner.as_deref(), signature);
        let plan = BorrowPlan(parameters);
        self.plans
            .entry(key.clone())
            .and_modify(|known| {
                if *known != plan {
                    self.ambiguous.insert(key.clone());
                }
            })
            .or_insert(plan);
    }
}

struct BorrowOnlySelfUses<'name> {
    name: &'name str,
    seen: bool,
    unsupported: bool,
}

impl Visit<'_> for BorrowOnlySelfUses<'_> {
    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if let syn::Expr::Field(field) = call.receiver.as_ref()
            && expression_is_name(&field.base, self.name)
            && matches!(
                call.method.to_string().as_str(),
                "iter" | "len" | "is_empty" | "get"
            )
        {
            self.seen = true;
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        if expression_is_name(&reference.expr, self.name) && reference.mutability.is_none() {
            self.seen = true;
            return;
        }
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.seen = true;
            self.unsupported = true;
        }
    }

    fn visit_item(&mut self, _item: &syn::Item) {}

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

struct BorrowOnlyTraitObjectUses<'facts> {
    name: &'facts str,
    trait_name: String,
    methods: &'facts HashSet<(String, String)>,
    seen: bool,
    unsupported: bool,
}

impl Visit<'_> for BorrowOnlyTraitObjectUses<'_> {
    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if expression_is_name(&call.receiver, self.name) {
            self.seen = true;
            if !self
                .methods
                .contains(&(self.trait_name.clone(), call.method.to_string()))
            {
                self.unsupported = true;
            }
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.seen = true;
            self.unsupported = true;
        }
    }

    fn visit_item(&mut self, _item: &syn::Item) {}

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

struct SignatureRewriter<'plans> {
    plans: &'plans HashMap<String, BorrowPlan>,
    modules: Vec<String>,
    owner: Option<String>,
}

impl VisitMut for SignatureRewriter<'_> {
    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        let Some((_, items)) = &mut module.content else {
            return;
        };
        self.modules.push(module.ident.to_string());
        for item in items {
            self.visit_item_mut(item);
        }
        self.modules.pop();
    }

    fn visit_item_impl_mut(&mut self, implementation: &mut syn::ItemImpl) {
        let previous = self.owner.replace(type_owner_name(&implementation.self_ty));
        visit_mut::visit_item_impl_mut(self, implementation);
        self.owner = previous;
    }

    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        self.rewrite_signature(&mut function.sig);
        visit_mut::visit_item_fn_mut(self, function);
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        self.rewrite_signature(&mut function.sig);
        visit_mut::visit_impl_item_fn_mut(self, function);
    }
}

impl SignatureRewriter<'_> {
    fn rewrite_signature(&self, signature: &mut syn::Signature) {
        let Some(plan) =
            self.plans
                .get(&scoped_key(&self.modules, self.owner.as_deref(), signature))
        else {
            return;
        };
        for (index, kind) in &plan.0 {
            let Some(parameter) = typed_input_mut(signature, *index) else {
                continue;
            };
            match kind {
                BorrowKind::Reference
                    if type_is_self(&parameter.ty) || type_is_owned_option(&parameter.ty) =>
                {
                    let ty = &parameter.ty;
                    *parameter.ty = syn::parse_quote!(&#ty);
                }
                BorrowKind::TraitObject => {
                    if let Some(trait_type) = boxed_trait_type(&parameter.ty) {
                        *parameter.ty = syn::parse_quote!(&#trait_type);
                    }
                }
                BorrowKind::Reference => {}
            }
        }
    }
}

struct CallRewriter<'plans> {
    plans: &'plans HashMap<String, BorrowPlan>,
    modules: Vec<String>,
    owner: Option<String>,
    binding_types: HashMap<String, String>,
}

impl VisitMut for CallRewriter<'_> {
    fn visit_item_mod_mut(&mut self, module: &mut syn::ItemMod) {
        let Some((_, items)) = &mut module.content else {
            return;
        };
        self.modules.push(module.ident.to_string());
        for item in items {
            self.visit_item_mut(item);
        }
        self.modules.pop();
    }

    fn visit_item_impl_mut(&mut self, implementation: &mut syn::ItemImpl) {
        let previous = self.owner.replace(type_owner_name(&implementation.self_ty));
        visit_mut::visit_item_impl_mut(self, implementation);
        self.owner = previous;
    }

    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        self.visit_function(&function.sig, &mut function.block);
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        self.visit_function(&function.sig, &mut function.block);
    }

    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.binding_types.clone();
        for statement in &mut block.stmts {
            self.visit_stmt_mut(statement);
            if let syn::Stmt::Local(local) = statement
                && let Some(name) = simple_pattern_name(&local.pat)
            {
                self.binding_types.remove(&name);
                if let syn::Pat::Type(typed) = &local.pat
                    && let Some(owner) = type_path_name(&typed.ty)
                {
                    self.binding_types.insert(name, owner);
                }
            }
        }
        self.binding_types = outer;
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        visit_mut::visit_expr_call_mut(self, call);
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return;
        };
        let Some(plan) = associated_or_free_plan(
            self.plans,
            &self.modules,
            self.owner.as_deref(),
            &path.path,
            call.args.len(),
        ) else {
            return;
        };
        rewrite_arguments(&mut call.args, plan);
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        let owner = if expression_is_name(&call.receiver, "self") {
            self.owner.clone()
        } else {
            expression_name(&call.receiver).and_then(|name| self.binding_types.get(&name).cloned())
        };
        let Some(owner) = owner else {
            return;
        };
        let Ok(path) = syn::parse_str::<syn::Path>(&format!("{owner}::{}", call.method)) else {
            return;
        };
        let plan = associated_or_free_plan(
            self.plans,
            &self.modules,
            self.owner.as_deref(),
            &path,
            call.args.len(),
        );
        let Some(plan) = plan else {
            return;
        };
        rewrite_arguments(&mut call.args, plan);
    }
}

impl CallRewriter<'_> {
    fn visit_function(&mut self, signature: &syn::Signature, block: &mut syn::Block) {
        let previous = std::mem::take(&mut self.binding_types);
        for input in &signature.inputs {
            if let syn::FnArg::Typed(parameter) = input
                && let Some(name) = simple_pattern_name(&parameter.pat)
                && let Some(owner) = type_path_name(&parameter.ty)
            {
                self.binding_types.insert(name, owner);
            }
        }
        self.visit_block_mut(block);
        self.binding_types = previous;
    }
}

fn rewrite_arguments(
    arguments: &mut syn::punctuated::Punctuated<syn::Expr, syn::Token![,]>,
    plan: &BorrowPlan,
) {
    for (index, kind) in &plan.0 {
        let Some(argument) = arguments.get_mut(*index) else {
            continue;
        };
        if *kind == BorrowKind::TraitObject
            && let syn::Expr::Call(boxing) = argument
            && matches!(boxing.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.len() == 2
                    && path.path.segments[0].ident == "Box"
                    && path.path.segments[1].ident == "new")
            && boxing.args.len() == 1
        {
            let value = boxing.args[0].clone();
            *argument = syn::parse_quote!(&#value);
            continue;
        }
        if !matches!(argument, syn::Expr::Reference(_)) {
            let value = argument.clone();
            *argument = syn::parse_quote!(&#value);
        }
    }
}

fn associated_or_free_plan<'plans>(
    plans: &'plans HashMap<String, BorrowPlan>,
    modules: &[String],
    self_owner: Option<&str>,
    path: &syn::Path,
    arguments: usize,
) -> Option<&'plans BorrowPlan> {
    let mut segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    if segments.first().is_some_and(|segment| segment == "Self") {
        segments[0] = self_owner?.to_string();
    }
    if path.leading_colon.is_some() {
        return None;
    }
    let qualified = super::scoped_imports::qualified_path(modules, &segments)?;
    plans.get(&format!("{}#{arguments}", qualified.join("::")))
}

fn scoped_key(modules: &[String], owner: Option<&str>, signature: &syn::Signature) -> String {
    let arguments = signature
        .inputs
        .iter()
        .filter(|input| matches!(input, syn::FnArg::Typed(_)))
        .count();
    scoped_name_key(modules, owner, &signature.ident.to_string(), arguments)
}

fn scoped_name_key(
    modules: &[String],
    owner: Option<&str>,
    name: &str,
    arguments: usize,
) -> String {
    let mut path = modules.to_vec();
    if let Some(owner) = owner {
        path.push(owner.to_string());
    }
    path.push(name.to_string());
    format!("{}#{arguments}", path.join("::"))
}

fn type_is_self(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.is_ident("Self"))
}

fn type_is_owned_option(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Option"))
}

fn boxed_trait_name(ty: &syn::Type) -> Option<String> {
    let trait_type = boxed_trait_type(ty)?;
    let syn::Type::TraitObject(trait_) = trait_type else {
        return None;
    };
    trait_.bounds.iter().find_map(|bound| {
        let syn::TypeParamBound::Trait(bound) = bound else {
            return None;
        };
        bound
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
    })
}

fn boxed_trait_type(ty: &syn::Type) -> Option<&syn::Type> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    let segment = path.path.segments.last()?;
    if segment.ident != "Box" {
        return None;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    arguments.args.iter().find_map(|argument| {
        let syn::GenericArgument::Type(ty @ syn::Type::TraitObject(_)) = argument else {
            return None;
        };
        Some(ty)
    })
}

fn type_owner_name(ty: &syn::Type) -> String {
    type_path_name(ty).unwrap_or_else(|| quote::quote!(#ty).to_string())
}

fn type_path_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) if path.qself.is_none() && path.path.leading_colon.is_none() => Some(
            path.path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        ),
        syn::Type::Reference(reference) => type_path_name(&reference.elem),
        _ => None,
    }
}

fn simple_pattern_name(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Ident(binding) if binding.subpat.is_none() => Some(binding.ident.to_string()),
        syn::Pat::Type(typed) => simple_pattern_name(&typed.pat),
        syn::Pat::Paren(paren) => simple_pattern_name(&paren.pat),
        _ => None,
    }
}

fn expression_name(expression: &syn::Expr) -> Option<String> {
    let syn::Expr::Path(path) = expression else {
        return None;
    };
    path.path.get_ident().map(ToString::to_string)
}

fn expression_is_name(expression: &syn::Expr, name: &str) -> bool {
    matches!(expression, syn::Expr::Path(path) if path.path.is_ident(name))
}

fn typed_input_mut(signature: &mut syn::Signature, index: usize) -> Option<&mut syn::PatType> {
    signature
        .inputs
        .iter_mut()
        .filter_map(|input| {
            let syn::FnArg::Typed(parameter) = input else {
                return None;
            };
            Some(parameter)
        })
        .nth(index)
}
