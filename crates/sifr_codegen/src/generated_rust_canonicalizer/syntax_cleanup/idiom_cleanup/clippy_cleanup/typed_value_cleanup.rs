struct OwnedStringLocalCollector<'returns> {
    names: HashSet<String>,
    option_names: HashSet<String>,
    tuple_string_fields: HashMap<String, Vec<bool>>,
    tuple_string_returns: &'returns HashMap<String, Vec<bool>>,
}

impl Visit<'_> for OwnedStringLocalCollector<'_> {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && type_is_option_string(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.option_names.insert(name);
        }
        if let syn::Pat::Type(typed) = &local.pat
            && type_is_owned_string(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.names.insert(name);
        }
        if let Some(name) = simple_pattern_name(&local.pat)
            && local
                .init
                .as_ref()
                .is_some_and(|init| expression_constructs_owned_string(&init.expr))
        {
            self.names.insert(name);
        }
        if let syn::Pat::Type(typed) = &local.pat
            && let syn::Type::Tuple(tuple) = typed.ty.as_ref()
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.tuple_string_fields
                .insert(name, tuple.elems.iter().map(type_is_owned_string).collect());
        }
        if let syn::Pat::Tuple(pattern) = &local.pat
            && let Some(initializer) = &local.init
            && let syn::Expr::Path(path) = initializer.expr.as_ref()
            && let Some(source) = path.path.get_ident()
            && let Some(fields) = self.tuple_string_fields.get(&source.to_string())
        {
            for (element, is_string) in pattern.elems.iter().zip(fields) {
                if *is_string && let Some(name) = simple_pattern_name(element) {
                    self.names.insert(name);
                }
            }
        }
        if let syn::Pat::Tuple(pattern) = &local.pat
            && let Some(initializer) = &local.init
            && let syn::Expr::Call(call) = initializer.expr.as_ref()
            && let syn::Expr::Path(path) = call.func.as_ref()
            && path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(function) = path.path.get_ident()
            && let Some(fields) =
                self.tuple_string_returns
                    .get(&format!("{}#{}", function, call.args.len()))
            && !fields.is_empty()
        {
            for (element, is_string) in pattern.elems.iter().zip(fields) {
                if *is_string && let Some(name) = simple_pattern_name(element) {
                    self.names.insert(name);
                }
            }
        }
        if let syn::Pat::TupleStruct(pattern) = &local.pat
            && pattern
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "Some")
            && pattern.elems.len() == 1
            && let Some(element) = pattern.elems.first()
            && let Some(name) = simple_pattern_name(element)
            && local.init.as_ref().is_some_and(|init| {
                matches!(init.expr.as_ref(), syn::Expr::Path(path)
                    if path.path.get_ident().is_some_and(|source|
                        self.option_names.contains(&source.to_string())))
            })
        {
            self.names.insert(name);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}

    fn visit_expr_if(&mut self, branch: &syn::ExprIf) {
        if let syn::Expr::Let(condition) = branch.cond.as_ref()
            && matches!(condition.expr.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|source|
                    self.option_names.contains(&source.to_string())))
        {
            collect_owned_pattern_names(&condition.pat, &mut self.names);
        }
        visit::visit_expr_if(self, branch);
    }
}

struct BorrowedStringBindingCollector<'names> {
    option_roots: &'names HashSet<String>,
    active: HashSet<String>,
}

impl VisitMut for BorrowedStringBindingCollector<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.active.clone();
        for statement in &mut block.stmts {
            self.visit_stmt_mut(statement);
            let syn::Stmt::Local(local) = statement else {
                continue;
            };
            let bound = pattern_binding_names(&local.pat);
            self.active.retain(|name| !bound.contains(name));
            if let Some(name) = borrowed_option_string_binding(local, self.option_roots) {
                self.active.insert(name);
            }
        }
        self.active = outer;
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if matches!(call.method.to_string().as_str(), "to_owned" | "to_string")
            && call.args.is_empty()
            && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.active.contains(&name.to_string())))
        {
            call.method = syn::Ident::new("clone", call.method.span());
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn borrowed_option_string_binding(
    local: &syn::Local,
    option_roots: &HashSet<String>,
) -> Option<String> {
    if let syn::Pat::TupleStruct(pattern) = &local.pat
        && pattern
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "Some")
        && pattern.elems.len() == 1
        && let Some(name) = pattern.elems.first().and_then(simple_pattern_name)
        && local.init.as_ref().is_some_and(|init| {
            matches!(init.expr.as_ref(), syn::Expr::MethodCall(call)
                    if call.method == "as_ref"
                        && call.args.is_empty()
                        && matches!(call.receiver.as_ref(), syn::Expr::Path(path)
                            if path.path.get_ident().is_some_and(|root|
                                option_roots.contains(&root.to_string()))))
        })
    {
        Some(name)
    } else {
        None
    }
}

