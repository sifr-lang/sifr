pub(super) fn rewrite_owned_string_clones(signature: &syn::Signature, body: &mut syn::Block) {
    if signature
        .receiver()
        .is_some_and(|receiver| matches!(receiver.kind, syn::ReceiverKind::Reference(..)))
    {
        SharedSelfBorrowRewriter.visit_block_mut(body);
    }
    let borrowed_parameters = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            matches!(parameter.ty.as_ref(), syn::Type::Reference(_))
                .then(|| simple_pattern_name(&parameter.pat))
                .flatten()
        })
        .collect::<HashSet<_>>();
    let mut owned_strings = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            if !type_is_owned_string(&parameter.ty) {
                return None;
            }
            simple_pattern_name(&parameter.pat)
        })
        .collect::<HashSet<_>>();
    let mut collector = OwnedStringLocalCollector::default();
    collector.visit_block(body);
    owned_strings.extend(collector.names);
    owned_strings.retain(|name| !borrowed_parameters.contains(name));
    OwnedStringCloneRewriter {
        names: &owned_strings,
    }
    .visit_block_mut(body);
    TypedStringInitializerRewriter {
        borrowed_roots: &borrowed_parameters,
    }
    .visit_block_mut(body);

    let mut optional_strings = OptionStringLocalCollector::default();
    optional_strings.visit_block(body);
    OwnedOptionStringIdentityRewriter {
        names: &optional_strings.names,
    }
    .visit_block_mut(body);

    let mut owned_vectors = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            (type_is_owned_vector(&parameter.ty))
                .then(|| simple_pattern_name(&parameter.pat))
                .flatten()
        })
        .collect::<HashSet<_>>();
    let mut vector_collector = OwnedVectorLocalCollector::default();
    vector_collector.visit_block(body);
    owned_vectors.extend(vector_collector.names);
    OwnedVectorCloneRewriter {
        names: &owned_vectors,
    }
    .visit_block_mut(body);

    let mut sifr_ints = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            type_is_sifr_int(&parameter.ty)
                .then(|| simple_pattern_name(&parameter.pat))
                .flatten()
        })
        .collect::<HashSet<_>>();
    let mut int_collector = SifrIntLocalCollector::default();
    int_collector.visit_block(body);
    sifr_ints.extend(int_collector.names);
    SifrIntOperationRewriter { names: &sifr_ints }.visit_block_mut(body);
    rewrite_residual_typed_calls(body, &sifr_ints);

    BorrowedCopyUnionCloneRewriter {
        borrowed_roots: &borrowed_parameters,
    }
    .visit_block_mut(body);

    let mut borrowed_slice_bindings = BorrowedSliceBindingCollector::default();
    borrowed_slice_bindings.visit_block(body);
    BorrowedBindingReferenceRewriter {
        names: &borrowed_slice_bindings.names,
    }
    .visit_block_mut(body);

    let borrowed_strings = signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let syn::FnArg::Typed(parameter) = argument else {
                return None;
            };
            matches!(parameter.ty.as_ref(), syn::Type::Reference(reference)
                if matches!(reference.elem.as_ref(), syn::Type::Path(path)
                    if path.path.is_ident("str")))
            .then(|| simple_pattern_name(&parameter.pat))
            .flatten()
        })
        .collect::<HashSet<_>>();
    BorrowedBindingReferenceRewriter {
        names: &borrowed_strings,
    }
    .visit_block_mut(body);

    DoubleReferenceCloneFromRewriter {
        borrowed: &borrowed_parameters,
    }
    .visit_block_mut(body);

    let mut copy_sources = CopyVectorSourceCollector::default();
    copy_sources.visit_block(body);
    CopyIteratorRewriter {
        sources: &copy_sources.sources,
    }
    .visit_block_mut(body);

    let mut generic_string_owners = GenericStringOwnerCollector::default();
    generic_string_owners.visit_block(body);
    GenericStringFieldConversionRewriter {
        owners: &generic_string_owners.names,
    }
    .visit_block_mut(body);
}

struct DoubleReferenceCloneFromRewriter<'names> {
    borrowed: &'names HashSet<String>,
}

