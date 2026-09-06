use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};

pub(super) fn collect_mutating_method_names(file: &syn::File) -> HashSet<String> {
    let mut collector = MutatingMethodCollector::default();
    collector.visit_file(file);
    collector.names
}

#[derive(Default)]
pub(super) struct LocalMethodFacts {
    returns_by_method: HashMap<String, HashSet<String>>,
    mutable: HashSet<(String, String)>,
    shared: HashSet<(String, String)>,
}

pub(super) fn collect_local_method_facts(file: &syn::File) -> LocalMethodFacts {
    let mut collector = LocalMethodFactCollector::default();
    collector.visit_file(file);
    collector.facts
}

#[derive(Default)]
struct LocalMethodFactCollector {
    facts: LocalMethodFacts,
}

impl<'ast> Visit<'ast> for LocalMethodFactCollector {
    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        let Some(owner) = type_owner_name(&implementation.self_ty) else {
            visit::visit_item_impl(self, implementation);
            return;
        };
        for item in &implementation.items {
            let syn::ImplItem::Fn(method) = item else {
                continue;
            };
            let key = (owner.clone(), method.sig.ident.to_string());
            if signature_has_mutable_receiver(&method.sig) {
                self.facts.mutable.insert(key);
            } else if method.sig.receiver().is_some() {
                self.facts.shared.insert(key);
            }
            if let syn::ReturnType::Type(_, ty) = &method.sig.output
                && let Some(returned) = type_owner_name(ty)
            {
                self.facts
                    .returns_by_method
                    .entry(method.sig.ident.to_string())
                    .or_default()
                    .insert(returned);
            }
        }
        visit::visit_item_impl(self, implementation);
    }
}

fn type_owner_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Reference(reference) => type_owner_name(&reference.elem),
        syn::Type::Group(group) => type_owner_name(&group.elem),
        syn::Type::Paren(paren) => type_owner_name(&paren.elem),
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

pub(super) fn remove_unneeded_parameter_mutability(
    signature: &mut syn::Signature,
    body: &syn::Block,
    mutating_methods: &HashSet<String>,
    local_method_facts: &LocalMethodFacts,
) {
    let mut collector = MutatingUseCollector::new(mutating_methods, local_method_facts);
    collector.visit_block(body);
    for argument in &mut signature.inputs {
        if let syn::FnArg::Typed(typed) = argument {
            remove_unneeded_pattern_mutability(&mut typed.pat, &collector.names);
        }
    }
}

pub(super) fn remove_unneeded_mutability(
    statements: &mut [syn::Stmt],
    mutating_methods: &HashSet<String>,
    local_method_facts: &LocalMethodFacts,
) {
    let mut collector = MutatingUseCollector::new(mutating_methods, local_method_facts);
    for statement in statements.iter() {
        collector.visit_stmt(statement);
    }
    for statement in statements {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        if local
            .init
            .as_ref()
            .is_some_and(|initializer| expression_is_closure(&initializer.expr))
        {
            continue;
        }
        remove_unneeded_pattern_mutability(&mut local.pat, &collector.names);
    }
}

pub(super) fn statements_mutate_name(
    statements: &[syn::Stmt],
    name: &str,
    mutating_methods: &HashSet<String>,
) -> bool {
    let empty_facts = LocalMethodFacts::default();
    let mut collector = MutatingUseCollector::new(mutating_methods, &empty_facts);
    for statement in statements {
        collector.visit_stmt(statement);
    }
    collector.names.contains(name)
}

fn expression_is_closure(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Closure(_) => true,
        syn::Expr::Group(group) => expression_is_closure(&group.expr),
        syn::Expr::Paren(paren) => expression_is_closure(&paren.expr),
        _ => false,
    }
}

