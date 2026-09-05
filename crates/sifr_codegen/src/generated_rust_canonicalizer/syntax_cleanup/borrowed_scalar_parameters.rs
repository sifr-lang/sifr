use std::collections::{HashMap, HashSet};

use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct ScalarBorrowPlan {
    owned: Vec<usize>,
    borrowed: Vec<usize>,
    optional: Vec<usize>,
    consumed: Vec<usize>,
}

pub(super) fn rewrite_borrow_only_scalar_parameters(file: &mut syn::File) {
    let plans = collect_plans(std::slice::from_ref(file));
    apply_plans(file, &plans);
}

pub(super) fn collect_project_plans(files: &[syn::File]) -> HashMap<String, ScalarBorrowPlan> {
    collect_plans(files)
}

pub(super) fn apply_project_plans(file: &mut syn::File, plans: &HashMap<String, ScalarBorrowPlan>) {
    apply_plans(file, plans);
}

fn collect_plans(files: &[syn::File]) -> HashMap<String, ScalarBorrowPlan> {
    let mut plans = HashMap::new();
    let mut ambiguous = HashSet::new();
    for file in files {
        let mut collector = PlanCollector {
            plans: &mut plans,
            ambiguous: &mut ambiguous,
            modules: Vec::new(),
            functions: Vec::new(),
            owner: None,
        };
        collector.visit_file(file);
    }
    let callable_value_uses = collect_callable_value_uses(files);
    let preserved_abis = callable_value_plan_keys(&plans, &callable_value_uses);
    plans.retain(|key, _| !ambiguous.contains(key));
    for key in preserved_abis {
        if let Some(plan) = plans.get_mut(&key) {
            plan.consumed.clone_from(&plan.owned);
            plan.owned.clear();
            plan.borrowed.clear();
            plan.optional.clear();
        }
    }
    plans
}

struct PlanCollector<'plans> {
    plans: &'plans mut HashMap<String, ScalarBorrowPlan>,
    ambiguous: &'plans mut HashSet<String>,
    modules: Vec<String>,
    functions: Vec<String>,
    owner: Option<String>,
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
        visit::visit_item_impl(self, implementation);
        self.owner = previous;
    }

    fn visit_item_fn(&mut self, function: &syn::ItemFn) {
        self.collect_signature(
            &function.sig,
            &function.block,
            None,
            !matches!(function.vis, syn::Visibility::Inherited),
        );
        self.functions.push(function.sig.ident.to_string());
        visit::visit_item_fn(self, function);
        self.functions.pop();
    }

    fn visit_impl_item_fn(&mut self, function: &syn::ImplItemFn) {
        let owner = self.owner.clone();
        self.collect_signature(
            &function.sig,
            &function.block,
            owner.as_deref(),
            !matches!(function.vis, syn::Visibility::Inherited),
        );
        self.functions.push(function.sig.ident.to_string());
        visit::visit_impl_item_fn(self, function);
        self.functions.pop();
    }
}

impl PlanCollector<'_> {
    fn collect_signature(
        &mut self,
        signature: &syn::Signature,
        block: &syn::Block,
        owner: Option<&str>,
        _public_api: bool,
    ) {
        let scalar_names = collect_sifr_int_bindings(signature, block);
        let mut owned = Vec::new();
        let mut borrowed = Vec::new();
        let mut optional = Vec::new();
        for (index, parameter) in signature
            .inputs
            .iter()
            .filter_map(|argument| {
                let syn::FnArg::Typed(parameter) = argument else {
                    return None;
                };
                Some(parameter)
            })
            .enumerate()
        {
            if borrowed_sifr_int_option_type(&parameter.ty)
                || owned_sifr_int_option_type(&parameter.ty)
            {
                optional.push(index);
            } else if borrowed_scalar_type(&parameter.ty) {
                borrowed.push(index);
            } else if owned_scalar_type(&parameter.ty)
                && simple_pattern_name(&parameter.pat).is_some_and(|name| {
                    let mut uses = BorrowCompatibleUses::new(&name, &scalar_names);
                    uses.visit_block(block);
                    uses.seen && !uses.unsupported
                })
            {
                owned.push(index);
                borrowed.push(index);
            }
        }
        let key = scoped_signature_key(&self.modules, &self.functions, owner, signature);
        let plan = ScalarBorrowPlan {
            owned,
            borrowed,
            optional,
            consumed: Vec::new(),
        };
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

fn apply_plans(file: &mut syn::File, plans: &HashMap<String, ScalarBorrowPlan>) {
    SignatureAndBodyRewriter {
        plans,
        modules: Vec::new(),
        functions: Vec::new(),
        owner: None,
    }
    .visit_file_mut(file);
    ScalarCallRewriter {
        plans,
        modules: Vec::new(),
        functions: Vec::new(),
        borrowed_bindings: HashSet::new(),
        optional_borrowed_bindings: HashSet::new(),
        owned_optional_bindings: HashSet::new(),
    }
    .visit_file_mut(file);
}

struct SignatureAndBodyRewriter<'plans> {
    plans: &'plans HashMap<String, ScalarBorrowPlan>,
    modules: Vec<String>,
    functions: Vec<String>,
    owner: Option<String>,
}

