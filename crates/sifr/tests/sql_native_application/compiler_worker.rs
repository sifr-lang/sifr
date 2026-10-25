//! Real canonical compiler worker; no mock semantic implementation.
use super::Project;
use sifr_sql_tool::compiler_session::CompilerSession;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

const SOURCE: &str = r#"from sifr.sql.migration import MigrationPlan, MigrationState, migration, rollback

@migration(id="all_kinds", parents=["parent_z", "parent_a"], author="Worker fields", created_at="2026-10-25T00:00:00Z")
def all_kinds[S: MigrationState](own plan: MigrationPlan[S]) -> MigrationPlan[S]:
    begun = plan.begin()
    created = begun.ddl(t"ALTER TABLE accounts ADD COLUMN status TEXT NULL")
    populated = created.sql_step(t"UPDATE accounts SET status = 'pending'")
    asserted = populated.assert_sql(t"SELECT status FROM accounts")
    marked = asserted.recovery_point("resume")
    return marked.commit()

@rollback(of="all_kinds")
def reverse[S: MigrationState](own plan: MigrationPlan[S]) -> MigrationPlan[S]:
    return plan.ddl(t"ALTER TABLE accounts DROP COLUMN status")
"#;

fn project(family: &str, version: &str) -> Project {
    let project = Project::with_provider(family, version);
    let mut command = Command::new("cargo");
    command
        .args(["generate-lockfile", "--offline"])
        .current_dir(project.root.path());
    let output = sifr_driver::process_execution::output(&mut command).expect("fixture lockfile");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::create_dir_all(project.root.path().join("migrations/app"))
        .expect("migration directory");
    std::fs::write(
        project.root.path().join("migrations/app/all_kinds.sifr"),
        SOURCE,
    )
    .expect("migration source");
    project
}

fn session(project: &Project) -> CompilerSession {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sifr"));
    command
        .arg("__sql-compiler-worker")
        .arg("--workspace")
        .arg(project.root.path())
        .current_dir(project.root.path());
    CompilerSession::spawn_admitted(
        &mut command,
        env!("SIFR_COMPILER_SOURCE_ID"),
        Duration::from_secs(60),
    )
    .expect("canonical compiler worker")
}

#[test]
fn canonical_profiles_and_all_migration_fields_match_in_process_semantics() {
    for (family, version) in [("sqlite", "3.53.4"), ("mysql", "8.4"), ("postgresql", "18")] {
        let project = project(family, version);
        let expected = sifr_semantic_services::load_sql_editor_profiles(
            project.root.path(),
            project.root.path(),
            None,
        )
        .expect("canonical profile inputs");
        let expected = expected
            .registry()
            .profile("app")
            .expect("app profile")
            .authority()
            .clone();
        let mut worker = session(&project);
        assert_eq!(
            worker.profile("app").expect("worker profile"),
            expected,
            "{family}"
        );
        let context = sifr_semantic_services::SemanticContext::new(
            sifr_identity::CompilerIdentity::product(env!("SIFR_COMPILER_BUILD_ID"))
                .expect("build identity"),
        );
        let expected = sifr_semantic_services::compile_sql_migration_source(&context, SOURCE)
            .expect("canonical migration");
        assert_eq!(
            worker.migration(SOURCE).expect("worker migration"),
            expected,
            "{family}"
        );
        worker.finish().expect("clean worker exit");
    }
}

#[test]
fn canonical_worker_rejects_lock_manifest_schema_and_component_drift() {
    for path in [
        "Cargo.lock",
        "sifr.toml",
        "db/schema.sql",
        "provider/components/sqlite.wasm",
    ] {
        let project = project("sqlite", "3.53.4");
        let mut worker = session(&project);
        worker.profile("app").expect("initial profile");
        let suffix = if path.ends_with(".sql") {
            "\n-- drift\n"
        } else {
            "\n# drift\n"
        };
        std::fs::OpenOptions::new()
            .append(true)
            .open(project.root.path().join(path))
            .expect("owned input")
            .write_all(suffix.as_bytes())
            .expect("change input");
        let error = worker.profile("app").expect_err("input drift must fail");
        assert!(error.contains("changed"), "{path}: {error}");
    }
}

#[test]
fn canonical_worker_admits_source_bytes_and_preserves_semantic_diagnostics() {
    let project = project("sqlite", "3.53.4");
    let mut worker = session(&project);
    assert!(
        worker
            .migration("outside source")
            .expect_err("unadmitted source")
            .contains("not an admitted")
    );
    let invalid = "def broken(:\n";
    std::fs::write(
        project.root.path().join("migrations/app/invalid.sifr"),
        invalid,
    )
    .expect("invalid source fixture");
    let context = sifr_semantic_services::SemanticContext::new(
        sifr_identity::CompilerIdentity::product(env!("SIFR_COMPILER_BUILD_ID"))
            .expect("build identity"),
    );
    let expected = sifr_semantic_services::compile_sql_migration_source(&context, invalid)
        .expect_err("canonical rejection")
        .into_iter()
        .map(|error| error.message)
        .collect::<Vec<_>>()
        .join("\n");
    let mut worker = session(&project);
    assert_eq!(
        worker.migration(invalid).expect_err("worker rejection"),
        expected
    );
}

#[test]
fn canonical_worker_rejects_migration_drift_between_requests() {
    let project = project("sqlite", "3.53.4");
    let mut worker = session(&project);
    worker.migration(SOURCE).expect("first migration");
    let changed = format!("{SOURCE}\n# edited\n");
    std::fs::write(
        project.root.path().join("migrations/app/all_kinds.sifr"),
        &changed,
    )
    .expect("edited migration");
    assert!(
        worker
            .migration(&changed)
            .expect_err("migration drift")
            .contains("changed")
    );
}
