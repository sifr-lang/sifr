use crate::session::Session;
use serde_json::json;

pub(super) const SOURCE: &str = "\
from sifr.python import PythonError

@python(math.sqrt)
def sqrt(value: float) -> Result[float, PythonError]: ...

def main() -> Result[float, PythonError]:
    return sqrt(9.0)
";

#[test]
fn python_declaration_completion_hover_and_navigation_share_compiler_status() {
    let (mut session, _temp, uri) = open_fixture(SOURCE);

    assert_eq!(session.python_declaration_cache_stats().misses, 0);

    let completion = request(&mut session, "textDocument/completion", &uri, 6, 15);
    assert_eq!(
        session.python_declaration_cache_stats(),
        crate::python_declarations::PythonDeclarationCacheStats {
            hits: 0,
            misses: 1,
            external_fingerprint_runs: 1,
            snapshot_builds: 1,
        }
    );
    let item = completion
        .get("items")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("label").and_then(serde_json::Value::as_str) == Some("sqrt"))
        })
        .expect("sqrt completion");
    assert_eq!(
        item.pointer("/data/pythonStatus")
            .and_then(serde_json::Value::as_str),
        Some("verified")
    );
    assert!(
        item.get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("math.sqrt") && detail.contains("verified"))
    );

    let hover = request(&mut session, "textDocument/hover", &uri, 6, 13);
    assert_eq!(session.python_declaration_cache_stats().hits, 1);
    assert_eq!(
        session
            .python_declaration_cache_stats()
            .external_fingerprint_runs,
        2
    );
    assert_eq!(session.python_declaration_cache_stats().snapshot_builds, 1);
    let hover = hover
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("hover markdown");
    assert!(hover.contains("Python target: `math.sqrt`"));
    assert!(hover.contains("Status: **verified**"));
    assert!(hover.contains("closed typed Python conversion grammar"));

    let definition = request(&mut session, "textDocument/definition", &uri, 6, 13);
    let target = definition
        .as_array()
        .and_then(|locations| locations.first())
        .expect("definition location");
    assert_eq!(
        target.get("uri").and_then(serde_json::Value::as_str),
        Some(uri.as_str())
    );
    assert_eq!(
        target
            .pointer("/range/start/line")
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );

    assert_eq!(session.python_declarations.probe_runs(), 1);
    assert_eq!(session.python_declarations.environment_probe_runs(), 1);
    let _ = request(&mut session, "textDocument/completion", &uri, 6, 15);
    assert_eq!(session.python_declarations.probe_runs(), 1);
    assert_eq!(session.python_declarations.environment_probe_runs(), 1);
    assert_eq!(session.python_declaration_cache_stats().hits, 2);
    assert_eq!(
        session
            .python_declaration_cache_stats()
            .external_fingerprint_runs,
        3
    );
    assert_eq!(session.python_declaration_cache_stats().snapshot_builds, 1);

    let non_target_edit = SOURCE.replace("9.0", "16.0");
    session
        .change_compacted(&uri, Some(2), &[json!({"text": non_target_edit})])
        .expect("change call argument");
    let _ = request(&mut session, "textDocument/completion", &uri, 6, 15);
    assert_eq!(session.python_declarations.probe_runs(), 1);
    assert_eq!(session.python_declarations.environment_probe_runs(), 1);
    assert_eq!(session.python_declaration_cache_stats().misses, 2);
    assert_eq!(
        session
            .python_declaration_cache_stats()
            .external_fingerprint_runs,
        4
    );
    assert_eq!(session.python_declaration_cache_stats().snapshot_builds, 2);
}