impl VisitMut for SignatureAndBodyRewriter<'_> {
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
        self.rewrite_signature(&mut function.sig, &mut function.block, None);
        self.functions.push(function.sig.ident.to_string());
        visit_mut::visit_item_fn_mut(self, function);
        self.functions.pop();
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        self.rewrite_signature(
            &mut function.sig,
            &mut function.block,
            self.owner.as_deref(),
        );
        self.functions.push(function.sig.ident.to_string());
        visit_mut::visit_impl_item_fn_mut(self, function);
        self.functions.pop();
    }
}

impl SignatureAndBodyRewriter<'_> {
    fn rewrite_signature(
        &self,
        signature: &mut syn::Signature,
        block: &mut syn::Block,
        owner: Option<&str>,
    ) {
        if let Some(plan) = self.plans.get(&scoped_signature_key(
            &self.modules,
            &self.functions,
            owner,
            signature,
        )) {
            let mut names = HashSet::new();
            let mut optional_names = Vec::new();
            for index in &plan.owned {
                let Some(parameter) = typed_input_mut(signature, *index) else {
                    continue;
                };
                if !owned_scalar_type(&parameter.ty) {
                    continue;
                }
                if let Some(name) = simple_pattern_name(&parameter.pat) {
                    names.insert(name);
                }
                let ty = parameter.ty.as_ref();
                *parameter.ty = syn::parse_quote!(&#ty);
            }
            for index in &plan.optional {
                let Some(parameter) = typed_input_mut(signature, *index) else {
                    continue;
                };
                let Some(inner) = owned_sifr_int_option_inner(&parameter.ty).cloned() else {
                    continue;
                };
                if let Some(name) = simple_pattern_name(&parameter.pat) {
                    optional_names.push(name);
                }
                *parameter.ty = syn::parse_quote!(Option<&#inner>);
            }
            for index in &plan.borrowed {
                if let Some(parameter) = typed_input(signature, *index)
                    && borrowed_scalar_type(&parameter.ty)
                    && let Some(name) = simple_pattern_name(&parameter.pat)
                {
                    names.insert(name);
                }
            }
            for index in &plan.consumed {
                if let Some(parameter) = typed_input(signature, *index)
                    && let Some(name) = simple_pattern_name(&parameter.pat)
                {
                    consume_tail_operation_parameter(block, &name);
                }
            }
            BorrowedUseRewriter { active: names }.visit_block_mut(block);
            for name in optional_names.into_iter().rev() {
                let name = syn::Ident::new(&name, proc_macro2::Span::call_site());
                block.stmts.insert(
                    0,
                    syn::parse_quote!(let #name: Option<SifrInt> = #name.cloned();),
                );
            }
        }
    }
}

include!("borrowed_scalar_parameters/call_rewriter.rs");

fn rewrite_borrowed_argument(argument: &mut syn::Expr, borrowed_bindings: &HashSet<String>) {
    if expression_is_borrowed_binding(argument, borrowed_bindings) {
        return;
    }
    if let syn::Expr::Reference(reference) = argument {
        if expression_is_borrowed_binding(&reference.expr, borrowed_bindings) {
            *argument = reference.expr.as_ref().clone();
        } else if let syn::Expr::MethodCall(clone) = reference.expr.as_ref()
            && clone.method == "clone"
            && clone.args.is_empty()
        {
            reference.expr = clone.receiver.clone();
        }
        return;
    }
    if let syn::Expr::MethodCall(clone) = argument
        && clone.method == "clone"
        && clone.args.is_empty()
    {
        if expression_is_borrowed_binding(&clone.receiver, borrowed_bindings) {
            *argument = clone.receiver.as_ref().clone();
            return;
        }
        if let syn::Expr::Unary(dereference) = clone.receiver.as_ref()
            && matches!(dereference.op, syn::UnOp::Deref(_))
            && expression_is_borrowed_binding(&dereference.expr, borrowed_bindings)
        {
            *argument = dereference.expr.as_ref().clone();
            return;
        }
    }
    let value = argument.clone();
    *argument = syn::parse_quote!(&#value);
}

fn expression_is_borrowed_binding(
    expression: &syn::Expr,
    borrowed_bindings: &HashSet<String>,
) -> bool {
    matches!(expression, syn::Expr::Path(path)
        if path.path.get_ident().is_some_and(|name|
            borrowed_bindings.contains(&name.to_string())))
}

struct BorrowCompatibleUses<'name> {
    name: &'name str,
    scalar_names: &'name HashSet<String>,
    seen: bool,
    unsupported: bool,
}

impl<'name> BorrowCompatibleUses<'name> {
    fn new(name: &'name str, scalar_names: &'name HashSet<String>) -> Self {
        Self {
            name,
            scalar_names,
            seen: false,
            unsupported: false,
        }
    }
}

impl Visit<'_> for BorrowCompatibleUses<'_> {
    fn visit_block(&mut self, block: &syn::Block) {
        for statement in &block.stmts {
            if let syn::Stmt::Local(local) = statement {
                if let Some(init) = &local.init {
                    self.visit_expr(&init.expr);
                    if let Some((_, diverge)) = &init.diverge {
                        self.visit_expr(diverge);
                    }
                }
                if pattern_binds_name(&local.pat, self.name) {
                    return;
                }
            } else {
                self.visit_stmt(statement);
            }
        }
    }

    fn visit_expr_closure(&mut self, closure: &syn::ExprClosure) {
        if expression_mentions_name(&closure.body, self.name) {
            self.unsupported = true;
            self.seen = true;
        }
    }

    fn visit_expr_call(&mut self, call: &syn::ExprCall) {
        if let Some(closure) = immediate_closure(&call.func) {
            self.visit_expr(&closure.body);
            for argument in &call.args {
                self.visit_expr(argument);
            }
            return;
        }
        visit::visit_expr_call(self, call);
    }

    fn visit_expr_if(&mut self, branch: &syn::ExprIf) {
        self.visit_expr(&branch.cond);
        if condition_binds_name(&branch.cond, self.name) {
            if let Some((_, alternative)) = &branch.else_branch {
                self.visit_expr(alternative);
            }
            return;
        }
        self.visit_block(&branch.then_branch);
        if let Some((_, alternative)) = &branch.else_branch {
            self.visit_expr(alternative);
        }
    }

    fn visit_expr_let(&mut self, let_: &syn::ExprLet) {
        if expression_is_name(&let_.expr, self.name) {
            self.seen = true;
        } else {
            visit::visit_expr_let(self, let_);
        }
    }

    fn visit_expr_binary(&mut self, binary: &syn::ExprBinary) {
        if matches!(binary.op, syn::BinOp::And(_)) {
            self.visit_expr(&binary.left);
            if !condition_binds_name(&binary.left, self.name) {
                self.visit_expr(&binary.right);
            }
            return;
        }
        if comparison_operator(&binary.op) {
            let left = expression_is_name(&binary.left, self.name);
            let right = expression_is_name(&binary.right, self.name);
            if left || right {
                self.seen = true;
                let counterpart = if left { &binary.right } else { &binary.left };
                if !(expression_is_name(counterpart, self.name)
                    || borrowable_sifr_int_comparison_operand(counterpart, self.scalar_names)
                    || expression_is_usize_measurement(counterpart))
                {
                    self.unsupported = true;
                }
            } else {
                visit::visit_expr_binary(self, binary);
            }
        } else {
            visit::visit_expr_binary(self, binary);
        }
    }

    fn visit_expr_async(&mut self, asynchronous: &syn::ExprAsync) {
        let mut finder = NameFinder::new(self.name);
        finder.visit_block(&asynchronous.block);
        if finder.found {
            self.unsupported = true;
            self.seen = true;
        }
    }

    fn visit_expr_reference(&mut self, reference: &syn::ExprReference) {
        if expression_is_name(&reference.expr, self.name) && reference.mutability.is_none() {
            self.seen = true;
        } else {
            visit::visit_expr_reference(self, reference);
        }
    }

    fn visit_expr_method_call(&mut self, call: &syn::ExprMethodCall) {
        if expression_is_name(&call.receiver, self.name) {
            self.seen = true;
            if call.method.to_string().starts_with("into_")
                || matches!(
                    call.method.to_string().as_str(),
                    "and_then"
                        | "map"
                        | "map_or"
                        | "map_or_else"
                        | "ok_or"
                        | "ok_or_else"
                        | "unwrap"
                        | "unwrap_or"
                        | "unwrap_or_else"
                )
            {
                self.unsupported = true;
            }
            for argument in &call.args {
                self.visit_expr(argument);
            }
        } else {
            visit::visit_expr_method_call(self, call);
        }
    }

    fn visit_expr_match(&mut self, match_: &syn::ExprMatch) {
        if expression_is_name(&match_.expr, self.name) {
            self.seen = true;
            for arm in &match_.arms {
                if pattern_binds_name(&arm.pat, self.name) {
                    continue;
                }
                self.visit_expr(&arm.body);
                if let syn::Pat::Guard(guard) = &arm.pat {
                    self.visit_expr(&guard.guard);
                }
            }
        } else {
            visit::visit_expr_match(self, match_);
        }
    }

    fn visit_expr_path(&mut self, path: &syn::ExprPath) {
        if path.path.is_ident(self.name) {
            self.seen = true;
            self.unsupported = true;
        }
    }

    fn visit_item(&mut self, _item: &syn::Item) {}

    fn visit_macro(&mut self, rust_macro: &syn::Macro) {
        if crate::generated_rust_canonicalizer::format_capture::names(rust_macro)
            .contains(self.name)
        {
            self.seen = true;
        }
        if let Ok(arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) {
            for argument in &arguments {
                self.visit_expr(argument);
            }
        }
    }
}

fn immediate_closure(expression: &syn::Expr) -> Option<&syn::ExprClosure> {
    match expression {
        syn::Expr::Closure(closure) => Some(closure),
        syn::Expr::Group(group) => immediate_closure(&group.expr),
        syn::Expr::Paren(paren) => immediate_closure(&paren.expr),
        _ => None,
    }
}

struct BorrowedUseRewriter {
    active: HashSet<String>,
}

impl VisitMut for BorrowedUseRewriter {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.active.clone();
        for statement in &mut block.stmts {
            self.visit_stmt_mut(statement);
            if let syn::Stmt::Local(local) = statement {
                for name in &outer {
                    if pattern_binds_name(&local.pat, name) {
                        self.active.remove(name);
                    }
                }
            }
        }
        self.active = outer;
    }

    fn visit_expr_binary_mut(&mut self, binary: &mut syn::ExprBinary) {
        if matches!(binary.op, syn::BinOp::And(_)) {
            self.visit_expr_mut(&mut binary.left);
            let outer = self.active.clone();
            for name in condition_binding_names(&binary.left) {
                self.active.remove(&name);
            }
            self.visit_expr_mut(&mut binary.right);
            self.active = outer;
            return;
        }
        visit_mut::visit_expr_binary_mut(self, binary);
        if !comparison_operator(&binary.op) {
            return;
        }
        let left = active_name(&binary.left, &self.active);
        let right = active_name(&binary.right, &self.active);
        if left.is_some()
            && right.is_none()
            && !matches!(binary.right.as_ref(), syn::Expr::Reference(_))
        {
            let value = binary.right.as_ref();
            binary.right = if expression_is_usize_measurement(value) {
                Box::new(syn::parse_quote!(&SifrInt::from(#value)))
            } else {
                Box::new(syn::parse_quote!(&#value))
            };
        }
        if right.is_some()
            && left.is_none()
            && !matches!(binary.left.as_ref(), syn::Expr::Reference(_))
        {
            let value = binary.left.as_ref();
            binary.left = if expression_is_usize_measurement(value) {
                Box::new(syn::parse_quote!(&SifrInt::from(#value)))
            } else {
                Box::new(syn::parse_quote!(&#value))
            };
        }
    }

    fn visit_expr_if_mut(&mut self, branch: &mut syn::ExprIf) {
        self.visit_expr_mut(&mut branch.cond);
        let outer = self.active.clone();
        for name in condition_binding_names(&branch.cond) {
            self.active.remove(&name);
        }
        self.visit_block_mut(&mut branch.then_branch);
        self.active = outer.clone();
        if let Some((_, alternative)) = &mut branch.else_branch {
            self.visit_expr_mut(alternative);
        }
        self.active = outer;
    }

    fn visit_expr_reference_mut(&mut self, reference: &mut syn::ExprReference) {
        visit_mut::visit_expr_reference_mut(self, reference);
        if reference.mutability.is_none() && active_name(&reference.expr, &self.active).is_some() {
            reference.and_token = syn::Token![&](reference.and_token.span);
        }
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if let syn::Expr::Reference(reference) = expression
            && reference.mutability.is_none()
            && active_name(&reference.expr, &self.active).is_some()
        {
            *expression = reference.expr.as_ref().clone();
        }
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if call.method == "clone"
            && call.args.is_empty()
            && active_name(&call.receiver, &self.active).is_some()
        {
            let receiver = call.receiver.as_ref();
            *call.receiver = syn::parse_quote!(*#receiver);
        }
    }

    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        self.visit_expr_mut(&mut match_.expr);
        let borrowed_scrutinee = active_name(&match_.expr, &self.active).is_some();
        for arm in &mut match_.arms {
            let outer = self.active.clone();
            self.active
                .retain(|name| !pattern_binds_name(&arm.pat, name));
            if let syn::Pat::Guard(guard) = &mut arm.pat {
                self.visit_expr_mut(&mut guard.guard);
            }
            self.visit_expr_mut(&mut arm.body);
            self.active = outer;
        }
        if borrowed_scrutinee {
            let value = match_.expr.as_ref();
            *match_.expr = syn::parse_quote!((*#value).clone());
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn active_name(expression: &syn::Expr, active: &HashSet<String>) -> Option<String> {
    let syn::Expr::Path(path) = expression else {
        return None;
    };
    path.path
        .get_ident()
        .map(ToString::to_string)
        .filter(|name| active.contains(name))
}

fn expression_is_name(expression: &syn::Expr, name: &str) -> bool {
    matches!(expression, syn::Expr::Path(path) if path.path.is_ident(name))
}

fn comparison_operator(operator: &syn::BinOp) -> bool {
    matches!(
        operator,
        syn::BinOp::Eq(_)
            | syn::BinOp::Ne(_)
            | syn::BinOp::Lt(_)
            | syn::BinOp::Le(_)
            | syn::BinOp::Gt(_)
            | syn::BinOp::Ge(_)
    )
}

fn borrowable_sifr_int_comparison_operand(
    expression: &syn::Expr,
    scalar_names: &HashSet<String>,
) -> bool {
    match expression {
        syn::Expr::Reference(_) => true,
        syn::Expr::Path(path) => path.path.get_ident().is_some_and(|name| {
            let name = name.to_string();
            scalar_names.contains(&name)
                || name == "current_index"
                || name.starts_with("sifr_generated_index")
        }),
        syn::Expr::Call(call) => matches!(call.func.as_ref(), syn::Expr::Path(path)
            if path.path.segments.iter().rev().nth(1).is_some_and(|segment|
                segment.ident == "SifrInt")
                || (path.path.segments.iter().any(|segment| segment.ident == "ops")
                    && path.path.segments.last().is_some_and(|segment|
                        matches!(segment.ident.to_string().as_str(), "add" | "sub" | "mul")))),
        syn::Expr::Paren(paren) => {
            borrowable_sifr_int_comparison_operand(&paren.expr, scalar_names)
        }
        _ => false,
    }
}

fn expression_is_usize_measurement(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::MethodCall(call)
        if call.args.is_empty()
            && matches!(call.method.to_string().as_str(), "len" | "count"))
}

fn collect_sifr_int_bindings(signature: &syn::Signature, body: &syn::Block) -> HashSet<String> {
    let mut names = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            type_is_sifr_int(&parameter.ty).then(|| simple_pattern_name(&parameter.pat))?
        })
        .collect::<HashSet<_>>();
    let mut collector = SifrIntBindingCollector::default();
    collector.visit_block(body);
    names.extend(collector.names);
    names
}

include!("borrowed_scalar_parameters/type_facts.rs");
include!("borrowed_scalar_parameters/optional_parameters.rs");

include!("borrowed_scalar_parameters/call_identity.rs");
include!("borrowed_scalar_parameters/callable_values.rs");
include!("borrowed_scalar_parameters/lexical_conditions.rs");
include!("borrowed_scalar_parameters/patterns.rs");
