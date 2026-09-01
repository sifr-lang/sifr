use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use std::collections::{BTreeSet, HashMap, HashSet};
use syn::parse::Parser;
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

mod api_cleanup;
mod field_name_cleanup;
mod identifier_canonicalizer;
mod identifier_policy;
mod local_name_cleanup;
mod member_demand;
mod method_demand;
mod source_expectations;
mod syntax_cleanup;

use api_cleanup::improve_generated_api_items;
use field_name_cleanup::canonicalize_generated_field_names;
pub use identifier_canonicalizer::canonicalize_generated_rust_identifier;
use member_demand::prune_unused_members;
use method_demand::{demanded_inherent_method_names, prune_inherent_methods};
use syntax_cleanup::canonicalize_syntax;

/// Canonicalize compiler-owned identifiers after every generated source fragment
/// has been assembled into one Rust file.
///
/// Identifier-only rewriting preserves source text, comments, and literals while
/// updating declarations, ordinary references, and macro token trees together.
/// Closed generated binaries that need structural simplification are rendered from
/// their parsed syntax tree; their literal values are preserved. Reserved prefixes
/// are escaped too, keeping the mapping injective when user code deliberately uses
/// a canonical prefix.
pub fn canonicalize_generated_rust_source(source: &str) -> Result<String, String> {
    let structurally_pruned = prune_closed_generated_binary(source)?;
    let source = structurally_pruned.as_deref().unwrap_or(source);
    let mut canonical = identifier_canonicalizer::canonicalize_identifiers(source)?;
    for _ in 0..16 {
        let rewritten = rewrite_format_captures(&canonical)?;
        let structurally_pruned = prune_closed_generated_binary(&rewritten)?;
        let next = structurally_pruned.unwrap_or(rewritten);
        if next == canonical {
            return Ok(next);
        }
        canonical = next;
    }
    Err("generated Rust canonicalization did not reach a fixed point".to_string())
}

/// Refresh compiler-owned API attributes after `rustfmt` has established the
/// exact source layout that downstream Rust lints inspect.
pub fn finalize_formatted_generated_rust_source(source: &str) -> Result<String, String> {
    let mut file = syn::parse_file(source)
        .map_err(|error| format!("failed to parse formatted generated Rust: {error}"))?;
    improve_generated_api_items(&mut file.items, source);
    Ok(prettyplease::unparse(&file))
}

fn prune_closed_generated_binary(source: &str) -> Result<Option<String>, String> {
    let mut file = syn::parse_file(source)
        .map_err(|error| format!("failed to parse assembled generated Rust: {error}"))?;
    let before = file.to_token_stream().to_string();
    if !file
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Fn(function) if function.sig.ident == "main"))
    {
        return Ok(None);
    }
    if file.items.iter().any(|item| {
        matches!(item, syn::Item::Mod(module) if module.content.is_none() && module.ident != "__sifr_bridge")
    }) {
        return Ok(None);
    }

    simplify_infallible_main(&mut file.items);
    prune_item_scope(&mut file.items, &HashSet::from(["main".to_string()]));
    let demanded_methods = demanded_inherent_method_names(&file);
    prune_inherent_methods(&mut file.items, &demanded_methods);
    prune_unused_members(&mut file);
    prune_item_scope(&mut file.items, &HashSet::from(["main".to_string()]));
    if file.to_token_stream().to_string() == before {
        Ok(None)
    } else {
        Ok(Some(prettyplease::unparse(&file)))
    }
}

fn simplify_infallible_main(items: &mut [syn::Item]) {
    for item in items {
        let syn::Item::Fn(function) = item else {
            continue;
        };
        if function.sig.ident != "main" || !signature_returns_result(&function.sig) {
            continue;
        }
        let Some(syn::Stmt::Expr(tail, _)) = function.block.stmts.last() else {
            continue;
        };
        if !is_unit_ok_call(tail) {
            continue;
        }
        let mut control = FallibleControlUse::default();
        for statement in &function.block.stmts[..function.block.stmts.len() - 1] {
            control.visit_stmt(statement);
        }
        if control.found {
            continue;
        }
        function.block.stmts.pop();
        function.sig.output = syn::ReturnType::Default;
    }
}