#[test]
fn python_status_is_attached_only_to_the_exact_declaring_symbol() {
    let source = "\
from sifr.python import PythonError

@python(math.sqrt)
def sqrt(value: float) -> Result[float, PythonError]: ...

def sqrt_table() -> int:
    return 1

def main() -> int:
    return sqrt_table()
";
    let (mut session, _temp, uri) = open_fixture(source);
    let completion = request(&mut session, "textDocument/completion", &uri, 9, 21);
    let item = completion
        .get("items")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                item.get("label").and_then(serde_json::Value::as_str) == Some("sqrt_table")
            })
        })
        .expect("sqrt_table completion");
    assert!(item.pointer("/data/pythonStatus").is_none());
    assert!(
        !item
            .get("detail")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|detail| detail.contains("math.sqrt"))
    );

    let hover = request(&mut session, "textDocument/hover", &uri, 9, 16);
    let markdown = hover
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("sqrt_table hover");
    assert!(!markdown.contains("Python target"));

    let shadowed = source.replace(
        "def main() -> int:\n    return sqrt_table()",
        "def main(sqrt: int) -> int:\n    return sqrt",
    );
    let (mut shadowed_session, _shadowed_temp, shadowed_uri) = open_fixture(&shadowed);
    let hover = request(
        &mut shadowed_session,
        "textDocument/hover",
        &shadowed_uri,
        9,
        12,
    );
    let markdown = hover
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("shadowing parameter hover");
    assert!(!markdown.contains("Python target"));
}

#[test]
fn same_named_symbol_in_another_module_is_not_certified() {
    let helper_source =
        "def sqrt() -> int:\n    return 1\n\ndef use_helper() -> int:\n    return sqrt()\n";
    let primary = SOURCE.replace(
        "from sifr.python import PythonError",
        "from sifr.python import PythonError\nfrom helper import use_helper",
    );
    let (mut session, temp, _uri) =
        open_fixture_with_extra(&primary, "main.sifr", Some(("helper.sifr", helper_source)));
    let helper_path = temp.path().join("helper.sifr");
    let helper_uri = url::Url::from_file_path(&helper_path)
        .expect("helper file uri")
        .to_string();
    session
        .open_document(
            helper_uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            helper_source.to_string(),
        )
        .expect("open helper document");
    let helper_file = session
        .with_document_analysis(&helper_uri, |_snapshot, _host, file, _source| Ok(file))
        .expect("helper file identity");

    let completion = request(&mut session, "textDocument/completion", &helper_uri, 4, 15);
    let helper_sqrt = completion
        .get("items")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                item.get("label").and_then(serde_json::Value::as_str) == Some("sqrt")
                    && item
                        .pointer("/data/sifrFile")
                        .and_then(serde_json::Value::as_u64)
                        == Some(u64::from(helper_file.as_u32()))
            })
        })
        .expect("helper sqrt completion");
    assert!(helper_sqrt.pointer("/data/pythonStatus").is_none());

    let hover = request(&mut session, "textDocument/hover", &helper_uri, 4, 12);
    let markdown = hover
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("helper sqrt hover");
    assert!(!markdown.contains("Python target"));
}

#[test]
fn imported_alias_does_not_inherit_an_unrelated_python_declaration_status() {
    let helper_source =
        "from ordinary import value as sqrt\n\ndef use_helper() -> int:\n    return sqrt()\n";
    let ordinary_source = "def value() -> int:\n    return 1\n";
    let primary = SOURCE.replace(
        "from sifr.python import PythonError",
        "from sifr.python import PythonError\nfrom helper import use_helper",
    );
    let (mut session, temp, _uri) = open_fixture_with_extras(
        &primary,
        "main.sifr",
        &[
            ("helper.sifr", helper_source),
            ("ordinary.sifr", ordinary_source),
        ],
    );
    let helper_path = temp.path().join("helper.sifr");
    let helper_uri = url::Url::from_file_path(&helper_path)
        .expect("helper file uri")
        .to_string();
    session
        .open_document(
            helper_uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            helper_source.to_string(),
        )
        .expect("open helper document");

    let hover = request(&mut session, "textDocument/hover", &helper_uri, 3, 12);
    let markdown = hover
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("aliased function hover");

    assert!(!markdown.contains("Python target"));
}

