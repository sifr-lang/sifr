use super::dedup_keys::dedup_impl_key;
use super::item_refs::referenced_item_names_via_ast;
use proc_macro2::{Group, Ident, TokenStream, TokenTree};
use quote::ToTokens;
use sifr_type_system::{is_global_rust_nominal_identity, stdlib_class_rust_name};
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::visit_mut::{self, VisitMut};
use syn::{Item, ItemUse, Type, UseTree};

#[derive(Clone)]
struct StdlibIrItem {
    name: String,
    item: Item,
    refs: HashSet<String>,
}

#[derive(Clone)]
enum StdlibIrEntry {
    Named(StdlibIrItem),
    Other(Item),
}

#[derive(Clone)]
struct StdlibIrFile {
    entries: Vec<StdlibIrEntry>,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SharedPreludeNeeds {
    pub(crate) collections: SharedPreludeCollectionNeeds,
    pub(crate) file_handles: SharedPreludeFileHandleNeeds,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SharedPreludeCollectionNeeds {
    pub(crate) needs_hashmap: bool,
    pub(crate) needs_hashset: bool,
    pub(crate) needs_vecdeque: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SharedPreludeFileHandleNeeds {
    pub(crate) needs_file_handles: bool,
    pub(crate) provides_file_handle_struct: bool,
}

pub(crate) struct PreparedStdlibModule {
    pub(crate) stripped_code: String,
    pub(crate) shared_needs: SharedPreludeNeeds,
}

/// Move canonical stdlib declarations behind compiler-owned Rust names.
///
/// Stdlib modules are flattened into the generated crate, while Sifr's nominal
/// identity still permits a user class to share a stdlib basename. Renaming the
/// canonical declarations and every stdlib reference before concatenation keeps
/// those two source-level identities distinct in Rust as well.
pub(crate) fn seal_canonical_stdlib_names(
    rust_code: &str,
    module: &str,
    nominal_types: &HashSet<String>,
) -> String {
    let replacements = nominal_types
        .iter()
        .filter(|name| !is_global_rust_nominal_identity(&format!("{module}.{name}")))
        .map(|name| (name.clone(), stdlib_class_rust_name(module, name)))
        .collect::<HashMap<_, _>>();
    let Ok(tokens) = rust_code.parse::<TokenStream>() else {
        return rust_code.to_string();
    };
    let rewritten = rewrite_canonical_stdlib_tokens(tokens, &replacements, false);
    let Ok(parsed) = syn::parse_file(&rewritten.to_string()) else {
        return rust_code.to_string();
    };
    render_items(&parsed.items)
}

/// Make generated stdlib references to external runtime crates immune to
/// source declarations with the same Rust identifier.
pub(crate) fn absolutize_external_crate_paths(rust_code: &str) -> String {
    let Ok(mut parsed) = syn::parse_file(rust_code) else {
        return rust_code.to_string();
    };
    ExternalCratePathAbsolutizer.visit_file_mut(&mut parsed);
    render_items(&parsed.items)
}

struct ExternalCratePathAbsolutizer;

impl VisitMut for ExternalCratePathAbsolutizer {
    fn visit_path_mut(&mut self, path: &mut syn::Path) {
        if path.leading_colon.is_none()
            && path.segments.first().is_some_and(|segment| {
                matches!(
                    segment.ident.to_string().as_str(),
                    "sifr_runtime" | "sifr_stdlib"
                )
            })
        {
            path.leading_colon = Some(syn::token::PathSep::default());
        }
        visit_mut::visit_path_mut(self, path);
    }

    fn visit_item_use_mut(&mut self, item: &mut ItemUse) {
        if item.leading_colon.is_none() && use_tree_starts_with_external_crate(&item.tree) {
            item.leading_colon = Some(syn::token::PathSep::default());
        }
        visit_mut::visit_item_use_mut(self, item);
    }
}

fn use_tree_starts_with_external_crate(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) => matches!(
            path.ident.to_string().as_str(),
            "sifr_runtime" | "sifr_stdlib"
        ),
        UseTree::Name(name) => matches!(
            name.ident.to_string().as_str(),
            "sifr_runtime" | "sifr_stdlib"
        ),
        UseTree::Rename(rename) => matches!(
            rename.ident.to_string().as_str(),
            "sifr_runtime" | "sifr_stdlib"
        ),
        UseTree::Glob(_) | UseTree::Group(_) => false,
    }
}

fn rewrite_canonical_stdlib_tokens(
    tokens: TokenStream,
    replacements: &HashMap<String, String>,
    rewrite_qualified_segments: bool,
) -> TokenStream {
    let tokens = tokens.into_iter().collect::<Vec<_>>();
    tokens
        .iter()
        .enumerate()
        .map(|(index, token)| match token {
            TokenTree::Ident(ident) => {
                let source = ident.to_string();
                let is_qualified_segment = index >= 2
                    && matches!(&tokens[index - 1], TokenTree::Punct(punct) if punct.as_char() == ':')
                    && matches!(&tokens[index - 2], TokenTree::Punct(punct) if punct.as_char() == ':');
                if is_qualified_segment && !rewrite_qualified_segments {
                    return TokenTree::Ident(ident.clone());
                }
                replacements.get(&source).map_or_else(
                    || TokenTree::Ident(ident.clone()),
                    |replacement| TokenTree::Ident(Ident::new(replacement, ident.span())),
                )
            }
            TokenTree::Group(group) => {
                let mut rewritten = Group::new(
                    group.delimiter(),
                    rewrite_canonical_stdlib_tokens(
                        group.stream(),
                        replacements,
                        rewrite_qualified_segments,
                    ),
                );
                rewritten.set_span(group.span());
                TokenTree::Group(rewritten)
            }
            other => other.clone(),
        })
        .collect()
}

/// Rewrite Rust identifier tokens in a compiler-owned source fragment without
/// touching string literals, character literals, or comments.
pub(crate) fn rewrite_rust_identifiers(
    rust_code: &str,
    replacements: &HashMap<String, String>,
) -> String {
    rust_code.parse::<TokenStream>().map_or_else(
        |_| rust_code.to_string(),
        |tokens| rewrite_canonical_stdlib_tokens(tokens, replacements, true).to_string(),
    )
}

const GLOBAL_INFRA_TYPES: &[&str] = &[
    "IOError",
    "ParseError",
    "ValueError",
    "TypeError",
    "RegexError",
    "KeyError",
    "IndexError",
    "AttributeError",
    "OverflowError",
    "ArithmeticLimitError",
    "FloatOverflowError",
    "FloatPrecisionLossError",
    "ZeroDivisionError",
    "RuntimeError",
    "NotImplementedError",
    "Error",
    "JSONDecodeError",
    "JsonIntegerRangeError",
    "JsonLimitError",
    "TOMLDecodeError",
    "FileNotFoundError",
    "PermissionError",
    "FileExistsError",
    "IsADirectoryError",
    "NotADirectoryError",
    "DirectoryNotEmptyError",
    "ScopeFailure",
    "TaskCancelled",
    "SecondaryError",
];

/// Strip per-module shared imports/infrastructure and return dependency flags.
pub(crate) fn collect_and_strip_shared_prelude(filtered: &str) -> PreparedStdlibModule {
    let Ok(parsed) = syn::parse_file(filtered) else {
        return PreparedStdlibModule {
            stripped_code: filtered.to_string(),
            shared_needs: derive_shared_needs_text_scan(filtered),
        };
    };

    let shared_needs = derive_shared_needs(&parsed.items);
    let kept_items: Vec<Item> = parsed
        .items
        .into_iter()
        .filter(|item| !is_shared_prelude_item(item))
        .collect();

    PreparedStdlibModule {
        stripped_code: render_items(&kept_items),
        shared_needs,
    }
}

/// Run stdlib IR DCE over compiled Rust source and keep only transitively-needed items.
pub(crate) fn filter_stdlib_ir_to_needed(
    rust_code: &str,
    imported_names: &HashSet<String>,
) -> String {
    let Some(ir) = parse_stdlib_ir_file(rust_code) else {
        return rust_code.to_string();
    };
    let deps = deps_by_item_name(&ir);
    let needed = transitive_needed_items(imported_names, &deps);
    render_needed_ir_items(&ir, &needed)
}

/// Run stdlib DCE after restoring identity-owned type references to the source
/// names used by top-level item declarations.
///
/// Checked stdlib HIR already carries canonical identities, so signatures and
/// expressions can contain `__SifrStdlib_*` references before the module-wide
/// sealing pass. The item filter must normalize those references or it cannot
/// discover dependencies on still-source-named declarations.
pub(crate) fn filter_canonical_stdlib_ir_to_needed(
    rust_code: &str,
    imported_names: &HashSet<String>,
    module: &str,
    nominal_types: &HashSet<String>,
) -> String {
    let replacements = nominal_types
        .iter()
        .filter(|name| !is_global_rust_nominal_identity(&format!("{module}.{name}")))
        .map(|name| (stdlib_class_rust_name(module, name), name.clone()))
        .collect::<HashMap<_, _>>();
    let Ok(tokens) = rust_code.parse::<TokenStream>() else {
        return filter_stdlib_ir_to_needed(rust_code, imported_names);
    };
    let restored = rewrite_canonical_stdlib_tokens(tokens, &replacements, true).to_string();
    filter_stdlib_ir_to_needed(&restored, imported_names)
}

/// Strip top-level items from Rust source whose names are already in `emitted_items`.
/// Items that survive are added to `emitted_items` so subsequent calls can deduplicate further.
///
/// Uses composite keys to distinguish struct/fn definitions from impl blocks:
/// - `struct X` / `fn X` -> key = "X"
/// - `impl X {` -> key = "impl X"
/// - `impl Trait for X {` -> key = "impl Trait for X"
///
/// The `skip_types` set contains type names (e.g., "`IOError`") for which ALL items
/// (struct, impl, trait impls) should be unconditionally stripped.
#[derive(Default)]
pub(crate) struct RustItemDeduper {
    fingerprints: HashMap<String, String>,
}

pub(crate) fn dedup_rust_items(
    rust_code: &str,
    emitted_items: &mut RustItemDeduper,
    skip_types: &HashSet<String>,
) -> String {
    let parsed = syn::parse_file(rust_code).unwrap_or_else(|error| {
        panic!("failed to parse stdlib support before deduplication: {error}")
    });

    let mut kept_items: Vec<Item> = Vec::new();
    for item in parsed.items {
        if let Some(name) = parse_item_name(&item) {
            if skip_types.contains(&name) {
                continue;
            }

            let dedup_key = dedup_item_key(&item);
            let fingerprint = item.to_token_stream().to_string();
            if let Some(previous) = emitted_items.fingerprints.get(&dedup_key) {
                if previous != &fingerprint {
                    assert!(
                        is_redundant_forwarding_adapter(&item, previous),
                        "conflicting generated support bodies share canonical key `{dedup_key}`"
                    );
                }
            } else {
                emitted_items.fingerprints.insert(dedup_key, fingerprint);
                kept_items.push(item);
            }
            continue;
        }
        let fingerprint = item.to_token_stream().to_string();
        let dedup_key = format!("unnamed:{fingerprint}");
        if emitted_items
            .fingerprints
            .insert(dedup_key, fingerprint)
            .is_none()
        {
            kept_items.push(item);
        }
    }

    render_items(&kept_items)
}

fn is_redundant_forwarding_adapter(item: &Item, previous: &str) -> bool {
    let Item::Fn(function) = item else {
        return false;
    };
    let Ok(Item::Fn(previous)) = syn::parse_str::<Item>(previous) else {
        return false;
    };
    if function.sig.to_token_stream().to_string() != previous.sig.to_token_stream().to_string() {
        return false;
    }
    let parameters = function
        .sig
        .inputs
        .iter()
        .map(|argument| match argument {
            syn::FnArg::Typed(argument) => match argument.pat.as_ref() {
                syn::Pat::Ident(ident) if ident.subpat.is_none() => Some(&ident.ident),
                _ => None,
            },
            syn::FnArg::Receiver(_) => None,
        })
        .collect::<Option<Vec<_>>>();
    let Some(parameters) = parameters else {
        return false;
    };
    let [syn::Stmt::Expr(expression, _)] = function.block.stmts.as_slice() else {
        return false;
    };
    let expression = match expression {
        syn::Expr::Return(return_expr) => return_expr.expr.as_deref(),
        expression => Some(expression),
    };
    let Some(syn::Expr::Call(call)) = expression else {
        return false;
    };
    let syn::Expr::Path(callee) = call.func.as_ref() else {
        return false;
    };
    if callee.qself.is_some()
        || callee.path.leading_colon.is_some()
        || callee.path.segments.len() != 1
        || call.args.len() != parameters.len()
    {
        return false;
    }
    call.args
        .iter()
        .zip(parameters)
        .all(|(argument, parameter)| match argument {
            syn::Expr::Path(path) => {
                path.qself.is_none()
                    && path.path.leading_colon.is_none()
                    && path.path.segments.len() == 1
                    && path.path.segments[0].ident == *parameter
            }
            _ => false,
        })
}

pub(crate) fn strip_rust_items_by_name(rust_code: &str, names: &HashSet<&str>) -> String {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return rust_code.to_string();
    };

