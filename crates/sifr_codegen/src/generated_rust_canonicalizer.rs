use quote::ToTokens;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};

mod api_cleanup;
mod enum_variant_cleanup;
mod field_name_cleanup;
mod format_capture;
mod identifier_canonicalizer;
mod identifier_policy;
mod item_demand;
mod item_dependencies;
mod local_name_cleanup;
mod member_demand;
mod method_demand;
mod project_support_pruning;
pub(crate) use project_support_pruning::{
    import_generated_support_in_project_nominals,
    import_project_prelude_bindings_in_generated_support, prune_generated_project_owners,
};
mod source_expectations;
mod syntax_cleanup;

use api_cleanup::improve_generated_api_items;
pub use api_cleanup::{
    discover_project_const_function_names,
    finalize_formatted_generated_rust_source_with_project_consts,
};
pub use identifier_canonicalizer::canonicalize_generated_rust_identifier;
#[cfg(test)]
use item_dependencies::IdentifierCollector;
use item_dependencies::{
    all_item_identifier_names, impl_self_type_name, item_definition_name, item_dependency_names,
};
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
    let mut sources = canonicalize_generated_rust_project(&BTreeMap::from([(
        String::new(),
        source.to_string(),
    )]))?;
    sources
        .remove("")
        .ok_or_else(|| "missing canonical crate root".to_string())
}

/// Canonicalize all physical modules together before per-file cleanup. Keys use
/// Rust module paths (`a::b`); the empty key is the crate root.
pub fn canonicalize_generated_rust_project(
    sources: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    let fields = field_name_cleanup::canonicalize_fields(sources)?;
    let names = identifier_canonicalizer::project_name_map(fields.values().map(String::as_str))?;
    fields
        .into_iter()
        .map(|(module, source)| {
            canonicalize_source_with_names(&source, &names).map(|source| (module, source))
        })
        .collect()
}

fn canonicalize_source_with_names(
    source: &str,
    names: &BTreeMap<String, String>,
) -> Result<String, String> {
    let structurally_pruned = prune_closed_generated_binary(source)?;
    let source = structurally_pruned.as_deref().unwrap_or(source);
    let mut canonical = identifier_canonicalizer::canonicalize_identifiers(source, names)?;
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

pub fn finalize_formatted_generated_rust_source(source: &str) -> Result<String, String> {
    finalize_formatted_generated_rust_source_with_project_consts(source, &HashSet::new())
}

fn prune_closed_generated_binary(source: &str) -> Result<Option<String>, String> {
    let mut file = syn::parse_file(source)
        .map_err(|error| format!("failed to parse assembled generated Rust: {error}"))?;
    let before = file.to_token_stream().to_string();
    enum_variant_cleanup::canonicalize_local_enum_variants(&mut file);
    if !file
        .items
        .iter()
        .any(|item| matches!(item, syn::Item::Fn(function) if function.sig.ident == "main"))
    {
        return Ok(None);
    }
    if file.items.iter().any(|item| {
        matches!(item, syn::Item::Mod(module)
            if module.content.is_none()
                && module.ident != "__sifr_bridge"
                && !module.ident.to_string().starts_with("sifr_generated_"))
    }) {
        return Ok(None);
    }

    simplify_infallible_main(&mut file.items);
    prune_item_scope(&mut file.items, &HashSet::from(["main".to_string()]), true);
    let demanded_methods = demanded_inherent_method_names(&file);
    prune_inherent_methods(&mut file.items, &demanded_methods);
    prune_unused_members(&mut file);
    prune_item_scope(&mut file.items, &HashSet::from(["main".to_string()]), true);
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
    let shorthand_changed = field_name_cleanup::compact_shorthand(&mut file);
    let syntax_changed = canonicalize_syntax_to_fixed_point(&mut file)?;
    let final_syntax = prettyplease::unparse(&file);
    let mut api_file = syn::parse_file(&final_syntax)
        .map_err(|error| format!("failed to reparse final generated Rust: {error}"))?;
    let before_api = api_file.to_token_stream().to_string();
    improve_generated_api_items(&mut api_file.items, &final_syntax);
    let api_changed = api_file.to_token_stream().to_string() != before_api;
    if !shorthand_changed && !syntax_changed && !api_changed {
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
        "assert" => 1,
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

fn prune_item_scope(
    items: &mut Vec<syn::Item>,
    external_roots: &HashSet<String>,
    is_crate_root: bool,
) {
    let definitions = items
        .iter()
        .filter_map(item_definition_name)
        .collect::<HashSet<_>>();
    let mut parent_binding_candidates = definitions.clone();
    for item in items.iter() {
        if let syn::Item::Use(item_use) = item {
            let mut bindings = BTreeSet::new();
            collect_use_bindings(&item_use.tree, &mut bindings);
            parent_binding_candidates.extend(bindings);
        }
    }
    let parent_demands =
        parent_items_demanded_by_modules(items, &parent_binding_candidates, is_crate_root);
    let mut roots = external_roots.clone();
    roots.extend(item_demand::concrete_trait_impl_roots(items));
    roots.extend(parent_demands.iter().cloned());
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
    used_names.extend(parent_demands);
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
            prune_item_scope(nested, &nested_roots, false);
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

fn parent_items_demanded_by_modules(
    items: &[syn::Item],
    definitions: &HashSet<String>,
    is_crate_root: bool,
) -> HashSet<String> {
    let mut roots = HashSet::new();
    for item in items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        let Some((_, nested)) = &module.content else {
            continue;
        };
        let mut collector = ParentScopeReferenceCollector {
            definitions,
            roots: &mut roots,
            is_crate_root,
            nested_module_depth: 0,
        };
        for nested_item in nested {
            collector.visit_item(nested_item);
        }
    }
    roots
}

struct ParentScopeReferenceCollector<'scope> {
    definitions: &'scope HashSet<String>,
    roots: &'scope mut HashSet<String>,
    is_crate_root: bool,
    nested_module_depth: usize,
}

impl ParentScopeReferenceCollector<'_> {
    fn collect_segments(&mut self, segments: &[String]) {
        let candidate = match segments {
            [qualifier, candidate, ..] if qualifier == "crate" && self.is_crate_root => {
                Some(candidate)
            }
            [qualifier, candidate, ..] if qualifier == "super" && self.nested_module_depth == 0 => {
                Some(candidate)
            }
            _ => None,
        };
        if let Some(candidate) = candidate
            && self.definitions.contains(candidate)
        {
            self.roots.insert(candidate.clone());
        }
    }

    fn collect_use_tree(&mut self, tree: &syn::UseTree, prefix: &mut Vec<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                self.collect_use_tree(&path.tree, prefix);
                prefix.pop();
            }
            syn::UseTree::Name(name) => {
                prefix.push(name.ident.to_string());
                self.collect_segments(prefix);
                prefix.pop();
            }
            syn::UseTree::Rename(rename) => {
                prefix.push(rename.ident.to_string());
                self.collect_segments(prefix);
                prefix.pop();
            }
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.collect_use_tree(tree, prefix);
                }
            }
            syn::UseTree::Glob(_) => {
                let imports_parent = matches!(prefix.as_slice(), [qualifier]
                    if qualifier == "super" && self.nested_module_depth == 0)
                    || matches!(prefix.as_slice(), [qualifier]
                        if qualifier == "crate" && self.is_crate_root);
                if imports_parent {
                    self.roots.extend(self.definitions.iter().cloned());
                }
            }
        }
    }
}

