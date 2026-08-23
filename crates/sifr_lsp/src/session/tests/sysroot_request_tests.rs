use crate::session::Session;
use serde_json::json;
use sifr_analysis::{ToolingSysrootDiagnostic, ToolingSysrootProbe, ToolingSysrootStatus};
use std::path::PathBuf;
use std::process::Command;

#[test]
fn definition_request_for_stdlib_import_returns_sysroot_uri() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/definition",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 24 }
        }),
    )
    .expect("definition should answer");
    let locations = response
        .as_array()
        .expect("definition response should be an array");

    assert_eq!(locations.len(), 1);
    let target_uri = locations[0]
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .expect("definition location should include uri");
    assert!(
        target_uri.ends_with("/stdlib/sifr/random.sifr"),
        "unexpected stdlib definition uri: {target_uri}"
    );
}

#[test]
fn definition_request_for_stdlib_call_returns_sysroot_uri() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/definition",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 4, "character": 17 }
        }),
    )
    .expect("definition should answer");
    let locations = response
        .as_array()
        .expect("definition response should be an array");

    assert_eq!(locations.len(), 1);
    let target_uri = locations[0]
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .expect("definition location should include uri");
    assert!(
        target_uri.ends_with("/stdlib/sifr/random.sifr"),
        "unexpected stdlib call definition uri: {target_uri}"
    );
}

#[test]
fn type_definition_request_for_stdlib_import_returns_sysroot_uri() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/typeDefinition",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 0, "character": 24 }
        }),
    )
    .expect("type definition should answer");
    let locations = response
        .as_array()
        .expect("type definition response should be an array");

    assert_eq!(locations.len(), 1);
    let target_uri = locations[0]
        .get("uri")
        .and_then(serde_json::Value::as_str)
        .expect("type definition location should include uri");
    assert!(
        target_uri.ends_with("/stdlib/sifr/random.sifr"),
        "unexpected stdlib type definition uri: {target_uri}"
    );
}

#[test]
fn hover_request_for_stdlib_call_reflects_installed_signature() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/hover",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 4, "character": 17 }
        }),
    )
    .expect("hover should answer");
    let contents = response
        .pointer("/contents/value")
        .and_then(serde_json::Value::as_str)
        .expect("hover response should contain markdown contents");

    assert!(contents.contains("randint"));
    assert!(contents.contains("minimum: int"));
    assert!(contents.contains("maximum: int"));
    assert!(contents.contains("Result[int, ValueError]"));
}

#[test]
fn completion_request_includes_public_stdlib_symbols_not_private_modules() {
    let (mut session, _temp, uri) = open_stdlib_import_fixture();

    let response = crate::requests::handle(
        &mut session,
        "textDocument/completion",
        json!({
            "textDocument": { "uri": uri },
            "position": { "line": 4, "character": 17 }
        }),
    )
    .expect("completion should answer");
    let items = response
        .get("items")
        .and_then(serde_json::Value::as_array)
        .expect("completion response should include items");
    let labels = items
        .iter()
        .filter_map(|item| item.get("label").and_then(serde_json::Value::as_str))
        .collect::<Vec<_>>();

    assert!(labels.contains(&"randint"));
    assert!(!labels.iter().any(|label| label.starts_with("_sifr")));
}

#[test]
fn development_sysroot_request_reports_same_root_as_cli() {
    let mut session = Session::new();
    let expected = cli_sysroot_status();

    let response = crate::requests::handle(
        &mut session,
        "sifr/sysroot",
        json!({
            "expectedRoot": expected.root.clone(),
            "expectedToolchainId": expected.toolchain_id.clone(),
        }),
    )
    .expect("sysroot request should answer");

    assert_eq!(
        response.get("ok").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        response.get("root").and_then(serde_json::Value::as_str),
        Some(expected.root.as_str())
    );
    assert_eq!(
        response
            .get("toolchainId")
            .and_then(serde_json::Value::as_str),
        Some(expected.toolchain_id.as_str())
    );
    assert_eq!(
        response
            .pointer("/observedPaths/sysroot")
            .and_then(serde_json::Value::as_str),
        Some(expected.root.as_str())
    );
}

