use sha2::{Digest as _, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream};
use syn::visit::Visit;
use syn::{Expr, LitStr, Macro, Token, Type};

pub(super) fn configure_hermetic_build_environment(command: &mut std::process::Command) {
    command.env("SQLX_OFFLINE", "true");
    command.env_remove("DATABASE_URL");
}

pub(super) fn validate_sqlx_offline_metadata(backend_root: &Path) -> Result<(), String> {
    let manifest_path = backend_root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "failed to read Rust package manifest '{}': {error}",
            manifest_path.display()
        )
    })?;
    if !manifest.contains("sqlx") || !manifest.contains("macros") {
        return Ok(());
    }
    let queries = collect_sqlx_queries(backend_root)?;
    for query in queries {
        validate_query_metadata(backend_root, &query)?;
    }
    Ok(())
}

fn collect_sqlx_queries(backend_root: &Path) -> Result<Vec<String>, String> {
    let mut rust_sources = Vec::new();
    let source_root = backend_root.join("src");
    if !source_root.is_dir() {
        return Ok(Vec::new());
    }
    collect_rust_sources(&source_root, &source_root, &mut rust_sources)?;
    let mut queries = Vec::new();
    for source_path in rust_sources {
        let source = fs::read_to_string(&source_path).map_err(|error| {
            format!(
                "failed to read SQLx bridge source '{}': {error}",
                source_path.display()
            )
        })?;
        let syntax = syn::parse_file(&source).map_err(|error| {
            format!(
                "failed to parse SQLx bridge source '{}': {error}",
                source_path.display()
            )
        })?;
        let mut visitor = SqlxQueryVisitor::default();
        visitor.visit_file(&syntax);
        if let Some(error) = visitor.error {
            return Err(format!("{} in '{}'", error, source_path.display()));
        }
        queries.extend(visitor.queries);
    }
    queries.sort();
    queries.dedup();
    Ok(queries)
}

fn collect_rust_sources(
    backend_root: &Path,
    current: &Path,
    sources: &mut Vec<PathBuf>,
) -> Result<(), String> {
    if current.is_file() {
        if current
            .extension()
            .is_some_and(|extension| extension == "rs")
        {
            sources.push(current.to_path_buf());
        }
        return Ok(());
    }
    let entries = fs::read_dir(current).map_err(|error| {
        format!(
            "failed to inspect Rust package path '{}': {error}",
            current.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to inspect Rust package path '{}': {error}",
                current.display()
            )
        })?;
        let path = entry.path();
        let relative = path.strip_prefix(backend_root).unwrap_or(&path);
        if relative
            .components()
            .any(|component| matches!(component.as_os_str().to_str(), Some("target" | ".git")))
        {
            continue;
        }
        collect_rust_sources(backend_root, &path, sources)?;
    }
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

#[derive(Default)]
struct SqlxQueryVisitor {
    queries: Vec<String>,
    error: Option<String>,
}

impl<'ast> Visit<'ast> for SqlxQueryVisitor {
    fn visit_macro(&mut self, node: &'ast Macro) {
        if self.error.is_none() {
            match sqlx_query_literal(node) {
                Ok(Some(query)) => self.queries.push(query),
                Ok(None) => {}
                Err(error) => self.error = Some(error),
            }
        }
        syn::visit::visit_macro(self, node);
    }
}

fn sqlx_query_literal(node: &Macro) -> Result<Option<String>, String> {
    let segments = node
        .path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    let Some(name) = segments.last().map(String::as_str) else {
        return Ok(None);
    };
    if segments.first().map(String::as_str) != Some("sqlx") {
        return Ok(None);
    }
    let literal = match name {
        "query" | "query_scalar" | "query_unchecked" | "query_scalar_unchecked" => {
            syn::parse2::<QueryInput>(node.tokens.clone())
                .map(|input| input.query)
                .map_err(|error| {
                    format!("SQLx {name}! must start with a string literal: {error}")
                })?
        }
        "query_as" | "query_as_unchecked" => syn::parse2::<QueryAsInput>(node.tokens.clone())
            .map(|input| input.query)
            .map_err(|error| {
                format!("SQLx {name}! must name a type and query string literal: {error}")
            })?,
        _ => return Ok(None),
    };
    Ok(Some(literal.value()))
}

struct QueryInput {
    query: LitStr,
}

impl Parse for QueryInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let query = input.parse()?;
        parse_bind_arguments(input)?;
        Ok(Self { query })
    }
}

struct QueryAsInput {
    query: LitStr,
}

impl Parse for QueryAsInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let _output: Type = input.parse()?;
        input.parse::<Token![,]>()?;
        let query = input.parse()?;
        parse_bind_arguments(input)?;
        Ok(Self { query })
    }
}

fn parse_bind_arguments(input: ParseStream<'_>) -> syn::Result<()> {
    while !input.is_empty() {
        input.parse::<Token![,]>()?;
        let _argument: Expr = input.parse()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{configure_hermetic_build_environment, validate_sqlx_offline_metadata};
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
    fn valid_checked_in_query_metadata_passes() {
        let fixture = SqlxFixture::new();
        fixture.write_metadata(fixture.query());

        assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
    }

    #[test]
    fn missing_and_stale_query_metadata_fail_closed() {
        let fixture = SqlxFixture::new();
        let missing = validate_sqlx_offline_metadata(&fixture.0)
            .expect_err("missing SQLx metadata must fail");
        assert!(missing.contains("there is no cached data for this query"));

        fixture.write_metadata("SELECT 12");
        let stale =
            validate_sqlx_offline_metadata(&fixture.0).expect_err("stale SQLx metadata must fail");
        assert!(stale.contains("hash collision for saved query data"));
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
            std::fs::write(
                root.join("src/lib.rs"),
                "fn query() { let _ = sqlx::query!(\"SELECT 13\"); }\n",
            )
            .expect("SQLx source should be written");
            std::fs::write(
                root.join("Cargo.toml"),
                "[dependencies]\nsqlx = { version = \"0.8\", features = [\"macros\"] }\n",
            )
            .expect("SQLx manifest should be written");
            Self(root)
        }

        const fn query(&self) -> &'static str {
            "SELECT 13"
        }

        fn write_metadata(&self, stored_query: &str) {
            let digest = Sha256::digest(self.query().as_bytes());
            let hash = hex(&digest);
            let metadata_root = self.0.join(".sqlx");
            std::fs::create_dir_all(&metadata_root).expect("metadata directory should be created");
            let body = serde_json::json!({
                "query": stored_query,
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
