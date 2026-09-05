use std::collections::HashMap;
use std::collections::HashSet;

use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

#[derive(Default)]
pub(crate) struct BorrowedStringSignatures {
    functions: HashMap<String, Vec<bool>>,
    methods: HashMap<String, Vec<bool>>,
    method_returns: HashMap<String, String>,
    field_types: HashMap<String, HashMap<String, String>>,
}

pub(super) fn collect_borrowed_string_params(file: &syn::File) -> BorrowedStringSignatures {
    let mut collector = BorrowedStringParamCollector::default();
    syn::visit::Visit::visit_file(&mut collector, file);
    collector.signatures
}

include!("borrowed_string_arguments/parameter_cleanup.rs");

pub(super) fn collect_owned_string_returns(file: &syn::File) -> HashSet<String> {
    let mut returns = HashSet::new();
    let mut collector = OwnedStringReturnCollector {
        returns: &mut returns,
    };
    syn::visit::Visit::visit_file(&mut collector, file);
    returns
}

struct OwnedStringReturnCollector<'returns> {
    returns: &'returns mut HashSet<String>,
}

impl syn::visit::Visit<'_> for OwnedStringReturnCollector<'_> {
    fn visit_signature(&mut self, signature: &syn::Signature) {
        if matches!(&signature.output, syn::ReturnType::Type(_, ty)
            if matches!(ty.as_ref(), syn::Type::Path(path) if path.path.is_ident("String")))
        {
            let argument_count = signature
                .inputs
                .iter()
                .filter(|argument| matches!(argument, syn::FnArg::Typed(_)))
                .count();
            self.returns
                .insert(signature_key(&signature.ident.to_string(), argument_count));
        }
        syn::visit::visit_signature(self, signature);
    }
}

pub(super) fn remove_returned_string_conversion(
    expression: &mut syn::Expr,
    returns: &HashSet<String>,
) {
    let syn::Expr::MethodCall(conversion) = expression else {
        return;
    };
    if !matches!(
        conversion.method.to_string().as_str(),
        "clone" | "to_owned" | "to_string"
    ) || !conversion.args.is_empty()
    {
        return;
    }
    let (name, argument_count) = match conversion.receiver.as_ref() {
        syn::Expr::Call(call) => {
            let syn::Expr::Path(path) = call.func.as_ref() else {
                return;
            };
            let Some(name) = path.path.segments.last() else {
                return;
            };
            (name.ident.to_string(), call.args.len())
        }
        syn::Expr::MethodCall(call) => (call.method.to_string(), call.args.len()),
        _ => return,
    };
    if returns.contains(&signature_key(&name, argument_count)) {
        *expression = conversion.receiver.as_ref().clone();
    }
}

pub(crate) fn collect_project_borrowed_string_params(
    files: &[syn::File],
) -> BorrowedStringSignatures {
    let mut merged = BorrowedStringSignatures::default();
    for file in files {
        let BorrowedStringSignatures {
            functions,
            methods,
            method_returns,
            field_types,
        } = collect_borrowed_string_params(file);
        for (target, params) in [
            (&mut merged.functions, functions),
            (&mut merged.methods, methods),
        ] {
            for (name, params) in params {
                target
                    .entry(name)
                    .and_modify(|known: &mut Vec<bool>| {
                        if *known != params {
                            known.clear();
                        }
                    })
                    .or_insert(params);
            }
        }
        merge_unique_values(&mut merged.method_returns, method_returns);
        for (owner, fields) in field_types {
            merged
                .field_types
                .entry(owner)
                .and_modify(|known| {
                    if *known != fields {
                        known.clear();
                    }
                })
                .or_insert(fields);
        }
    }
    merged
}

pub(crate) fn rewrite_project_borrowed_string_literals(
    file: &mut syn::File,
    signatures: &BorrowedStringSignatures,
) {
    ProjectBorrowedStringLiteralRewriter {
        signatures,
        owner: None,
    }
    .visit_file_mut(file);
}