#[test]
fn sysroot_request_handler_reports_expected_root_mismatch() {
    let mut session = Session::new();
    let response = crate::requests::handle(
        &mut session,
        "sifr/sysroot",
        json!({
            "expectedRoot": "/tmp/not-the-lsp-sysroot",
            "expectedToolchainId": "not-the-lsp-toolchain",
        }),
    )
    .expect("sysroot request should answer");

    assert_eq!(
        response.get("ok").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        response.get("kind").and_then(serde_json::Value::as_str),
        Some("mismatch")
    );
    assert_eq!(
        response
            .get("expectedRoot")
            .and_then(serde_json::Value::as_str),
        Some("/tmp/not-the-lsp-sysroot")
    );
    assert!(
        response
            .pointer("/observedPaths/sysroot")
            .and_then(serde_json::Value::as_str)
            .is_some()
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/expectedToolchainId")
            .and_then(serde_json::Value::as_str),
        Some("not-the-lsp-toolchain")
    );
}

#[test]
fn sysroot_request_reports_expected_root_mismatch_with_observed_path() {
    let observed = PathBuf::from("/opt/sifr/current");
    let expected = "/opt/sifr/from-cli";
    let response = crate::requests::sysroot_status_from_probe(
        ToolingSysrootProbe {
            status: Some(ToolingSysrootStatus {
                root: observed.clone(),
                toolchain_id: "0.1.0:test-target:abc123".to_string(),
            }),
            diagnostic: None,
        },
        Some(expected),
        None,
    );

    assert_eq!(
        response.get("ok").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        response.get("kind").and_then(serde_json::Value::as_str),
        Some("mismatch")
    );
    assert_eq!(
        response
            .pointer("/observedPaths/sysroot")
            .and_then(serde_json::Value::as_str),
        Some(observed.to_string_lossy().as_ref())
    );
    assert_eq!(
        response
            .get("expectedRoot")
            .and_then(serde_json::Value::as_str),
        Some(expected)
    );
    let diagnostic = response
        .pointer("/diagnostics/0")
        .and_then(|diagnostic| diagnostic.get("message"))
        .and_then(serde_json::Value::as_str)
        .expect("mismatch should include diagnostic");
    assert!(diagnostic.contains(expected));
    assert!(diagnostic.contains(observed.to_string_lossy().as_ref()));
    assert_eq!(
        response
            .pointer("/diagnostics/0/observedRoot")
            .and_then(serde_json::Value::as_str),
        Some(observed.to_string_lossy().as_ref())
    );
}

#[test]
fn sysroot_request_handler_reports_expected_root_and_toolchain_mismatch() {
    let mut session = Session::new();
    let response = crate::requests::handle(
        &mut session,
        "sifr/sysroot",
        json!({
            "expectedRoot": "/opt/sifr/from-cli",
            "expectedToolchainId": "0.1.0-release-aarch64-apple-darwin"
        }),
    )
    .expect("sysroot request should answer");

    assert_eq!(
        response.get("ok").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        response.get("kind").and_then(serde_json::Value::as_str),
        Some("mismatch")
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/kind")
            .and_then(serde_json::Value::as_str),
        Some("mismatch")
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/expectedRoot")
            .and_then(serde_json::Value::as_str),
        Some("/opt/sifr/from-cli")
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/expectedToolchainId")
            .and_then(serde_json::Value::as_str),
        Some("0.1.0-release-aarch64-apple-darwin")
    );
    assert!(
        response
            .pointer("/diagnostics/0/observedRoot")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|root| !root.is_empty())
    );
}

#[test]
fn sysroot_request_reports_expected_toolchain_mismatch() {
    let observed = PathBuf::from("/opt/sifr/current");
    let response = crate::requests::sysroot_status_from_probe(
        ToolingSysrootProbe {
            status: Some(ToolingSysrootStatus {
                root: observed.clone(),
                toolchain_id: "0.1.0-dev-aarch64-apple-darwin".to_string(),
            }),
            diagnostic: None,
        },
        Some(observed.to_string_lossy().as_ref()),
        Some("0.1.0-release-aarch64-apple-darwin"),
    );

    assert_eq!(
        response.get("ok").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/expectedToolchainId")
            .and_then(serde_json::Value::as_str),
        Some("0.1.0-release-aarch64-apple-darwin")
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/observedToolchainId")
            .and_then(serde_json::Value::as_str),
        Some("0.1.0-dev-aarch64-apple-darwin")
    );
}

