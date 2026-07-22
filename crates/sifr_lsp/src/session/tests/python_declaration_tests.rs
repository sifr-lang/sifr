use crate::request_queue::CancellationTarget;
use crate::session::Session;
use lsp_server::RequestId;
use serde_json::json;

const SOURCE: &str = "\
from sifr.python import PythonError

@python(math.sqrt)
def sqrt(value: float) -> Result[float, PythonError]: ...

def main() -> Result[float, PythonError]:
    return sqrt(9.0)
";

#[test]
fn python_declaration_completion_hover_and_navigation_share_compiler_status() {
    let (mut session, _temp, uri) = open_fixture(SOURCE);

    let completion = request(&mut session, "textDocument/completion", &uri, 6, 15);
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
    assert!(item
        .get("detail")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|detail| detail.contains("math.sqrt") && detail.contains("verified")));

    let hover = request(&mut session, "textDocument/hover", &uri, 6, 13);
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

    let non_target_edit = SOURCE.replace("9.0", "16.0");
    session
        .change_compacted(&uri, Some(2), &[json!({"text": non_target_edit})])
        .expect("change call argument");
    let _ = request(&mut session, "textDocument/completion", &uri, 6, 15);
    assert_eq!(session.python_declarations.probe_runs(), 1);
    assert_eq!(session.python_declarations.environment_probe_runs(), 1);
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
    assert!(!item
        .get("detail")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|detail| detail.contains("math.sqrt")));

    let hover = request(&mut session, "textDocument/hover", &uri, 9, 16);
    let markdown = hover
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("sqrt_table hover");
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
    std::fs::write(temp.path().join("sifr.python-bindings.json"), "{}\n")
        .expect("write drifted artifact");
    session.record_watcher_events(1);

    let diagnostics =
        crate::diagnostics::document_diagnostics(&mut session, &uri).expect("artifact diagnostics");
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) == Some("SIFR-PYENV-0011")
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
    assert!(diagnostics.iter().all(|diagnostic| {
        diagnostic.get("code").and_then(serde_json::Value::as_str) != Some("SIFR-PYENV-0001")
    }));
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

#[test]
fn cancelled_python_declaration_request_stops_before_probe() {
    let (mut session, _temp, uri) = open_fixture(SOURCE);
    let id = RequestId::from(150);
    session
        .enqueue_request(
            &id,
            "textDocument/completion",
            crate::scheduler::WorkLane::LatencySensitive,
        )
        .expect("enqueue request");
    let scheduled = session.start_next_request().expect("start request");
    session
        .begin_request_execution(scheduled.id())
        .expect("begin request");
    assert_eq!(session.cancel_request(&id), CancellationTarget::InFlight);

    let error = session
        .python_declaration_snapshot(&uri)
        .expect_err("cancelled request must stop");
    assert!(error.message().contains("cancelled"));
    assert_eq!(session.python_declarations.probe_runs(), 0);
    session.finish_request(&id);
}

fn request(
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

fn open_fixture(source: &str) -> (Session, tempfile::TempDir, String) {
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
    if let Some((name, contents)) = extra_source {
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