fn signature_returns_result(signature: &syn::Signature) -> bool {
    let syn::ReturnType::Type(_, ty) = &signature.output else {
        return false;
    };
    matches!(ty.as_ref(), syn::Type::Path(path)
        if path.path.segments.last().is_some_and(|segment| segment.ident == "Result"))
}

fn is_unit_ok_call(expression: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    matches!(call.func.as_ref(), syn::Expr::Path(path) if path.qself.is_none() && path.path.is_ident("Ok"))
        && matches!(call.args.first(), Some(syn::Expr::Tuple(tuple)) if tuple.elems.is_empty())
        && call.args.len() == 1
}

#[derive(Default)]
struct FallibleControlUse {
    found: bool,
}

impl<'ast> Visit<'ast> for FallibleControlUse {
    fn visit_expr_try(&mut self, _expression: &'ast syn::ExprTry) {
        self.found = true;
    }

    fn visit_expr_return(&mut self, _expression: &'ast syn::ExprReturn) {
        self.found = true;
    }
}

fn rewrite_format_captures(source: &str) -> Result<String, String> {
    let mut file = syn::parse_file(source)
        .map_err(|error| format!("failed to parse canonical generated Rust: {error}"))?;
    let field_names_changed = canonicalize_generated_field_names(&mut file);
    let syntax_changed = canonicalize_syntax_to_fixed_point(&mut file)?;
    let final_syntax = prettyplease::unparse(&file);
    let mut api_file = syn::parse_file(&final_syntax)
        .map_err(|error| format!("failed to reparse final generated Rust: {error}"))?;
    let before_api = api_file.to_token_stream().to_string();
    improve_generated_api_items(&mut api_file.items, &final_syntax);
    let api_changed = api_file.to_token_stream().to_string() != before_api;
    if !field_names_changed && !syntax_changed && !api_changed {
        return Ok(source.to_string());
    }
    let first_api_source = prettyplease::unparse(&api_file);
    improve_final_api_source(first_api_source)
}

fn canonicalize_syntax_to_fixed_point(file: &mut syn::File) -> Result<bool, String> {
    let mut changed = false;
    for _ in 0..4 {
        let before = file.to_token_stream().to_string();
        canonicalize_syntax(file);
        if file.to_token_stream().to_string() == before {
            let mut format_rewriter = FormatCaptureRewriter { changed: false };
            format_rewriter.visit_file_mut(file);
            return Ok(changed || format_rewriter.changed);
        }
        changed = true;
    }
    Err("generated Rust syntax cleanup did not reach a stable final form".to_string())
}

fn improve_final_api_source(mut source: String) -> Result<String, String> {
    for _ in 0..4 {
        let mut file = syn::parse_file(&source)
            .map_err(|error| format!("failed to reparse final generated Rust: {error}"))?;
        improve_generated_api_items(&mut file.items, &source);
        let improved = prettyplease::unparse(&file);
        if improved == source {
            return Ok(source);
        }
        source = improved;
    }
    Err("generated Rust API cleanup did not reach a stable final form".to_string())
}

struct FormatCaptureRewriter {
    changed: bool,
}

impl VisitMut for FormatCaptureRewriter {
    fn visit_macro_mut(&mut self, rust_macro: &mut syn::Macro) {
        visit_mut::visit_macro_mut(self, rust_macro);
        if inline_simple_format_arguments(rust_macro) {
            self.changed = true;
            return;
        }
        let Ok(mut arguments) = rust_macro.parse_body_with(
            syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
        ) else {
            return;
        };
        let arguments_before = arguments.to_token_stream().to_string();
        for argument in &mut arguments {
            self.visit_expr_mut(argument);
        }
        if arguments.to_token_stream().to_string() != arguments_before {
            rust_macro.tokens = arguments.into_token_stream();
            self.changed = true;
        }
    }
}

