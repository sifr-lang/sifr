use serde_json::Value;
use sifr_diagnostics::{DiagnosticArg, DiagnosticCode, RenderedDiagnostic, Severity};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORTED_TARGETS: &[&str] = &[
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-gnu",
    "aarch64-unknown-linux-gnu",
];

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

#[derive(Debug, Clone)]
pub(crate) struct ReceiptDiscoveryEnv {
    pub(crate) current_executable: PathBuf,
    pub(crate) manifest_dir: Option<PathBuf>,
    pub(crate) home_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct DiscoveredReceipt {
    pub(crate) receipt: InstallReceipt,
    pub(crate) receipt_path: PathBuf,
    pub(crate) current_executable: PathBuf,
    pub(crate) matches_receipt: bool,
}

impl ReceiptDiscoveryEnv {
    pub(crate) fn production() -> Result<Self, Box<RenderedDiagnostic>> {
        let current_executable = std::env::current_exe().map_err(|error| {
            unmanaged_receipt_diagnostic(format!(
                "could not determine current Sifr executable for self-update eligibility: {error}"
            ))
        })?;
        let current_executable = current_executable.canonicalize().map_err(|error| {
            unmanaged_receipt_diagnostic(format!(
                "could not canonicalize current Sifr executable {} for self-update eligibility: {error}",
                current_executable.display()
            ))
        })?;
        Ok(Self {
            current_executable,
            manifest_dir: std::env::var_os("SIFR_INSTALL_MANIFEST_DIR").map(PathBuf::from),
            home_dir: std::env::var_os("HOME").map(PathBuf::from),
        })
    }
}

pub(crate) fn discover_install_receipt(
    env: &ReceiptDiscoveryEnv,
) -> Result<DiscoveredReceipt, Box<RenderedDiagnostic>> {
    let receipt_path = discover_receipt_path(env)?;
    let input = fs::read_to_string(&receipt_path).map_err(|error| {
        unmanaged_receipt_diagnostic(format!(
            "standalone install receipt {} could not be read: {error}; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the managed install rules",
            receipt_path.display()
        ))
    })?;
    let receipt = parse_install_receipt_json(&input)?;
    validate_receipt_eligibility(&receipt, &receipt_path, env)?;
    Ok(DiscoveredReceipt {
        receipt,
        receipt_path,
        current_executable: env.current_executable.clone(),
        matches_receipt: true,
    })
}

fn discover_receipt_path(env: &ReceiptDiscoveryEnv) -> Result<PathBuf, Box<RenderedDiagnostic>> {
    if let Some(manifest_dir) = &env.manifest_dir {
        let receipt_path = manifest_dir.join("install.json");
        if receipt_path.is_file() {
            return Ok(receipt_path);
        }
        return Err(missing_receipt_diagnostic(format!(
            "standalone install receipt is missing at {} from SIFR_INSTALL_MANIFEST_DIR",
            receipt_path.display()
        )));
    }

    if let Some(parent) = env.current_executable.parent() {
        let receipt_path = parent.join("install.json");
        if receipt_path.is_file() {
            return Ok(receipt_path);
        }
    }

    if let Some(home_dir) = &env.home_dir {
        let default_binary = home_dir.join(".sifr/bin/sifr");
        if same_file(&env.current_executable, &default_binary).unwrap_or(false) {
            let receipt_path = home_dir.join(".sifr/install.json");
            if receipt_path.is_file() {
                return Ok(receipt_path);
            }
        }
    }

    Err(missing_receipt_diagnostic(
        "standalone install receipt is missing; use your package manager for package-managed installs or re-run `curl -LsSf https://sifr.sh/install | sh` to enter the managed install rules",
    ))
}

fn validate_receipt_eligibility(
    receipt: &InstallReceipt,
    receipt_path: &Path,
    env: &ReceiptDiscoveryEnv,
) -> Result<(), Box<RenderedDiagnostic>> {
    if !SUPPORTED_TARGETS.contains(&receipt.target.as_str()) {
        return Err(unmanaged_receipt_diagnostic(format!(
            "standalone install receipt target {} is not supported by the preview distribution",
            receipt.target
        )));
    }
    if receipt.channel != "alpha" && receipt.channel != "beta" {
        return Err(unmanaged_receipt_diagnostic(format!(
            "standalone install receipt channel {} is not supported while stable release channels are disabled; use alpha or beta",
            receipt.channel
        )));
    }
    if !same_file(&env.current_executable, Path::new(&receipt.binary_path)).map_err(|error| {
        unmanaged_receipt_diagnostic(format!(
            "could not compare current executable {} with receipt binary {}: {error}",
            env.current_executable.display(),
            receipt.binary_path
        ))
    })? {
        return Err(unmanaged_receipt_diagnostic(format!(
            "standalone install receipt belongs to {}, but the current executable is {}",
            receipt.binary_path,
            env.current_executable.display()
        )));
    }

    let install_dir = canonicalize_for_receipt(Path::new(&receipt.install_dir), "install_dir")?;
    let binary_path = canonicalize_for_receipt(Path::new(&receipt.binary_path), "binary_path")?;
    let binary_parent = binary_path.parent().ok_or_else(|| {
        unmanaged_receipt_diagnostic(
            "standalone install receipt binary_path has no parent directory",
        )
    })?;
    if !paths_same_after_canonicalization(&install_dir, binary_parent) {
        return Err(unmanaged_receipt_diagnostic(format!(
            "standalone install receipt binary_path {} is outside install_dir {}",
            receipt.binary_path, receipt.install_dir
        )));
    }
    if !receipt_path.is_file() {
        return Err(missing_receipt_diagnostic(format!(
            "standalone install receipt is missing at {}",
            receipt_path.display()
        )));
    }
    Ok(())
}

fn canonicalize_for_receipt(path: &Path, field: &str) -> Result<PathBuf, Box<RenderedDiagnostic>> {
    path.canonicalize().map_err(|error| {
        unmanaged_receipt_diagnostic(format!(
            "standalone install receipt field `{field}` could not be canonicalized at {}: {error}",
            path.display()
        ))
    })
}

fn paths_same_after_canonicalization(left: &Path, right: &Path) -> bool {
    #[cfg(unix)]
    {
        same_metadata(left, right).unwrap_or_else(|_| left == right)
    }
    #[cfg(not(unix))]
    {
        left == right
    }
}

fn same_file(left: &Path, right: &Path) -> Result<bool, std::io::Error> {
    #[cfg(unix)]
    {
        same_metadata(left, right)
    }
    #[cfg(not(unix))]
    {
        Ok(left.canonicalize()? == right.canonicalize()?)
    }
}

#[cfg(unix)]
fn same_metadata(left: &Path, right: &Path) -> Result<bool, std::io::Error> {
    use std::os::unix::fs::MetadataExt as _;

    let left = fs::metadata(left)?;
    let right = fs::metadata(right)?;
    Ok(left.dev() == right.dev() && left.ino() == right.ino())
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
) -> Result<InstallReceipt, Box<RenderedDiagnostic>> {
    let value = serde_json::from_str::<Value>(input).map_err(|error| {
        unmanaged_receipt_diagnostic(format!(
            "standalone install receipt is not valid JSON: {error}; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the self-update-managed install rules"
        ))
    })?;
    let object = value.as_object().ok_or_else(|| {
        unmanaged_receipt_diagnostic(
            "standalone install receipt must be a JSON object; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the self-update-managed install rules",
        )
    })?;

    let expected = RECEIPT_FIELDS.iter().copied().collect::<BTreeSet<_>>();
    let actual = object.keys().map(String::as_str).collect::<BTreeSet<_>>();
    if actual != expected {
        return Err(unmanaged_receipt_diagnostic(
            "standalone install receipt predates or diverges from the schema-versioned self-update rules; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the managed install rules",
        ));
    }

    let schema_version = object
        .get("schema_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| malformed_field("schema_version"))?;
    if schema_version != 1 {
        return Err(unmanaged_receipt_diagnostic(format!(
            "standalone install receipt schema_version {schema_version} is unsupported; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the managed install rules"
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
) -> Result<&'a str, Box<RenderedDiagnostic>> {
    object
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| malformed_field(field))
}

fn malformed_field(field: &str) -> Box<RenderedDiagnostic> {
    unmanaged_receipt_diagnostic(format!(
        "standalone install receipt field `{field}` is missing or malformed; re-run `curl -LsSf https://sifr.sh/install | sh` to enter the self-update-managed install rules"
    ))
}

fn unmanaged_receipt_diagnostic(message: impl Into<String>) -> Box<RenderedDiagnostic> {
    let message = message.into();
    let mut args = BTreeMap::new();
    args.insert("message".to_owned(), DiagnosticArg::String(message.clone()));
    Box::new(RenderedDiagnostic {
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
    })
}

fn missing_receipt_diagnostic(message: impl Into<String>) -> Box<RenderedDiagnostic> {
    let mut diagnostic = unmanaged_receipt_diagnostic(message);
    diagnostic.help = Some(
        "self-update is available only for official standalone installs with install.json receipts"
            .to_owned(),
    );
    diagnostic
}

#[cfg(test)]
mod tests {
    use super::{
        discover_install_receipt, parse_install_receipt_json, InstallReceipt, ReceiptDiscoveryEnv,
    };
    use sifr_diagnostics::DiagnosticCode;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

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

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "sifr-self-update-{name}-{}-{nonce}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create temp dir");
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(path, b"sifr").expect("write file");
    }

    fn receipt_json(
        version: &str,
        channel: &str,
        install_dir: &Path,
        binary_path: &Path,
    ) -> String {
        serde_json::json!({
            "schema_version": 1,
            "name": "sifr",
            "version": version,
            "channel": channel,
            "target": "aarch64-apple-darwin",
            "install_dir": install_dir.display().to_string(),
            "binary_path": binary_path.display().to_string(),
            "artifact": format!("sifr-{version}-aarch64-apple-darwin.tar.gz"),
            "modify_path": true,
        })
        .to_string()
    }

    fn write_receipt(path: &Path, version: &str, channel: &str, binary_path: &Path) {
        let install_dir = binary_path.parent().expect("binary parent");
        fs::create_dir_all(path.parent().expect("receipt parent")).expect("create receipt parent");
        fs::write(
            path,
            receipt_json(version, channel, install_dir, binary_path),
        )
        .expect("write receipt");
    }

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

    #[test]
    fn discovers_manifest_dir_before_adjacent_manifest() {
        let tmp = TestDir::new("discovery-order");
        let binary = tmp.path().join("bin/sifr");
        touch(&binary);
        write_receipt(
            &tmp.path().join("manifest/install.json"),
            "0.1.0-beta.2",
            "beta",
            &binary,
        );
        write_receipt(
            &tmp.path().join("bin/install.json"),
            "0.1.0-alpha.1",
            "alpha",
            &binary,
        );

        let discovered = discover_install_receipt(&ReceiptDiscoveryEnv {
            current_executable: binary,
            manifest_dir: Some(tmp.path().join("manifest")),
            home_dir: None,
        })
        .expect("receipt discovers");

        assert_eq!(discovered.receipt.version, "0.1.0-beta.2");
        assert!(discovered
            .receipt_path
            .ends_with(Path::new("manifest/install.json")));
    }

    #[test]
    fn discovers_default_home_manifest_only_for_default_binary() {
        let tmp = TestDir::new("default-home");
        let binary = tmp.path().join(".sifr/bin/sifr");
        touch(&binary);
        write_receipt(
            &tmp.path().join(".sifr/install.json"),
            "0.1.0-beta.2",
            "beta",
            &binary,
        );

        let discovered = discover_install_receipt(&ReceiptDiscoveryEnv {
            current_executable: binary,
            manifest_dir: None,
            home_dir: Some(tmp.path().to_path_buf()),
        })
        .expect("default receipt discovers");

        assert_eq!(discovered.receipt.version, "0.1.0-beta.2");
    }

    #[test]
    fn rejects_receipt_for_different_executable() {
        let tmp = TestDir::new("mismatch");
        let current = tmp.path().join("bin/sifr");
        let other = tmp.path().join("other/sifr");
        touch(&current);
        touch(&other);
        write_receipt(
            &tmp.path().join("manifest/install.json"),
            "0.1.0-beta.2",
            "beta",
            &other,
        );

        let error = discover_install_receipt(&ReceiptDiscoveryEnv {
            current_executable: current,
            manifest_dir: Some(tmp.path().join("manifest")),
            home_dir: None,
        })
        .expect_err("mismatched executable is rejected");

        assert!(error.message.contains("belongs to"));
    }

    #[cfg(unix)]
    #[test]
    fn accepts_symlinked_current_executable_using_same_file_metadata() {
        use std::os::unix::fs::symlink;

        let tmp = TestDir::new("symlink");
        let binary = tmp.path().join("bin/sifr");
        let symlink_path = tmp.path().join("linked-sifr");
        touch(&binary);
        symlink(&binary, &symlink_path).expect("create symlink");
        write_receipt(
            &tmp.path().join("manifest/install.json"),
            "0.1.0-beta.2",
            "beta",
            &binary,
        );

        let discovered = discover_install_receipt(&ReceiptDiscoveryEnv {
            current_executable: symlink_path,
            manifest_dir: Some(tmp.path().join("manifest")),
            home_dir: None,
        })
        .expect("symlink matches receipt binary");

        assert!(discovered.matches_receipt);
    }

    #[cfg(unix)]
    #[test]
    fn accepts_hardlinked_current_executable_using_same_file_metadata() {
        let tmp = TestDir::new("hardlink");
        let binary = tmp.path().join("bin/sifr");
        let hardlink_path = tmp.path().join("hardlinked-sifr");
        touch(&binary);
        fs::hard_link(&binary, &hardlink_path).expect("create hardlink");
        write_receipt(
            &tmp.path().join("manifest/install.json"),
            "0.1.0-beta.2",
            "beta",
            &binary,
        );

        let discovered = discover_install_receipt(&ReceiptDiscoveryEnv {
            current_executable: hardlink_path,
            manifest_dir: Some(tmp.path().join("manifest")),
            home_dir: None,
        })
        .expect("hardlink matches receipt binary");

        assert!(discovered.matches_receipt);
    }
}