impl VisitMut for DoubleReferenceCloneFromRewriter<'_> {
    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        visit_mut::visit_expr_method_call_mut(self, call);
        if call.method != "clone_from" || call.args.len() != 1 {
            return;
        }
        let Some(syn::Expr::Reference(reference)) = call.args.first() else {
            return;
        };
        if reference.mutability.is_none()
            && matches!(reference.expr.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.borrowed.contains(&name.to_string())))
        {
            call.args[0] = reference.expr.as_ref().clone();
        }
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

#[derive(Default)]
struct GenericStringOwnerCollector {
    names: HashSet<String>,
}

impl Visit<'_> for GenericStringOwnerCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && type_has_string_argument(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.names.insert(name);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

fn type_has_string_argument(ty: &syn::Type) -> bool {
    let syn::Type::Path(path) = ty else {
        return false;
    };
    path.path.segments.iter().any(|segment| {
        matches!(&segment.arguments, syn::PathArguments::AngleBracketed(arguments)
            if arguments.args.iter().any(|argument|
                matches!(argument, syn::GenericArgument::Type(inner)
                    if type_is_owned_string(inner))))
    })
}

struct GenericStringFieldConversionRewriter<'names> {
    owners: &'names HashSet<String>,
}

impl VisitMut for GenericStringFieldConversionRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(conversion) = expression else {
            return;
        };
        if !matches!(conversion.method.to_string().as_str(), "clone" | "to_owned" | "to_string")
            || !conversion.args.is_empty()
            || !matches!(conversion.receiver.as_ref(), syn::Expr::Field(field)
                if matches!(field.base.as_ref(), syn::Expr::Path(path)
                    if path.path.get_ident().is_some_and(|name|
                        self.owners.contains(&name.to_string()))))
        {
            return;
        }
        *expression = conversion.receiver.as_ref().clone();
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
struct BorrowedSliceBindingCollector {
    names: HashSet<String>,
}

impl Visit<'_> for BorrowedSliceBindingCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        let borrowed_source = local.init.as_ref().is_some_and(|init| {
            matches!(init.expr.as_ref(), syn::Expr::MethodCall(call)
                if call.method == "as_slice" && call.args.is_empty())
                || matches!(init.expr.as_ref(), syn::Expr::Path(path)
                    if path.path.get_ident().is_some_and(|name|
                        self.names.contains(&name.to_string())))
        });
        if borrowed_source {
            collect_owned_pattern_names(&local.pat, &mut self.names);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

struct BorrowedBindingReferenceRewriter<'names> {
    names: &'names HashSet<String>,
}

impl VisitMut for BorrowedBindingReferenceRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if let syn::Expr::Reference(reference) = expression
            && reference.mutability.is_none()
            && matches!(reference.expr.as_ref(), syn::Expr::Path(path)
                if path.path.get_ident().is_some_and(|name|
                    self.names.contains(&name.to_string())))
        {
            *expression = reference.expr.as_ref().clone();
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
struct OwnedStringLocalCollector {
    names: HashSet<String>,
    tuple_string_fields: HashMap<String, Vec<bool>>,
}

impl Visit<'_> for OwnedStringLocalCollector {
    fn visit_local(&mut self, local: &syn::Local) {
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
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
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
            mapper.body = Box::new(syn::parse_quote!(#binding));
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
}

impl VisitMut for OwnedStringCloneRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if rewrite_identity_string_concat(expression, self.names) {
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
struct OwnedVectorLocalCollector {
    names: HashSet<String>,
}

impl Visit<'_> for OwnedVectorLocalCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && type_is_owned_vector(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.names.insert(name);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

struct OwnedVectorCloneRewriter<'names> {
    names: &'names HashSet<String>,
}

impl VisitMut for OwnedVectorCloneRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        let syn::Expr::MethodCall(call) = expression else {
            return;
        };
        if call.method == "to_vec"
            && call.args.is_empty()
            && expression_root_name(&call.receiver).is_some_and(|name| self.names.contains(&name))
        {
            call.method = syn::Ident::new("clone", call.method.span());
            return;
        }
        if call.method != "collect"
            || !call.args.is_empty()
            || !matches!(call.turbofish.as_ref().and_then(|fish| fish.args.first()),
                Some(syn::GenericArgument::Type(syn::Type::Path(path)))
                    if path.path.segments.last().is_some_and(|segment| segment.ident == "Vec"))
        {
            return;
        }
        let syn::Expr::MethodCall(cloned) = call.receiver.as_ref() else {
            return;
        };
        let syn::Expr::MethodCall(iterated) = cloned.receiver.as_ref() else {
            return;
        };
        if cloned.method == "cloned"
            && cloned.args.is_empty()
            && iterated.method == "iter"
            && iterated.args.is_empty()
            && expression_root_name(&iterated.receiver)
                .is_some_and(|name| self.names.contains(&name))
        {
            let receiver = iterated.receiver.as_ref();
            *expression = syn::parse_quote!(#receiver.clone());
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
struct SifrIntLocalCollector {
    names: HashSet<String>,
}

impl Visit<'_> for SifrIntLocalCollector {
    fn visit_local(&mut self, local: &syn::Local) {
        if let syn::Pat::Type(typed) = &local.pat
            && type_is_sifr_int(&typed.ty)
            && let Some(name) = simple_pattern_name(&typed.pat)
        {
            self.names.insert(name);
        }
        visit::visit_local(self, local);
    }

    fn visit_item(&mut self, _item: &syn::Item) {}
}

struct SifrIntOperationRewriter<'names> {
    names: &'names HashSet<String>,
}

impl VisitMut for SifrIntOperationRewriter<'_> {
    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        if let syn::Expr::MethodCall(call) = expression
            && matches!(
                call.method.to_string().as_str(),
                "checked_to_f64" | "to_bigint"
            )
            && call.args.is_empty()
            && let syn::Expr::MethodCall(clone) = call.receiver.as_ref()
            && clone.method == "clone"
            && clone.args.is_empty()
            && expression_root_name(&clone.receiver).is_some_and(|name| self.names.contains(&name))
        {
            call.receiver = clone.receiver.clone();
            return;
        }
        let syn::Expr::Binary(binary) = expression else {
            return;
        };
        let Some((operation, method)) = exact_integer_operation(&binary.op) else {
            return;
        };
        if !expression_root_name(&binary.left).is_some_and(|name| self.names.contains(&name))
            && !expression_root_name(&binary.right).is_some_and(|name| self.names.contains(&name))
        {
            return;
        }
        let left = binary.left.clone();
        let right = binary.right.clone();
        let operation = syn::Ident::new(operation, proc_macro2::Span::call_site());
        let method = syn::Ident::new(method, proc_macro2::Span::call_site());
        *expression = syn::parse_quote!(::std::ops::#operation::#method(#left, #right));
    }

    fn visit_item_mut(&mut self, _item: &mut syn::Item) {}
}

fn rewrite_identity_string_concat(expression: &mut syn::Expr, owned: &HashSet<String>) -> bool {
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
        value = owned_string_from_as_str(argument, owned);
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

fn owned_string_from_as_str(expression: &syn::Expr, owned: &HashSet<String>) -> Option<syn::Expr> {
    let syn::Expr::MethodCall(as_str) = expression else {
        return None;
    };
    if as_str.method != "as_str" || !as_str.args.is_empty() {
        return None;
    }
    match as_str.receiver.as_ref() {
        syn::Expr::Path(path)
            if path.qself.is_none()
                && path.path.segments.len() == 1
                && path
                    .path
                    .segments
                    .first()
                    .is_some_and(|segment| owned.contains(&segment.ident.to_string())) =>
        {
            let receiver = syn::Expr::Path(path.clone());
            Some(syn::parse_quote!(#receiver.clone()))
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

fn type_is_owned_vector(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path)
        if path.qself.is_none()
            && path.path.segments.last().is_some_and(|segment| segment.ident == "Vec"))
}

fn type_is_sifr_int(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(path)
        if path.path.segments.last().is_some_and(|segment| segment.ident == "SifrInt"))
}