    let kept_items: Vec<Item> = parsed
        .items
        .into_iter()
        .filter(|item| {
            parse_item_name(item)
                .as_deref()
                .is_none_or(|name| !names.contains(name))
        })
        .collect();

    render_items(&kept_items)
}

pub(crate) fn partition_rust_items_by_name(
    rust_code: &str,
    names: &HashSet<&str>,
) -> (String, String) {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return (String::new(), rust_code.to_string());
    };
    let (selected, remaining): (Vec<_>, Vec<_>) = parsed.items.into_iter().partition(|item| {
        parse_item_name(item)
            .as_deref()
            .is_some_and(|name| names.contains(name))
    });
    (render_items(&selected), render_items(&remaining))
}

pub(crate) fn rust_source_references_item_name(rust_code: &str, name: &str) -> bool {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return false;
    };
    let item_names = HashSet::from([name.to_string()]);
    let global_types = HashSet::new();
    parsed.items.iter().any(|item| {
        let current_name = parse_item_name(item).unwrap_or_default();
        referenced_item_names_via_ast(item, &item_names, &current_name, &global_types)
            .contains(name)
    })
}

pub(crate) fn rust_source_defines_item_name(rust_code: &str, name: &str) -> bool {
    syn::parse_file(rust_code).is_ok_and(|parsed| {
        parsed
            .items
            .iter()
            .any(|item| parse_item_name(item).as_deref() == Some(name))
    })
}