impl<'ast> Visit<'ast> for ParentScopeReferenceCollector<'_> {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        self.collect_segments(&segments);
        visit::visit_path(self, path);
    }

    fn visit_item_use(&mut self, item: &'ast syn::ItemUse) {
        self.collect_use_tree(&item.tree, &mut Vec::new());
    }

    fn visit_item_mod(&mut self, module: &'ast syn::ItemMod) {
        self.nested_module_depth += 1;
        visit::visit_item_mod(self, module);
        self.nested_module_depth -= 1;
    }
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
        if let syn::Item::Mod(module) = item {
            if let Some((_, nested)) = &module.content {
                let referenced_names = item_dependency_names(item, definitions);
                collect_nested_module_use_roots(
                    nested,
                    module_name,
                    definitions,
                    &referenced_names,
                    &mut roots,
                );
            }
            let mut collector = QualifiedModuleReferenceCollector {
                module_name,
                definitions,
                roots: &mut roots,
            };
            collector.visit_item(item);
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

fn collect_nested_module_use_roots(
    items: &[syn::Item],
    module_name: &str,
    definitions: &HashSet<String>,
    referenced_names: &HashSet<String>,
    roots: &mut HashSet<String>,
) {
    for item in items {
        match item {
            syn::Item::Use(item_use) => collect_module_use_roots(
                &item_use.tree,
                module_name,
                false,
                definitions,
                referenced_names,
                roots,
            ),
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    collect_nested_module_use_roots(
                        nested,
                        module_name,
                        definitions,
                        referenced_names,
                        roots,
                    );
                }
            }
            _ => {}
        }
    }
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

#[cfg(test)]
#[path = "generated_rust_canonicalizer_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "generated_rust_canonicalizer_transforms_tests.rs"]
mod transforms_tests;

#[cfg(test)]
#[path = "generated_rust_canonicalizer_effects_tests.rs"]
mod effects_tests;

#[cfg(test)]
#[path = "generated_rust_canonicalizer_semantics_tests.rs"]
mod semantics_tests;

#[cfg(test)]
#[path = "generated_rust_canonicalizer_support_demand_tests.rs"]
mod support_demand_tests;