struct ProjectBorrowedStringLiteralRewriter<'signatures> {
    signatures: &'signatures BorrowedStringSignatures,
    owner: Option<String>,
}

impl VisitMut for ProjectBorrowedStringLiteralRewriter<'_> {
    fn visit_item_impl_mut(&mut self, implementation: &mut syn::ItemImpl) {
        let previous = self.owner.replace(type_owner_name(&implementation.self_ty));
        visit_mut::visit_item_impl_mut(self, implementation);
        self.owner = previous;
    }

    fn visit_item_fn_mut(&mut self, function: &mut syn::ItemFn) {
        rewrite_lexical_borrowed_string_arguments(
            &function.sig,
            &mut function.block,
            self.signatures,
            None,
        );
        visit_mut::visit_item_fn_mut(self, function);
    }

    fn visit_impl_item_fn_mut(&mut self, function: &mut syn::ImplItemFn) {
        rewrite_lexical_borrowed_string_arguments(
            &function.sig,
            &mut function.block,
            self.signatures,
            self.owner.as_deref(),
        );
        visit_mut::visit_impl_item_fn_mut(self, function);
    }

    fn visit_expr_mut(&mut self, expression: &mut syn::Expr) {
        visit_mut::visit_expr_mut(self, expression);
        rewrite_borrowed_string_literal_arguments(expression, self.signatures);
    }
}

pub(super) fn rewrite_lexical_borrowed_string_arguments(
    signature: &syn::Signature,
    body: &mut syn::Block,
    signatures: &BorrowedStringSignatures,
    owner: Option<&str>,
) {
    let mut rewriter = LexicalBorrowedStringArgumentRewriter {
        signatures,
        bindings: HashMap::new(),
        types: HashMap::new(),
        owner: owner.map(str::to_owned),
    };
    for argument in &signature.inputs {
        if let syn::FnArg::Typed(parameter) = argument
            && let Some(name) = simple_pattern_name(&parameter.pat)
        {
            rewriter
                .bindings
                .insert(name.clone(), string_binding_kind(&parameter.ty));
            if let Some(owner) = type_path_name(&parameter.ty) {
                rewriter.types.insert(name, owner);
            }
        }
    }
    rewriter.visit_block_mut(body);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StringBindingKind {
    Owned,
    Borrowed,
    Other,
}

struct LexicalBorrowedStringArgumentRewriter<'signatures> {
    signatures: &'signatures BorrowedStringSignatures,
    bindings: HashMap<String, StringBindingKind>,
    types: HashMap<String, String>,
    owner: Option<String>,
}