fn inline_simple_format_arguments(rust_macro: &mut syn::Macro) -> bool {
    let Some(name) = rust_macro
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
    else {
        return false;
    };
    let format_index = match name.as_str() {
        "format" => 0,
        "print" | "println" | "eprint" | "eprintln" => 0,
        "write" | "writeln" => 1,
        _ => return false,
    };
    let Ok(mut arguments) = rust_macro.parse_body_with(
        syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated,
    ) else {
        return false;
    };
    if arguments.len() <= format_index + 1 {
        return false;
    }
    let Some(syn::Expr::Lit(format_expression)) = arguments.iter().nth(format_index) else {
        return false;
    };
    let syn::Lit::Str(format_literal) = &format_expression.lit else {
        return false;
    };
    let format_span = format_literal.span();
    let mut payload = arguments
        .iter()
        .skip(format_index + 1)
        .cloned()
        .collect::<Vec<_>>();
    let mut format = format_literal.value();
    let placeholders = sequential_format_placeholders(&format);
    if placeholders.len() != payload.len() {
        return false;
    }

    let mut changed = false;
    for (argument, (_, _, specifier)) in payload.iter_mut().zip(&placeholders) {
        if specifier.is_empty()
            && let syn::Expr::MethodCall(call) = argument
            && call.method == "to_string"
            && call.args.is_empty()
        {
            *argument = call.receiver.as_ref().clone();
            changed = true;
        }
    }
    let replacements = payload
        .iter()
        .zip(&placeholders)
        .map(|(argument, (_, _, specifier))| match argument {
            syn::Expr::Path(path) if path.qself.is_none() && path.path.segments.len() == 1 => path
                .path
                .segments
                .first()
                .map(|segment| format!("{{{}{specifier}}}", segment.ident)),
            syn::Expr::Lit(literal) if specifier.is_empty() => match &literal.lit {
                syn::Lit::Str(value) => Some(value.value().replace('{', "{{").replace('}', "}}")),
                _ => None,
            },
            _ => None,
        })
        .collect::<Option<Vec<_>>>();
    if let Some(replacements) = replacements {
        for ((start, end, _), replacement) in placeholders.iter().zip(&replacements).rev() {
            format.replace_range(*start..=*end, replacement);
        }
        payload.clear();
        changed = true;
    }

    if !changed {
        return false;
    }

    let retained = arguments
        .iter()
        .take(format_index)
        .cloned()
        .collect::<Vec<_>>();
    arguments.clear();
    arguments.extend(retained);
    arguments.push(syn::Expr::Lit(syn::ExprLit {
        attrs: Vec::new(),
        lit: syn::Lit::Str(syn::LitStr::new(&format, format_span)),
    }));
    arguments.extend(payload);
    rust_macro.tokens = arguments.into_token_stream();
    true
}

fn sequential_format_placeholders(format: &str) -> Vec<(usize, usize, String)> {
    let bytes = format.as_bytes();
    let mut placeholders = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'{' {
            index += 1;
            continue;
        }
        if bytes.get(index + 1) == Some(&b'{') {
            index += 2;
            continue;
        }
        let Some(relative_end) = format[index + 1..].find('}') else {
            break;
        };
        let end = index + 1 + relative_end;
        let interior = &format[index + 1..end];
        if interior.is_empty() || interior.starts_with(':') {
            placeholders.push((index, end, interior.to_string()));
        }
        index = end + 1;
    }
    placeholders
}