pub(crate) fn rust_source_defined_item_names(rust_code: &str) -> HashSet<String> {
    syn::parse_file(rust_code).map_or_else(
        |_| HashSet::new(),
        |parsed| {
            parsed
                .items
                .into_iter()
                .flat_map(crate::task_local_support::split_declarations)
                .filter_map(|item| parse_item_name(&item))
                .collect()
        },
    )
}

pub(crate) fn rust_source_identifier_names(rust_code: &str) -> HashSet<String> {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return HashSet::new();
    };
    let mut collector = RustIdentifierCollector::default();
    collector.visit_file(&parsed);
    collector.names
}

#[derive(Default)]
struct RustIdentifierCollector {
    names: HashSet<String>,
}

impl<'ast> Visit<'ast> for RustIdentifierCollector {
    fn visit_ident(&mut self, ident: &'ast Ident) {
        self.names.insert(ident.to_string());
    }

    fn visit_macro(&mut self, rust_macro: &'ast syn::Macro) {
        collect_macro_token_refs_rec(&rust_macro.tokens, &HashSet::new(), |name| {
            self.names.insert(name.to_string());
        });
        visit::visit_macro(self, rust_macro);
    }
}

fn parse_stdlib_ir_file(rust_code: &str) -> Option<StdlibIrFile> {
    let Ok(mut parsed) = syn::parse_file(rust_code) else {
        return None;
    };
    parsed.items = parsed
        .items
        .into_iter()
        .flat_map(crate::task_local_support::split_declarations)
        .collect();

    let item_names: HashSet<String> = parsed.items.iter().filter_map(parse_item_name).collect();
    let global_types: HashSet<String> = GLOBAL_INFRA_TYPES
        .iter()
        .map(|s| (*s).to_string())
        .collect();

    let entries = parsed
        .items
        .into_iter()
        .map(|item| {
            if let Some(name) = parse_item_name(&item) {
                let refs = referenced_item_names_via_ast(&item, &item_names, &name, &global_types);
                StdlibIrEntry::Named(StdlibIrItem { name, item, refs })
            } else {
                StdlibIrEntry::Other(item)
            }
        })
        .collect();

    Some(StdlibIrFile { entries })
}