impl VisitMut for LexicalBorrowedStringArgumentRewriter<'_> {
    fn visit_block_mut(&mut self, block: &mut syn::Block) {
        let outer = self.bindings.clone();
        let outer_types = self.types.clone();
        visit_mut::visit_block_mut(self, block);
        self.bindings = outer;
        self.types = outer_types;
    }

    fn visit_local_mut(&mut self, local: &mut syn::Local) {
        if let Some(initializer) = &mut local.init {
            self.visit_expr_mut(&mut initializer.expr);
            if let Some((_, diverge)) = &mut initializer.diverge {
                self.visit_expr_mut(diverge);
            }
        }
        let Some(name) = simple_pattern_name(&local.pat) else {
            return;
        };
        self.types.remove(&name);
        let owner = match &local.pat {
            syn::Pat::Type(typed) => type_path_name(&typed.ty),
            _ => local
                .init
                .as_ref()
                .and_then(|init| self.expression_type_owner(&init.expr)),
        };
        if let Some(owner) = owner {
            self.types.insert(name.clone(), owner);
        }
        let kind = match &local.pat {
            syn::Pat::Type(typed) => string_binding_kind(&typed.ty),
            _ => local
                .init
                .as_ref()
                .map_or(StringBindingKind::Other, |init| {
                    self.expression_string_kind(&init.expr)
                }),
        };
        self.bindings.insert(name, kind);
    }

    fn visit_expr_call_mut(&mut self, call: &mut syn::ExprCall) {
        self.visit_expr_mut(&mut call.func);
        for argument in &mut call.args {
            self.visit_expr_mut(argument);
        }
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return;
        };
        if path.qself.is_some() {
            return;
        }
        let parameters = if let Some(function) = path.path.get_ident() {
            self.signatures
                .functions
                .get(&signature_key(&function.to_string(), call.args.len()))
        } else {
            let mut tail = path.path.segments.iter().rev();
            let method = tail.next().map(|segment| segment.ident.to_string());
            let owner = tail.next().map(|segment| {
                if segment.ident == "Self" {
                    self.owner.clone().unwrap_or_else(|| "Self".to_string())
                } else {
                    segment.ident.to_string()
                }
            });
            owner.zip(method).and_then(|(owner, method)| {
                self.signatures
                    .methods
                    .get(&method_signature_key(&owner, &method, call.args.len()))
            })
        };
        let Some(parameters) = parameters.filter(|params| !params.is_empty()) else {
            return;
        };
        for (argument, borrowed) in call.args.iter_mut().zip(parameters) {
            if *borrowed {
                self.rewrite_argument(argument);
            }
        }
    }

    fn visit_expr_method_call_mut(&mut self, call: &mut syn::ExprMethodCall) {
        self.visit_expr_mut(&mut call.receiver);
        for argument in &mut call.args {
            self.visit_expr_mut(argument);
        }
        if call.method == "to_string"
            && call.args.is_empty()
            && self.receiver_is_owned_string_field(&call.receiver)
        {
            call.method = syn::Ident::new("clone", call.method.span());
            return;
        }
        let Some(owner) = self.expression_type_owner(&call.receiver) else {
            return;
        };
        let key = method_signature_key(&owner, &call.method.to_string(), call.args.len());
        let Some(parameters) = self
            .signatures
            .methods
            .get(&key)
            .filter(|params| !params.is_empty())
        else {
            return;
        };
        for (argument, borrowed) in call.args.iter_mut().zip(parameters) {
            if *borrowed {
                self.rewrite_argument(argument);
            }
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
        rust_macro.tokens = quote::quote!(#arguments);
    }
}

impl LexicalBorrowedStringArgumentRewriter<'_> {
    fn expression_type_owner(&self, expression: &syn::Expr) -> Option<String> {
        match expression {
            syn::Expr::Reference(reference) => self.expression_type_owner(&reference.expr),
            syn::Expr::Group(group) => self.expression_type_owner(&group.expr),
            syn::Expr::Paren(paren) => self.expression_type_owner(&paren.expr),
            syn::Expr::Path(path) if path.path.is_ident("self") => self.owner.clone(),
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.types.get(&name.to_string()).cloned()),
            syn::Expr::Struct(struct_) => struct_
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string()),
            syn::Expr::Call(call) => match call.func.as_ref() {
                syn::Expr::Path(path) if path.path.segments.len() > 1 => path
                    .path
                    .segments
                    .iter()
                    .rev()
                    .nth(1)
                    .map(|segment| segment.ident.to_string()),
                _ => None,
            },
            syn::Expr::Field(field) => {
                let owner = self.expression_type_owner(&field.base)?;
                let syn::Member::Named(name) = &field.member else {
                    return None;
                };
                self.signatures
                    .field_types
                    .get(&owner)
                    .and_then(|fields| fields.get(&name.to_string()))
                    .cloned()
            }
            syn::Expr::MethodCall(call) => {
                let owner = self.expression_type_owner(&call.receiver)?;
                let key = method_signature_key(&owner, &call.method.to_string(), call.args.len());
                self.signatures.method_returns.get(&key).cloned()
            }
            _ => None,
        }
    }

    fn receiver_is_owned_string_field(&self, expression: &syn::Expr) -> bool {
        let syn::Expr::Field(field) = expression else {
            return false;
        };
        let Some(owner) = self.expression_type_owner(&field.base) else {
            return false;
        };
        let syn::Member::Named(name) = &field.member else {
            return false;
        };
        self.signatures
            .field_types
            .get(&owner)
            .and_then(|fields| fields.get(&name.to_string()))
            .is_some_and(|field_type| field_type == "String")
    }

    fn rewrite_argument(&self, argument: &mut syn::Expr) {
        if let syn::Expr::Path(value) = argument
            && let Some(name) = value.path.get_ident()
            && self.bindings.get(&name.to_string()) == Some(&StringBindingKind::Owned)
        {
            let value = syn::Expr::Path(value.clone());
            *argument = syn::parse_quote!(#value.as_str());
            return;
        }
        if let syn::Expr::Path(value) = argument
            && let Some(name) = value.path.get_ident()
            && self.bindings.get(&name.to_string()) == Some(&StringBindingKind::Borrowed)
        {
            return;
        }
        let syn::Expr::Reference(reference) = argument else {
            if let syn::Expr::MethodCall(conversion) = argument
                && conversion.args.is_empty()
                && matches!(
                    conversion.method.to_string().as_str(),
                    "to_owned" | "to_string"
                )
            {
                let receiver = conversion.receiver.as_ref().clone();
                *argument = match self.expression_string_kind(&receiver) {
                    StringBindingKind::Owned => syn::parse_quote!(#receiver.as_str()),
                    StringBindingKind::Borrowed => receiver,
                    StringBindingKind::Other => return,
                };
            }
            return;
        };
        if let syn::Expr::Path(value) = reference.expr.as_ref()
            && let Some(name) = value.path.get_ident()
        {
            match self.bindings.get(&name.to_string()) {
                Some(StringBindingKind::Owned) => {
                    let value = syn::Expr::Path(value.clone());
                    *argument = syn::parse_quote!(#value.as_str());
                    return;
                }
                Some(StringBindingKind::Borrowed) => {
                    *argument = reference.expr.as_ref().clone();
                    return;
                }
                _ => {}
            }
        }
        if matches!(reference.expr.as_ref(), syn::Expr::Call(call)
            if matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.len() == 2
                    && path.path.segments[0].ident == "String"
                    && path.path.segments[1].ident == "new")
                && call.args.is_empty())
        {
            *argument = syn::parse_quote!("");
            return;
        }
        if let syn::Expr::MethodCall(conversion) = reference.expr.as_ref()
            && conversion.args.is_empty()
            && matches!(
                conversion.method.to_string().as_str(),
                "to_owned" | "to_string"
            )
            && matches!(conversion.receiver.as_ref(), syn::Expr::Lit(literal)
                if matches!(literal.lit, syn::Lit::Str(_)))
        {
            *argument = conversion.receiver.as_ref().clone();
        }
    }

    fn expression_string_kind(&self, expression: &syn::Expr) -> StringBindingKind {
        match expression {
            syn::Expr::Path(path) => path
                .path
                .get_ident()
                .and_then(|name| self.bindings.get(&name.to_string()).copied())
                .unwrap_or(StringBindingKind::Other),
            syn::Expr::Lit(literal) if matches!(literal.lit, syn::Lit::Str(_)) => {
                StringBindingKind::Borrowed
            }
            syn::Expr::MethodCall(call)
                if call.args.is_empty()
                    && matches!(call.method.to_string().as_str(), "to_owned" | "to_string")
                    && matches!(
                        self.expression_string_kind(&call.receiver),
                        StringBindingKind::Owned | StringBindingKind::Borrowed
                    ) =>
            {
                StringBindingKind::Owned
            }
            syn::Expr::MethodCall(call)
                if call.args.is_empty()
                    && call.method == "clone"
                    && self.expression_string_kind(&call.receiver) == StringBindingKind::Owned =>
            {
                StringBindingKind::Owned
            }
            syn::Expr::Call(call)
                if matches!(call.func.as_ref(), syn::Expr::Path(path)
                    if path.path.segments.len() == 2
                        && path.path.segments[0].ident == "String") =>
            {
                StringBindingKind::Owned
            }
            _ => StringBindingKind::Other,
        }
    }
}

fn string_binding_kind(ty: &syn::Type) -> StringBindingKind {
    if matches!(ty, syn::Type::Path(path) if path.path.is_ident("String")) {
        StringBindingKind::Owned
    } else if matches!(ty, syn::Type::Reference(reference)
        if matches!(reference.elem.as_ref(), syn::Type::Path(path) if path.path.is_ident("str")))
    {
        StringBindingKind::Borrowed
    } else {
        StringBindingKind::Other
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

#[derive(Default)]
struct BorrowedStringParamCollector {
    signatures: BorrowedStringSignatures,
}

impl<'ast> syn::visit::Visit<'ast> for BorrowedStringParamCollector {
    fn visit_item_struct(&mut self, item: &'ast syn::ItemStruct) {
        let fields = item
            .fields
            .iter()
            .filter_map(|field| {
                let name = field.ident.as_ref()?.to_string();
                type_path_name(&field.ty).map(|field_type| (name, field_type))
            })
            .collect::<HashMap<_, _>>();
        self.signatures
            .field_types
            .entry(item.ident.to_string())
            .and_modify(|known| {
                if *known != fields {
                    known.clear();
                }
            })
            .or_insert(fields);
        syn::visit::visit_item_struct(self, item);
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        insert_signature(
            &mut self.signatures.functions,
            signature_key(
                &function.sig.ident.to_string(),
                typed_argument_count(&function.sig),
            ),
            borrowed_string_parameters(&function.sig),
        );
        syn::visit::visit_item_fn(self, function);
    }

    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        let owner = type_owner_name(&implementation.self_ty);
        for item in &implementation.items {
            if let syn::ImplItem::Fn(method) = item {
                if let syn::ReturnType::Type(_, return_type) = &method.sig.output
                    && let Some(mut returned) = type_path_name(return_type)
                {
                    if returned == "Self" {
                        returned.clone_from(&owner);
                    }
                    let key = method_signature_key(
                        &owner,
                        &method.sig.ident.to_string(),
                        typed_argument_count(&method.sig),
                    );
                    insert_unique_value(&mut self.signatures.method_returns, key, returned);
                }
                insert_signature(
                    &mut self.signatures.methods,
                    method_signature_key(
                        &owner,
                        &method.sig.ident.to_string(),
                        typed_argument_count(&method.sig),
                    ),
                    borrowed_string_parameters(&method.sig),
                );
            }
        }
        syn::visit::visit_item_impl(self, implementation);
    }
}

fn borrowed_string_parameters(signature: &syn::Signature) -> Vec<bool> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            syn::FnArg::Typed(parameter) => Some(
                matches!(parameter.ty.as_ref(), syn::Type::Reference(reference)
                    if matches!(reference.elem.as_ref(), syn::Type::Path(path)
                        if path.path.is_ident("str"))),
            ),
            syn::FnArg::Receiver(_) => None,
        })
        .collect()
}

