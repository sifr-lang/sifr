#![allow(clippy::expect_used)]
//! Real Sifr packages, actual provider components, and native SQLite execution.
use sha2::{Digest as _, Sha256};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

struct Project {
    root: tempfile::TempDir,
    database: PathBuf,
}
impl Project {
    fn new() -> Self {
        let root = tempfile::tempdir().expect("project directory");
        let database = root.path().join("actual.sqlite3");
        rusqlite::Connection::open(&database)
            .expect("actual database")
            .execute_batch("VACUUM")
            .expect("initialize empty catalog");
        let provider = root.path().join("provider");
        for directory in [
            root.path().join("src"),
            root.path().join("db"),
            provider.join("src"),
            provider.join("components"),
        ] {
            std::fs::create_dir_all(directory).expect("fixture directory");
        }
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let bytes =
            std::fs::read(repository.join("crates/sifr_sql_sqlite/components/sqlite-3.53.4.wasm"))
                .expect("actual component");
        let sha256: String = Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        std::fs::write(provider.join("components/sqlite.wasm"), bytes).expect("component fixture");
        let diagnostics = (1..=11)
            .map(|index| format!("{{ code = \"SIFR-SQLITE-{index:04}\", lifecycle = \"active\" }}"))
            .collect::<Vec<_>>()
            .join(", ");
        std::fs::write(
            provider.join("sifr.toml"),
            format!(
                r#"[package]
name="sifr_sql_sqlite"
edition="2026"
sifr-version=">=0.3,<0.4"
[source]
root="src"
[compiler-components.sqlite]
kind="embedded-language-provider"
artifact="components/sqlite.wasm"
version="1.0.0"
sha256="{sha256}"
protocol-min=1
protocol-max=1
processors=["sifr.sql.sqlite.schema","sifr.sql.sqlite.sql"]
diagnostic-namespace="SQLITE"
diagnostics=[{diagnostics}]
"#
            ),
        )
        .expect("provider manifest");
        std::fs::write(provider.join("Cargo.toml"), "[package]\nname=\"sql-provider\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[package.metadata.sifr]\nmanifest=\"sifr.toml\"\n").expect("provider Cargo owner");
        std::fs::write(provider.join("src/lib.rs"), "").expect("provider Rust marker");
        std::fs::write(provider.join("src/__init__.sifr"), "").expect("provider source marker");
        std::fs::write(root.path().join("Cargo.toml"), "[package]\nname=\"sql-app\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[package.metadata.sifr]\nmanifest=\"sifr.toml\"\n[dependencies]\nprovider={package=\"sql-provider\",path=\"provider\"}\n[workspace]\n").expect("real Cargo package");
        std::fs::write(root.path().join("src/main.rs"), "fn main() {}")
            .expect("application Rust marker");
        std::fs::write(root.path().join("sifr.toml"), format!("[package]\nname=\"sql_app\"\nedition=\"2026\"\nsifr-version=\">=0.3,<0.4\"\n[source]\nroot=\"src\"\n{}", profile("app"))).expect("application profile");
        std::fs::write(root.path().join("db/schema.sql"), "").expect("empty declared catalog");
        Self { root, database }
    }
    fn source(&self, name: &str, source: &str) {
        std::fs::write(self.root.path().join("src").join(name), source)
            .expect("application source");
    }
    fn cli(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_sifr"))
            .args(args)
            .current_dir(self.root.path())
            .output()
            .expect("actual Sifr CLI")
    }
    fn rejects(&self, source: &str, message: &str) {
        self.source("main.sifr", source);
        let output = self.cli(&["check", "src/main.sifr", "--offline"]);
        let diagnostics = String::from_utf8_lossy(&output.stderr);
        assert!(
            !output.status.success(),
            "negative application compiled: {source}"
        );
        assert!(diagnostics.contains(message), "{diagnostics}");
    }
}
fn profile(name: &str) -> String {
    format!(
        "[sql.profiles.{name}]\nprovider=\"provider\"\nfamily=\"sqlite\"\nsource=\"db/schema.sql\"\nserver-version=\"3.53.4\"\nsearch-path=[\"main\"]\nschema-evidence=\"introspection\"\nschema-strictness=\"exact\"\npooling=\"session\"\n"
    )
}