fn pattern_binding_names(pattern: &syn::Pat) -> HashSet<String> {
    struct Collector(HashSet<String>);
    impl Visit<'_> for Collector {
        fn visit_pat_ident(&mut self, binding: &syn::PatIdent) {
            self.0.insert(binding.ident.to_string());
            visit::visit_pat_ident(self, binding);
        }
    }
    let mut collector = Collector(HashSet::new());
    collector.visit_pat(pattern);
    collector.0
}

struct TypedStringInitializerRewriter<'names> {
    borrowed_roots: &'names HashSet<String>,
}

impl VisitMut for TypedStringInitializerRewriter<'_> {
    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        visit_mut::visit_local_mut(self, local);
        let syn::Pat::Type(typed) = &local.pat else {
            return;
        };
        if !type_is_owned_string(&typed.ty) {
            return;
        }
        let Some(init) = &mut local.init else {
            return;
        };
        let syn::Expr::MethodCall(clone) = init.expr.as_mut() else {
            return;
        };
        if clone.method == "clone"
            && clone.args.is_empty()
            && matches!(clone.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.borrowed_roots.contains(&name.to_string())))
        {
            clone.method = syn::Ident::new("to_string", clone.method.span());
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn expression_constructs_owned_string(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::MethodCall(call)
        if matches!(call.method.to_string().as_str(), "to_owned" | "to_string")
            && call.args.is_empty())
        || matches!(expression, syn::Expr::Call(call)
            if matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.len() >= 2
                    && path.path.segments.iter().rev().nth(1).is_some_and(|segment|
                        segment.ident == "String")))
        || matches!(expression, syn::Expr::Macro(rust_macro)
            if rust_macro.mac.path.is_ident("format"))
        || matches!(expression, syn::Expr::Match(match_)
            if !match_.arms.is_empty()
                && match_.arms.iter().all(|arm|
                    expression_constructs_owned_string(&arm.body)
                        || expression_diverges(&arm.body)))
}

fn expression_diverges(expression: &syn::Expr) -> bool {
    matches!(expression, syn::Expr::Return(_))
        || matches!(expression, syn::Expr::Block(block)
            if block_ends_control_flow(&block.block))
}

struct BorrowedCopyUnionCloneRewriter<'names> {
    borrowed_roots: &'names HashSet<String>,
}

