use super::rust_interop_digest::digest_path;
use super::rust_interop_probe::{
    canonical_sifr_target_path, PendingRustBridgeProbe, ProbeExecutionFailure,
};
use sha2::{Digest as _, Sha256};
use sifr_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{
    Arm, Attribute, Expr, ForeignItem, ImplItem, Item, ItemUse, LitStr, Macro, Stmt, Token,
    TraitItem, Type, UseTree,
};

const INLINE_QUERY_MACROS: &[&str] = &[
    "query",
    "query_as",
    "query_as_unchecked",
    "query_scalar",
    "query_scalar_unchecked",
    "query_unchecked",
];
const FILE_QUERY_MACROS: &[&str] = &[
    "query_file",
    "query_file_as",
    "query_file_as_unchecked",
    "query_file_scalar",
    "query_file_scalar_unchecked",
    "query_file_unchecked",
];

pub(super) fn configure_hermetic_build_environment(command: &mut std::process::Command) {
    command.env("SQLX_OFFLINE", "true");
    command.env_remove("DATABASE_URL");
}

pub(super) fn validate_probe_sqlx_offline_metadata(
    probe: &PendingRustBridgeProbe,
    backend_root: &Path,
) -> Result<(), ProbeExecutionFailure> {
    if probe.backend.cargo_source.is_some() {
        return Ok(());
    }
    validate_sqlx_offline_metadata(backend_root).map_err(|reason| ProbeExecutionFailure {
        code: DiagnosticCode::RUST_CARGO_METADATA,
        message_template: "Rust bridge SQLx offline metadata failed for `{target}`: {reason}",
        args: vec![
            ("target", canonical_sifr_target_path(&probe.declaration)),
            ("reason", reason),
        ],
        notes: vec![
            "Sifr validates recognized checked-in SQLx query metadata before Cargo, includes the complete .sqlx directory in cache identity, and never inherits DATABASE_URL for Rust bridge builds"
                .to_string(),
        ],
    })
}

pub(super) fn sqlx_offline_metadata_digest(backend_root: &Path) -> Option<String> {
    combined_sqlx_offline_metadata_digest([backend_root])
}

pub(super) fn combined_sqlx_offline_metadata_digest<'a>(
    backend_roots: impl IntoIterator<Item = &'a Path>,
) -> Option<String> {
    let mut identities = BTreeMap::new();
    for backend_root in backend_roots {
        let Some(metadata_roots) = sqlx_metadata_roots(backend_root) else {
            continue;
        };
        for metadata_root in metadata_roots {
            if metadata_root.is_dir() {
                identities
                    .entry(metadata_root.clone())
                    .or_insert_with(|| digest_path(&metadata_root));
            }
        }
    }
    if identities.is_empty() {
        return None;
    }
    let mut bytes = Vec::new();
    for (path, digest) in identities {
        bytes.extend_from_slice(path.to_string_lossy().as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(digest.as_bytes());
        bytes.push(0);
    }
    Some(sha256_hex(&bytes))
}

pub(super) fn validate_sqlx_offline_metadata(backend_root: &Path) -> Result<(), String> {
    let sqlx_crates = sqlx_dependency_crate_names(backend_root)?;
    if sqlx_crates.is_empty() {
        return Ok(());
    }
    let Some(metadata_roots) = sqlx_metadata_roots(backend_root) else {
        return Ok(());
    };
    for query in collect_sqlx_queries(backend_root, &sqlx_crates) {
        validate_query_metadata(&metadata_roots, &query)?;
    }
    Ok(())
}

fn sqlx_dependency_crate_names(backend_root: &Path) -> Result<BTreeSet<String>, String> {
    let manifest_path = backend_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "failed to read Rust package manifest '{}': {error}",
            manifest_path.display()
        )
    })?;
    let table = manifest.parse::<toml::Table>().map_err(|error| {
        format!(
            "failed to parse Rust package manifest '{}': {error}",
            manifest_path.display()
        )
    })?;
    let workspace_dependencies = workspace_dependency_packages(backend_root);
    let mut names = BTreeSet::new();
    collect_sqlx_dependency_table(
        table.get("dependencies"),
        &workspace_dependencies,
        &mut names,
    );
    if let Some(targets) = table.get("target").and_then(toml::Value::as_table) {
        for target in targets.values().filter_map(toml::Value::as_table) {
            collect_sqlx_dependency_table(
                target.get("dependencies"),
                &workspace_dependencies,
                &mut names,
            );
        }
    }
    Ok(names)
}