fn deps_by_item_name(ir: &StdlibIrFile) -> HashMap<String, HashSet<String>> {
    let mut deps = HashMap::new();
    for entry in &ir.entries {
        if let StdlibIrEntry::Named(item) = entry {
            // Multiple blocks with the same name (e.g., impl X + impl Display for X)
            // should contribute dependencies together.
            deps.entry(item.name.clone())
                .or_insert_with(HashSet::new)
                .extend(item.refs.iter().cloned());
        }
    }
    deps
}

pub(super) fn transitive_needed_items(
    imported_names: &HashSet<String>,
    deps: &HashMap<String, HashSet<String>>,
) -> HashSet<String> {
    let mut needed: HashSet<String> = imported_names.clone();
    let mut worklist: Vec<String> = imported_names.iter().cloned().collect();

    while let Some(name) = worklist.pop() {
        if let Some(called) = deps.get(&name) {
            for dep in called {
                if needed.insert(dep.clone()) {
                    worklist.push(dep.clone());
                }
            }
        }
    }
    needed
}

fn render_needed_ir_items(ir: &StdlibIrFile, needed: &HashSet<String>) -> String {
    let needed_declared_traits: HashSet<String> = ir
        .entries
        .iter()
        .filter_map(|entry| match entry {
            StdlibIrEntry::Named(item)
                if matches!(item.item, Item::Trait(_)) && needed.contains(&item.name) =>
            {
                Some(item.name.clone())
            }
            _ => None,
        })
        .collect();
    let mut kept_items: Vec<Item> = Vec::new();
    for entry in &ir.entries {
        match entry {
            StdlibIrEntry::Named(item) => {
                if needed.contains(&item.name)
                    || trait_impl_name(&item.item)
                        .is_some_and(|name| needed_declared_traits.contains(&name))
                {
                    kept_items.push(item.item.clone());
                }
            }
            StdlibIrEntry::Other(item) => kept_items.push(item.clone()),
        }
    }
    render_items(&kept_items)
}