impl VisitMut for BorrowedCopyUnionCloneRewriter<'_> {
    fn visit_expr_match_mut(&mut self, match_: &mut syn::ExprMatch) {
        self.visit_expr_mut(&mut match_.expr);
        let borrowed_match = expression_root_name(&match_.expr)
            .is_some_and(|name| self.borrowed_roots.contains(&name));
        for arm in &mut match_.arms {
            if borrowed_match && pattern_is_copy_union_variant(&arm.pat) {
                let mut bindings = HashSet::new();
                collect_owned_pattern_names(&arm.pat, &mut bindings);
                BorrowedCopyBindingCloneRewriter {
                    bindings: &bindings,
                }
                .visit_expr_mut(&mut arm.body);
            }
            self.visit_expr_mut(&mut arm.body);
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn pattern_is_copy_union_variant(pattern: &syn::Pat) -> bool {
    let syn::Pat::TupleStruct(tuple) = pattern else {
        return false;
    };
    tuple.path.segments.last().is_some_and(|segment| {
        let name = segment.ident.to_string();
        name.contains("atom4X3abool") || name.contains("atom5X3afloat")
    })
}

struct BorrowedCopyBindingCloneRewriter<'names> {
    bindings: &'names HashSet<String>,
}

impl VisitMut for BorrowedCopyBindingCloneRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(clone) = expression else {
            return;
        };
        if clone.method == "clone"
            && clone.args.is_empty()
            && matches!(clone.receiver.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.bindings.contains(&name.to_string())))
        {
            let receiver = clone.receiver.as_ref();
            *expression = syn::parse_quote!(*#receiver);
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

#[derive(Default)]
struct OptionStringLocalCollector {
    names: HashSet<String>,
}

impl Visit<'_> for OptionStringLocalCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && type_is_option_string(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.names.insert(name);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

struct OwnedOptionStringIdentityRewriter<'names> {
    names: &'names HashSet<String>,
}

impl VisitMut for OwnedOptionStringIdentityRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if call.method != "map_or_else"
            || call.args.len() != 2
            || !expression_root_name(&call.receiver).is_some_and(|name| self.names.contains(&name))
        {
            return;
        }
        let Some(syn::Expr::Closure(mapper)) = call.args.iter_mut().nth(1) else {
            return;
        };
        let Some(binding) = mapper.inputs.first().and_then(simple_pattern_name) else {
            return;
        };
        if mapper.inputs.len() == 1
            && matches!(mapper.body.as_ref(), syn::Expr::MethodCall(conversion)
                if matches!(conversion.method.to_string().as_str(), "to_owned" | "to_string" | "clone")
                    && conversion.args.is_empty()
                    && matches!(conversion.receiver.as_ref(), syn::Expr::Path(path)
                        if path.path.is_ident(&binding)))
        {
            let binding = syn::Ident::new(&binding, proc_macro2::Span::call_site());
            *mapper.body = syn::parse_quote!(#binding);
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        let Ok(mut arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        rust_macro.tokens = arguments.to_token_stream();
    }
}

struct OwnedStringCloneRewriter<'names> {
    names: &'names HashSet<String>,
    borrowed: &'names HashSet<String>,
}

impl VisitMut for OwnedStringCloneRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if rewrite_identity_string_concat(expression, self.names, self.borrowed) {
            return;
        }
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if !matches!(call.method.to_string().as_str(), "to_owned" | "to_string")
            || !call.args.is_empty()
        {
            return;
        }
        if let syn::Expr::MethodCall(clone) = call.receiver.as_ref()
            && clone.method == "clone"
            && clone.args.is_empty()
            && expression_root_name(&clone.receiver).is_some_and(|name| self.names.contains(&name))
        {
            *expression = syn::Expr::MethodCall(clone.clone());
            return;
        }
        if matches!(call.receiver.as_ref(), syn::Expr::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && path.path.segments.first().is_some_and(|segment|
                    self.names.contains(&segment.ident.to_string())))
        {
            call.method = syn::Ident::new("clone", call.method.span());
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}

    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        let Ok(mut arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        rust_macro.tokens = arguments.to_token_stream();
    }
}

#[derive(Default)]
struct UsizeLocalCollector {
    names: HashSet<String>,
}

impl Visit<'_> for UsizeLocalCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let Some(name) = simple_pattern_name(&local.pat) {
            let typed = matches!(&local.pat, syn::Pat::Type(typed)
                if matches!(typed.ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("usize")));
            let inferred = local
                .init
                .as_ref()
                .is_some_and(|init| match init.expr.as_ref() {
                    syn::Expr::Lit(literal) => matches!(&literal.lit, syn::Lit::Int(value)
                    if value.suffix() == "usize"
                        || (name.starts_with("sifr_generated_count")
                            && value.base10_digits() == "0")),
                    syn::Expr::MethodCall(call) => matches!(
                        call.method.to_string().as_str(),
                        "len" | "clamp_slice_bound"
                    ),
                    syn::Expr::Path(path) => path
                        .path
                        .get_ident()
                        .is_some_and(|source| self.names.contains(&source.to_string())),
                    _ => false,
                });
            if typed || inferred {
                self.names.insert(name);
            }
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

struct UsizeCounterRewriter<'names> {
    names: &'names HashSet<String>,
}