fn collect_sqlx_dependency_table(
    dependencies: Option<&toml::Value>,
    workspace_dependencies: &BTreeMap<String, String>,
    names: &mut BTreeSet<String>,
) {
    let Some(dependencies) = dependencies.and_then(toml::Value::as_table) else {
        return;
    };
    for (alias, specification) in dependencies {
        let specification_table = specification.as_table();
        let package_name = specification_table
            .and_then(|table| table.get("package"))
            .and_then(toml::Value::as_str)
            .or_else(|| {
                specification_table
                    .and_then(|table| table.get("workspace"))
                    .and_then(toml::Value::as_bool)
                    .is_some_and(|workspace| workspace)
                    .then(|| workspace_dependencies.get(alias))
                    .flatten()
                    .map(String::as_str)
            })
            .unwrap_or(alias);
        if package_name == "sqlx" {
            names.insert(alias.replace('-', "_"));
        }
    }
}

fn workspace_dependency_packages(backend_root: &Path) -> BTreeMap<String, String> {
    let Some(workspace_root) = cargo_workspace_root(backend_root) else {
        return BTreeMap::new();
    };
    let Ok(source) = fs::read_to_string(workspace_root.join("Cargo.toml")) else {
        return BTreeMap::new();
    };
    let Ok(table) = source.parse::<toml::Table>() else {
        return BTreeMap::new();
    };
    let Some(dependencies) = table
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(toml::Value::as_table)
    else {
        return BTreeMap::new();
    };
    dependencies
        .iter()
        .map(|(alias, specification)| {
            let package = specification
                .as_table()
                .and_then(|specification| specification.get("package"))
                .and_then(toml::Value::as_str)
                .unwrap_or(alias)
                .to_string();
            (alias.clone(), package)
        })
        .collect()
}

fn sqlx_metadata_roots(backend_root: &Path) -> Option<Vec<PathBuf>> {
    if dotenv_defines_offline_dir(&backend_root.join(".env")) {
        return None;
    }
    let mut roots = vec![backend_root.join(".sqlx")];
    if !backend_may_resolve_sqlx_metadata(backend_root) {
        return Some(roots);
    }
    if let Some(workspace_root) = cargo_workspace_root(backend_root) {
        let workspace_metadata = workspace_root.join(".sqlx");
        if !roots.contains(&workspace_metadata) {
            roots.push(workspace_metadata);
        }
    }
    Some(roots)
}

fn backend_may_resolve_sqlx_metadata(backend_root: &Path) -> bool {
    if backend_root.join(".sqlx").is_dir() {
        return true;
    }
    let Ok(source) = fs::read_to_string(backend_root.join("Cargo.toml")) else {
        return false;
    };
    let Ok(table) = source.parse::<toml::Table>() else {
        return false;
    };
    if dependency_table_may_resolve_sqlx(table.get("dependencies")) {
        return true;
    }
    table
        .get("target")
        .and_then(toml::Value::as_table)
        .is_some_and(|targets| {
            targets
                .values()
                .filter_map(toml::Value::as_table)
                .any(|target| dependency_table_may_resolve_sqlx(target.get("dependencies")))
        })
}

fn dependency_table_may_resolve_sqlx(dependencies: Option<&toml::Value>) -> bool {
    dependencies
        .and_then(toml::Value::as_table)
        .is_some_and(|dependencies| {
            dependencies.iter().any(|(alias, specification)| {
                if alias == "sqlx" {
                    return true;
                }
                specification.as_table().is_some_and(|specification| {
                    specification.get("package").and_then(toml::Value::as_str) == Some("sqlx")
                        || specification
                            .get("workspace")
                            .and_then(toml::Value::as_bool)
                            .is_some_and(|workspace| workspace)
                })
            })
        })
}

