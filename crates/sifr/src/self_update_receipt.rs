use serde_json::Value;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic, Severity};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct InstallReceipt {
    pub(crate) name: String,
    pub(crate) version: String,
    pub(crate) channel: String,
    pub(crate) target: String,
    pub(crate) install_dir: String,
    pub(crate) binary_path: String,
    pub(crate) artifact: String,
    pub(crate) modify_path: bool,
}

const RECEIPT_FIELDS: &[&str] = &[
    "schema_version",
    "name",
    "version",
    "channel",
    "target",
    "install_dir",
    "binary_path",
    "artifact",
    "modify_path",
];

pub(crate) fn parse_install_receipt_json(
    input: &str,
) -> Result<InstallReceipt, RenderedDiagnostic> {
    let value = serde_json::from_str::<Value>(input).map_err(|error| {
        unmanaged_receipt_diagnostic(format!(
            "standalone install receipt is not valid JSON: {error}; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the self-update-managed install contract"
        ))
    })?;
    let object = value.as_object().ok_or_else(|| {
        unmanaged_receipt_diagnostic(
            "standalone install receipt must be a JSON object; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the self-update-managed install contract",
        )
    })?;

    let expected = RECEIPT_FIELDS.iter().copied().collect::<BTreeSet<_>>();
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(unmanaged_receipt_diagnostic(format!(
            "standalone install receipt predates or diverges from the schema-versioned self-update contract; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the managed contract"
        )));
    }

    let schema_version = object
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| malformed_field("schema_version"))?;
    if schema_version != 1 {
        return Err(unmanaged_receipt_diagnostic(format!(
            "standalone install receipt schema_version {schema_version} is unsupported; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the managed contract"
        )));
    }

    let name = string_field(object, "name")?;
    if name != "sifr" {
        return Err(unmanaged_receipt_diagnostic(format!(
            "standalone install receipt belongs to {name}, not sifr"
        )));
    }

    Ok(InstallReceipt {
        name: name.to_owned(),
        version: string_field(object, "version")?.to_owned(),
        channel: string_field(object, "channel")?.to_owned(),
        target: string_field(object, "target")?.to_owned(),
        install_dir: string_field(object, "install_dir")?.to_owned(),
        binary_path: string_field(object, "binary_path")?.to_owned(),
        artifact: string_field(object, "artifact")?.to_owned(),
        modify_path: object
            .get("modify_path")
            .and_then(Value::as_bool)
            .ok_or_else(|| malformed_field("modify_path"))?,
    })
}

fn string_field<'a>(
    object: &'a serde_json::Map<String, Value>,
    field: &str,
) -> Result<&'a str, RenderedDiagnostic> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| malformed_field(field))
}

fn malformed_field(field: &str) -> RenderedDiagnostic {
    unmanaged_receipt_diagnostic(format!(
        "standalone install receipt field `{field}` is missing or malformed; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the self-update-managed install contract"
    ))
}