#[test]
fn imported_alias_uses_its_exact_python_declaration_identity() {
    let declaration_source = "from sifr.python import PythonError\n\n@python(math.sqrt)\ndef sqrt(value: float) -> Result[float, PythonError]: ...\n";
    let helper_source = "from sifr.python import PythonError\nfrom declarations import sqrt as root\n\ndef use_helper() -> Result[float, PythonError]:\n    return root(9.0)\n";
    let primary = "from sifr.python import PythonError\nfrom declarations import sqrt\nfrom helper import use_helper\n\ndef main() -> Result[float, PythonError]:\n    return use_helper()\n";
    let (mut session, temp, _uri) = open_fixture_with_extras(
        primary,
        "main.sifr",
        &[
            ("declarations.sifr", declaration_source),
            ("helper.sifr", helper_source),
        ],
    );
    let helper_path = temp.path().join("helper.sifr");
    let helper_uri = url::Url::from_file_path(&helper_path)
        .expect("helper file uri")
        .to_string();
    session
        .open_document(
            helper_uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            helper_source.to_string(),
        )
        .expect("open helper document");

    let hover = request(&mut session, "textDocument/hover", &helper_uri, 4, 13);
    let markdown = hover
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("aliased Python function hover");

    assert!(markdown.contains("Python target: `math.sqrt`"));
    assert!(markdown.contains("Status: **verified**"));
}

#[test]
fn declarations_sharing_a_target_report_the_same_verified_status() {
    let source = SOURCE.replace(
        "def sqrt(value: float) -> Result[float, PythonError]: ...",
        "def sqrt(value: float) -> Result[float, PythonError]: ...\n\n@python(math.sqrt)\ndef other_sqrt(value: float) -> Result[float, PythonError]: ...",
    );
    let (mut session, _temp, uri) = open_fixture(&source);
    let completion = request(&mut session, "textDocument/completion", &uri, 9, 15);
    let items = completion
        .get("items")
        .and_then(serde_json::Value::as_array)
        .expect("completion items");
    for label in ["sqrt", "other_sqrt"] {
        let item = items
            .iter()
            .find(|item| item.get("label").and_then(serde_json::Value::as_str) == Some(label))
            .expect("shared-target completion");
        assert_eq!(
            item.pointer("/data/pythonStatus")
                .and_then(serde_json::Value::as_str),
            Some("verified")
        );
    }
    assert_eq!(session.python_declarations.probe_runs(), 1);
}

#[test]
fn python_declaration_diagnostics_and_source_drift_invalidate_cached_status() {
    let (mut session, _temp, uri) = open_fixture(SOURCE);
    let diagnostics =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("initial diagnostics");
    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
    assert_eq!(session.python_declarations.probe_runs(), 1);

    let invalid = SOURCE.replace("math.sqrt", "math.pi");
    session
        .change_compacted(&uri, Some(2), &[json!({"text": invalid})])
        .expect("change target");
    let diagnostics =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("drift diagnostics");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYCALL-0001")
    }));
    assert_eq!(session.python_declarations.probe_runs(), 2);
    assert_eq!(session.python_declarations.environment_probe_runs(), 1);
}

#[test]
fn declaration_diagnostic_is_scoped_to_its_declaring_document() {
    let helper_source = "def helper() -> int:\n    return 1\n";
    let invalid = SOURCE
        .replace(
            "from sifr.python import PythonError",
            "from sifr.python import PythonError\nfrom helper import helper",
        )
        .replace("math.sqrt", "math.pi");
    let (mut session, temp, uri) =
        open_fixture_with_extra(&invalid, "main.sifr", Some(("helper.sifr", helper_source)));
    let helper_path = temp.path().join("helper.sifr");
    let helper_uri = url::Url::from_file_path(&helper_path)
        .expect("helper file uri")
        .to_string();
    session
        .open_document(
            helper_uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            helper_source.to_string(),
        )
        .expect("open helper document");

    let declaring =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("main diagnostics");
    assert!(declaring.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYCALL-0001")
            && diagnostic
                .pointer("/range/start/line")
                .and_then(serde_json::Value::as_u64)
                == Some(3)
    }));
    let helper = crate::diagnostics::document_diagnostics(&mut session, &helper_uri)
        .expect("helper diagnostics");
    assert!(helper.iter().all(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) != Some("SIFR-PYCALL-0001")
    }));
}

#[test]
fn watcher_drift_revalidates_authoring_artifacts() {
    let (mut session, temp, uri) = open_fixture(SOURCE);
    let _ =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("initial diagnostics");
    std::fs::write(
        temp.path().join("sifr.python-bindings.json"),
        "{\"schema_version\":1,\"environment_digest\":\"stale\",\"bindings\":[]}\n",
    )
    .expect("write drifted artifact");
    session.record_watcher_events(1);

    let diagnostics =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("artifact diagnostics");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYCONV-0001")
            && diagnostic
                .get("message")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|message| message.contains("binding artifact"))
    }));
    assert_eq!(session.python_declarations.probe_runs(), 1);
}