#[test]
fn sysroot_request_reports_broken_sysroot_observed_paths() {
    let response = crate::requests::sysroot_status_from_probe(
        ToolingSysrootProbe {
            status: None,
            diagnostic: Some(sample_sysroot_diagnostic()),
        },
        None,
        None,
    );

    assert_eq!(
        response.get("ok").and_then(serde_json::Value::as_bool),
        Some(false)
    );
    assert_eq!(
        response.get("kind").and_then(serde_json::Value::as_str),
        Some("broken")
    );
    assert_eq!(
        response
            .pointer("/observedPaths/binary")
            .and_then(serde_json::Value::as_str),
        Some("/tmp/sifr/bin/sifr")
    );
    assert_eq!(
        response
            .pointer("/observedPaths/attemptedSysroot")
            .and_then(serde_json::Value::as_str),
        Some("/tmp/sifr")
    );
    assert_eq!(
        response
            .pointer("/observedPaths/asset")
            .and_then(serde_json::Value::as_str),
        Some("/tmp/sifr/sysroot.toml")
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/binaryPath")
            .and_then(serde_json::Value::as_str),
        Some("/tmp/sifr/bin/sifr")
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/attemptedSysroot")
            .and_then(serde_json::Value::as_str),
        Some("/tmp/sifr")
    );
    assert_eq!(
        response
            .pointer("/diagnostics/0/assetPath")
            .and_then(serde_json::Value::as_str),
        Some("/tmp/sifr/sysroot.toml")
    );
}

#[test]
fn initialized_sysroot_notification_includes_resolver_paths() {
    let notification =
        crate::notifications::tooling_sysroot_notification(&sample_sysroot_diagnostic());

    assert_eq!(notification.method, "window/showMessage");
    assert_eq!(
        notification
            .params
            .get("type")
            .and_then(serde_json::Value::as_i64),
        Some(1)
    );
    let message = notification
        .params
        .get("message")
        .and_then(serde_json::Value::as_str)
        .expect("notification should include message");
    assert!(message.contains("missing manifest"));
    assert!(message.contains("/tmp/sifr/bin/sifr"));
    assert!(message.contains("/tmp/sifr"));
    assert!(message.contains("/tmp/sifr/sysroot.toml"));
}

fn sample_sysroot_diagnostic() -> ToolingSysrootDiagnostic {
    ToolingSysrootDiagnostic {
        message: "missing manifest".to_string(),
        binary_path: PathBuf::from("/tmp/sifr/bin/sifr"),
        attempted_sysroot: PathBuf::from("/tmp/sifr"),
        asset_path: Some(PathBuf::from("/tmp/sifr/sysroot.toml")),
    }
}

struct CliSysrootStatus {
    root: String,
    toolchain_id: String,
}

fn cli_sysroot_status() -> CliSysrootStatus {
    let output = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string()))
        .args([
            "run", "--locked", "-q", "-p", "sifr", "--", "--print", "sysroot", "--json",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("sifr CLI should run");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("CLI sysroot JSON should parse");
    CliSysrootStatus {
        root: value
            .get("root")
            .and_then(serde_json::Value::as_str)
            .expect("CLI sysroot JSON should include root")
            .to_string(),
        toolchain_id: value
            .get("toolchain_id")
            .and_then(serde_json::Value::as_str)
            .expect("CLI sysroot JSON should include toolchain_id")
            .to_string(),
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .expect("sifr_lsp crate should live under workspace crates directory")
        .to_path_buf()
}

fn open_stdlib_import_fixture() -> (Session, tempfile::TempDir, String) {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("main.sifr");
    let source = "\
from sifr.random import randint

def main() -> int | None:
    try:
        x: int = randint(1, 2)
        return x
    except ValueError:
        return None
";
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
