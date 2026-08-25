use super::{
    collect_sqlx_queries, combined_sqlx_offline_metadata_digest,
    configure_hermetic_build_environment, sqlx_dependency_crate_names,
    sqlx_offline_metadata_digest, validate_sqlx_offline_metadata,
};
use sha2::{Digest as _, Sha256};
use std::path::{Path, PathBuf};
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

    fixture
        .write_manifest("[dependencies]\ndatabase = { package = \"sqlx\", version = \"0.9\" }\n");
    fixture.write_source("fn query() { let _ = database::query!(\"SELECT 13\"); }\n");
    assert!(
        validate_sqlx_offline_metadata(&fixture.0)
            .expect_err("renamed SQLx dependency must activate preflight")
            .contains("there is no cached data")
    );
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
fn syn_3_safe_foreign_items_do_not_hide_sqlx_queries() {
    let fixture = SqlxFixture::new();
    fixture.write_source(
        r#"
unsafe extern "C" {
    safe fn host_callback();
}

fn query() {
    let _ = sqlx::query!("SELECT 27");
}
"#,
    );
    let crate_names = sqlx_dependency_crate_names(&fixture.0).expect("manifest should parse");

    assert_eq!(
        collect_sqlx_queries(&fixture.0, &crate_names),
        vec!["SELECT 27".to_string()]
    );
}

#[test]
fn syntax_outside_preflight_understanding_falls_through_to_cargo() {
    let fixture = SqlxFixture::new();
    fixture.write_source(
        r#"
const QUERY: &str = "SELECT 13";
macro_rules! wrapper { ($query:expr) => { $query }; }
mod unparseable;
fn query() {
    use sqlx::query;
    let _ = sqlx::query!(QUERY);
    let _ = query!("SELECT 14");
    let _ = wrapper!(sqlx::query!("SELECT 15"));
}
"#,
    );
    let crate_names = sqlx_dependency_crate_names(&fixture.0).expect("manifest should parse");
    assert_eq!(
        collect_sqlx_queries(&fixture.0, &crate_names),
        Vec::<String>::new()
    );
    assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
    std::fs::write(fixture.0.join("src/unparseable.rs"), "fn unfinished(")
        .expect("unparseable source should be written");
    assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
}

#[test]
fn cfg_gated_queries_fall_through_to_cargo() {
    let fixture = SqlxFixture::new();
    fixture.write_source(
        r#"
fn active() {
    let _ = sqlx::query!("SELECT 13");
}

#[cfg(test)]
mod tests {
    fn query() {
        let _ = sqlx::query!("SELECT 91");
    }
}

#[cfg(test)]
fn test_only_query() {
    let _ = sqlx::query!("SELECT 92");
}

#[cfg(feature = "mysql-variant")]
fn disabled_feature_query() {
    let _ = sqlx::query!("SELECT 93");
}

#[cfg_attr(any(), allow(dead_code))]
fn cfg_attr_query() {
    let _ = sqlx::query!("SELECT 96");
}

#[cfg_attr(feature = "conditional-tests", cfg(test))]
fn conditionally_disabled_query() {
    let _ = sqlx::query!("SELECT 98");
}

struct QueryHolder;

impl QueryHolder {
    #[cfg(test)]
    fn test_only_query() {
        let _ = sqlx::query!("SELECT 97");
    }
}

struct GatedFieldHolder {
    #[cfg(test)]
    value: (),
}

fn gated_field_value() {
    let _ = GatedFieldHolder {
        #[cfg(test)]
        value: {
            let _ = sqlx::query!("SELECT 99");
        },
    };
}

fn gated_statements() {
    #[cfg(test)]
    let _ = sqlx::query!("SELECT 94");

    #[cfg(feature = "mysql-variant")]
    sqlx::query!("SELECT 95");
}
"#,
    );
    let crate_names = sqlx_dependency_crate_names(&fixture.0).expect("manifest should parse");
    assert_eq!(
        collect_sqlx_queries(&fixture.0, &crate_names),
        vec!["SELECT 13".to_string(), "SELECT 96".to_string()]
    );
    for query in ["SELECT 13", "SELECT 96"] {
        fixture.write_metadata_for(query, query);
    }
    assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
}

