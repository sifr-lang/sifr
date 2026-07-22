use crate::request_queue::CancellationTarget;
use crate::session::Session;
use lsp_server::RequestId;
use serde_json::json;
use std::path::PathBuf;

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
    let _ = request(&mut session, "textDocument/completion", &uri, 6, 15);
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
    assert_eq!(session.python_declarations.probe_runs(), 2);
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
    let temp = tempfile::tempdir().expect("temp dir");
    let src = temp.path().join("src");
    std::fs::create_dir(&src).expect("create src");
    let interpreter = python_interpreter();
    let local_interpreter = temp.path().join(if cfg!(windows) {
        "python-bin.exe"
    } else {
        "python-bin"
    });
    link_interpreter(&interpreter, &local_interpreter);
    std::fs::write(
        temp.path().join("sifr.toml"),
        format!(
            "[package]\nname = \"lsp-python\"\nedition = \"2026\"\nsifr-version = \">=0.3,<0.4\"\n\n[source]\nroot = \"src\"\n\n[python]\ninterpreter = \"{}\"\n",
            local_interpreter
                .file_name()
                .and_then(|name| name.to_str())
                .expect("local interpreter name")
        ),
    )
    .expect("write manifest");
    let path = src.join("main.sifr");
    std::fs::write(&path, source).expect("write source");
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

fn link_interpreter(source: &std::path::Path, destination: &std::path::Path) {
    #[cfg(unix)]
    std::os::unix::fs::symlink(source, destination).expect("link Python interpreter");
    #[cfg(windows)]
    std::fs::hard_link(source, destination).expect("link Python interpreter");
}

fn python_interpreter() -> PathBuf {
    for candidate in ["python3", "python"] {
        let output = std::process::Command::new(candidate)
            .args(["-c", "import sys; print(sys.executable)"])
            .output();
        if let Ok(output) = output {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return PathBuf::from(path);
                }
            }
        }
    }
    panic!("Python interpreter is required for Python LSP tests");
}