fn trait_impl_name(item: &Item) -> Option<String> {
    let Item::Impl(item_impl) = item else {
        return None;
    };
    item_impl
        .trait_
        .as_ref()?
        .0
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

pub(super) fn derive_shared_needs(items: &[Item]) -> SharedPreludeNeeds {
    let mut shared_needs = SharedPreludeNeeds::default();
    for item in items {
        match item {
            Item::Use(item_use) => {
                let mut imported_paths = Vec::new();
                collect_use_paths(&item_use.tree, &mut Vec::new(), &mut imported_paths);
                for path in &imported_paths {
                    mark_collection_use_path(path, &mut shared_needs);
                }
            }
            Item::Struct(item_struct) if item_struct.ident == "FileHandle" => {
                shared_needs.file_handles.provides_file_handle_struct = true;
            }
            Item::Static(item_static)
                if item_static.ident == "__SIFR_FILE_HANDLES"
                    || item_static.ident == "__SIFR_NEXT_FILE_HANDLE_ID" =>
            {
                shared_needs.file_handles.needs_file_handles = true;
            }
            _ => {}
        }
    }

    let mut collector = SharedNeedsCollector { shared_needs };
    for item in items {
        collector.visit_item(item);
    }
    collector.shared_needs
}

pub(super) fn derive_shared_needs_text_scan(code: &str) -> SharedPreludeNeeds {
    SharedPreludeNeeds {
        collections: SharedPreludeCollectionNeeds {
            needs_hashmap: code.contains("HashMap"),
            needs_hashset: code.contains("HashSet"),
            needs_vecdeque: code.contains("VecDeque"),
        },
        file_handles: SharedPreludeFileHandleNeeds {
            needs_file_handles: code.contains("__SIFR_FILE_HANDLES")
                || code.contains("__SIFR_NEXT_FILE_HANDLE_ID")
                || code.contains("__sifr_next_file_handle_id"),
            provides_file_handle_struct: code.contains("struct FileHandle"),
        },
    }
}

#[derive(Debug, Default)]
struct SharedNeedsCollector {
    shared_needs: SharedPreludeNeeds,
}

impl<'ast> Visit<'ast> for SharedNeedsCollector {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if let Some(segment) = path.segments.last() {
            let ident = segment.ident.to_string();
            match ident.as_str() {
                "HashMap" => self.shared_needs.collections.needs_hashmap = true,
                "HashSet" => self.shared_needs.collections.needs_hashset = true,
                "VecDeque" => self.shared_needs.collections.needs_vecdeque = true,
                "__SIFR_FILE_HANDLES"
                | "__SIFR_NEXT_FILE_HANDLE_ID"
                | "__sifr_next_file_handle_id" => {
                    self.shared_needs.file_handles.needs_file_handles = true;
                }
                _ => {}
            }
        }
        visit::visit_path(self, path);
    }
}