#[test]
fn certification_drift_is_checked_against_the_live_environment_digest() {
    let (mut session, temp, uri) = open_fixture(SOURCE);
    std::fs::write(
        temp.path().join("sifr.python-certifications.json"),
        "{\"schema_version\":3,\"environment_digest\":\"stale\",\"arrow\":[],\"dlpack\":[]}\n",
    )
    .expect("write stale certification artifact");
    session.record_watcher_events(1);

    let diagnostics =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("drift diagnostics");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYZC-0001")
            && diagnostic
                .get("message")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|message| message.contains("environment digest"))
    }));
}

#[test]
fn configured_environment_is_validated_without_python_declarations() {
    let source = "def main() -> int:\n    return 1\n";
    let (mut session, temp, uri) = open_fixture(source);
    std::fs::write(
        temp.path().join("sifr.python-certifications.json"),
        "{\"schema_version\":3,\"environment_digest\":\"stale\",\"arrow\":[],\"dlpack\":[]}\n",
    )
    .expect("write stale certification artifact");
    session.record_watcher_events(1);

    let diagnostics = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("environment diagnostics");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYZC-0001")
            && diagnostic
                .get("message")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|message| message.contains("environment digest"))
    }));
}

#[test]
fn lockfile_less_package_without_python_inputs_has_no_editor_diagnostic() {
    let source = "def main() -> int:\n    return 1\n";
    let (mut session, temp, uri) = open_fixture(source);
    std::fs::write(
        temp.path().join("sifr.toml"),
        "[package]\nname = \"lsp-pure\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    )
    .expect("write pure Sifr manifest");
    std::fs::remove_file(temp.path().join("Cargo.lock")).expect("remove fixture lockfile");
    session.record_watcher_events(1);

    let diagnostics = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("lockfile-less pure package diagnostics");

    assert!(
        diagnostics.is_empty(),
        "unexpected diagnostics: {diagnostics:?}"
    );
    assert!(!temp.path().join("Cargo.lock").exists());
    assert_eq!(session.python_declarations.environment_probe_runs(), 0);
}

#[test]
fn lockfile_less_nontrivial_packages_defer_without_mutation() {
    let pure_source = "def main() -> int:\n    return 1\n";

    let (mut dependency_session, dependency_temp, dependency_uri) = open_fixture(pure_source);
    std::fs::write(
        dependency_temp.path().join("sifr.toml"),
        "[package]\nname = \"lsp-dependency\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    )
    .expect("write dependency Sifr manifest");
    let cargo_manifest = std::fs::read_to_string(dependency_temp.path().join("Cargo.toml"))
        .expect("read dependency Cargo manifest")
        .replace(
            "\n[workspace]\n",
            "\n[dependencies]\nserde = \"1\"\n\n[workspace]\n",
        );
    std::fs::write(dependency_temp.path().join("Cargo.toml"), cargo_manifest)
        .expect("write dependency Cargo manifest");
    std::fs::remove_file(dependency_temp.path().join("Cargo.lock"))
        .expect("remove dependency fixture lockfile");
    dependency_session.record_watcher_events(1);
    let dependency_diagnostics =
        crate::diagnostics::document_diagnostics(&mut dependency_session, &dependency_uri)
            .expect("lockfile-less dependency diagnostics");
    assert!(dependency_diagnostics.is_empty());
    assert!(!dependency_temp.path().join("Cargo.lock").exists());

    let (mut configured_session, configured_temp, configured_uri) = open_fixture(pure_source);
    std::fs::remove_file(configured_temp.path().join("Cargo.lock"))
        .expect("remove configured fixture lockfile");
    configured_session.record_watcher_events(1);
    let configured_diagnostics =
        crate::diagnostics::document_diagnostics(&mut configured_session, &configured_uri)
            .expect("lockfile-less configured diagnostics");
    assert!(configured_diagnostics.is_empty());
    assert!(!configured_temp.path().join("Cargo.lock").exists());

    let (mut declaration_session, declaration_temp, declaration_uri) = open_fixture(SOURCE);
    std::fs::remove_file(declaration_temp.path().join("Cargo.lock"))
        .expect("remove declaration fixture lockfile");
    declaration_session.record_watcher_events(1);
    let declaration_diagnostics =
        crate::diagnostics::document_diagnostics(&mut declaration_session, &declaration_uri)
            .expect("lockfile-less declaration diagnostics");
    assert!(declaration_diagnostics.is_empty());
    let completion = request(
        &mut declaration_session,
        "textDocument/completion",
        &declaration_uri,
        6,
        15,
    );
    let sqrt = completion
        .get("items")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("label").and_then(serde_json::Value::as_str) == Some("sqrt"))
        })
        .expect("deferred sqrt completion");
    assert_eq!(
        sqrt.pointer("/data/pythonStatus")
            .and_then(serde_json::Value::as_str),
        Some("deferred")
    );
    assert!(!declaration_temp.path().join("Cargo.lock").exists());
}

