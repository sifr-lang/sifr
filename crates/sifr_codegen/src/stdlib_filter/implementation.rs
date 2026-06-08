use proc_macro2::{TokenStream, TokenTree};
use std::collections::{HashMap, HashSet};
use syn::visit::{self, Visit};
use syn::{Item, ItemImpl, ItemUse, Type, UseTree};

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
    pub(crate) process_status: SharedPreludeProcessStatusNeeds,
    pub(crate) process_async: SharedPreludeProcessAsyncNeeds,
    pub(crate) process_children: SharedPreludeProcessChildNeeds,
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

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SharedPreludeProcessStatusNeeds {
    pub(crate) needs_process_status: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SharedPreludeProcessAsyncNeeds {
    pub(crate) needs_run: bool,
    pub(crate) needs_run_timeout: bool,
    pub(crate) needs_output: bool,
    pub(crate) needs_output_timeout: bool,
    pub(crate) needs_spawn: bool,
    pub(crate) needs_wait: bool,
    pub(crate) needs_kill: bool,
    pub(crate) needs_terminate: bool,
}

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct SharedPreludeProcessChildNeeds {
    pub(crate) needs_process_children: bool,
}

pub(crate) struct PreparedStdlibModule {
    pub(crate) stripped_code: String,
    pub(crate) shared_needs: SharedPreludeNeeds,
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
pub(crate) fn dedup_rust_items(
    rust_code: &str,
    emitted_items: &mut HashSet<String>,
    skip_types: &HashSet<String>,
) -> String {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return rust_code.to_string();
    };

    let mut kept_items: Vec<Item> = Vec::new();
    for item in parsed.items {
        if let Some(name) = parse_item_name(&item) {
            if skip_types.contains(&name) {
                continue;
            }

            let dedup_key = dedup_item_key(&item);
            if emitted_items.insert(dedup_key) {
                kept_items.push(item);
            }
            continue;
        }

        kept_items.push(item);
    }

    render_items(&kept_items)
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

fn parse_stdlib_ir_file(rust_code: &str) -> Option<StdlibIrFile> {
    let Ok(parsed) = syn::parse_file(rust_code) else {
        return None;
    };

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
    let mut kept_items: Vec<Item> = Vec::new();
    for entry in &ir.entries {
        match entry {
            StdlibIrEntry::Named(item) => {
                if needed.contains(&item.name) {
                    kept_items.push(item.item.clone());
                }
            }
            StdlibIrEntry::Other(item) => kept_items.push(item.clone()),
        }
    }
    render_items(&kept_items)
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
            Item::Static(item_static)
                if item_static.ident == "__SIFR_PROCESS_CHILDREN"
                    || item_static.ident == "__SIFR_NEXT_PROCESS_CHILD_ID" =>
            {
                shared_needs.process_children.needs_process_children = true;
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
        process_children: SharedPreludeProcessChildNeeds {
            needs_process_children: code.contains("__SIFR_PROCESS_CHILDREN")
                || code.contains("__SIFR_PROCESS_PIPE_READERS")
                || code.contains("__SIFR_PROCESS_PIPE_WRITERS")
                || code.contains("__SIFR_NEXT_PROCESS_CHILD_ID")
                || code.contains("__sifr_next_process_child_id")
                || code.contains("__sifr_process_spawn")
                || code.contains("__sifr_process_terminate")
                || code.contains("__sifr_process_child_stdin")
                || code.contains("__sifr_process_child_stdout")
                || code.contains("__sifr_process_child_stderr")
                || code.contains("__sifr_process_pipe_read_all")
                || code.contains("__sifr_process_pipe_write_all")
                || code.contains("__sifr_process_pipe_close")
                || code.contains("__sifr_process_stdio_from_mode"),
        },
        process_status: SharedPreludeProcessStatusNeeds {
            needs_process_status: code.contains("__sifr_process_exit_signal"),
        },
        process_async: SharedPreludeProcessAsyncNeeds {
            needs_run: code.contains("__sifr_process_async_run("),
            needs_run_timeout: code.contains("__sifr_process_async_run_timeout"),
            needs_output: code.contains("__sifr_process_async_output("),
            needs_output_timeout: code.contains("__sifr_process_async_output_timeout"),
            needs_spawn: code.contains("__sifr_process_async_spawn("),
            needs_wait: code.contains("__sifr_process_async_wait("),
            needs_kill: code.contains("__sifr_process_async_kill("),
            needs_terminate: code.contains("__sifr_process_async_terminate("),
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
                "__SIFR_PROCESS_CHILDREN"
                | "__SIFR_PROCESS_PIPE_READERS"
                | "__SIFR_PROCESS_PIPE_WRITERS"
                | "__SIFR_NEXT_PROCESS_CHILD_ID"
                | "__sifr_next_process_child_id"
                | "__sifr_process_spawn"
                | "__sifr_process_terminate"
                | "__sifr_process_child_stdin"
                | "__sifr_process_child_stdout"
                | "__sifr_process_child_stderr"
                | "__sifr_process_pipe_read_all"
                | "__sifr_process_pipe_write_all"
                | "__sifr_process_pipe_close"
                | "__sifr_process_stdio_from_mode" => {
                    self.shared_needs.process_children.needs_process_children = true;
                }
                "__sifr_process_exit_signal" => {
                    self.shared_needs.process_status.needs_process_status = true;
                }
                "__sifr_process_async_run" => {
                    self.shared_needs.process_async.needs_run = true;
                }
                "__sifr_process_async_run_timeout" => {
                    self.shared_needs.process_async.needs_run_timeout = true;
                }
                "__sifr_process_async_output" => {
                    self.shared_needs.process_async.needs_output = true;
                }
                "__sifr_process_async_output_timeout" => {
                    self.shared_needs.process_async.needs_output_timeout = true;
                }
                "__SIFR_PROCESS_ASYNC_CHILDREN"
                | "__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID"
                | "__sifr_next_process_async_child_id"
                | "__sifr_process_async_spawn" => {
                    self.shared_needs.process_async.needs_spawn = true;
                }
                "__sifr_process_async_wait" => {
                    self.shared_needs.process_async.needs_wait = true;
                }
                "__sifr_process_async_kill" => {
                    self.shared_needs.process_async.needs_kill = true;
                }
                "__sifr_process_async_terminate" => {
                    self.shared_needs.process_async.needs_terminate = true;
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
                || item_static.ident == "__SIFR_PROCESS_CHILDREN"
                || item_static.ident == "__SIFR_PROCESS_PIPE_READERS"
                || item_static.ident == "__SIFR_PROCESS_PIPE_WRITERS"
                || item_static.ident == "__SIFR_NEXT_PROCESS_CHILD_ID"
                || item_static.ident == "__SIFR_PROCESS_ASYNC_CHILDREN"
                || item_static.ident == "__SIFR_NEXT_PROCESS_ASYNC_CHILD_ID"
                || item_static.ident == "__SIFR_GLOBAL_LOG_LEVEL"
        }
        Item::Fn(item_fn) => {
            item_fn.sig.ident == "__sifr_next_file_handle_id"
                || item_fn.sig.ident == "__sifr_next_process_child_id"
                || item_fn.sig.ident == "__sifr_process_spawn"
                || item_fn.sig.ident == "__sifr_process_terminate"
                || item_fn.sig.ident == "__sifr_process_child_stdin"
                || item_fn.sig.ident == "__sifr_process_child_stdout"
                || item_fn.sig.ident == "__sifr_process_child_stderr"
                || item_fn.sig.ident == "__sifr_process_pipe_read_all"
                || item_fn.sig.ident == "__sifr_process_pipe_write_all"
                || item_fn.sig.ident == "__sifr_process_pipe_close"
                || item_fn.sig.ident == "__sifr_process_stdio_from_mode"
                || item_fn.sig.ident == "__sifr_process_exit_signal"
                || item_fn.sig.ident == "__sifr_process_async_run"
                || item_fn.sig.ident == "__sifr_process_async_run_timeout"
                || item_fn.sig.ident == "__sifr_process_async_output"
                || item_fn.sig.ident == "__sifr_process_async_output_timeout"
                || item_fn.sig.ident == "__sifr_next_process_async_child_id"
                || item_fn.sig.ident == "__sifr_process_async_spawn"
                || item_fn.sig.ident == "__sifr_process_async_wait"
                || item_fn.sig.ident == "__sifr_process_async_kill"
                || item_fn.sig.ident == "__sifr_process_async_terminate"
        }
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
    )
}

pub(super) fn referenced_item_names_via_ast(
    item: &Item,
    item_names: &HashSet<String>,
    current_name: &str,
    global_types: &HashSet<String>,
) -> HashSet<String> {
    let mut local_bindings = LocalBindingCollector::default();
    local_bindings.visit_item(item);

    let mut collector = ItemRefCollector::new(
        item_names,
        current_name,
        global_types,
        local_bindings.locals,
    );
    collector.visit_item(item);
    collector.refs
}

#[derive(Default)]
struct LocalBindingCollector {
    locals: HashSet<String>,
}

impl<'ast> Visit<'ast> for LocalBindingCollector {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.locals.insert(node.ident.to_string());
        visit::visit_pat_ident(self, node);
    }
}

struct ItemRefCollector<'a> {
    item_names: &'a HashSet<String>,
    current_name: &'a str,
    global_types: &'a HashSet<String>,
    locals: HashSet<String>,
    refs: HashSet<String>,
}

impl<'a> ItemRefCollector<'a> {
    fn new(
        item_names: &'a HashSet<String>,
        current_name: &'a str,
        global_types: &'a HashSet<String>,
        locals: HashSet<String>,
    ) -> Self {
        Self {
            item_names,
            current_name,
            global_types,
            locals,
            refs: HashSet::new(),
        }
    }

    fn try_insert_ref(&mut self, ident: &str) {
        if ident == self.current_name {
            return;
        }
        if self.global_types.contains(ident) {
            return;
        }
        if self.item_names.contains(ident) {
            self.refs.insert(ident.to_string());
        }
    }

    fn collect_macro_token_refs(&mut self, macro_tokens: &TokenStream) {
        let locals = self.locals.clone();
        collect_macro_token_refs_rec(macro_tokens, &locals, |ident| {
            self.try_insert_ref(ident);
        });
    }
}

impl<'ast> Visit<'ast> for ItemRefCollector<'_> {
    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        if let Some((_, trait_path, _)) = &node.trait_ {
            if let Some(first) = trait_path.segments.first() {
                self.try_insert_ref(&first.ident.to_string());
            }
        }
        if let Some(name) = impl_self_type_ident(node.self_ty.as_ref()) {
            self.try_insert_ref(&name);
        }
        visit::visit_item_impl(self, node);
    }

    fn visit_type_path(&mut self, node: &'ast syn::TypePath) {
        if let Some(first) = node.path.segments.first() {
            self.try_insert_ref(&first.ident.to_string());
        }
        visit::visit_type_path(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(first) = node.segments.first() {
            let ident = first.ident.to_string();
            let is_single_local = node.leading_colon.is_none()
                && node.segments.len() == 1
                && self.locals.contains(&ident);
            if !is_single_local {
                self.try_insert_ref(&ident);
            }
        }
        visit::visit_path(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node.qself.is_none() {
            if let Some(first) = node.path.segments.first() {
                let ident = first.ident.to_string();
                let is_single_segment = node.path.segments.len() == 1;
                if !(is_single_segment && self.locals.contains(&ident)) {
                    self.try_insert_ref(&ident);
                }
            }
        }
        visit::visit_expr_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if let Some(first) = node.path.segments.first() {
            self.try_insert_ref(&first.ident.to_string());
        }
        self.collect_macro_token_refs(&node.tokens);
        visit::visit_macro(self, node);
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

pub(super) fn dedup_impl_key(item_impl: &ItemImpl) -> String {
    let self_ty = dedup_type_key(item_impl.self_ty.as_ref());
    if let Some((_, trait_path, _)) = &item_impl.trait_ {
        format!("impl {} for {}", dedup_path_key(trait_path), self_ty)
    } else {
        format!("impl {self_ty}")
    }
}

pub(super) fn dedup_path_key(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<String>>()
        .join("::")
}

pub(super) fn dedup_type_key(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => dedup_path_key(&type_path.path),
        Type::Reference(reference) => dedup_type_key(reference.elem.as_ref()),
        Type::Paren(paren) => dedup_type_key(paren.elem.as_ref()),
        Type::Group(group) => dedup_type_key(group.elem.as_ref()),
        Type::Slice(slice) => format!("[{}]", dedup_type_key(slice.elem.as_ref())),
        Type::Array(array) => format!("[{}]", dedup_type_key(array.elem.as_ref())),
        Type::Tuple(tuple) => {
            let elems = tuple
                .elems
                .iter()
                .map(dedup_type_key)
                .collect::<Vec<String>>()
                .join(",");
            format!("({elems})")
        }
        _ => "__unknown_type__".to_string(),
    }
}

pub(super) fn render_items(items: &[Item]) -> String {
    if items.is_empty() {
        return String::new();
    }

    prettyplease::unparse(&syn::File {
        shebang: None,
        attrs: Vec::new(),
        items: items.to_vec(),
    })
}