pub(super) fn is_shared_prelude_item(item: &Item) -> bool {
    match item {
        Item::Use(item_use) => is_shared_prelude_use(item_use),
        Item::Enum(item_enum) => item_enum.ident == "SifrFileHandle",
        Item::Static(item_static) => {
            item_static.ident == "__SIFR_FILE_HANDLES"
                || item_static.ident == "__SIFR_NEXT_FILE_HANDLE_ID"
        }
        Item::Fn(item_fn) => item_fn.sig.ident == "__sifr_next_file_handle_id",
        _ => false,
    }
}

pub(super) fn is_shared_prelude_use(item_use: &ItemUse) -> bool {
    let mut imported_paths = Vec::new();
    collect_use_paths(&item_use.tree, &mut Vec::new(), &mut imported_paths);

    !imported_paths.is_empty() && imported_paths.iter().all(|path| is_shared_use_path(path))
}

pub(super) fn collect_use_paths(
    tree: &UseTree,
    prefix: &mut Vec<String>,
    out: &mut Vec<Vec<String>>,
) {
    match tree {
        UseTree::Path(path) => {
            prefix.push(path.ident.to_string());
            collect_use_paths(&path.tree, prefix, out);
            prefix.pop();
        }
        UseTree::Name(name) => {
            let mut path = prefix.clone();
            path.push(name.ident.to_string());
            out.push(path);
        }
        UseTree::Rename(rename) => {
            let mut path = prefix.clone();
            path.push(rename.ident.to_string());
            out.push(path);
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_use_paths(item, prefix, out);
            }
        }
        UseTree::Glob(_) => {
            out.push(prefix.clone());
        }
    }
}

