use crate::{
    DocumentVersion, FrontendContext, ModuleId, ProjectRoot, SourcePath, SourceText,
    WorkspaceDirtyReason, WorkspaceDirtyScope,
};

#[test]
fn frontend_export_policy_hides_private_math_bridge_aliases() {
    assert!(!crate::query_diagnostics::should_export_callable(
        "sifr.math",
        "_dist_impl"
    ));
    assert!(!crate::query_diagnostics::should_export_callable(
        "sifr.math",
        "_fsum_impl"
    ));
    assert!(!crate::query_diagnostics::should_export_callable(
        "sifr.math",
        "_sumprod_impl"
    ));
    assert!(crate::query_diagnostics::should_export_callable(
        "sifr.math",
        "dist"
    ));
    assert!(crate::query_diagnostics::should_export_callable(
        "sifr.heapq",
        "_heapify_max"
    ));
    assert!(!crate::query_diagnostics::should_export_callable(
        "sifr.math",
        "_copy_float_list"
    ));
}

#[test]
fn project_exports_preserve_imported_error_class_status() {
    let dir = temp_project_dir("imported_error_class_status");
    std::fs::write(
        dir.join("main.sifr"),
        "from api import ApiError\n\ndef fail() -> Result[int, ApiError]:\n    raise ApiError(\"failed\")\n\ndef main() -> Result[None, ApiError]:\n    try:\n        value: int = fail()\n        assert value == 1\n    except ApiError as error:\n        raise error\n    return None\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("errors.sifr"),
        "class PackageError(Error):\n    message: str\n",
    )
    .expect("error module should be written");
    std::fs::write(
        dir.join("api.sifr"),
        "from errors import PackageError as ApiError\n",
    )
    .expect("facade module should be written");
    let mut context = load_temp_project(&dir);

    let diagnostics = context.diagnostics_for_project().into_value().diagnostics;

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
}

#[test]
fn public_constant_value_update_invalidates_reverse_dependents() {
    let dir = temp_project_dir("constant_export_signature_invalidation");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import LIMIT\n\ndef main() -> int:\n    return LIMIT\n",
    )
    .expect("main should be written");
    std::fs::write(dir.join("helper.sifr"), "LIMIT: int = 1\n").expect("helper should be written");
    let mut context = load_temp_project(&dir);
    let helper = ModuleId(1);
    let main = ModuleId(0);
    let _ = context.diagnostics_for_project();

    let report = context
        .update_module_source(
            helper,
            SourceText::new("LIMIT: int = 2\n"),
            Some(DocumentVersion::new(2)),
        )
        .expect("helper update should succeed");

    assert_eq!(report.invalidated_modules, vec![main, helper]);
    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::ReverseDependencies {
            path: SourcePath::new(dir.join("helper.sifr"))
        }
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::ExportSignatureChanged
        ]
    );
}

#[test]
fn dunder_method_signature_update_invalidates_reverse_dependents() {
    let dir = temp_project_dir("dunder_export_signature_invalidation");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import Box\n\ndef main():\n    value = Box(1)\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "class Box:\n    def __init__(self, value: int):\n        self.value = value\n",
    )
    .expect("helper should be written");
    let mut context = load_temp_project(&dir);
    let helper = ModuleId(1);
    let main = ModuleId(0);

    let report = context
        .update_module_source(
            helper,
            SourceText::new(
                "class Box:\n    def __init__(self, value: str):\n        self.value = value\n",
            ),
            Some(DocumentVersion::new(2)),
        )
        .expect("helper update should succeed");

    assert_eq!(report.invalidated_modules, vec![main, helper]);
    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::ReverseDependencies {
            path: SourcePath::new(dir.join("helper.sifr"))
        }
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::ExportSignatureChanged
        ]
    );
}

#[test]
fn single_underscore_method_signature_update_invalidates_reverse_dependents() {
    let dir = temp_project_dir("underscore_method_export_signature_invalidation");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import Box\n\ndef main() -> int:\n    value = Box()\n    return value._helper()\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "class Box:\n    def _helper(self) -> int:\n        return 1\n",
    )
    .expect("helper should be written");
    let mut context = load_temp_project(&dir);
    let helper = ModuleId(1);
    let main = ModuleId(0);

    let report = context
        .update_module_source(
            helper,
            SourceText::new(
                "class Box:\n    def _helper(self) -> str:\n        return \"changed\"\n",
            ),
            Some(DocumentVersion::new(2)),
        )
        .expect("helper update should succeed");

    assert_eq!(report.invalidated_modules, vec![main, helper]);
    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::ReverseDependencies {
            path: SourcePath::new(dir.join("helper.sifr"))
        }
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::ExportSignatureChanged
        ]
    );
}

#[test]
fn class_decorator_update_invalidates_reverse_dependents() {
    let dir = temp_project_dir("class_decorator_export_signature_invalidation");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import Box\n\ndef main():\n    value = Box()\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "@old_decorator\nclass Box:\n    pass\n",
    )
    .expect("helper should be written");
    let mut context = load_temp_project(&dir);
    let helper = ModuleId(1);
    let main = ModuleId(0);

    let report = context
        .update_module_source(
            helper,
            SourceText::new("@new_decorator\nclass Box:\n    pass\n"),
            Some(DocumentVersion::new(2)),
        )
        .expect("helper update should succeed");

    assert_eq!(report.invalidated_modules, vec![main, helper]);
    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::ReverseDependencies {
            path: SourcePath::new(dir.join("helper.sifr"))
        }
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![
            WorkspaceDirtyReason::SourceTextChanged,
            WorkspaceDirtyReason::ExportSignatureChanged
        ]
    );
}

#[test]
fn leading_whitespace_edit_preserves_export_signature_scope() {
    let dir = temp_project_dir("leading_whitespace_export_signature");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main() -> int:\n    return value()\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def value() -> int:\n    return 1\n",
    )
    .expect("helper should be written");
    let mut context = load_temp_project(&dir);
    let helper = ModuleId(1);
    let main = ModuleId(0);

    let report = context
        .update_module_source(
            helper,
            SourceText::new("\ndef value() -> int:\n    return 1\n"),
            Some(DocumentVersion::new(2)),
        )
        .expect("helper update should succeed");

    assert_eq!(report.invalidated_modules, vec![helper]);
    assert!(!report.invalidated_modules.contains(&main));
    assert_eq!(
        report.dirty_scope_report.scope,
        WorkspaceDirtyScope::OneModule {
            path: SourcePath::new(dir.join("helper.sifr"))
        }
    );
    assert_eq!(
        report.dirty_scope_report.reasons,
        vec![WorkspaceDirtyReason::SourceTextChanged]
    );
}

fn temp_project_dir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "sifr_frontend_{name}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("temp project should be created");
    dir
}

fn load_temp_project(dir: &std::path::Path) -> FrontendContext {
    FrontendContext::load_project(&ProjectRoot {
        root: SourcePath::new(dir),
        entrypoint: SourcePath::new(dir.join("main.sifr")),
    })
    .expect("project should load")
}