fn prune_item_scope(items: &mut Vec<syn::Item>, external_roots: &HashSet<String>) {
    let definitions = items
        .iter()
        .filter_map(item_definition_name)
        .collect::<HashSet<_>>();
    let mut roots = external_roots.clone();
    for item in items.iter() {
        let local_impl_owner = match item {
            syn::Item::Impl(item_impl) => impl_self_type_name(item_impl.self_ty.as_ref())
                .filter(|owner| definitions.contains(owner)),
            _ => None,
        };
        if item_definition_name(item).is_none()
            && !matches!(item, syn::Item::Use(_) | syn::Item::Mod(_))
            && local_impl_owner.is_none()
        {
            roots.extend(
                item_dependency_names(item, &definitions)
                    .intersection(&definitions)
                    .cloned(),
            );
        }
    }

    let mut dependencies = HashMap::<String, HashSet<String>>::new();
    for item in items.iter() {
        if let Some(name) = item_definition_name(item) {
            dependencies.entry(name).or_default().extend(
                item_dependency_names(item, &definitions)
                    .intersection(&definitions)
                    .cloned(),
            );
        } else if let syn::Item::Impl(item_impl) = item
            && let Some(owner) = impl_self_type_name(item_impl.self_ty.as_ref())
            && definitions.contains(&owner)
        {
            dependencies.entry(owner).or_default().extend(
                item_dependency_names(item, &definitions)
                    .intersection(&definitions)
                    .cloned(),
            );
        }
    }
    let mut reachable = roots.clone();
    let mut worklist = roots.into_iter().collect::<Vec<_>>();
    while let Some(name) = worklist.pop() {
        if let Some(references) = dependencies.get(&name) {
            for reference in references {
                if reachable.insert(reference.clone()) {
                    worklist.push(reference.clone());
                }
            }
        }
    }

    items.retain(|item| {
        if let Some(name) = item_definition_name(item) {
            return reachable.contains(&name);
        }
        if let syn::Item::Impl(item_impl) = item
            && let Some(owner) = impl_self_type_name(item_impl.self_ty.as_ref())
            && definitions.contains(&owner)
        {
            return reachable.contains(&owner);
        }
        true
    });

    let mut used_names = external_roots.clone();
    for item in items.iter() {
        if !matches!(item, syn::Item::Use(_) | syn::Item::Mod(_)) {
            used_names.extend(all_item_identifier_names(item));
        }
    }

    for index in 0..items.len() {
        let nested_roots = if let syn::Item::Mod(module) = &items[index] {
            module.content.as_ref().map(|(_, nested)| {
                let nested_definitions = nested
                    .iter()
                    .filter_map(item_definition_name)
                    .collect::<HashSet<_>>();
                module_roots_from_parent_scope(
                    items,
                    index,
                    &module.ident.to_string(),
                    &nested_definitions,
                    &used_names,
                )
            })
        } else {
            None
        };
        if let (Some(nested_roots), syn::Item::Mod(module)) = (nested_roots, &mut items[index])
            && let Some((_, nested)) = &mut module.content
        {
            prune_item_scope(nested, &nested_roots);
        }
    }

    items.retain(|item| {
        let syn::Item::Use(item_use) = item else {
            return true;
        };
        let mut bindings = BTreeSet::new();
        collect_use_bindings(&item_use.tree, &mut bindings)
            || bindings.iter().any(|binding| used_names.contains(binding))
    });
}

fn module_roots_from_parent_scope(
    items: &[syn::Item],
    module_index: usize,
    module_name: &str,
    definitions: &HashSet<String>,
    used_names: &HashSet<String>,
) -> HashSet<String> {
    let mut roots = HashSet::new();
    for (index, item) in items.iter().enumerate() {
        if index == module_index {
            continue;
        }
        if let syn::Item::Use(item_use) = item {
            collect_module_use_roots(
                &item_use.tree,
                module_name,
                false,
                definitions,
                used_names,
                &mut roots,
            );
            continue;
        }
        if matches!(item, syn::Item::Mod(_)) {
            continue;
        }
        let mut collector = QualifiedModuleReferenceCollector {
            module_name,
            definitions,
            roots: &mut roots,
        };
        collector.visit_item(item);
    }
    roots.retain(|name| definitions.contains(name));
    roots
}

