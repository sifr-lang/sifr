use quote::{ToTokens, quote};
use std::collections::HashSet;
use syn::visit::{self, Visit};

use super::source_expectations::{
    refresh_const_expectations, refresh_function_expectations, refresh_struct_expectations,
};

pub(super) fn improve_generated_api_items(items: &mut [syn::Item], source: &str) {
    loop {
        let before_const = const_callable_paths(items);
        let before_eq = derived_eq_owners(items);
        improve_generated_api_items_once(items, &before_const, &before_eq, source);
        if const_callable_paths(items) == before_const && derived_eq_owners(items) == before_eq {
            break;
        }
    }
}

fn improve_generated_api_items_once(
    items: &mut [syn::Item],
    const_callables: &HashSet<String>,
    eq_owners: &HashSet<String>,
    source: &str,
) {
    let display_owners = display_implementation_owners(items);
    for item in items {
        match item {
            syn::Item::Fn(function) => improve_function_api(
                &mut function.attrs,
                &function.vis,
                &mut function.sig,
                &function.block,
                ApiContext {
                    allow_const: true,
                    owner: None,
                    owner_has_display: false,
                    const_callables,
                    source,
                },
            ),
            syn::Item::Impl(item_impl) => {
                let allow_const = item_impl.trait_.is_none();
                let owner = impl_self_type_name(item_impl.self_ty.as_ref());
                let owner_has_display = owner
                    .as_ref()
                    .is_some_and(|owner| display_owners.contains(owner));
                for impl_item in &mut item_impl.items {
                    if let syn::ImplItem::Fn(method) = impl_item {
                        improve_function_api(
                            &mut method.attrs,
                            &method.vis,
                            &mut method.sig,
                            &method.block,
                            ApiContext {
                                allow_const,
                                owner: owner.as_deref(),
                                owner_has_display,
                                const_callables,
                                source,
                            },
                        );
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &mut module.content {
                    improve_generated_api_items_once(nested, const_callables, eq_owners, source);
                }
            }
            syn::Item::Struct(item_struct) => {
                refresh_struct_expectations(&mut item_struct.attrs, &item_struct.fields);
                let generic_types = generic_type_names(&item_struct.generics);
                add_eq_to_eligible_derives(
                    &mut item_struct.attrs,
                    item_struct.fields.iter().map(|field| &field.ty),
                    eq_owners,
                    &generic_types,
                );
            }
            syn::Item::Enum(item_enum) => {
                let generic_types = generic_type_names(&item_enum.generics);
                add_eq_to_eligible_derives(
                    &mut item_enum.attrs,
                    item_enum
                        .variants
                        .iter()
                        .flat_map(|variant| variant.fields.iter().map(|field| &field.ty)),
                    eq_owners,
                    &generic_types,
                );
            }
            syn::Item::Trait(item_trait) => add_trait_method_contracts(item_trait),
            syn::Item::Const(item_const) => {
                refresh_const_expectations(&mut item_const.attrs, &item_const.expr);
            }
            _ => {}
        }
    }
}

fn const_callable_paths(items: &[syn::Item]) -> HashSet<String> {
    let mut paths = HashSet::new();
    for item in items {
        match item {
            syn::Item::Fn(function) if function.sig.constness.is_some() => {
                paths.insert(function.sig.ident.to_string());
            }
            syn::Item::Impl(item_impl) if item_impl.trait_.is_none() => {
                if let Some(owner) = impl_self_type_name(item_impl.self_ty.as_ref()) {
                    for impl_item in &item_impl.items {
                        if let syn::ImplItem::Fn(method) = impl_item
                            && method.sig.constness.is_some()
                        {
                            paths.insert(format!("{owner}::{}", method.sig.ident));
                        }
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    paths.extend(const_callable_paths(nested));
                }
            }
            _ => {}
        }
    }
    paths
}

#[derive(Clone, Copy)]
struct ApiContext<'context> {
    allow_const: bool,
    owner: Option<&'context str>,
    owner_has_display: bool,
    const_callables: &'context HashSet<String>,
    source: &'context str,
}

fn improve_function_api(
    attrs: &mut Vec<syn::Attribute>,
    visibility: &syn::Visibility,
    signature: &mut syn::Signature,
    body: &syn::Block,
    context: ApiContext<'_>,
) {
    replace_unused_parameters(signature, body);
    let rendered_body_lines = rendered_function_body_lines(body, context.source);
    add_source_shape_expectations(attrs, signature, rendered_body_lines);
    refresh_function_expectations(attrs, signature, body, context.owner_has_display);
    if matches!(visibility, syn::Visibility::Public(_)) {
        if !matches!(signature.output, syn::ReturnType::Default)
            && !returns_result(signature)
            && !attrs.iter().any(|attr| attr.path().is_ident("must_use"))
        {
            attrs.push(syn::parse_quote!(#[must_use]));
        }
        if returns_result(signature) && !attrs.iter().any(is_errors_doc_attribute) {
            attrs.push(syn::parse_quote!(#[doc = "# Errors"]));
            attrs.push(
                syn::parse_quote!(#[doc = "Returns the typed error produced by this operation."]),
            );
        }
    }
    if context.allow_const
        && signature.constness.is_none()
        && signature.asyncness.is_none()
        && block_is_const_compatible(
            body,
            context.owner,
            context.const_callables,
            signature.inputs.iter().any(
                |argument| matches!(argument, syn::FnArg::Receiver(receiver) if matches!(receiver.kind, syn::ReceiverKind::Reference(..))),
            ),
        )
    {
        signature.constness = Some(syn::token::Const::default());
    }
}

fn replace_unused_parameters(signature: &mut syn::Signature, body: &syn::Block) {
    let mut references = BodyReferenceCollector::default();
    references.visit_block(body);
    for input in &mut signature.inputs {
        let syn::FnArg::Typed(parameter) = input else {
            continue;
        };
        let syn::Pat::Ident(binding) = parameter.pat.as_ref() else {
            continue;
        };
        let name = binding.ident.to_string();
        if !references.names.contains(&name) {
            *parameter.pat = syn::parse_quote!(_);
        }
    }
}

#[derive(Default)]
struct BodyReferenceCollector {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for BodyReferenceCollector {
    fn visit_expr_path(&mut self, path: &'ast syn::ExprPath) {
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && let Some(segment) = path.path.segments.first()
        {
            self.names.insert(segment.ident.to_string());
        }
        visit::visit_expr_path(self, path);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        collect_macro_identifier_tokens(rust_macro.tokens.clone(), &mut self.names);
        collect_format_capture_identifiers(rust_macro, &mut self.names);
        visit::visit_macro(self, rust_macro);
    }
}

fn collect_macro_identifier_tokens(tokens: proc_macro2::TokenStream, names: &mut HashSet<String>) {
    for token in tokens {
        match token {
            proc_macro2::TokenTree::Ident(identifier) => {
                names.insert(identifier.to_string());
            }
            proc_macro2::TokenTree::Group(group) => {
                collect_macro_identifier_tokens(group.stream(), names);
            }
            _ => {}
        }
    }
}

fn collect_format_capture_identifiers(rust_macro: &syn::Macro, names: &mut HashSet<String>) {
    let Some(macro_name) = rust_macro
        .path
        .segments
        .last()
        .map(|segment| &segment.ident)
    else {
        return;
    };
    let format_index = match macro_name.to_string().as_str() {
        "format" | "print" | "println" | "eprint" | "eprintln" => 0,
        "write" | "writeln" => 1,
        _ => return,
    };
    let Ok(arguments) = rust_macro.parse_body_with(
        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
    ) else {
        return;
    };
    let Some(syn::Expr::Lit(format_expression)) = arguments.iter().nth(format_index) else {
        return;
    };
    let syn::Lit::Str(format_literal) = &format_expression.lit else {
        return;
    };
    for field in format_literal.value().split(['{', '}']).skip(1).step_by(2) {
        let name = field.split(':').next().unwrap_or_default();
        if !name.is_empty()
            && name
                .chars()
                .all(|character| character == '_' || character.is_ascii_alphanumeric())
        {
            names.insert(name.to_string());
        }
    }
}

fn rendered_function_body_lines(body: &syn::Block, source: &str) -> usize {
    let start = body.brace_token.span.open().start().line;
    let end = body.brace_token.span.close().end().line;
    if end <= start + 1 {
        return usize::from(end == start && source.lines().nth(start.saturating_sub(1)).is_some());
    }
    count_code_lines(
        source
            .lines()
            .skip(start)
            .take(end.saturating_sub(start + 1)),
    )
}

fn count_code_lines<'line>(lines: impl Iterator<Item = &'line str>) -> usize {
    let mut in_block_comment = false;
    let mut count = 0;
    for mut line in lines {
        let mut has_code = false;
        loop {
            line = line.trim_start();
            if line.is_empty() {
                break;
            }
            if in_block_comment {
                let Some(end) = line.find("*/") else {
                    break;
                };
                line = &line[end + 2..];
                in_block_comment = false;
                continue;
            }
            let block_comment = line.find("/*").unwrap_or(line.len());
            let line_comment = line.find("//").unwrap_or(line.len());
            has_code |= block_comment > 0 && line_comment > 0;
            if block_comment < line_comment {
                line = &line[block_comment + 2..];
                in_block_comment = true;
                continue;
            }
            break;
        }
        count += usize::from(has_code);
    }
    count
}

fn add_source_shape_expectations(
    attrs: &mut Vec<syn::Attribute>,
    signature: &syn::Signature,
    rendered_body_lines: usize,
) {
    attrs.retain(|attribute| !is_generated_source_shape_expectation(attribute));
    if signature.inputs.len() > 7
        && !attrs.iter().any(|attribute| {
            attribute.path().is_ident("expect")
                && attribute
                    .meta
                    .to_token_stream()
                    .to_string()
                    .contains("too_many_arguments")
        })
    {
        attrs.push(syn::parse_quote!(
            #[expect(
                clippy::too_many_arguments,
                reason = "generated signature preserves the typed Sifr callable contract"
            )]
        ));
    }
    if rendered_body_lines > 100
        && !attrs.iter().any(|attribute| {
            attribute.path().is_ident("expect")
                && attribute
                    .meta
                    .to_token_stream()
                    .to_string()
                    .contains("too_many_lines")
        })
    {
        attrs.push(syn::parse_quote!(
            #[expect(
                clippy::too_many_lines,
                reason = "one generated Rust function preserves one typed Sifr function"
            )]
        ));
    }
}

fn display_implementation_owners(items: &[syn::Item]) -> HashSet<String> {
    let mut owners = HashSet::new();
    for item in items {
        match item {
            syn::Item::Impl(item_impl)
                if item_impl.trait_.as_ref().is_some_and(|(path, _)| {
                    path.segments
                        .last()
                        .is_some_and(|segment| segment.ident == "Display")
                }) =>
            {
                if let Some(owner) = impl_self_type_name(&item_impl.self_ty) {
                    owners.insert(owner);
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    owners.extend(display_implementation_owners(nested));
                }
            }
            _ => {}
        }
    }
    owners
}

fn add_trait_method_contracts(item_trait: &mut syn::ItemTrait) {
    for item in &mut item_trait.items {
        let syn::TraitItem::Fn(method) = item else {
            continue;
        };
        if signature_returns_self(&method.sig)
            && !method
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("must_use"))
        {
            method.attrs.push(syn::parse_quote!(#[must_use]));
        }
    }
}

fn signature_returns_self(signature: &syn::Signature) -> bool {
    matches!(&signature.output,
        syn::ReturnType::Type(_, ty)
            if matches!(ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("Self")))
}

fn is_generated_source_shape_expectation(attribute: &syn::Attribute) -> bool {
    if !attribute.path().is_ident("expect") {
        return false;
    }
    let rendered = attribute.meta.to_token_stream().to_string();
    rendered.contains("generated signature preserves the typed Sifr callable contract")
        || rendered.contains("one generated Rust function preserves one typed Sifr function")
}

fn is_errors_doc_attribute(attr: &syn::Attribute) -> bool {
    let syn::Meta::NameValue(meta) = &attr.meta else {
        return false;
    };
    let syn::Expr::Lit(expression) = &meta.value else {
        return false;
    };
    matches!(&expression.lit, syn::Lit::Str(value) if value.value().contains("# Errors"))
}

fn returns_result(signature: &syn::Signature) -> bool {
    let syn::ReturnType::Type(_, ty) = &signature.output else {
        return false;
    };
    matches!(ty.as_ref(), syn::Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Result"))
}

fn block_is_const_compatible(
    block: &syn::Block,
    owner: Option<&str>,
    const_callables: &HashSet<String>,
    has_borrowed_self: bool,
) -> bool {
    let mut checker = ConstCompatibilityChecker {
        compatible: true,
        owner,
        const_callables,
        has_borrowed_self,
    };
    checker.visit_block(block);
    checker.compatible
}

struct ConstCompatibilityChecker<'scope> {
    compatible: bool,
    owner: Option<&'scope str>,
    const_callables: &'scope HashSet<String>,
    has_borrowed_self: bool,
}

impl<'ast> Visit<'ast> for ConstCompatibilityChecker<'_> {
    fn visit_macro(&mut self, _rust_macro: &'ast syn::Macro) {
        self.compatible = false;
    }

    fn visit_expr_method_call(&mut self, _call: &'ast syn::ExprMethodCall) {
        self.compatible = false;
    }

    fn visit_expr_binary(&mut self, _expression: &'ast syn::ExprBinary) {
        self.compatible = false;
    }

    fn visit_expr_unary(&mut self, _expression: &'ast syn::ExprUnary) {
        self.compatible = false;
    }

    fn visit_expr_field(&mut self, expression: &'ast syn::ExprField) {
        if !self.has_borrowed_self || !expression_is_rooted_in_self(&expression.base) {
            self.compatible = false;
            return;
        }
        visit::visit_expr_field(self, expression);
    }

    fn visit_expr_index(&mut self, _expression: &'ast syn::ExprIndex) {
        self.compatible = false;
    }

    fn visit_expr_call(&mut self, call: &'ast syn::ExprCall) {
        let is_const_constructor = matches!(call.func.as_ref(), syn::Expr::Path(path)
            if path.path.segments.last().is_some_and(|segment| segment.ident == "from_i64")
                && path.path.segments.iter().rev().nth(1).is_some_and(|segment| segment.ident == "SifrInt"))
            || matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.last().is_some_and(|segment| segment.ident == "new")
                    && path.path.segments.iter().rev().nth(1).is_some_and(|segment| matches!(segment.ident.to_string().as_str(), "String" | "Vec")))
            || matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.last().is_some_and(|segment| segment.ident.to_string().chars().next().is_some_and(char::is_uppercase)));
        let is_known_external_const = matches!(call.func.as_ref(), syn::Expr::Path(path)
        if path.path.segments.last().is_some_and(|segment| matches!(
            segment.ident.to_string().as_str(),
            "isnan" | "isinf" | "isfinite" | "signbit"
        )));
        let known_const = match call.func.as_ref() {
            syn::Expr::Path(path) => {
                let segments = path.path.segments.iter().collect::<Vec<_>>();
                match segments.as_slice() {
                    [function] => self.const_callables.contains(&function.ident.to_string()),
                    [.., owner, function] => {
                        let owner_name = if owner.ident == "Self" {
                            self.owner.unwrap_or("Self").to_string()
                        } else {
                            owner.ident.to_string()
                        };
                        self.const_callables
                            .contains(&format!("{owner_name}::{}", function.ident))
                    }
                    _ => false,
                }
            }
            _ => false,
        };
        if !is_const_constructor && !is_known_external_const && !known_const {
            self.compatible = false;
        }
        visit::visit_expr_call(self, call);
    }
}

fn expression_is_rooted_in_self(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Path(path) => path.qself.is_none() && path.path.is_ident("self"),
        syn::Expr::Field(field) => expression_is_rooted_in_self(&field.base),
        syn::Expr::Paren(paren) => expression_is_rooted_in_self(&paren.expr),
        _ => false,
    }
}