fn dotenv_defines_offline_dir(path: &Path) -> bool {
    let Ok(source) = fs::read_to_string(path) else {
        return false;
    };
    source.lines().any(|line| {
        let line = line.trim();
        let line = line.strip_prefix("export ").unwrap_or(line).trim_start();
        line.strip_prefix("SQLX_OFFLINE_DIR")
            .is_some_and(|suffix| suffix.trim_start().starts_with('='))
    })
}

fn cargo_workspace_root(backend_root: &Path) -> Option<PathBuf> {
    resolve_cargo_workspace_root(backend_root)
}

fn resolve_cargo_workspace_root(backend_root: &Path) -> Option<PathBuf> {
    let manifest_path = backend_root.join("Cargo.toml");
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--format-version=1",
            "--no-deps",
            "--offline",
            "--manifest-path",
        ])
        .arg(&manifest_path)
        .env("SQLX_OFFLINE", "true")
        .env_remove("DATABASE_URL")
        .output()
        .ok()?;
    if !output.status.success() {
        return nearest_declared_workspace_root(backend_root);
    }
    serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .ok()
        .and_then(|metadata| {
            metadata
                .get("workspace_root")
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from)
        })
        .or_else(|| nearest_declared_workspace_root(backend_root))
}

fn nearest_declared_workspace_root(backend_root: &Path) -> Option<PathBuf> {
    backend_root.ancestors().find_map(|ancestor| {
        let source = fs::read_to_string(ancestor.join("Cargo.toml")).ok()?;
        let table = source.parse::<toml::Table>().ok()?;
        table
            .contains_key("workspace")
            .then(|| ancestor.to_path_buf())
    })
}

fn collect_sqlx_queries(backend_root: &Path, sqlx_crates: &BTreeSet<String>) -> Vec<String> {
    let mut queries = Vec::new();
    for source_path in collect_rust_sources(&backend_root.join("src")) {
        let Ok(source) = fs::read_to_string(&source_path) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&source) else {
            continue;
        };
        collect_module_queries(&syntax.items, backend_root, sqlx_crates, &mut queries);
    }
    queries.sort();
    queries.dedup();
    queries
}

fn collect_rust_sources(source_root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![source_root.to_path_buf()];
    let mut sources = Vec::new();
    while let Some(path) = pending.pop() {
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_file() {
            if path.extension().is_some_and(|extension| extension == "rs") {
                sources.push(path);
            }
            continue;
        }
        if !metadata.is_dir() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        pending.extend(entries.filter_map(Result::ok).map(|entry| entry.path()));
    }
    sources.sort();
    sources
}

fn collect_module_queries(
    items: &[Item],
    backend_root: &Path,
    sqlx_crates: &BTreeSet<String>,
    queries: &mut Vec<String>,
) {
    let aliases = module_sqlx_aliases(items, sqlx_crates);
    for item in items {
        if let Item::Mod(module) = item {
            if has_cfg_attribute(&module.attrs) {
                continue;
            }
            if let Some((_, nested_items)) = &module.content {
                collect_module_queries(nested_items, backend_root, sqlx_crates, queries);
                continue;
            }
        }
        let mut visitor = SqlxQueryVisitor {
            aliases: &aliases,
            backend_root,
            queries,
        };
        visitor.visit_item(item);
    }
}

#[derive(Default)]
struct SqlxAliases {
    crate_names: BTreeSet<String>,
    macro_names: BTreeMap<String, String>,
}

fn module_sqlx_aliases(items: &[Item], sqlx_crates: &BTreeSet<String>) -> SqlxAliases {
    let mut aliases = SqlxAliases {
        crate_names: sqlx_crates.clone(),
        macro_names: BTreeMap::new(),
    };
    for item in items {
        if item_has_cfg_attribute(item) {
            continue;
        }
        match item {
            Item::ExternCrate(item) if sqlx_crates.contains(&item.ident.to_string()) => {
                if let Some((_, rename)) = &item.rename {
                    aliases.crate_names.insert(rename.to_string());
                }
            }
            Item::Use(item) => collect_sqlx_use_aliases(item, sqlx_crates, &mut aliases),
            _ => {}
        }
    }
    aliases
}