fn collect_module_use_roots(
    tree: &syn::UseTree,
    module_name: &str,
    inside_module: bool,
    definitions: &HashSet<String>,
    used_names: &HashSet<String>,
    roots: &mut HashSet<String>,
) {
    match tree {
        syn::UseTree::Path(path) => collect_module_use_roots(
            &path.tree,
            module_name,
            inside_module || path.ident == module_name,
            definitions,
            used_names,
            roots,
        ),
        syn::UseTree::Name(name)
            if inside_module && used_names.contains(&name.ident.to_string()) =>
        {
            roots.insert(name.ident.to_string());
        }
        syn::UseTree::Rename(rename)
            if inside_module && used_names.contains(&rename.rename.to_string()) =>
        {
            roots.insert(rename.ident.to_string());
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_module_use_roots(
                    item,
                    module_name,
                    inside_module,
                    definitions,
                    used_names,
                    roots,
                );
            }
        }
        syn::UseTree::Glob(_) if inside_module => {
            roots.extend(definitions.intersection(used_names).cloned());
        }
        syn::UseTree::Name(_) | syn::UseTree::Rename(_) | syn::UseTree::Glob(_) => {}
    }
}

struct QualifiedModuleReferenceCollector<'scope> {
    module_name: &'scope str,
    definitions: &'scope HashSet<String>,
    roots: &'scope mut HashSet<String>,
}

impl<'ast> Visit<'ast> for QualifiedModuleReferenceCollector<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments = path.segments.iter().collect::<Vec<_>>();
        for pair in segments.windows(2) {
            if pair[0].ident == self.module_name {
                let candidate = pair[1].ident.to_string();
                if self.definitions.contains(&candidate) {
                    self.roots.insert(candidate);
                }
            }
        }
        visit::visit_path(self, path);
    }
}

fn collect_use_bindings(tree: &syn::UseTree, bindings: &mut BTreeSet<String>) -> bool {
    match tree {
        syn::UseTree::Name(name) => {
            bindings.insert(name.ident.to_string());
            false
        }
        syn::UseTree::Rename(rename) => {
            bindings.insert(rename.rename.to_string());
            false
        }
        syn::UseTree::Path(path) => collect_use_bindings(&path.tree, bindings),
        syn::UseTree::Group(group) => group
            .items
            .iter()
            .any(|tree| collect_use_bindings(tree, bindings)),
        syn::UseTree::Glob(_) => true,
    }
}

fn item_definition_name(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Const(item) => Some(item.ident.to_string()),
        syn::Item::Enum(item) => Some(item.ident.to_string()),
        syn::Item::Fn(item) => Some(item.sig.ident.to_string()),
        syn::Item::Impl(item) => impl_self_type_name(item.self_ty.as_ref()),
        syn::Item::Static(item) => Some(item.ident.to_string()),
        syn::Item::Struct(item) => Some(item.ident.to_string()),
        syn::Item::Trait(item) => Some(item.ident.to_string()),
        syn::Item::Type(item) => Some(item.ident.to_string()),
        syn::Item::Union(item) => Some(item.ident.to_string()),
        _ => None,
    }
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

fn item_dependency_names(item: &syn::Item, definitions: &HashSet<String>) -> HashSet<String> {
    let mut bindings = BindingCollector::default();
    bindings.visit_item(item);
    let mut collector = ScopedItemReferenceCollector {
        bindings: &bindings.names,
        definitions,
        references: HashSet::new(),
    };
    collector.visit_item(item);
    collector.references
}

fn all_item_identifier_names(item: &syn::Item) -> HashSet<String> {
    let mut collector = IdentifierCollector::default();
    collector.visit_item(item);
    collector.names.into_iter().collect()
}

#[derive(Default)]
struct IdentifierCollector {
    names: BTreeSet<String>,
}

impl IdentifierCollector {
    fn collect_tokens(&mut self, tokens: TokenStream) {
        for token in tokens {
            match token {
                TokenTree::Ident(identifier) => {
                    self.names.insert(identifier.to_string());
                }
                TokenTree::Group(group) => self.collect_tokens(group.stream()),
                _ => {}
            }
        }
    }
}

