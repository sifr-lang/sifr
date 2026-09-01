use std::collections::HashSet;
use syn::visit::{self, Visit};

pub(super) fn collect_mutating_method_names(file: &syn::File) -> HashSet<String> {
    let mut collector = MutatingMethodCollector::default();
    collector.visit_file(file);
    collector.names
}

pub(super) fn remove_unneeded_parameter_mutability(
    signature: &mut syn::Signature,
    body: &syn::Block,
    mutating_methods: &HashSet<String>,
) {
    let mut collector = MutatingUseCollector::new(mutating_methods);
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
) {
    let mut collector = MutatingUseCollector::new(mutating_methods);
    for statement in statements.iter() {
        collector.visit_stmt(statement);
    }
    for statement in statements {
        let syn::Stmt::Local(local) = statement else {
            continue;
        };
        remove_unneeded_pattern_mutability(&mut local.pat, &collector.names);
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

#[derive(Default)]
struct MutatingUseCollector {
    names: HashSet<String>,
    mutating_methods: HashSet<String>,
}

impl MutatingUseCollector {
    fn new(mutating_methods: &HashSet<String>) -> Self {
        Self {
            names: HashSet::new(),
            mutating_methods: mutating_methods.clone(),
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

    fn method_requires_mutability(&self, method: &syn::Ident) -> bool {
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
                    | "truncate"
                    | "try_recv"
                    | "values_mut"
                    | "write"
                    | "write_all"
            )
    }
}

impl<'ast> Visit<'ast> for MutatingUseCollector {
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
        if self.method_requires_mutability(&call.method)
            && !Self::is_read_only_generated_cache_call(call)
        {
            self.collect_place(&call.receiver);
        }
        visit::visit_expr_method_call(self, call);
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