fn collect_sqlx_use_aliases(
    item: &ItemUse,
    sqlx_crates: &BTreeSet<String>,
    aliases: &mut SqlxAliases,
) {
    match &item.tree {
        UseTree::Name(name) if sqlx_crates.contains(&name.ident.to_string()) => {
            aliases.crate_names.insert(name.ident.to_string());
        }
        UseTree::Rename(rename) if sqlx_crates.contains(&rename.ident.to_string()) => {
            aliases.crate_names.insert(rename.rename.to_string());
        }
        UseTree::Path(path) if sqlx_crates.contains(&path.ident.to_string()) => {
            collect_sqlx_use_tail(&path.tree, aliases);
        }
        _ => {}
    }
}

fn collect_sqlx_use_tail(tree: &UseTree, aliases: &mut SqlxAliases) {
    match tree {
        UseTree::Name(name) => {
            register_sqlx_macro_alias(&name.ident.to_string(), &name.ident.to_string(), aliases);
        }
        UseTree::Rename(rename) if rename.ident == "self" => {
            aliases.crate_names.insert(rename.rename.to_string());
        }
        UseTree::Rename(rename) => register_sqlx_macro_alias(
            &rename.ident.to_string(),
            &rename.rename.to_string(),
            aliases,
        ),
        UseTree::Glob(_) => {
            for name in INLINE_QUERY_MACROS.iter().chain(FILE_QUERY_MACROS) {
                register_sqlx_macro_alias(name, name, aliases);
            }
        }
        UseTree::Group(group) => {
            for item in &group.items {
                collect_sqlx_use_tail(item, aliases);
            }
        }
        UseTree::Path(_) => {}
    }
}

fn register_sqlx_macro_alias(name: &str, alias: &str, aliases: &mut SqlxAliases) {
    if is_sqlx_query_macro(name) {
        aliases
            .macro_names
            .insert(alias.to_string(), name.to_string());
    }
}

fn is_sqlx_query_macro(name: &str) -> bool {
    INLINE_QUERY_MACROS.contains(&name) || FILE_QUERY_MACROS.contains(&name)
}

struct SqlxQueryVisitor<'a> {
    aliases: &'a SqlxAliases,
    backend_root: &'a Path,
    queries: &'a mut Vec<String>,
}

