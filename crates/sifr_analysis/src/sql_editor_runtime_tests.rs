use super::*;
use sifr_frontend::{
    CacheFamily, CompilerFingerprint, FrontendMode, PackageContextFingerprint, SourcePath,
    WorkspaceContextFingerprint, WorkspacePackageConfigIdentity,
};

fn context() -> CacheKeyContext {
    CacheKeyContext::new(
        CacheFamily::EmbeddedAnalysis,
        CompilerFingerprint::current(),
        WorkspaceContextFingerprint::single_file(
            &SourcePath::new("main.sifr"),
            FrontendMode::SingleFile,
        ),
        PackageContextFingerprint::from_identity(&WorkspacePackageConfigIdentity {
            workspace_root: None,
            entrypoint: None,
        }),
    )
}

#[test]
fn no_profile_queries_preserve_cancellation_without_host_residency() {
    let mut runtime = SqlEditorRuntime::new(PreparedSqlProfiles::default()).expect("empty runtime");
    assert!(runtime.host.is_none());
    let flag = Arc::new(AtomicBool::new(true));
    runtime.set_cancellation(Some(Arc::clone(&flag)));
    assert!(matches!(
        runtime.enrich(Vec::new(), &context(), "main.sifr"),
        Err(SqlEditorRuntimeError::Operation(
            EmbeddedProviderOperationError::Cancelled(_)
        ))
    ));
    runtime
        .replace_profiles(PreparedSqlProfiles::default())
        .expect("empty reload");
    assert!(runtime.host.is_none());
    assert!(Arc::ptr_eq(
        runtime
            .cancellation
            .as_ref()
            .expect("retained cancellation"),
        &flag
    ));
    flag.store(false, Ordering::Release);
    assert!(
        runtime
            .enrich(Vec::new(), &context(), "main.sifr")
            .expect("uncancelled")
            .is_empty()
    );
    assert!(runtime.host.is_none());
}

#[test]
fn profile_failures_remain_diagnostic_and_reload_clears_them_without_host() {
    let error = ComponentError::new(
        sifr_compiler_component::ComponentErrorKind::Execution,
        "configuration unavailable",
    );
    let diagnostic = sql_editor_initialization_diagnostic(&error);
    let profiles = PreparedSqlProfiles::from_initialization_failure(vec![diagnostic.clone()]);
    let mut runtime = SqlEditorRuntime::new(profiles.clone()).expect("failed profiles retained");
    assert!(runtime.host.is_none());
    assert_eq!(
        runtime.diagnostics_for_source("first.sifr"),
        vec![diagnostic.clone()]
    );
    runtime
        .replace_profiles(PreparedSqlProfiles::default())
        .expect("disable");
    assert!(runtime.diagnostics_for_source("first.sifr").is_empty());
    runtime
        .replace_profiles(profiles)
        .expect("later configuration failure");
    assert_eq!(
        runtime.diagnostics_for_source("second.sifr"),
        vec![diagnostic]
    );
    assert!(runtime.host.is_none());
}