pub(super) fn is_shared_use_path(path: &[String]) -> bool {
    matches!(
        path,
        [std, collections, symbol]
            if std == "std"
                && collections == "collections"
                && matches!(symbol.as_str(), "HashMap" | "HashSet" | "VecDeque")
    ) || matches!(
        path,
        [std, sync, symbol]
            if std == "std"
                && sync == "sync"
                && symbol == "Mutex"
    ) || matches!(
        path,
        [runtime, symbol]
            if runtime == "sifr_runtime"
                && symbol == "SifrInt"
    ) || matches!(
        path,
        [numeric_crate, symbol]
            if matches!(
                (numeric_crate.as_str(), symbol.as_str()),
                ("num_bigint", "BigInt")
                    | ("rust_decimal", "Decimal")
                    | ("bigdecimal", "BigDecimal")
            )
    )
}

#[derive(Default)]
pub(super) struct LocalBindingCollector {
    pub(super) locals: HashSet<String>,
}

impl<'ast> Visit<'ast> for LocalBindingCollector {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.locals.insert(node.ident.to_string());
        visit::visit_pat_ident(self, node);
    }
}

pub(super) fn collect_macro_token_refs_rec<F>(
    tokens: &TokenStream,
    locals: &HashSet<String>,
    mut on_ident: F,
) where
    F: FnMut(&str),
{
    fn visit_tree<F>(tree: TokenTree, locals: &HashSet<String>, on_ident: &mut F)
    where
        F: FnMut(&str),
    {
        match tree {
            TokenTree::Ident(ident) => {
                let name = ident.to_string();
                if !locals.contains(&name) {
                    on_ident(&name);
                }
            }
            TokenTree::Group(group) => {
                for token in group.stream() {
                    visit_tree(token, locals, on_ident);
                }
            }
            TokenTree::Punct(_) | TokenTree::Literal(_) => {}
        }
    }

    for token in tokens.clone() {
        visit_tree(token, locals, &mut on_ident);
    }
}

pub(super) fn mark_collection_use_path(path: &[String], shared_needs: &mut SharedPreludeNeeds) {
    match path {
        [std, collections, symbol] if std == "std" && collections == "collections" => {
            match symbol.as_str() {
                "HashMap" => shared_needs.collections.needs_hashmap = true,
                "HashSet" => shared_needs.collections.needs_hashset = true,
                "VecDeque" => shared_needs.collections.needs_vecdeque = true,
                _ => {}
            }
        }
        _ => {}
    }
}

pub(super) fn parse_item_name(item: &Item) -> Option<String> {
    match item {
        Item::Fn(item_fn) => Some(item_fn.sig.ident.to_string()),
        Item::Const(item_const) => Some(item_const.ident.to_string()),
        Item::Static(item_static) => Some(item_static.ident.to_string()),
        Item::Struct(item_struct) => Some(item_struct.ident.to_string()),
        Item::Type(item_type) => Some(item_type.ident.to_string()),
        Item::Enum(item_enum) => Some(item_enum.ident.to_string()),
        Item::Trait(item_trait) => Some(item_trait.ident.to_string()),
        Item::Impl(item_impl) => impl_self_type_ident(item_impl.self_ty.as_ref()),
        Item::Macro(item) => {
            crate::task_local_support::declarations(&item.mac).and_then(|declarations| {
                if let [declaration] = declarations.0.as_slice() {
                    Some(declaration.name.to_string())
                } else {
                    None
                }
            })
        }
        _ => None,
    }
}

pub(super) fn impl_self_type_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        Type::Reference(reference) => impl_self_type_ident(reference.elem.as_ref()),
        Type::Paren(paren) => impl_self_type_ident(paren.elem.as_ref()),
        Type::Group(group) => impl_self_type_ident(group.elem.as_ref()),
        _ => None,
    }
}

pub(super) fn dedup_item_key(item: &Item) -> String {
    match item {
        Item::Impl(item_impl) => dedup_impl_key(item_impl),
        _ => parse_item_name(item).unwrap_or_else(|| "__unnamed_item__".to_string()),
    }
}

pub(super) fn render_items(items: &[Item]) -> String {
    if items.is_empty() {
        return String::new();
    }

    prettyplease::unparse(&syn::File {
        shebang: None,
        frontmatter: None,
        attrs: Vec::new(),
        items: items.to_vec(),
    })
}