fn remove_unneeded_pattern_mutability(pattern: &mut syn::Pat, mutating: &HashSet<String>) {
    match pattern {
        syn::Pat::Ident(binding) => {
            if binding.mutability.is_some() && !mutating.contains(&binding.ident.to_string()) {
                binding.mutability = None;
            }
            if let Some((_, subpattern)) = &mut binding.subpat {
                remove_unneeded_pattern_mutability(subpattern, mutating);
            }
        }
        syn::Pat::Tuple(tuple) => {
            for element in &mut tuple.elems {
                remove_unneeded_pattern_mutability(element, mutating);
            }
        }
        syn::Pat::TupleStruct(tuple) => {
            for element in &mut tuple.elems {
                remove_unneeded_pattern_mutability(element, mutating);
            }
        }
        syn::Pat::Struct(struct_) => {
            for field in &mut struct_.fields {
                remove_unneeded_pattern_mutability(&mut field.pat, mutating);
            }
        }
        syn::Pat::Slice(slice) => {
            for element in &mut slice.elems {
                remove_unneeded_pattern_mutability(element, mutating);
            }
        }
        syn::Pat::Reference(reference) => {
            remove_unneeded_pattern_mutability(&mut reference.pat, mutating);
        }
        syn::Pat::Type(typed) => remove_unneeded_pattern_mutability(&mut typed.pat, mutating),
        syn::Pat::Paren(paren) => remove_unneeded_pattern_mutability(&mut paren.pat, mutating),
        syn::Pat::Const(_)
        | syn::Pat::Lit(_)
        | syn::Pat::Macro(_)
        | syn::Pat::Or(_)
        | syn::Pat::Path(_)
        | syn::Pat::Range(_)
        | syn::Pat::Rest(_)
        | syn::Pat::Verbatim(_)
        | syn::Pat::Wild(_)
        | _ => {}
    }
}

fn macro_is_read_only(path: &syn::Path) -> bool {
    path.segments.last().is_some_and(|segment| {
        matches!(
            segment.ident.to_string().as_str(),
            "assert"
                | "assert_eq"
                | "assert_ne"
                | "dbg"
                | "eprint"
                | "eprintln"
                | "format"
                | "format_args"
                | "print"
                | "println"
                | "vec"
                | "write"
                | "writeln"
        )
    })
}

pub(super) fn collect_token_identifiers(
    tokens: proc_macro2::TokenStream,
    names: &mut HashSet<String>,
) {
    for token in tokens {
        match token {
            proc_macro2::TokenTree::Ident(identifier) => {
                names.insert(identifier.to_string());
            }
            proc_macro2::TokenTree::Group(group) => {
                collect_token_identifiers(group.stream(), names);
            }
            _ => {}
        }
    }
}

struct MutatingUseCollector<'facts> {
    names: HashSet<String>,
    mutating_methods: HashSet<String>,
    binding_owners: HashMap<String, HashSet<String>>,
    local_method_facts: &'facts LocalMethodFacts,
}

impl<'facts> MutatingUseCollector<'facts> {
    fn new(
        mutating_methods: &HashSet<String>,
        local_method_facts: &'facts LocalMethodFacts,
    ) -> Self {
        Self {
            names: HashSet::new(),
            mutating_methods: mutating_methods.clone(),
            binding_owners: HashMap::new(),
            local_method_facts,
        }
    }

    fn collect_place(&mut self, expression: &syn::Expr) {
        match expression {
            syn::Expr::Path(path) if path.path.segments.len() == 1 => {
                if let Some(segment) = path.path.segments.first() {
                    self.names.insert(segment.ident.to_string());
                }
            }
            syn::Expr::Field(field) => self.collect_place(&field.base),
            syn::Expr::Index(index) => self.collect_place(&index.expr),
            syn::Expr::Paren(paren) => self.collect_place(&paren.expr),
            _ => {}
        }
    }

    fn is_read_only_generated_cache_call(call: &syn::ExprMethodCall) -> bool {
        let syn::Expr::Path(path) = call.receiver.as_ref() else {
            return false;
        };
        path.qself.is_none()
            && path.path.segments.len() == 1
            && path.path.segments[0]
                .ident
                .to_string()
                .starts_with("sifr_generated_chars_")
            && matches!(
                call.method.to_string().as_str(),
                "as_slice" | "clone" | "first" | "get" | "is_empty" | "iter" | "last" | "len"
            )
    }