#[test]
fn ordinary_source_imports_and_executes_decorated_and_standalone_typed_queries() {
    let project = Project::new();
    project.source("queries.sifr", "from sifr.sql.schemas import app\n\n@app.query\ndef selected(value: int64, text: str):\n    return app.sql(t\"SELECT {value} AS z, {text} AS a\")\n\ndef standalone(value: str):\n    return app.sql(t\"SELECT {value} AS label\")\n");
    project.source(
        "main.sifr",
        &format!(
            r#"from sifr.sql.schemas import app
from sql_app.queries import selected, standalone
async def main():
    try:
        db = await app.connect({path:?})
        value: int64 = 41
        one: int64 = 1
        row = await db.fetch_one(selected(value, "column order"))
        number: int64 = row.z
        result: int = number + one
        print(result)
        text: str = row.a
        print(text)
        text_row = await db.fetch_one(standalone("bound text"))
        label: str = text_row.label
        print(label)
        rows = await db.fetch_all(selected(value, "again"), max_rows=1)
        for item in rows:
            print(item.z)
        empty = await db.fetch_optional(app.sql(t"SELECT {{value}} AS value WHERE 0"))
        print(empty is None)
    except:
        print("SQL execution failed")
"#,
            path = project.database.display().to_string()
        ),
    );
    let build = project.cli(&["build", "src/main.sifr", "--offline", "-o", "native"]);
    assert!(
        build.status.success(),
        "{}",
        String::from_utf8_lossy(&build.stderr)
    );
    let binary = project
        .root
        .path()
        .join("native/sifr_output/target/final")
        .join(if cfg!(windows) {
            "sifr_output.exe"
        } else {
            "sifr_output"
        });
    let run = Command::new(&binary)
        .output()
        .expect("compiled native application");
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout),
        "42\ncolumn order\nbound text\n41\nTrue\n"
    );
    assert!(
        std::fs::read(&project.database)
            .expect("database bytes")
            .starts_with(b"SQLite format 3\0")
    );
    // The same binary must reject an observed catalog differing from its contract.
    rusqlite::Connection::open(&project.database)
        .expect("catalog mutation")
        .execute_batch("CREATE TABLE drift(value TEXT)")
        .expect("real schema drift");
    let drift = Command::new(&binary)
        .output()
        .expect("native drift rejection");
    assert_eq!(
        String::from_utf8_lossy(&drift.stdout),
        "SQL execution failed\n"
    );
}

#[test]
fn standalone_sql_is_validated_before_erasure() {
    Project::new().rejects(
        "from sifr.sql.schemas import app\ndef main():\n    query = app.sql(t\"SELEC broken\")\n",
        "SIFR-COMPONENT",
    );
}
#[test]
fn imported_query_rows_reject_unknown_fields() {
    let project = Project::new();
    project.source("queries.sifr", "from sifr.sql.schemas import app\ndef selected(value: int64):\n    return app.sql(t\"SELECT {value} AS value\")\n");
    project.rejects("from sifr.sql.schemas import app\nfrom sql_app.queries import selected\nasync def main():\n    try:\n        db = await app.connect(\"db.sqlite3\")\n        value: int64 = 1\n        row = await db.fetch_one(selected(value))\n        print(row.missing)\n    except:\n        pass\n", "missing");
}
#[test]
fn exact_pool_profile_is_required_for_imported_bound_queries() {
    let project = Project::new();
    let manifest = project.root.path().join("sifr.toml");
    let mut source = std::fs::read_to_string(&manifest).expect("manifest");
    source.push_str(&profile("analytics"));
    std::fs::write(manifest, source).expect("second nominal profile");
    project.rejects("from sifr.sql.schemas import app, analytics\nasync def main():\n    try:\n        db = await app.connect(\"db.sqlite3\")\n        value: int64 = 1\n        row = await db.fetch_one(analytics.sql(t\"SELECT {value} AS value\"))\n    except:\n        pass\n", "profile identities differ");
}
#[test]
fn decorated_query_must_return_its_identity_on_every_path() {
    Project::new().rejects("from sifr.sql.schemas import app\n@app.query\ndef selected(value: int64, condition: bool):\n    if condition:\n        return app.sql(t\"SELECT {value} AS value\")\n    return None\n", "return type mismatch");
}
#[test]
fn native_connection_reports_the_populated_profile_boundary() {
    let project = Project::new();
    std::fs::write(
        project.root.path().join("db/schema.sql"),
        "CREATE TABLE users(id INTEGER PRIMARY KEY) STRICT",
    )
    .expect("populated profile");
    project.rejects("from sifr.sql.schemas import app\nasync def main():\n    try:\n        db = await app.connect(\"db.sqlite3\")\n    except:\n        pass\n", "populated profiles require provider runtime introspection");
}

#[test]
fn bounded_fetch_rejects_zero_and_missing_limits() {
    for suffix in [", max_rows=0", ""] {
        let source = format!(
            "from sifr.sql.schemas import app\nasync def main():\n    try:\n        db = await app.connect(\"db.sqlite3\")\n        value: int64 = 1\n        rows = await db.fetch_all(app.sql(t\"SELECT {{value}} AS value\"){suffix})\n    except:\n        pass\n"
        );
        Project::new().rejects(&source, "bound");
    }
}
#[test]
fn native_query_rejects_unsupported_exact_integer_codec() {
    Project::new().rejects("from sifr.sql.schemas import app\ndef main():\n    value: int = 1\n    query = app.sql(t\"SELECT {value} AS value\")\n", "codec");
}