#[test]
fn watched_live_bridge_sources_without_inventory_are_validated_and_fingerprinted() {
    let source = "def main() -> int:\n    return 1\n";
    let (mut session, temp, uri) = open_fixture(source);
    let initial =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("initial diagnostics");
    assert!(initial.is_empty());
    let bridge_root = temp.path().join("src/python_bridges");
    std::fs::create_dir_all(&bridge_root).expect("create bridge source root");
    std::fs::write(
        bridge_root.join("util.py"),
        "import numpy\n\ndef value():\n    return numpy.array([1])\n",
    )
    .expect("write live bridge source");
    session.record_watcher_events(1);

    let diagnostics = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("live bridge diagnostics after watcher invalidation");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYTRUST-0005")
    }));
    assert!(!bridge_root.join("__sifr_inventory__.json").exists());
}

#[test]
fn workspace_member_python_requirements_are_validated() {
    let source = "def main() -> int:\n    return 1\n";
    let (mut session, temp, uri) = open_fixture(source);
    std::fs::write(
        temp.path().join("sifr.toml"),
        "[package]\nname = \"lsp-pure-root\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    )
    .expect("write pure root Sifr manifest");
    let cargo_manifest = std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .expect("read root Cargo manifest")
        .replace("[workspace]\n", "[workspace]\nmembers = [\"member\"]\n");
    std::fs::write(temp.path().join("Cargo.toml"), cargo_manifest)
        .expect("write workspace Cargo manifest");
    let member = temp.path().join("member");
    std::fs::create_dir_all(member.join("src")).expect("create member source root");
    std::fs::write(
        member.join("Cargo.toml"),
        "[package]\nname = \"lsp-python-member\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n",
    )
    .expect("write member Cargo manifest");
    std::fs::write(member.join("src/lib.rs"), "").expect("write member pure marker");
    std::fs::write(
        member.join("sifr.toml"),
        "[package]\nname = \"lsp-python-member\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[python]\nrequires-imports = [\"numpy\"]\n",
    )
    .expect("write member Sifr manifest");
    let cargo_lock =
        std::fs::read_to_string(temp.path().join("Cargo.lock")).expect("read root Cargo lock");
    std::fs::write(
        temp.path().join("Cargo.lock"),
        format!("{cargo_lock}\n[[package]]\nname = \"lsp-python-member\"\nversion = \"0.0.0\"\n"),
    )
    .expect("write workspace Cargo lock");
    session.record_watcher_events(1);

    let diagnostics = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("workspace member Python diagnostics");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYTRUST-0005")
    }));
}

#[test]
fn misplaced_bridge_root_is_reported_for_an_otherwise_pure_package() {
    let source = "def main() -> int:\n    return 1\n";
    let (mut session, temp, uri) = open_fixture(source);
    std::fs::write(
        temp.path().join("sifr.toml"),
        "[package]\nname = \"lsp-pure-root\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    )
    .expect("write pure root Sifr manifest");
    let misplaced_root = temp.path().join("python_bridges");
    std::fs::create_dir_all(&misplaced_root).expect("create misplaced bridge root");
    std::fs::write(
        misplaced_root.join("util.py"),
        "def value():\n    return 1\n",
    )
    .expect("write misplaced bridge source");
    session.record_watcher_events(1);

    let diagnostics = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("misplaced bridge diagnostics");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYIMP-0002")
    }));
}