    fn method_requires_mutability(&self, call: &syn::ExprMethodCall) -> bool {
        if self.call_is_proven_shared(call) {
            return false;
        }
        let method = &call.method;
        self.mutating_methods.contains(&method.to_string())
            || matches!(
                method.to_string().as_str(),
                "append"
                    | "as_mut_slice"
                    | "blocking_recv"
                    | "clear"
                    | "dedup"
                    | "drain"
                    | "entry"
                    | "extend"
                    | "flush"
                    | "get_mut"
                    | "insert"
                    | "iter_mut"
                    | "join_next"
                    | "join_next_with_id"
                    | "make_contiguous"
                    | "next"
                    | "pop"
                    | "push"
                    | "push_back"
                    | "push_front"
                    | "push_str"
                    | "read"
                    | "read_exact"
                    | "read_to_end"
                    | "read_to_string"
                    | "recv"
                    | "recv_many"
                    | "remove"
                    | "resize"
                    | "retain"
                    | "reverse"
                    | "seek"
                    | "shutdown"
                    | "sort"
                    | "sort_by"
                    | "sort_by_key"
                    | "spawn"
                    | "spawn_blocking"
                    | "spawn_local"
                    | "spawn_on"
                    | "split_at_mut"
                    | "swap"
                    | "swap_remove"
                    | "take"
                    | "truncate"
                    | "try_recv"
                    | "values_mut"
                    | "write"
                    | "write_all"
            )
    }

    fn call_is_proven_shared(&self, call: &syn::ExprMethodCall) -> bool {
        let syn::Expr::Path(path) = call.receiver.as_ref() else {
            return false;
        };
        let Some(name) = path.path.get_ident().map(ToString::to_string) else {
            return false;
        };
        let Some(owners) = self.binding_owners.get(&name) else {
            return false;
        };
        let method = call.method.to_string();
        let relevant = owners
            .iter()
            .filter(|owner| {
                contains_method_fact(&self.local_method_facts.shared, owner, &method)
                    || contains_method_fact(&self.local_method_facts.mutable, owner, &method)
            })
            .collect::<Vec<_>>();
        !relevant.is_empty()
            && relevant
                .iter()
                .all(|owner| contains_method_fact(&self.local_method_facts.shared, owner, &method))
    }

    fn expression_owner_candidates(&self, expression: &syn::Expr) -> HashSet<String> {
        match expression {
            syn::Expr::Reference(reference) => self.expression_owner_candidates(&reference.expr),
            syn::Expr::Group(group) => self.expression_owner_candidates(&group.expr),
            syn::Expr::Paren(paren) => self.expression_owner_candidates(&paren.expr),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|ident| self.binding_owners.get(&ident.to_string()))
                .cloned()
                .unwrap_or_default(),
            syn::Expr::Struct(struct_) => struct_
                .path
                .segments
                .last()
                .map(|segment| HashSet::from([segment.ident.to_string()]))
                .unwrap_or_default(),
            syn::Expr::Call(call) => match call.func.as_ref() {
                syn::Expr::Path(path) if path.path.segments.len() > 1 => {
                    HashSet::from([path.path.segments[path.path.segments.len() - 2]
                        .ident
                        .to_string()])
                }
                _ => HashSet::new(),
            },
            syn::Expr::MethodCall(call) => self
                .local_method_facts
                .returns_by_method
                .get(&call.method.to_string())
                .cloned()
                .unwrap_or_default(),
            _ => HashSet::new(),
        }
    }
}

fn contains_method_fact(facts: &HashSet<(String, String)>, owner: &str, method: &str) -> bool {
    facts
        .iter()
        .any(|(known_owner, known_method)| known_owner == owner && known_method == method)
}