fn add_eq_to_eligible_derives<'type_ref>(
    attrs: &mut [syn::Attribute],
    field_types: impl Iterator<Item = &'type_ref syn::Type>,
    eq_owners: &HashSet<String>,
    generic_types: &HashSet<String>,
) {
    let field_types = field_types.collect::<Vec<_>>();
    if field_types
        .iter()
        .any(|ty| !type_is_proven_eq(ty, eq_owners, generic_types))
    {
        return;
    }
    for attr in attrs {
        let syn::Meta::List(meta) = &mut attr.meta else {
            continue;
        };
        if !meta.path.is_ident("derive") {
            continue;
        }
        let tokens = meta.tokens.to_string();
        let has_partial_eq = tokens
            .split(|character: char| !character.is_alphanumeric() && character != '_')
            .any(|name| name == "PartialEq");
        let has_eq = tokens
            .split(|character: char| !character.is_alphanumeric() && character != '_')
            .any(|name| name == "Eq");
        if has_partial_eq && !has_eq {
            meta.tokens.extend(quote!(, Eq));
        }
    }
}

fn type_is_proven_eq(
    ty: &syn::Type,
    eq_owners: &HashSet<String>,
    generic_types: &HashSet<String>,
) -> bool {
    match ty {
        syn::Type::Array(array) => type_is_proven_eq(&array.elem, eq_owners, generic_types),
        syn::Type::Paren(paren) => type_is_proven_eq(&paren.elem, eq_owners, generic_types),
        syn::Type::Reference(reference) => {
            type_is_proven_eq(&reference.elem, eq_owners, generic_types)
        }
        syn::Type::Slice(slice) => type_is_proven_eq(&slice.elem, eq_owners, generic_types),
        syn::Type::Tuple(tuple) => tuple
            .elems
            .iter()
            .all(|element| type_is_proven_eq(element, eq_owners, generic_types)),
        syn::Type::Path(path) if path.qself.is_none() => {
            let Some(segment) = path.path.segments.last() else {
                return false;
            };
            let owner = segment.ident.to_string();
            let known_leaf = matches!(
                owner.as_str(),
                "bool"
                    | "char"
                    | "i8"
                    | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "isize"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "usize"
                    | "str"
                    | "String"
                    | "SifrInt"
            ) || eq_owners.contains(&owner)
                || generic_types.contains(&owner);
            if known_leaf {
                return true;
            }
            if !matches!(
                owner.as_str(),
                "Arc"
                    | "BTreeMap"
                    | "BTreeSet"
                    | "Box"
                    | "HashMap"
                    | "HashSet"
                    | "Option"
                    | "PhantomData"
                    | "Rc"
                    | "Result"
                    | "Vec"
                    | "VecDeque"
            ) {
                return false;
            }
            let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
                return false;
            };
            arguments.args.iter().all(|argument| match argument {
                syn::GenericArgument::Type(ty) => type_is_proven_eq(ty, eq_owners, generic_types),
                syn::GenericArgument::Lifetime(_) => true,
                _ => false,
            })
        }
        _ => false,
    }
}

fn generic_type_names(generics: &syn::Generics) -> HashSet<String> {
    generics
        .type_params()
        .map(|parameter| parameter.ident.to_string())
        .collect()
}

fn derived_eq_owners(items: &[syn::Item]) -> HashSet<String> {
    let mut owners = HashSet::new();
    for item in items {
        match item {
            syn::Item::Struct(struct_) if attributes_derive_eq(&struct_.attrs) => {
                owners.insert(struct_.ident.to_string());
            }
            syn::Item::Enum(enum_) if attributes_derive_eq(&enum_.attrs) => {
                owners.insert(enum_.ident.to_string());
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    owners.extend(derived_eq_owners(nested));
                }
            }
            _ => {}
        }
    }
    owners
}

fn attributes_derive_eq(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|attribute| {
        let syn::Meta::List(meta) = &attribute.meta else {
            return false;
        };
        meta.path.is_ident("derive")
            && meta
                .tokens
                .to_string()
                .split(|character: char| !character.is_alphanumeric() && character != '_')
                .any(|name| name == "Eq")
    })
}

fn impl_self_type_name(ty: &syn::Type) -> Option<String> {
    let syn::Type::Path(path) = ty else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}