#[test]
fn stale_existing_lockfile_remains_a_package_error() {
    let source = "def main() -> int:\n    return 1\n";
    let (mut session, temp, uri) = open_fixture(source);
    let cargo_manifest = std::fs::read_to_string(temp.path().join("Cargo.toml"))
        .expect("read Cargo manifest")
        .replace(
            "\n[workspace]\n",
            "\n[dependencies]\nserde = \"1\"\n\n[workspace]\n",
        );
    std::fs::write(temp.path().join("Cargo.toml"), cargo_manifest)
        .expect("write stale Cargo manifest");
    session.record_watcher_events(1);

    let diagnostics = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("stale lockfile diagnostics");

    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PACKAGE-0101")
    }));
    assert!(temp.path().join("Cargo.lock").exists());
}

#[test]
fn mixed_shared_target_constraints_are_reported_without_wrong_kind_attribution() {
    let source = "from sifr.python import PythonError\n\n@python(math.sqrt)\ndef sqrt(value: float) -> Result[float, PythonError]: ...\n\n@python.opaque(type=math.sqrt, cleanup=drop)\nclass Root:\n    pass\n\ndef main() -> Result[float, PythonError]:\n    return sqrt(9.0)\n";
    let (mut session, _temp, uri) = open_fixture(source);

    let diagnostics = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("mixed shared-target diagnostics");
    let target_diagnostics = diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYCALL-0001")
        })
        .collect::<Vec<_>>();

    assert_eq!(target_diagnostics.len(), 2);
    assert!(target_diagnostics.iter().all(|diagnostic| {
        diagnostic
            .get("message")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|message| {
                message.contains("a declaration requires") && !message.contains("opaque")
            })
    }));
}

#[test]
fn app_trust_policy_is_enforced_by_the_editor_environment_path() {
    let (mut session, temp, uri) = open_fixture(SOURCE);
    let manifest = std::fs::read_to_string(temp.path().join("sifr.toml"))
        .expect("read fixture manifest")
        .replace("[\"builtins\", \"math\"]", "[\"builtins\"]");
    std::fs::write(temp.path().join("sifr.toml"), manifest).expect("remove math trust");
    session.record_watcher_events(1);

    let diagnostics =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("trust diagnostics");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYTRUST-0005")
    }));
    assert_eq!(session.python_declarations.probe_runs(), 0);
}

#[test]
fn library_without_root_environment_selection_defers_python_status() {
    let (mut session, temp, uri) = open_fixture_named(SOURCE, "library.sifr");
    std::fs::write(
        temp.path().join("sifr.toml"),
        "[package]\nname = \"lsp-python\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n",
    )
    .expect("write library manifest without environment selection");
    session.record_watcher_events(1);

    let diagnostics = crate::diagnostics::document_diagnostics(&mut session, &uri)
        .expect("deferred library diagnostics");
    assert!(
        diagnostics.is_empty(),
        "unexpected deferred-library diagnostics: {diagnostics:?}"
    );
    let completion = request(&mut session, "textDocument/completion", &uri, 6, 15);
    let item = completion
        .get("items")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items
                .iter()
                .find(|item| item.get("label").and_then(serde_json::Value::as_str) == Some("sqrt"))
        })
        .expect("deferred sqrt completion");
    assert_eq!(
        item.pointer("/data/pythonStatus")
            .and_then(serde_json::Value::as_str),
        Some("deferred")
    );
    assert_eq!(session.python_declarations.environment_probe_runs(), 0);
}

#[test]
fn uninspectable_target_is_runtime_checked_and_unsupported_types_are_never_verified() {
    let runtime_checked = "\
from sifr.python import PythonError

@python(builtins.dir)
def listing() -> Result[list[str], PythonError]: ...

def main() -> Result[list[str], PythonError]:
    return listing()
";
    let (mut session, _temp, uri) = open_fixture(runtime_checked);
    let completion = request(&mut session, "textDocument/completion", &uri, 6, 15);
    let item = completion
        .get("items")
        .and_then(serde_json::Value::as_array)
        .and_then(|items| {
            items.iter().find(|item| {
                item.get("label").and_then(serde_json::Value::as_str) == Some("listing")
            })
        })
        .expect("runtime-checked completion");
    assert_eq!(
        item.pointer("/data/pythonStatus")
            .and_then(serde_json::Value::as_str),
        Some("runtime-checked")
    );

    let unsupported = SOURCE.replace("value: float", "value: object");
    let (mut unsupported_session, _unsupported_temp, unsupported_uri) = open_fixture(&unsupported);
    let diagnostics =
        crate::diagnostics::document_diagnostics(&mut unsupported_session, &unsupported_uri)
            .expect("unsupported diagnostics");
    assert!(!diagnostics.is_empty());
    let completion = request(
        &mut unsupported_session,
        "textDocument/completion",
        &unsupported_uri,
        6,
        15,
    );
    let verified_unsupported = completion
        .get("items")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .any(|item| {
            item.get("label").and_then(serde_json::Value::as_str) == Some("sqrt")
                && item
                    .pointer("/data/pythonStatus")
                    .and_then(serde_json::Value::as_str)
                    == Some("verified")
        });
    assert!(!verified_unsupported);
}