#[test]
fn crate_module_graph_skips_gated_and_orphan_source_files() {
    let fixture = SqlxFixture::new();
    fixture.write_source(
        r#"
mod active;

#[path = "redirected/active.rs"]
mod redirected_active;

#[path = "alt_inline"]
mod inline_redirected {
    mod child;
}

#[cfg(test)]
mod tests;

#[cfg(feature = "mysql-variant")]
mod variant;

mod inner_gated;

#[cfg(test)]
#[path = "redirected/gated.rs"]
mod redirected_gated;
"#,
    );
    fixture.write_rust_file(
        "src/active.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 13\"); }\n",
    );
    fixture.write_rust_file(
        "src/redirected/active.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 14\"); }\n",
    );
    fixture.write_rust_file(
        "src/alt_inline/child.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 15\"); }\n",
    );
    fixture.write_rust_file(
        "src/inline_redirected/child.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 90\"); }\n",
    );
    fixture.write_rust_file(
        "src/tests.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 91\"); }\n",
    );
    fixture.write_rust_file(
        "src/variant.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 92\"); }\n",
    );
    fixture.write_rust_file(
        "src/inner_gated.rs",
        "#![cfg(test)]\nfn query() { let _ = sqlx::query!(\"SELECT 95\"); }\n",
    );
    fixture.write_rust_file(
        "src/redirected/gated.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 93\"); }\n",
    );
    fixture.write_rust_file(
        "src/bin/tool.rs",
        "fn main() { let _ = sqlx::query!(\"SELECT 94\"); }\n",
    );

    let crate_names = sqlx_dependency_crate_names(&fixture.0).expect("manifest should parse");
    let expected = vec![
        "SELECT 13".to_string(),
        "SELECT 14".to_string(),
        "SELECT 15".to_string(),
    ];
    assert_eq!(collect_sqlx_queries(&fixture.0, &crate_names), expected);
    for query in ["SELECT 13", "SELECT 14", "SELECT 15"] {
        fixture.write_metadata_for(query, query);
    }
    assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
}

#[test]
fn explicit_paths_from_file_modules_follow_rust_directory_rules() {
    let fixture = SqlxFixture::new();
    fixture.write_source("mod outer;\n");
    fixture.write_rust_file(
        "src/outer.rs",
        r#"
#[path = "direct.rs"]
mod direct;

#[path = "alt_inline"]
mod inline_redirected {
    mod child;
}

#[path = "loaded.rs"]
mod loaded;

mod r#async;

mod r#type {
    mod child;
}
"#,
    );

    fixture.write_rust_file(
        "src/direct.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 13\"); }\n",
    );
    fixture.write_rust_file(
        "src/outer/direct.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 90\"); }\n",
    );
    fixture.write_rust_file(
        "src/alt_inline/child.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 14\"); }\n",
    );
    fixture.write_rust_file(
        "src/outer/alt_inline/child.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 91\"); }\n",
    );
    fixture.write_rust_file("src/loaded.rs", "mod child;\n");
    fixture.write_rust_file(
        "src/child.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 15\"); }\n",
    );
    fixture.write_rust_file(
        "src/loaded/child.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 92\"); }\n",
    );
    fixture.write_rust_file(
        "src/outer/async.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 16\"); }\n",
    );
    fixture.write_rust_file(
        "src/outer/r#async.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 93\"); }\n",
    );
    fixture.write_rust_file(
        "src/outer/type/child.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 17\"); }\n",
    );
    fixture.write_rust_file(
        "src/outer/r#type/child.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 94\"); }\n",
    );

    let crate_names = sqlx_dependency_crate_names(&fixture.0).expect("manifest should parse");
    let expected = vec![
        "SELECT 13".to_string(),
        "SELECT 14".to_string(),
        "SELECT 15".to_string(),
        "SELECT 16".to_string(),
        "SELECT 17".to_string(),
    ];
    assert_eq!(collect_sqlx_queries(&fixture.0, &crate_names), expected);
    for query in expected {
        fixture.write_metadata_for(&query, &query);
    }
    assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
}

#[test]
fn cargo_entrypoint_selection_follows_lib_path_then_main_fallback() {
    let library_fixture = SqlxFixture::new();
    library_fixture.write_manifest(
        "[lib]\npath = \"source/entry.rs\"\n\n[dependencies]\nsqlx = { version = \"0.9\", features = [\"macros\"] }\n",
    );
    library_fixture.write_rust_file(
        "source/entry.rs",
        "fn query() { let _ = sqlx::query!(\"SELECT 13\"); }\n",
    );
    let library_names =
        sqlx_dependency_crate_names(&library_fixture.0).expect("library manifest should parse");
    assert_eq!(
        collect_sqlx_queries(&library_fixture.0, &library_names),
        vec!["SELECT 13".to_string()]
    );

    let main_fixture = SqlxFixture::new();
    std::fs::remove_file(main_fixture.0.join("src/lib.rs"))
        .expect("default library entry should be removed");
    main_fixture.write_rust_file(
        "src/main.rs",
        "fn main() { let _ = sqlx::query!(\"SELECT 14\"); }\n",
    );
    let main_names =
        sqlx_dependency_crate_names(&main_fixture.0).expect("main manifest should parse");
    assert_eq!(
        collect_sqlx_queries(&main_fixture.0, &main_names),
        vec!["SELECT 14".to_string()]
    );
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
    let missing =
        validate_sqlx_offline_metadata(&fixture.0).expect_err("missing SQLx metadata must fail");
    assert!(missing.contains("there is no cached data for this query"));

    fixture.write_metadata_for(fixture.query(), "SELECT 12");
    let stale =
        validate_sqlx_offline_metadata(&fixture.0).expect_err("stale SQLx metadata must fail");
    assert!(stale.contains("saved SQLx query text does not match query identity"));
}