impl<'ast> Visit<'ast> for SqlxQueryVisitor<'_> {
    fn visit_item(&mut self, node: &'ast Item) {
        if !item_has_cfg_attribute(node) {
            syn::visit::visit_item(self, node);
        }
    }

    fn visit_stmt(&mut self, node: &'ast Stmt) {
        if !stmt_has_cfg_attribute(node) {
            syn::visit::visit_stmt(self, node);
        }
    }

    fn visit_expr(&mut self, node: &'ast Expr) {
        if !expr_has_cfg_attribute(node) {
            syn::visit::visit_expr(self, node);
        }
    }

    fn visit_arm(&mut self, node: &'ast Arm) {
        if !has_cfg_attribute(&node.attrs) {
            syn::visit::visit_arm(self, node);
        }
    }

    fn visit_impl_item(&mut self, node: &'ast ImplItem) {
        if !impl_item_has_cfg_attribute(node) {
            syn::visit::visit_impl_item(self, node);
        }
    }

    fn visit_trait_item(&mut self, node: &'ast TraitItem) {
        if !trait_item_has_cfg_attribute(node) {
            syn::visit::visit_trait_item(self, node);
        }
    }

    fn visit_foreign_item(&mut self, node: &'ast ForeignItem) {
        if !foreign_item_has_cfg_attribute(node) {
            syn::visit::visit_foreign_item(self, node);
        }
    }

    fn visit_macro(&mut self, node: &'ast Macro) {
        if let Some(query) = sqlx_query_text(node, self.aliases, self.backend_root) {
            self.queries.push(query);
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_item_mod(&mut self, _node: &'ast syn::ItemMod) {}
}

fn has_cfg_attribute(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
}

fn item_has_cfg_attribute(item: &Item) -> bool {
    let attrs = match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        Item::Verbatim(_) => return false,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn stmt_has_cfg_attribute(stmt: &Stmt) -> bool {
    match stmt {
        Stmt::Local(local) => has_cfg_attribute(&local.attrs),
        Stmt::Item(item) => item_has_cfg_attribute(item),
        Stmt::Expr(expr, _) => expr_has_cfg_attribute(expr),
        Stmt::Macro(stmt) => has_cfg_attribute(&stmt.attrs),
    }
}

fn expr_has_cfg_attribute(expr: &Expr) -> bool {
    let attrs = match expr {
        Expr::Array(expr) => &expr.attrs,
        Expr::Assign(expr) => &expr.attrs,
        Expr::Async(expr) => &expr.attrs,
        Expr::Await(expr) => &expr.attrs,
        Expr::Binary(expr) => &expr.attrs,
        Expr::Block(expr) => &expr.attrs,
        Expr::Break(expr) => &expr.attrs,
        Expr::Call(expr) => &expr.attrs,
        Expr::Cast(expr) => &expr.attrs,
        Expr::Closure(expr) => &expr.attrs,
        Expr::Const(expr) => &expr.attrs,
        Expr::Continue(expr) => &expr.attrs,
        Expr::Field(expr) => &expr.attrs,
        Expr::ForLoop(expr) => &expr.attrs,
        Expr::Group(expr) => &expr.attrs,
        Expr::If(expr) => &expr.attrs,
        Expr::Index(expr) => &expr.attrs,
        Expr::Infer(expr) => &expr.attrs,
        Expr::Let(expr) => &expr.attrs,
        Expr::Lit(expr) => &expr.attrs,
        Expr::Loop(expr) => &expr.attrs,
        Expr::Macro(expr) => &expr.attrs,
        Expr::Match(expr) => &expr.attrs,
        Expr::MethodCall(expr) => &expr.attrs,
        Expr::Paren(expr) => &expr.attrs,
        Expr::Path(expr) => &expr.attrs,
        Expr::Range(expr) => &expr.attrs,
        Expr::RawAddr(expr) => &expr.attrs,
        Expr::Reference(expr) => &expr.attrs,
        Expr::Repeat(expr) => &expr.attrs,
        Expr::Return(expr) => &expr.attrs,
        Expr::Struct(expr) => &expr.attrs,
        Expr::Try(expr) => &expr.attrs,
        Expr::TryBlock(expr) => &expr.attrs,
        Expr::Tuple(expr) => &expr.attrs,
        Expr::Unary(expr) => &expr.attrs,
        Expr::Unsafe(expr) => &expr.attrs,
        Expr::Verbatim(_) => return false,
        Expr::While(expr) => &expr.attrs,
        Expr::Yield(expr) => &expr.attrs,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn impl_item_has_cfg_attribute(item: &ImplItem) -> bool {
    let attrs = match item {
        ImplItem::Const(item) => &item.attrs,
        ImplItem::Fn(item) => &item.attrs,
        ImplItem::Type(item) => &item.attrs,
        ImplItem::Macro(item) => &item.attrs,
        ImplItem::Verbatim(_) => return false,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn trait_item_has_cfg_attribute(item: &TraitItem) -> bool {
    let attrs = match item {
        TraitItem::Const(item) => &item.attrs,
        TraitItem::Fn(item) => &item.attrs,
        TraitItem::Type(item) => &item.attrs,
        TraitItem::Macro(item) => &item.attrs,
        TraitItem::Verbatim(_) => return false,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn foreign_item_has_cfg_attribute(item: &ForeignItem) -> bool {
    let attrs = match item {
        ForeignItem::Fn(item) => &item.attrs,
        ForeignItem::Static(item) => &item.attrs,
        ForeignItem::Type(item) => &item.attrs,
        ForeignItem::Macro(item) => &item.attrs,
        ForeignItem::Verbatim(_) => return false,
        _ => return true,
    };
    has_cfg_attribute(attrs)
}

fn sqlx_query_text(node: &Macro, aliases: &SqlxAliases, backend_root: &Path) -> Option<String> {
    let segments = node
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let invoked_name = segments.last()?;
    let name = if segments.len() >= 2 && aliases.crate_names.contains(&segments[0]) {
        invoked_name.clone()
    } else if segments.len() == 1 {
        aliases.macro_names.get(invoked_name)?.clone()
    } else {
        return None;
    };
    let input = if matches!(
        name.as_str(),
        "query" | "query_scalar" | "query_unchecked" | "query_scalar_unchecked"
    ) {
        syn::parse2::<InlineQueryInput>(node.tokens.clone())
            .ok()
            .map(|input| QuerySource::Inline(input.query))
    } else if matches!(name.as_str(), "query_as" | "query_as_unchecked") {
        syn::parse2::<InlineQueryAsInput>(node.tokens.clone())
            .ok()
            .map(|input| QuerySource::Inline(input.query))
    } else if matches!(
        name.as_str(),
        "query_file" | "query_file_scalar" | "query_file_unchecked" | "query_file_scalar_unchecked"
    ) {
        syn::parse2::<FileQueryInput>(node.tokens.clone())
            .ok()
            .map(|input| QuerySource::File(input.path))
    } else if matches!(name.as_str(), "query_file_as" | "query_file_as_unchecked") {
        syn::parse2::<FileQueryAsInput>(node.tokens.clone())
            .ok()
            .map(|input| QuerySource::File(input.path))
    } else {
        None
    }?;
    input.resolve(backend_root)
}

enum QuerySource {
    Inline(String),
    File(LitStr),
}

impl QuerySource {
    fn resolve(self, backend_root: &Path) -> Option<String> {
        match self {
            Self::Inline(query) => Some(query),
            Self::File(path) => fs::read_to_string(backend_root.join(path.value())).ok(),
        }
    }
}

struct InlineQueryInput {
    query: String,
}

impl Parse for InlineQueryInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let query = parse_inline_query(input)?;
        parse_bind_arguments(input)?;
        Ok(Self { query })
    }
}

struct InlineQueryAsInput {
    query: String,
}

impl Parse for InlineQueryAsInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _output: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let query = parse_inline_query(input)?;
        parse_bind_arguments(input)?;
        Ok(Self { query })
    }
}

struct FileQueryInput {
    path: LitStr,
}

impl Parse for FileQueryInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path = input.parse()?;
        parse_bind_arguments(input)?;
        Ok(Self { path })
    }
}

struct FileQueryAsInput {
    path: LitStr,
}

impl Parse for FileQueryAsInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _output: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let path = input.parse()?;
        parse_bind_arguments(input)?;
        Ok(Self { path })
    }
}

fn parse_inline_query(input: ParseStream<'_>) -> syn::Result<String> {
    Ok(
        Punctuated::<LitStr, Token![+]>::parse_separated_nonempty(input)?
            .iter()
            .map(LitStr::value)
            .collect(),
    )
}

fn parse_bind_arguments(input: ParseStream<'_>) -> syn::Result<()> {
    if input.is_empty() {
        return Ok(());
    }
    input.parse::<Token![,]>()?;
    if input.is_empty() {
        return Ok(());
    }
    let _arguments = Punctuated::<Expr, Token![,]>::parse_terminated(input)?;
    Ok(())
}

fn validate_query_metadata(metadata_roots: &[PathBuf], query: &str) -> Result<(), String> {
    let hash = sha256_hex(query.as_bytes());
    let file_name = format!("query-{hash}.json");
    let metadata_path = metadata_roots
        .iter()
        .map(|root| root.join(&file_name))
        .find(|path| path.is_file())
        .ok_or_else(|| {
            format!("`SQLX_OFFLINE=true` but there is no cached data for this query: {query}")
        })?;
    let raw = fs::read_to_string(&metadata_path).map_err(|_| {
        format!("`SQLX_OFFLINE=true` but there is no cached data for this query: {query}")
    })?;
    let metadata: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "failed to read saved query path '{}': {error}",
            metadata_path.display()
        )
    })?;
    if metadata.get("query").and_then(serde_json::Value::as_str) != Some(query) {
        return Err(format!(
            "saved SQLx query text does not match query identity: {query}"
        ));
    }
    Ok(())
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
#[path = "rust_interop_sqlx_offline_tests.rs"]
mod tests;