fn unmanaged_receipt_diagnostic(message: impl Into<String>) -> RenderedDiagnostic {
    let message = message.into();
    let mut args = BTreeMap::new();
    args.insert("message".to_owned(), DiagnosticArg::String(message.clone()));
    RenderedDiagnostic {
        code: DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT.code().to_owned(),
        severity: Severity::Error,
        message,
        message_template: "{message}".to_owned(),
        args,
        url: DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT.docs_url(),
        spans: Vec::new(),
        children: Vec::new(),
        help: Some(
            "standalone self-update requires a schema-versioned install.json written by the official Sifr installer".to_owned(),
        ),
        suggestions: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_install_receipt_json, InstallReceipt};
    use sifr_diagnostics::DiagnosticCode;

    const VALID_RECEIPT: &str = r#"{
  "schema_version": 1,
  "name": "sifr",
  "version": "0.1.0-beta.2",
  "channel": "beta",
  "target": "aarch64-apple-darwin",
  "install_dir": "/Users/example/.sifr/bin",
  "binary_path": "/Users/example/.sifr/bin/sifr",
  "artifact": "sifr-0.1.0-beta.2-aarch64-apple-darwin.tar.gz",
  "modify_path": true
}"#;

    #[test]
    fn parses_schema_versioned_receipt_shape() {
        let receipt = parse_install_receipt_json(VALID_RECEIPT).expect("receipt parses");
        assert_eq!(
            receipt,
            InstallReceipt {
                name: "sifr".to_owned(),
                version: "0.1.0-beta.2".to_owned(),
                channel: "beta".to_owned(),
                target: "aarch64-apple-darwin".to_owned(),
                install_dir: "/Users/example/.sifr/bin".to_owned(),
                binary_path: "/Users/example/.sifr/bin/sifr".to_owned(),
                artifact: "sifr-0.1.0-beta.2-aarch64-apple-darwin.tar.gz".to_owned(),
                modify_path: true,
            }
        );
    }

    #[test]
    fn rejects_pre_schema_receipt_with_remediation() {
        let error = parse_install_receipt_json(r#"{"name":"sifr","version":"0.1.0-beta.1"}"#)
            .expect_err("pre-schema receipts are unmanaged");
        assert_eq!(
            error.code,
            DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT.code()
        );
        assert!(error.message.contains("predates"));
        assert!(error
            .message
            .contains("curl -LsSf https://sifr.sh/install | sh"));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let error = parse_install_receipt_json(
            r#"{
  "schema_version": 2,
  "name": "sifr",
  "version": "0.1.0-beta.2",
  "channel": "beta",
  "target": "aarch64-apple-darwin",
  "install_dir": "/Users/example/.sifr/bin",
  "binary_path": "/Users/example/.sifr/bin/sifr",
  "artifact": "sifr-0.1.0-beta.2-aarch64-apple-darwin.tar.gz",
  "modify_path": true
}"#,
        )
        .expect_err("unsupported schema versions are rejected");
        assert_eq!(
            error.code,
            DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT.code()
        );
        assert!(error.message.contains("schema_version 2 is unsupported"));
    }

    #[test]
    fn rejects_empty_receipt_json() {
        let error = parse_install_receipt_json("").expect_err("empty receipts are rejected");
        assert_eq!(
            error.code,
            DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT.code()
        );
        assert!(error.message.contains("not valid JSON"));
    }

    #[test]
    fn rejects_invalid_receipt_json() {
        let error = parse_install_receipt_json("{").expect_err("invalid JSON is rejected");
        assert_eq!(
            error.code,
            DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT.code()
        );
        assert!(error.message.contains("not valid JSON"));
    }

    #[test]
    fn rejects_unknown_fields() {
        let error = parse_install_receipt_json(
            r#"{
  "schema_version": 1,
  "name": "sifr",
  "version": "0.1.0-beta.2",
  "channel": "beta",
  "target": "aarch64-apple-darwin",
  "install_dir": "/Users/example/.sifr/bin",
  "binary_path": "/Users/example/.sifr/bin/sifr",
  "artifact": "sifr-0.1.0-beta.2-aarch64-apple-darwin.tar.gz",
  "modify_path": true,
  "installer_url": "https://example.invalid"
}"#,
        )
        .expect_err("unknown fields are rejected");
        assert_eq!(
            error.code,
            DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT.code()
        );
    }

    #[test]
    fn rejects_wrong_field_types() {
        let error = parse_install_receipt_json(
            r#"{
  "schema_version": 1,
  "name": "sifr",
  "version": "0.1.0-beta.2",
  "channel": "beta",
  "target": "aarch64-apple-darwin",
  "install_dir": "/Users/example/.sifr/bin",
  "binary_path": "/Users/example/.sifr/bin/sifr",
  "artifact": "sifr-0.1.0-beta.2-aarch64-apple-darwin.tar.gz",
  "modify_path": "false"
}"#,
        )
        .expect_err("wrong field types are rejected");
        assert!(error.message.contains("modify_path"));
    }
}