#[test]
fn workspace_metadata_and_workspace_dependency_renames_are_resolved() {
    let fixture = SqlxFixture::new_workspace_member();
    fixture.write_manifest("[dependencies]\ndatabase = { workspace = true }\n");
    fixture.write_source("fn query() { let _ = database::query!(\"SELECT 13\"); }\n");
    fixture.write_workspace_metadata_for(fixture.query(), fixture.query());

    assert_eq!(
        sqlx_dependency_crate_names(&fixture.0),
        Ok(["database".to_string()].into_iter().collect())
    );
    assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
    assert!(
        sqlx_offline_metadata_digest(&fixture.0).is_some(),
        "workspace-root metadata must participate in cache identity"
    );
}

#[test]
fn explicit_offline_directory_disengages_conservative_preflight() {
    let fixture = SqlxFixture::new();
    std::fs::write(
        fixture.0.join(".env"),
        "SQLX_OFFLINE_DIR=/external/sqlx-cache\n",
    )
    .expect("offline directory policy should be written");

    assert_eq!(validate_sqlx_offline_metadata(&fixture.0), Ok(()));
    assert_eq!(sqlx_offline_metadata_digest(&fixture.0), None);
}

#[test]
fn complete_metadata_directory_participates_in_cache_identity() {
    let fixture = SqlxFixture::new();
    let bridge_fixture = SqlxFixture::new();
    fixture.write_metadata_for(fixture.query(), fixture.query());
    bridge_fixture.write_metadata_for(bridge_fixture.query(), bridge_fixture.query());
    let before =
        combined_sqlx_offline_metadata_digest([fixture.0.as_path(), bridge_fixture.0.as_path()])
            .expect("metadata digest should exist");
    let path = bridge_fixture.metadata_path(bridge_fixture.query());
    let source = std::fs::read_to_string(&path).expect("metadata should be readable");
    std::fs::write(
        &path,
        source.replace("\"describe\":null", "\"describe\":{\"columns\":[]}"),
    )
    .expect("metadata describe mutation should be written");
    let after =
        combined_sqlx_offline_metadata_digest([fixture.0.as_path(), bridge_fixture.0.as_path()])
            .expect("metadata digest should still exist");
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
            "[dependencies]\nsqlx = { version = \"0.9\", features = [\"macros\"] }\n",
        );
        fixture
    }

    fn new_workspace_member() -> Self {
        let workspace = std::env::temp_dir().join(format!(
            "sifr_sqlx_workspace_{}_{}",
            std::process::id(),
            NONCE.fetch_add(1, Ordering::Relaxed)
        ));
        let root = workspace.join("member");
        std::fs::create_dir_all(root.join("src")).expect("source directory should exist");
        std::fs::write(
            workspace.join("Cargo.toml"),
            "[workspace]\nmembers = [\"member\"]\nresolver = \"3\"\n\n[workspace.dependencies]\ndatabase = { package = \"sqlx\", version = \"0.9\", features = [\"macros\"] }\n",
        )
        .expect("workspace manifest should be written");
        let fixture = Self(root);
        fixture.write_source("fn query() { let _ = database::query!(\"SELECT 13\"); }\n");
        fixture.write_manifest("[dependencies]\ndatabase = { workspace = true }\n");
        fixture
    }

    const fn query(&self) -> &'static str {
        "SELECT 13"
    }

    fn write_manifest(&self, source: &str) {
        std::fs::write(self.0.join("Cargo.toml"), source).expect("SQLx manifest should be written");
    }

    fn write_source(&self, source: &str) {
        std::fs::write(self.0.join("src/lib.rs"), source).expect("SQLx source should be written");
    }

    fn write_rust_file(&self, relative_path: &str, source: &str) {
        let path = self.0.join(relative_path);
        std::fs::create_dir_all(path.parent().expect("Rust source should have a parent"))
            .expect("Rust source directory should be created");
        std::fs::write(path, source).expect("Rust source should be written");
    }

    fn metadata_path(&self, query: &str) -> PathBuf {
        let hash = hex(&Sha256::digest(query.as_bytes()));
        self.0.join(".sqlx").join(format!("query-{hash}.json"))
    }

    fn write_metadata_for(&self, query: &str, stored_query: &str) {
        self.write_metadata_at(&self.0.join(".sqlx"), query, stored_query);
    }

    fn write_workspace_metadata_for(&self, query: &str, stored_query: &str) {
        self.write_metadata_at(
            &self
                .0
                .parent()
                .expect("workspace member should have a parent")
                .join(".sqlx"),
            query,
            stored_query,
        );
    }

    fn write_metadata_at(&self, metadata_root: &Path, query: &str, stored_query: &str) {
        let hash = hex(&Sha256::digest(query.as_bytes()));
        std::fs::create_dir_all(metadata_root).expect("metadata directory should be created");
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
        let workspace = self.0.parent().filter(|parent| {
            parent
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("sifr_sqlx_workspace_"))
        });
        let _ = std::fs::remove_dir_all(workspace.unwrap_or(&self.0));
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