impl<'ast> Visit<'ast> for IdentifierCollector {
    fn visit_ident(&mut self, identifier: &'ast proc_macro2::Ident) {
        self.names.insert(identifier.to_string());
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        visit::visit_macro(self, rust_macro);
        self.collect_tokens(rust_macro.tokens.clone());
    }

    fn visit_meta_list(&mut self, meta: &'ast syn::MetaList) {
        visit::visit_meta_list(self, meta);
        self.collect_tokens(meta.tokens.clone());
    }
}

#[derive(Default)]
struct BindingCollector {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for BindingCollector {
    fn visit_pat_ident(&mut self, pattern: &'ast syn::PatIdent) {
        self.names.insert(pattern.ident.to_string());
        visit::visit_pat_ident(self, pattern);
    }

    fn visit_generic_param(&mut self, parameter: &'ast syn::GenericParam) {
        match parameter {
            syn::GenericParam::Type(type_) => {
                self.names.insert(type_.ident.to_string());
            }
            syn::GenericParam::Const(const_) => {
                self.names.insert(const_.ident.to_string());
            }
            syn::GenericParam::Lifetime(_) => {}
        }
        visit::visit_generic_param(self, parameter);
    }
}

struct ScopedItemReferenceCollector<'scope> {
    bindings: &'scope HashSet<String>,
    definitions: &'scope HashSet<String>,
    references: HashSet<String>,
}

impl ScopedItemReferenceCollector<'_> {
    fn collect_path(&mut self, path: &syn::Path) {
        let segments = path.segments.iter().collect::<Vec<_>>();
        let candidate = segments.first().and_then(|first| {
            if matches!(first.ident.to_string().as_str(), "crate" | "self" | "super") {
                segments.get(1)
            } else {
                Some(first)
            }
        });
        if let Some(segment) = candidate {
            let name = segment.ident.to_string();
            if self.definitions.contains(&name) && !self.bindings.contains(&name) {
                self.references.insert(name);
            }
        }
    }

    fn collect_macro_tokens(&mut self, tokens: TokenStream) {
        let parser = syn::punctuated::Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated;
        if let Ok(expressions) = parser.parse2(tokens.clone()) {
            for expression in &expressions {
                self.visit_expr(expression);
            }
            return;
        }
        let tokens = tokens.into_iter().collect::<Vec<_>>();
        for (index, token) in tokens.iter().enumerate() {
            if let TokenTree::Group(group) = token {
                self.collect_macro_tokens(group.stream());
                continue;
            }
            let TokenTree::Ident(identifier) = token else {
                continue;
            };
            let preceded_by_member_access = matches!(tokens.get(index.wrapping_sub(1)), Some(TokenTree::Punct(punctuation)) if punctuation.as_char() == '.');
            let followed_by_field_separator = matches!(tokens.get(index + 1), Some(TokenTree::Punct(first)) if first.as_char() == ':')
                && !matches!(tokens.get(index + 2), Some(TokenTree::Punct(second)) if second.as_char() == ':');
            let name = identifier.to_string();
            if !preceded_by_member_access
                && !followed_by_field_separator
                && self.definitions.contains(&name)
                && !self.bindings.contains(&name)
            {
                self.references.insert(name);
            }
        }
    }
}

impl<'ast> Visit<'ast> for ScopedItemReferenceCollector<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        self.collect_path(path);
        visit::visit_path(self, path);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        visit::visit_expr_path(self, expression);
    }

    fn visit_type_path(&mut self, ty: &'ast syn::TypePath) {
        visit::visit_type_path(self, ty);
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        self.collect_macro_tokens(rust_macro.tokens.clone());
        visit::visit_macro(self, rust_macro);
    }
}

#[cfg(test)]
#[path = "generated_rust_canonicalizer_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "generated_rust_canonicalizer_item8_tests.rs"]
mod item8_tests;