pub(super) fn request(
    session: &mut Session,
    method: &str,
    uri: &str,
    line: u64,
    character: u64,
) -> serde_json::Value {
    crate::requests::handle(
        session,
        method,
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": line, "character": character }
        }),
    )
    .unwrap_or_else(|error| panic!("{method} failed: {error:?}"))
}

pub(super) fn open_fixture(source: &str) -> (Session, tempfile::TempDir, String) {
    open_fixture_named(source, "main.sifr")
}

fn open_fixture_named(source: &str, source_name: &str) -> (Session, tempfile::TempDir, String) {
    open_fixture_with_extra(source, source_name, None)
}

fn open_fixture_with_extra(
    source: &str,
    source_name: &str,
    extra_source: Option<(&str, &str)>,
) -> (Session, tempfile::TempDir, String) {
    let extra_sources = extra_source.into_iter().collect::<Vec<_>>();
    open_fixture_with_extras(source, source_name, &extra_sources)
}

fn open_fixture_with_extras(
    source: &str,
    source_name: &str,
    extra_sources: &[(&str, &str)],
) -> (Session, tempfile::TempDir, String) {
    let temp = tempfile::tempdir().expect("temp dir");
    let src = temp.path().join("src");
    std::fs::create_dir(&src).expect("create src");
    let repository = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let python_project = repository.join("verification/areas/python_interop");
    let local_venv = temp.path().join(".venv");
    link_directory(&python_project.join(".venv"), &local_venv);
    std::fs::write(
        temp.path().join("Cargo.toml"),
        "[package]\nname = \"lsp-python-fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[package.metadata.sifr]\nmanifest = \"sifr.toml\"\n\n[workspace]\n",
    )
    .expect("write Cargo manifest");
    std::fs::write(
        temp.path().join("Cargo.lock"),
        "# This file is automatically @generated by Cargo.\n# It is not intended for manual editing.\nversion = 4\n\n[[package]]\nname = \"lsp-python-fixture\"\nversion = \"0.0.0\"\n",
    )
    .expect("write Cargo lock");
    std::fs::write(src.join("lib.rs"), "").expect("write pure marker");
    std::fs::copy(
        python_project.join("pyproject.toml"),
        temp.path().join("pyproject.toml"),
    )
    .expect("copy Python project");
    std::fs::copy(python_project.join("uv.lock"), temp.path().join("uv.lock"))
        .expect("copy Python lock");
    std::fs::write(
        temp.path().join("sifr.toml"),
        "[package]\nname = \"lsp-python\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[python]\nvenv = \".venv\"\npyproject = \"pyproject.toml\"\nlock = \"uv.lock\"\n\n[trust]\npython = [\"builtins\", \"math\"]\n",
    )
    .expect("write manifest");
    let path = src.join(source_name);
    std::fs::write(&path, source).expect("write source");
    for (name, contents) in extra_sources {
        std::fs::write(temp.path().join(name), contents).expect("write extra source");
    }
    let uri = url::Url::from_file_path(&path)
        .expect("file uri")
        .to_string();
    let mut session = Session::new();
    session
        .open_document(
            uri.clone(),
            crate::capabilities::LANGUAGE_ID,
            Some(1),
            source.to_string(),
        )
        .expect("open document");
    (session, temp, uri)
}

fn link_directory(source: &std::path::Path, destination: &std::path::Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, destination).expect("link Python interpreter");
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(source, destination).expect("link Python environment");
}