fn typed_argument_count(signature: &syn::Signature) -> usize {
    signature
        .inputs
        .iter()
        .filter(|argument| matches!(argument, syn::FnArg::Typed(_)))
        .count()
}

fn insert_signature(target: &mut HashMap<String, Vec<bool>>, key: String, params: Vec<bool>) {
    target
        .entry(key)
        .and_modify(|known| {
            if *known != params {
                known.clear();
            }
        })
        .or_insert(params);
}

fn insert_unique_value(target: &mut HashMap<String, String>, key: String, value: String) {
    target
        .entry(key)
        .and_modify(|known| {
            if *known != value {
                known.clear();
            }
        })
        .or_insert(value);
}

fn merge_unique_values(target: &mut HashMap<String, String>, values: HashMap<String, String>) {
    for (key, value) in values {
        insert_unique_value(target, key, value);
    }
    target.retain(|_, value| !value.is_empty());
}

pub(super) fn type_owner_name(ty: &syn::Type) -> String {
    type_path_name(ty).unwrap_or_else(|| quote::quote!(#ty).to_string())
}

fn type_path_name(ty: &syn::Type) -> Option<String> {
    match ty {
        syn::Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        syn::Type::Reference(reference) => type_path_name(&reference.elem),
        _ => None,
    }
}

fn signature_key(name: &str, argument_count: usize) -> String {
    format!("{name}#{argument_count}")
}

fn method_signature_key(owner: &str, name: &str, argument_count: usize) -> String {
    format!("{owner}::{name}#{argument_count}")
}

pub(super) fn rewrite_borrowed_string_literal_arguments(
    expression: &mut syn::Expr,
    signatures: &BorrowedStringSignatures,
) {
    let (parameters, arguments) = match expression {
        syn::Expr::Call(call) => {
            let syn::Expr::Path(path) = call.func.as_ref() else {
                return;
            };
            let Some(name) = path
                .path
                .segments
                .last()
                .map(|segment| segment.ident.to_string())
            else {
                return;
            };
            let parameters = if path.path.segments.len() == 1 {
                signatures
                    .functions
                    .get(&signature_key(&name, call.args.len()))
            } else {
                let mut tail = path.path.segments.iter().rev();
                let method = tail.next().map(|segment| segment.ident.to_string());
                let owner = tail.next().map(|segment| segment.ident.to_string());
                owner.zip(method).and_then(|(owner, method)| {
                    signatures
                        .methods
                        .get(&method_signature_key(&owner, &method, call.args.len()))
                })
            };
            (parameters, &mut call.args)
        }
        _ => return,
    };
    let known_params = parameters.filter(|params| !params.is_empty());
    if known_params.is_none() {
        return;
    }
    for (index, argument) in arguments.iter_mut().enumerate() {
        let borrowed_string = known_params
            .and_then(|params| params.get(index))
            .copied()
            .unwrap_or(false);
        if !borrowed_string {
            continue;
        }
        let syn::Expr::Reference(reference) = argument else {
            if matches!(argument, syn::Expr::Lit(literal) if matches!(literal.lit, syn::Lit::Str(_)))
            {
                continue;
            }
            if matches!(argument, syn::Expr::MethodCall(call)
                if call.method == "as_str" && call.args.is_empty())
            {
                continue;
            }
            if matches!(argument, syn::Expr::Path(_)) {
                continue;
            }
            if let syn::Expr::MethodCall(conversion) = argument
                && conversion.args.is_empty()
                && matches!(
                    conversion.method.to_string().as_str(),
                    "clone" | "to_owned" | "to_string"
                )
            {
                let receiver = conversion.receiver.as_ref().clone();
                *argument = syn::parse_quote!(&#receiver);
                continue;
            }
            let value = argument.clone();
            *argument = syn::parse_quote!(&#value);
            continue;
        };
        if matches!(reference.expr.as_ref(), syn::Expr::Call(call)
            if matches!(call.func.as_ref(), syn::Expr::Path(path)
                if path.path.segments.len() == 2
                    && path.path.segments[0].ident == "String"
                    && path.path.segments[1].ident == "new")
                && call.args.is_empty())
        {
            *argument = syn::parse_quote!("");
            continue;
        }
        let syn::Expr::MethodCall(conversion) = reference.expr.as_mut() else {
            continue;
        };
        if matches!(
            conversion.method.to_string().as_str(),
            "to_owned" | "to_string"
        ) && conversion.args.is_empty()
            && matches!(conversion.receiver.as_ref(), syn::Expr::Lit(literal)
                if matches!(literal.lit, syn::Lit::Str(_)))
        {
            *argument = conversion.receiver.as_ref().clone();
        } else if conversion.method == "clone"
            && conversion.args.is_empty()
            && matches!(conversion.receiver.as_ref(), syn::Expr::Path(_))
        {
            conversion.method = syn::Ident::new("to_string", conversion.method.span());
        }
    }
}
