use crate::{
    create_invocation_workspace, discover_test_root_modules, parse_import_closure_modules,
    DiscoveryDiagnosticStyle, ModuleResolver, SifrWorkspaceConfig, WorkspaceRoot,
};
use sifr_diagnostics::{render_compact_diagnostics, DiagnosticArg, DiagnosticCode};
use sifr_frontend::DiskSourceProvider;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[test]
fn test_create_invocation_workspace_returns_unique_paths() {
    let first =
        create_invocation_workspace("workspace_unique").expect("first workspace should be created");
    let second = create_invocation_workspace("workspace_unique")
        .expect("second workspace should be created");
    assert_ne!(first, second);
    assert!(first.exists());
    assert!(second.exists());

    let _ = std::fs::remove_dir_all(first);
    let _ = std::fs::remove_dir_all(second);
}

#[test]
fn test_discover_test_root_modules_is_deterministic() {
    let unique = format!(
        "sifr_test_root_discovery_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("test dir should be created");
    std::fs::write(dir.join("z_test.sifr"), "def test_z():\n    assert True\n")
        .expect("z_test should be written");
    std::fs::write(dir.join("test_a.sifr"), "def test_a():\n    assert True\n")
        .expect("test_a should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "def helper() -> int:\n    return 1\n",
    )
    .expect("helper should be written");

    let roots = discover_test_root_modules(&dir, &mut DiskSourceProvider::new());
    let names: Vec<String> = roots.keys().cloned().collect();
    assert_eq!(names, vec!["test_a".to_string(), "z_test".to_string()]);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_project_and_test_discovery_share_import_closure_membership() {
    let unique = format!(
        "sifr_discovery_parity_positive_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("test_parity.sifr"),
        "from helper import value\n\ndef test_value():\n    assert value() == 42\n",
    )
    .expect("test_parity should be written");
    std::fs::write(
        dir.join("helper.sifr"),
        "from shared import BASE\n\ndef value() -> int:\n    return BASE\n",
    )
    .expect("helper should be written");
    std::fs::write(dir.join("shared.sifr"), "BASE: int = 42\n").expect("shared should be written");
    std::fs::write(dir.join("unrelated_bad.sifr"), "def unrelated(:\n")
        .expect("unrelated sibling should be written");

    let project_roots = BTreeSet::from(["main".to_string()]);
    let test_roots = BTreeSet::from(["test_parity".to_string()]);
    let resolver = ModuleResolver::entry_parent(&dir);
    let project_modules = parse_import_closure_modules(
        &resolver,
        &project_roots,
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .expect("project closure discovery should succeed");
    let test_modules = parse_import_closure_modules(
        &resolver,
        &test_roots,
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .expect("test closure discovery should succeed");

    let project_support: BTreeSet<String> = project_modules
        .keys()
        .filter(|name| !project_roots.contains(*name))
        .cloned()
        .collect();
    let test_support: BTreeSet<String> = test_modules
        .keys()
        .filter(|name| !test_roots.contains(*name))
        .cloned()
        .collect();

    assert_eq!(
        project_support,
        BTreeSet::from(["helper".to_string(), "shared".to_string()])
    );
    assert_eq!(project_support, test_support);
    assert!(!project_support.contains("unrelated_bad"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_project_and_test_discovery_parity_reports_reachable_parse_errors() {
    let unique = format!(
        "sifr_discovery_parity_negative_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    std::fs::create_dir_all(&dir).expect("project dir should be created");
    std::fs::write(
        dir.join("main.sifr"),
        "from helper import value\n\ndef main():\n    print(value())\n",
    )
    .expect("main should be written");
    std::fs::write(
        dir.join("test_parity.sifr"),
        "from helper import value\n\ndef test_value():\n    assert value() == 1\n",
    )
    .expect("test_parity should be written");
    std::fs::write(dir.join("helper.sifr"), "def value(:\n").expect("helper should be written");
    std::fs::write(
        dir.join("unrelated_ok.sifr"),
        "def spare() -> int:\n    return 1\n",
    )
    .expect("unrelated should be written");

    let project_roots = BTreeSet::from(["main".to_string()]);
    let test_roots = BTreeSet::from(["test_parity".to_string()]);
    let resolver = ModuleResolver::entry_parent(&dir);

    let project_errors = parse_import_closure_modules(
        &resolver,
        &project_roots,
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .err()
    .expect("project closure should fail on reachable parse error");
    let test_errors = parse_import_closure_modules(
        &resolver,
        &test_roots,
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .err()
    .expect("test closure should fail on reachable parse error");

    assert!(project_errors.iter().any(|e| e
        .children
        .iter()
        .any(|child| child.message == "while parsing helper")));
    assert!(test_errors.iter().any(|e| e
        .children
        .iter()
        .any(|child| child.message == "while parsing helper")));

    let _ = std::fs::remove_dir_all(&dir);
}

fn workspace_resolver(
    entry_parent: &std::path::Path,
    workspace_root: &std::path::Path,
    source_root: &str,
) -> ModuleResolver {
    ModuleResolver::with_workspace(
        entry_parent,
        WorkspaceRoot {
            dir: workspace_root.to_path_buf(),
            config: SifrWorkspaceConfig {
                source_root: PathBuf::from(source_root),
                package_name: None,
            },
        },
    )
}

#[test]
fn test_workspace_resolver_prefers_entry_parent_over_workspace_sources() {
    let unique = format!(
        "sifr_workspace_sibling_wins_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    let entry_dir = dir.join("cases");
    let source_dir = dir.join("lib");
    std::fs::create_dir_all(&entry_dir).expect("entry dir should be created");
    std::fs::create_dir_all(&source_dir).expect("source dir should be created");
    std::fs::write(entry_dir.join("main.sifr"), "from helper import value\n")
        .expect("main should be written");
    std::fs::write(entry_dir.join("helper.sifr"), "ENTRY: int = 1\n")
        .expect("entry helper should be written");
    std::fs::write(source_dir.join("helper.sifr"), "WORKSPACE: int = 2\n")
        .expect("workspace helper should be written");

    let resolver = workspace_resolver(&entry_dir, &dir, "lib");
    let resolved = resolver
        .resolve("helper", &mut DiskSourceProvider::new())
        .expect("entry helper should resolve first");

    assert_eq!(resolved.path, entry_dir.join("helper.sifr"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_workspace_resolver_finds_declared_source_root_and_dotted_paths() {
    let unique = format!(
        "sifr_workspace_dotted_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    let entry_dir = dir.join("cases");
    let helper_dir = dir.join("lib/helpers");
    std::fs::create_dir_all(&entry_dir).expect("entry dir should be created");
    std::fs::create_dir_all(&helper_dir).expect("helper dir should be created");
    std::fs::write(
        entry_dir.join("main.sifr"),
        "from helpers.nodes import LinkedNode\n",
    )
    .expect("main should be written");
    std::fs::write(
        helper_dir.join("nodes.sifr"),
        "class LinkedNode:\n    pass\n",
    )
    .expect("helper should be written");

    let resolver = workspace_resolver(&entry_dir, &dir, "lib");
    let resolved = resolver
        .resolve("helpers.nodes", &mut DiskSourceProvider::new())
        .expect("dotted helper should resolve");

    assert_eq!(resolved.path, helper_dir.join("nodes.sifr"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_workspace_resolver_reports_unresolved_tried_paths() {
    let unique = format!(
        "sifr_workspace_unresolved_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    let entry_dir = dir.join("cases");
    std::fs::create_dir_all(&entry_dir).expect("entry dir should be created");
    std::fs::create_dir_all(dir.join("lib")).expect("lib should be created");
    std::fs::write(entry_dir.join("main.sifr"), "from missing import value\n")
        .expect("main should be written");

    let resolver = workspace_resolver(&entry_dir, &dir, "lib");
    let errors = parse_import_closure_modules(
        &resolver,
        &BTreeSet::from(["main".to_string()]),
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .expect_err("missing workspace helper should fail");

    assert_eq!(
        errors[0].code,
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE.code()
    );
    assert!(errors[0]
        .message
        .contains("unknown import target: 'missing'"));
    assert!(errors[0].spans.iter().any(|span| span.is_primary));
    assert!(errors[0]
        .children
        .iter()
        .any(|child| child.message.contains("cases/missing.sifr")));
    assert!(errors[0]
        .children
        .iter()
        .any(|child| child.message.contains("lib/missing.sifr")));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_workspace_resolver_renders_missing_root_with_unknown_location() {
    let unique = format!(
        "sifr_workspace_missing_root_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    let entry_dir = dir.join("cases");
    std::fs::create_dir_all(&entry_dir).expect("entry dir should be created");
    std::fs::create_dir_all(dir.join("lib")).expect("lib should be created");

    let resolver = workspace_resolver(&entry_dir, &dir, "lib");
    let errors = parse_import_closure_modules(
        &resolver,
        &BTreeSet::from(["missing".to_string()]),
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .expect_err("missing root module should fail");

    let diagnostic = &errors[0];
    assert_eq!(
        diagnostic.code,
        DiagnosticCode::IMPORT_UNKNOWN_SOURCE_MODULE.code()
    );
    assert_eq!(diagnostic.message, "unknown import target: 'missing'");
    assert_eq!(
        diagnostic.args.get("resolution_scope"),
        Some(&DiagnosticArg::String("workspace".to_string()))
    );
    assert!(diagnostic.args.contains_key("tried_paths"));
    assert_eq!(diagnostic.children.len(), 2);
    assert!(diagnostic.spans.is_empty());
    assert!(render_compact_diagnostics(&errors)
        .contains("E SIFR-IMPORT-0002 <unknown> unknown import target: 'missing'"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_workspace_resolver_reclassifies_unresolved_bare_stdlib_import() {
    let unique = format!(
        "sifr_workspace_bare_stdlib_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    let entry_dir = dir.join("cases");
    std::fs::create_dir_all(&entry_dir).expect("entry dir should be created");
    std::fs::create_dir_all(dir.join("lib")).expect("lib should be created");
    std::fs::write(entry_dir.join("main.sifr"), "from math import sqrt\n")
        .expect("main should be written");

    let resolver = workspace_resolver(&entry_dir, &dir, "lib");
    let errors = parse_import_closure_modules(
        &resolver,
        &BTreeSet::from(["main".to_string()]),
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .expect_err("bare stdlib import should fail");

    assert_eq!(errors[0].code, DiagnosticCode::IMPORT_BARE_STDLIB.code());
    assert_eq!(
        errors[0].message,
        "bare stdlib import 'math'; Sifr stdlib lives under 'sifr.*'"
    );
    assert_eq!(errors[0].args["bare_module"], "math".into());
    assert_eq!(errors[0].args["suggested_module"], "sifr.math".into());
    assert_eq!(errors[0].args["imported_names"], "sqrt".into());
    assert_eq!(
        errors[0].help.as_deref(),
        Some("use 'from sifr.math import sqrt'")
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_workspace_resolver_prefers_real_user_module_over_bare_stdlib_match() {
    let unique = format!(
        "sifr_workspace_bare_stdlib_user_wins_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    let entry_dir = dir.join("cases");
    std::fs::create_dir_all(&entry_dir).expect("entry dir should be created");
    std::fs::create_dir_all(dir.join("lib")).expect("lib should be created");
    std::fs::write(entry_dir.join("main.sifr"), "from math import sqrt\n")
        .expect("main should be written");
    std::fs::write(
        dir.join("lib/math.sifr"),
        "def sqrt(value: int) -> int:\n    return value\n",
    )
    .expect("math module should be written");

    let resolver = workspace_resolver(&entry_dir, &dir, "lib");
    let parsed = parse_import_closure_modules(
        &resolver,
        &BTreeSet::from(["main".to_string()]),
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .expect("real user module should win over bare stdlib diagnostic");

    assert!(parsed.contains_key("math"));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_workspace_resolver_keeps_stdlib_imports_out_of_filesystem_resolution() {
    let unique = format!(
        "sifr_workspace_stdlib_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    let entry_dir = dir.join("cases");
    std::fs::create_dir_all(&entry_dir).expect("entry dir should be created");
    std::fs::create_dir_all(dir.join("lib")).expect("lib should be created");
    std::fs::write(
        entry_dir.join("main.sifr"),
        "from sifr.statistics import mean\nfrom _sifr.core import panic\nfrom typing import List\n",
    )
    .expect("main should be written");

    let resolver = workspace_resolver(&entry_dir, &dir, "lib");
    let parsed = parse_import_closure_modules(
        &resolver,
        &BTreeSet::from(["main".to_string()]),
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .expect("stdlib imports should not require workspace files");

    assert_eq!(parsed.len(), 1);
    assert!(parsed.contains_key("main"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_workspace_resolver_rejects_namespace_file_collision() {
    let unique = format!(
        "sifr_workspace_namespace_collision_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time should move forward")
            .as_nanos()
    );
    let dir = std::env::temp_dir().join(unique);
    let entry_dir = dir.join("cases");
    std::fs::create_dir_all(&entry_dir).expect("entry dir should be created");
    std::fs::create_dir_all(dir.join("lib/helpers")).expect("helper dir should be created");
    std::fs::write(
        entry_dir.join("main.sifr"),
        "from helpers.nodes import LinkedNode\n",
    )
    .expect("main should be written");
    std::fs::write(dir.join("lib/helpers.sifr"), "VALUE: int = 1\n")
        .expect("parent helper should be written");
    std::fs::write(
        dir.join("lib/helpers/nodes.sifr"),
        "class LinkedNode:\n    pass\n",
    )
    .expect("dotted helper should be written");

    let resolver = workspace_resolver(&entry_dir, &dir, "lib");
    let errors = parse_import_closure_modules(
        &resolver,
        &BTreeSet::from(["main".to_string()]),
        DiscoveryDiagnosticStyle::ModuleName,
        &mut DiskSourceProvider::new(),
    )
    .expect_err("namespace collision should fail");

    assert_eq!(
        errors[0].code,
        DiagnosticCode::IMPORT_NAMESPACE_COLLISION.code()
    );
    assert!(errors[0]
        .message
        .contains("import target 'helpers.nodes' collides with a namespace package"));
    assert!(errors[0].spans.iter().any(|span| span.is_primary));
    assert!(errors[0]
        .children
        .iter()
        .any(|child| child.message.contains("helpers.sifr")));

    let _ = std::fs::remove_dir_all(&dir);
}