impl VisitMut for UsizeCounterRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::Binary(binary) = expression else {
            return;
        };
        if !matches!(binary.op, syn::BinOp::AddAssign(_))
            || !matches!(binary.left.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name| self.names.contains(&name.to_string())))
            || !matches!(binary.right.as_ref(), syn::Expr::Lit(literal)
                if matches!(&literal.lit, syn::Lit::Int(value)
                    if value.base10_digits() == "1"))
        {
            return;
        }
        let left = binary.left.as_ref();
        *expression = syn::parse_quote!(#left = (#left).saturating_add(1usize));
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn rewrite_identity_string_concat(
    expression: &mut syn::Expr,
    owned: &HashSet<String>,
    borrowed: &HashSet<String>,
) -> bool {
    let syn::Expr::Block(block) = expression else {
        return false;
    };
    let Some(syn::Stmt::Local(initial)) = block.block.stmts.first() else {
        return false;
    };
    let Some(buffer) = simple_pattern_name(&initial.pat) else {
        return false;
    };
    let Some(initializer) = &initial.init else {
        return false;
    };
    if !is_string_buffer_initializer(&initializer.expr) {
        return false;
    }
    let Some(syn::Stmt::Expr(syn::Expr::Path(tail), None)) = block.block.stmts.last() else {
        return false;
    };
    if !tail.path.is_ident(&buffer) {
        return false;
    }
    let mut value = None;
    for statement in &block.block.stmts[1..block.block.stmts.len() - 1] {
        let syn::Stmt::Expr(syn::Expr::MethodCall(push), Some(_)) = statement else {
            return false;
        };
        if push.method != "push_str"
            || push.args.len() != 1
            || !matches!(push.receiver.as_ref(), syn::Expr::Path(path) if path.path.is_ident(&buffer))
        {
            return false;
        }
        let Some(argument) = push.args.first() else {
            return false;
        };
        if matches!(argument, syn::Expr::Lit(literal)
            if matches!(&literal.lit, syn::Lit::Str(text) if text.value().is_empty()))
        {
            continue;
        }
        if value.is_some() {
            return false;
        }
        value = copied_string_from_as_str(argument, owned, borrowed);
        if value.is_none() {
            return false;
        }
    }
    let Some(value) = value else {
        return false;
    };
    *expression = value;
    true
}

fn is_string_buffer_initializer(expression: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    let syn::Expr::Path(path) = call.func.as_ref() else {
        return false;
    };
    let segments = path.path.segments.iter().collect::<Vec<_>>();
    segments.len() == 2
        && segments[0].ident == "String"
        && matches!(
            segments[1].ident.to_string().as_str(),
            "new" | "with_capacity"
        )
}

fn copied_string_from_as_str(
    expression: &syn::Expr,
    owned: &HashSet<String>,
    borrowed: &HashSet<String>,
) -> Option<syn::Expr> {
    if let syn::Expr::Path(path) = expression
        && path.qself.is_none()
        && path.path.segments.len() == 1
        && borrowed.contains(&path.path.segments[0].ident.to_string())
    {
        let receiver = syn::Expr::Path(path.clone());
        return Some(syn::parse_quote!(#receiver.to_string()));
    }
    let syn::Expr::MethodCall(as_str) = expression else {
        return None;
    };
    if as_str.method != "as_str" || !as_str.args.is_empty() {
        return None;
    }
    match as_str.receiver.as_ref() {
        syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => {
            let receiver = syn::Expr::Path(path.clone());
            let name = path.path.segments.first()?.ident.to_string();
            if owned.contains(&name) {
                Some(syn::parse_quote!(#receiver.clone()))
            } else if borrowed.contains(&name) {
                Some(syn::parse_quote!(#receiver.to_string()))
            } else {
                None
            }
        }
        syn::Expr::MethodCall(clone) if clone.method == "clone" && clone.args.is_empty() => {
            Some(syn::Expr::MethodCall(clone.clone()))
        }
        _ => None,
    }
}

fn type_is_owned_string(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path) if path.qself.is_none() && path.path.is_ident("String"))
}

fn type_is_option_string(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    segment.ident == "Option"
        && matches!(arguments.args.first(), Some(syn::GenericArgument::Type(inner)) if type_is_owned_string(inner))
}

fn type_is_sifr_int(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path)
        if path.path.segments.last().is_some_and(|segment| segment.ident == "SifrInt"))
}