impl<'ast> Visit<'ast> for MutatingUseCollector<'_> {
    fn visit_local(&mut self, local: &'ast syn::Local) {
        if let Some(init) = &local.init {
            self.visit_expr(&init.expr);
            if let Some(name) = simple_pattern_name(&local.pat) {
                let explicit = pattern_type_owner(&local.pat).map(|owner| HashSet::from([owner]));
                let owners =
                    explicit.unwrap_or_else(|| self.expression_owner_candidates(&init.expr));
                if !owners.is_empty() {
                    self.binding_owners.insert(name, owners);
                }
            }
            if let Some((_, diverge)) = &init.diverge {
                self.visit_expr(diverge);
            }
        }
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        if macro_is_read_only(&rust_macro.path) {
            let Ok(arguments) = rust_macro.parse_body_with(
                syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
            ) else {
                return;
            };
            let writes_first_argument = rust_macro.path.segments.last().is_some_and(|segment| {
                matches!(segment.ident.to_string().as_str(), "write" | "writeln")
            });
            for (index, argument) in arguments.iter().enumerate() {
                if writes_first_argument && index == 0 {
                    self.collect_place(argument);
                }
                self.visit_expr(argument);
            }
        } else {
            collect_token_identifiers(rust_macro.tokens.clone(), &mut self.names);
        }
    }

    fn visit_expr_assign(&mut self, assign: &'ast syn::ExprAssign) {
        self.collect_place(&assign.left);
        visit::visit_expr_assign(self, assign);
    }

    fn visit_expr_binary(&mut self, binary: &'ast syn::ExprBinary) {
        if matches!(
            binary.op,
            syn::BinOp::AddAssign(_)
                | syn::BinOp::SubAssign(_)
                | syn::BinOp::MulAssign(_)
                | syn::BinOp::DivAssign(_)
                | syn::BinOp::RemAssign(_)
                | syn::BinOp::BitXorAssign(_)
                | syn::BinOp::BitAndAssign(_)
                | syn::BinOp::BitOrAssign(_)
                | syn::BinOp::ShlAssign(_)
                | syn::BinOp::ShrAssign(_)
        ) {
            self.collect_place(&binary.left);
        }
        visit::visit_expr_binary(self, binary);
    }

    fn visit_expr_reference(&mut self, reference: &'ast syn::ExprReference) {
        if reference.mutability.is_some() {
            self.collect_place(&reference.expr);
        }
        visit::visit_expr_reference(self, reference);
    }

    fn visit_expr_method_call(&mut self, call: &'ast syn::ExprMethodCall) {
        if self.method_requires_mutability(call) && !Self::is_read_only_generated_cache_call(call) {
            self.collect_place(&call.receiver);
        }
        visit::visit_expr_method_call(self, call);
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

fn pattern_type_owner(pattern: &syn::Pat) -> Option<String> {
    match pattern {
        syn::Pat::Type(typed) => type_owner_name(&typed.ty),
        syn::Pat::Paren(paren) => pattern_type_owner(&paren.pat),
        _ => None,
    }
}

#[derive(Default)]
struct MutatingMethodCollector {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for MutatingMethodCollector {
    fn visit_impl_item_fn(&mut self, method: &'ast syn::ImplItemFn) {
        if signature_has_mutable_receiver(&method.sig) {
            self.names.insert(method.sig.ident.to_string());
        }
        visit::visit_impl_item_fn(self, method);
    }

    fn visit_trait_item_fn(&mut self, method: &'ast syn::TraitItemFn) {
        if signature_has_mutable_receiver(&method.sig) {
            self.names.insert(method.sig.ident.to_string());
        }
        visit::visit_trait_item_fn(self, method);
    }
}

fn signature_has_mutable_receiver(signature: &syn::Signature) -> bool {
    signature.inputs.iter().any(|argument| {
        matches!(argument, syn::FnArg::Receiver(receiver)
            if receiver.mutability.is_some()
                || matches!(receiver.kind, syn::ReceiverKind::Reference(_, _, Some(_))))
    })
}
