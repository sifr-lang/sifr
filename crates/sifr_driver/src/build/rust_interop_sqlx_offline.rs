use super::rust_interop_digest::digest_path;
use super::rust_interop_probe::{
    canonical_sifr_target_path, PendingRustBridgeProbe, ProbeExecutionFailure,
};
use sha2::{Digest as _, Sha256};
use sifr_diagnostics::DiagnosticCode;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
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
    let metadata_root = backend_root.join(".sqlx");
    metadata_root.is_dir().then(|| digest_path(&metadata_root))
}

pub(super) fn validate_sqlx_offline_metadata(backend_root: &Path) -> Result<(), String> {
    let sqlx_crates = sqlx_dependency_crate_names(backend_root)?;
    if sqlx_crates.is_empty() {
        return Ok(());
    }
    for query in collect_sqlx_queries(backend_root, &sqlx_crates) {
        validate_query_metadata(backend_root, &query)?;
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
    let mut names = BTreeSet::new();
    collect_sqlx_dependency_table(table.get("dependencies"), &mut names);
    if let Some(targets) = table.get("target").and_then(toml::Value::as_table) {
        for target in targets.values().filter_map(toml::Value::as_table) {
            collect_sqlx_dependency_table(target.get("dependencies"), &mut names);
        }
    }
    Ok(names)
}

fn collect_sqlx_dependency_table(dependencies: Option<&toml::Value>, names: &mut BTreeSet<String>) {
    let Some(dependencies) = dependencies.and_then(toml::Value::as_table) else {
        return;
    };
    for (alias, specification) in dependencies {
        let package_name = specification
            .as_table()
            .and_then(|table| table.get("package"))
            .and_then(toml::Value::as_str)
            .unwrap_or(alias);
        if package_name == "sqlx" {
            names.insert(alias.replace('-', "_"));
        }
    }
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
    fn visit_macro(&mut self, node: &'ast Macro) {
        if let Some(query) = sqlx_query_text(node, self.aliases, self.backend_root) {
            self.queries.push(query);
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_item_mod(&mut self, _node: &'ast syn::ItemMod) {}
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

fn validate_query_metadata(backend_root: &Path, query: &str) -> Result<(), String> {
    let hash = sha256_hex(query.as_bytes());
    let metadata_path = backend_root
        .join(".sqlx")
        .join(format!("query-{hash}.json"));
    let raw = fs::read_to_string(&metadata_path).map_err(|_| {
        format!("`SQLX_OFFLINE=true` but there is no cached data for this query: {query}")
    })?;
    let metadata: serde_json::Value = serde_json::from_str(&raw).map_err(|error| {
        format!(
            "failed to read saved query path '{}': {error}",
            metadata_path.display()
        )
    })?;
    if metadata.get("query").and_then(serde_json::Value::as_str) != Some(query)
        || metadata.get("hash").and_then(serde_json::Value::as_str) != Some(hash.as_str())
    {
        return Err(format!("hash collision for saved query data: {query}"));
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
mod tests {
    use super::{
        collect_sqlx_queries, configure_hermetic_build_environment, sqlx_dependency_crate_names,
        sqlx_offline_metadata_digest, validate_sqlx_offline_metadata,
    };
    use sha2::{Digest as _, Sha256};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NONCE: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn hermetic_build_environment_forces_sqlx_offline_without_database_url() {
        let mut command = std::process::Command::new("cargo");
        command.env("DATABASE_URL", "postgres://127.0.0.1:1/sifr");
        configure_hermetic_build_environment(&mut command);
        let environment = command
            .get_envs()
            .map(|(key, value)| {
                (
                    key.to_string_lossy().into_owned(),
                    value.map(|value| value.to_string_lossy().into_owned()),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            environment.get("SQLX_OFFLINE"),
            Some(&Some("true".to_owned()))
        );
        assert_eq!(environment.get("DATABASE_URL"), Some(&None));
    }

    #[test]
    fn real_dependency_table_controls_sqlx_preflight() {
        let fixture = SqlxFixture::new();
        assert_eq!(
            sqlx_dependency_crate_names(&fixture.0),
            Ok(["sqlx".to_string()].into_iter().collect())
        );
        fixture.write_manifest("[dependencies]\nother = \"1\"\n# sqlx macros\n");
        assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));

        fixture.write_manifest(
            "[dependencies]\ndatabase = { package = \"sqlx\", version = \"0.8\" }\n",
        );
        fixture.write_source("fn query() { let _ = database::query!(\"SELECT 13\"); }\n");
        assert!(validate_sqlx_offline_metadata(&fixture.0)
            .expect_err("renamed SQLx dependency must activate preflight")
            .contains("there is no cached data"));
    }

    #[test]
    fn supported_macro_forms_include_aliases_files_concatenation_and_trailing_commas() {
        let fixture = SqlxFixture::new();
        std::fs::create_dir_all(fixture.0.join("queries")).expect("query directory should exist");
        for value in 21..=26 {
            std::fs::write(
                fixture.0.join(format!("queries/value-{value}.sql")),
                format!("SELECT {value}"),
            )
            .expect("query file should be written");
        }
        fixture.write_source(
            r#"
use sqlx::{query as imported_query, query_file_scalar};
use sqlx as database;
struct Row;
fn queries(value: i32) {
    let _ = sqlx::query!("SELECT " + "13", value,);
    let _ = imported_query!("SELECT 14");
    let _ = database::query_as!(Row, "SELECT 15");
    let _ = sqlx::query_unchecked!("SELECT 16");
    let _ = sqlx::query_scalar!("SELECT 17");
    let _ = sqlx::query_scalar_unchecked!("SELECT 18");
    let _ = sqlx::query_as_unchecked!(Row, "SELECT 19");
    let _ = sqlx::query_file!("queries/value-21.sql");
    let _ = sqlx::query_file_unchecked!("queries/value-22.sql");
    let _ = query_file_scalar!("queries/value-23.sql",);
    let _ = sqlx::query_file_scalar_unchecked!("queries/value-24.sql");
    let _ = sqlx::query_file_as!(Row, "queries/value-25.sql");
    let _ = sqlx::query_file_as_unchecked!(Row, "queries/value-26.sql");
}
"#,
        );
        let crate_names = sqlx_dependency_crate_names(&fixture.0).expect("manifest should parse");
        let expected = (13..=19)
            .chain(21..=26)
            .map(|value| format!("SELECT {value}"))
            .collect::<Vec<_>>();
        assert_eq!(collect_sqlx_queries(&fixture.0, &crate_names), expected);
        for query in expected {
            fixture.write_metadata_for(&query, &query);
        }
        assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
    }

    #[test]
    fn syntax_outside_preflight_understanding_falls_through_to_cargo() {
        let fixture = SqlxFixture::new();
        fixture.write_source(
            "const QUERY: &str = \"SELECT 13\";\nfn query() { let _ = sqlx::query!(QUERY); }\n",
        );
        assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
        std::fs::write(fixture.0.join("src/unparseable.rs"), "fn unfinished(")
            .expect("unparseable source should be written");
        assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
    }

    #[test]
    fn valid_checked_in_query_metadata_passes() {
        let fixture = SqlxFixture::new();
        fixture.write_metadata_for(fixture.query(), fixture.query());

        assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
    }

    #[test]
    fn missing_and_stale_query_metadata_fail_closed() {
        let fixture = SqlxFixture::new();
        let missing = validate_sqlx_offline_metadata(&fixture.0)
            .expect_err("missing SQLx metadata must fail");
        assert!(missing.contains("there is no cached data for this query"));

        fixture.write_metadata_for(fixture.query(), "SELECT 12");
        let stale =
            validate_sqlx_offline_metadata(&fixture.0).expect_err("stale SQLx metadata must fail");
        assert!(stale.contains("hash collision for saved query data"));
    }

    #[test]
    fn complete_metadata_directory_participates_in_cache_identity() {
        let fixture = SqlxFixture::new();
        fixture.write_metadata_for(fixture.query(), fixture.query());
        let before =
            sqlx_offline_metadata_digest(&fixture.0).expect("metadata digest should exist");
        let path = fixture.metadata_path(fixture.query());
        let source = std::fs::read_to_string(&path).expect("metadata should be readable");
        std::fs::write(
            &path,
            source.replace("\"describe\":null", "\"describe\":{\"columns\":[]}"),
        )
        .expect("metadata describe mutation should be written");
        let after =
            sqlx_offline_metadata_digest(&fixture.0).expect("metadata digest should still exist");
        assert_ne!(before, after);
    }

    struct SqlxFixture(PathBuf);

    impl SqlxFixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "sifr_sqlx_offline_{}_{}",
                std::process::id(),
                NONCE.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(root.join("src")).expect("source directory should exist");
            let fixture = Self(root);
            fixture.write_source("fn query() { let _ = sqlx::query!(\"SELECT 13\"); }\n");
            fixture.write_manifest(
                "[dependencies]\nsqlx = { version = \"0.8\", features = [\"macros\"] }\n",
            );
            fixture
        }

        const fn query(&self) -> &'static str {
            "SELECT 13"
        }

        fn write_manifest(&self, source: &str) {
            std::fs::write(self.0.join("Cargo.toml"), source)
                .expect("SQLx manifest should be written");
        }

        fn write_source(&self, source: &str) {
            std::fs::write(self.0.join("src/lib.rs"), source)
                .expect("SQLx source should be written");
        }

        fn metadata_path(&self, query: &str) -> PathBuf {
            let hash = hex(&Sha256::digest(query.as_bytes()));
            self.0.join(".sqlx").join(format!("query-{hash}.json"))
        }

        fn write_metadata_for(&self, query: &str, stored_query: &str) {
            let hash = hex(&Sha256::digest(query.as_bytes()));
            let metadata_root = self.0.join(".sqlx");
            std::fs::create_dir_all(&metadata_root).expect("metadata directory should be created");
            let body = serde_json::json!({
                "db_name": "PostgreSQL",
                "query": stored_query,
                "describe": serde_json::Value::Null,
                "hash": hash,
            });
            std::fs::write(
                metadata_root.join(format!("query-{hash}.json")),
                serde_json::to_vec(&body).expect("metadata should serialize"),
            )
            .expect("metadata should be written");
        }
    }

    impl Drop for SqlxFixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }
}
