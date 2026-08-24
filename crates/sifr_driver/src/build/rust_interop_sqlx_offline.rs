use self::cfg_filter::{has_cfg_attribute, item_has_cfg_attribute};
use super::rust_interop_digest::digest_path;
use super::rust_interop_probe::{PendingRustBridgeProbe, ProbeExecutionFailure};
use super::rust_interop_sqlx_modules::reachable_rust_modules;
use sifr_diagnostics::DiagnosticCode;
use sifr_sysroot::sha256_hex;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::visit::Visit;
use syn::{Expr, Item, ItemUse, LitStr, Macro, Token, Type, UseTree};

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
type WorkspaceRootCacheKey = (PathBuf, String);
type WorkspaceRootCache = Mutex<BTreeMap<WorkspaceRootCacheKey, Option<PathBuf>>>;

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
        message_template: "Rust bridge package SQLx offline metadata failed: {reason}",
        args: vec![("reason", reason)],
        notes: vec![
            "This preflight is package-scoped. Sifr validates recognized checked-in SQLx query metadata before Cargo, includes the complete .sqlx directory in cache identity, and never inherits DATABASE_URL for Rust bridge builds"
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
    let workspace_aliases = workspace_dependency_aliases(&table);
    let workspace_dependencies = workspace_dependency_packages(backend_root, &workspace_aliases);
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

fn workspace_dependency_aliases(table: &toml::Table) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    collect_workspace_dependency_aliases(table.get("dependencies"), &mut aliases);
    if let Some(targets) = table.get("target").and_then(toml::Value::as_table) {
        for target in targets.values().filter_map(toml::Value::as_table) {
            collect_workspace_dependency_aliases(target.get("dependencies"), &mut aliases);
        }
    }
    aliases
}

fn collect_workspace_dependency_aliases(
    dependencies: Option<&toml::Value>,
    aliases: &mut BTreeSet<String>,
) {
    let Some(dependencies) = dependencies.and_then(toml::Value::as_table) else {
        return;
    };
    for (alias, specification) in dependencies {
        let uses_workspace = specification
            .as_table()
            .and_then(|specification| specification.get("workspace"))
            .and_then(toml::Value::as_bool)
            .is_some_and(|workspace| workspace);
        if uses_workspace {
            aliases.insert(alias.clone());
        }
    }
}

fn workspace_dependency_packages(
    backend_root: &Path,
    aliases: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    if aliases.is_empty() {
        return BTreeMap::new();
    }
    let Some(workspace_root) = declared_workspace_root(backend_root) else {
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
        .filter(|(alias, _)| aliases.contains(*alias))
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
    sqlx_dependency_crate_names(backend_root).is_ok_and(|names| !names.is_empty())
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
    static ROOTS: OnceLock<WorkspaceRootCache> = OnceLock::new();
    let key = (
        backend_root.to_path_buf(),
        workspace_resolution_fingerprint(backend_root),
    );
    let roots = ROOTS.get_or_init(|| Mutex::new(BTreeMap::new()));
    if let Ok(roots) = roots.lock() {
        if let Some(root) = roots.get(&key) {
            return root.clone();
        }
    }
    let resolved = resolve_cargo_workspace_root(backend_root);
    if let Ok(mut roots) = roots.lock() {
        // Retain one fingerprint per backend so long-lived compiler services
        // invalidate workspace changes without accumulating stale entries.
        roots.retain(|(root, _), _| root != backend_root);
        roots.insert(key, resolved.clone());
    }
    resolved
}

fn workspace_resolution_fingerprint(backend_root: &Path) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(backend_root.to_string_lossy().as_bytes());
    bytes.push(0);
    for ancestor in backend_root.ancestors() {
        let manifest = ancestor.join("Cargo.toml");
        bytes.extend_from_slice(manifest.to_string_lossy().as_bytes());
        bytes.push(0);
        if let Ok(source) = fs::read(&manifest) {
            bytes.extend_from_slice(&source);
        }
        bytes.push(0);
    }
    sha256_hex(&bytes)
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

fn declared_workspace_root(backend_root: &Path) -> Option<PathBuf> {
    let source = fs::read_to_string(backend_root.join("Cargo.toml")).ok()?;
    let table = source.parse::<toml::Table>().ok()?;
    if let Some(workspace) = table
        .get("package")
        .and_then(toml::Value::as_table)
        .and_then(|package| package.get("workspace"))
        .and_then(toml::Value::as_str)
    {
        return Some(backend_root.join(workspace));
    }
    nearest_declared_workspace_root(backend_root)
}

fn collect_sqlx_queries(backend_root: &Path, sqlx_crates: &BTreeSet<String>) -> Vec<String> {
    let mut queries = Vec::new();
    for syntax in reachable_rust_modules(backend_root) {
        collect_module_queries(&syntax.items, backend_root, sqlx_crates, &mut queries);
    }
    queries.sort();
    queries.dedup();
    queries
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

#[path = "rust_interop_sqlx_cfg.rs"]
mod cfg_filter;

#[cfg(test)]
#[path = "rust_interop_sqlx_offline_tests.rs"]
mod tests;
