use sifr_sql_contract::{MigrationSourceDeclaration, ProfileAuthority};
use sifr_sql_sqlite_tools as tools;
use std::cell::Cell;
use std::path::Path;

fn compile_never(_: &str) -> Result<Vec<MigrationSourceDeclaration>, String> {
    panic!("command must not request migration compilation");
}

#[tokio::test]
async fn invalid_schema_arguments_do_not_load_profiles() {
    let root = tempfile::tempdir().expect("fixture");
    let calls = Cell::new(0);
    let load = |_: &Path, _: &str| -> Result<ProfileAuthority, String> {
        calls.set(calls.get() + 1);
        Err("unexpected profile request".into())
    };
    let result = tools::run_schema_command_with_profile_loader(
        &load,
        &["unknown".into()],
        root.path(),
        None,
        &mut |_| Ok(()),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(calls.get(), 0);
}

#[tokio::test]
async fn profile_failure_preserves_message_and_prevents_schema_writes() {
    let root = tempfile::tempdir().expect("fixture");
    let calls = Cell::new(0);
    let load = |path: &Path, profile: &str| -> Result<ProfileAuthority, String> {
        assert_eq!(path, root.path());
        assert_eq!(profile, "app");
        calls.set(calls.get() + 1);
        Err("canonical profile rejection\noriginal detail".into())
    };
    let error = tools::run_schema_command_with_profile_loader(
        &load,
        &["build".into(), "--profile".into(), "app".into()],
        root.path(),
        None,
        &mut |_| panic!("failed preparation must not emit"),
    )
    .await
    .expect_err("profile failure");
    assert_eq!(
        error.message,
        "canonical profile rejection\noriginal detail"
    );
    assert_eq!(calls.get(), 1);
    assert_eq!(
        std::fs::read_dir(root.path()).expect("directory").count(),
        0
    );
}

#[tokio::test]
async fn profile_failure_prevents_migration_compilation_and_writes() {
    let root = tempfile::tempdir().expect("fixture");
    let load = |_: &Path, _: &str| -> Result<ProfileAuthority, String> {
        Err("canonical profile rejection".into())
    };
    let error = tools::run_migration_command_with_compiler(
        &compile_never,
        &load,
        &["build".into(), "--profile".into(), "app".into()],
        root.path(),
        None,
    )
    .await
    .expect_err("profile failure");
    assert_eq!(error.message, "canonical profile rejection");
    assert_eq!(
        std::fs::read_dir(root.path()).expect("directory").count(),
        0
    );
}

#[tokio::test]
async fn reading_a_plan_does_not_request_compiler_capabilities() {
    let root = tempfile::tempdir().expect("fixture");
    let load = |_: &Path, _: &str| -> Result<ProfileAuthority, String> {
        panic!("plan reads must not prepare a profile");
    };
    let error = tools::run_migration_command_with_compiler(
        &compile_never,
        &load,
        &["plan".into(), "--profile".into(), "app".into()],
        root.path(),
        None,
    )
    .await
    .expect_err("missing plan");
    assert!(
        error.message.contains("migration plan"),
        "{}",
        error.message
    );
}

#[tokio::test]
async fn profile_failure_prevents_provisioning_side_effects() {
    let root = tempfile::tempdir().expect("fixture");
    let load = |_: &Path, _: &str| -> Result<ProfileAuthority, String> {
        Err("canonical provision rejection".into())
    };
    let error = tools::provision_test_database_with_profile_loader(&load, root.path(), "app")
        .await
        .expect_err("profile failure");
    assert_eq!(error.message, "canonical provision rejection");
    assert_eq!(
        std::fs::read_dir(root.path()).expect("directory").count(),
        0
    );
}
