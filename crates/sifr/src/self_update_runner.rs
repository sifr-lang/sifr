use crate::cli_model_and_entrypoint::{EXIT_USER_DIAGNOSTIC, diagnostic_with_code};
use crate::self_update_metadata::{UpdateAction, UpdatePlan};
use crate::self_update_receipt::DiscoveredReceipt;
use sifr_diagnostics::{DiagnosticCode, RenderedDiagnostic};
use sifr_sysroot::sha256_hex;
use std::fs;
use std::io::{self, BufRead as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const MIN_INSTALLER_BYTES: u64 = 1024;
const LOCK_POLL_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub(crate) struct RunnerError {
    pub(crate) diagnostic: Box<RenderedDiagnostic>,
    pub(crate) exit_code: i32,
}

#[derive(Debug, Clone)]
pub(crate) struct SelfUpdateRunner {
    curl_program: PathBuf,
}

impl SelfUpdateRunner {
    pub(crate) fn production() -> Self {
        Self {
            curl_program: PathBuf::from("curl"),
        }
    }

    pub(crate) fn run(
        &self,
        plan: &UpdatePlan,
        discovered: &DiscoveredReceipt,
    ) -> Result<i32, RunnerError> {
        if plan.action == UpdateAction::NoOp {
            return Ok(0);
        }

        let temp_dir = TempWorkDir::create("sifr-self-update")?;
        let installer_path = self.download_installer(plan, temp_dir.path())?;
        validate_installer(&installer_path, &plan.installer_sha256)?;
        make_executable(&installer_path)?;

        let install_dir = Path::new(&discovered.receipt.install_dir);
        let lock = InstallLock::acquire(install_dir)?;
        let status = Self::run_installer(plan, discovered, &installer_path)?;
        drop(lock);

        if status.success() {
            return Ok(status.code().unwrap_or(0));
        }

        Err(runner_error_with_exit(
            format!(
                "self-update installer exited with status {}; installer stdout/stderr above is preserved",
                status.code().map_or_else(
                    || "terminated by signal".to_owned(),
                    |code| code.to_string()
                )
            ),
            status.code().unwrap_or(EXIT_USER_DIAGNOSTIC),
        ))
    }

    fn download_installer(
        &self,
        plan: &UpdatePlan,
        temp_dir: &Path,
    ) -> Result<PathBuf, RunnerError> {
        let partial_path = temp_dir.join("installer.download");
        let final_path = temp_dir.join("installer.sh");
        let url = plan.target_version.installer_url();
        let status = Command::new(&self.curl_program)
            .args([
                "-fsSL",
                "--proto",
                "=https",
                "--proto-redir",
                "=https",
                &url,
                "-o",
            ])
            .arg(&partial_path)
            .status()
            .map_err(|error| {
                runner_error(format!(
                    "could not run curl to download self-update installer {url}: {error}"
                ))
            })?;
        if !status.success() {
            return Err(runner_error(format!(
                "self-update installer download failed for {url}; curl exited with status {}",
                status.code().map_or_else(
                    || "terminated by signal".to_owned(),
                    |code| code.to_string()
                )
            )));
        }
        fs::rename(&partial_path, &final_path).map_err(|error| {
            runner_error(format!(
                "could not finalize downloaded self-update installer {}: {error}",
                final_path.display()
            ))
        })?;
        Ok(final_path)
    }

    fn run_installer(
        plan: &UpdatePlan,
        discovered: &DiscoveredReceipt,
        installer_path: &Path,
    ) -> Result<std::process::ExitStatus, RunnerError> {
        let mut command = Command::new(installer_path);
        if plan.force {
            command.arg("--force");
        }
        command.env("SIFR_INSTALL_DIR", &discovered.receipt.install_dir);
        command.env("SIFR_SYSROOT_INSTALL_DIR", &discovered.receipt.sysroot_path);
        command.env("SIFR_INSTALL_LOCK_HELD", "1");
        if !discovered.receipt.modify_path {
            command.env("SIFR_NO_MODIFY_PATH", "1");
        }
        if let Some(manifest_dir) = Self::manifest_dir_override(discovered)? {
            command.env("SIFR_INSTALL_MANIFEST_DIR", manifest_dir);
        }
        command.status().map_err(|error| {
            runner_error(format!(
                "could not execute self-update installer {}: {error}",
                installer_path.display()
            ))
        })
    }

    fn manifest_dir_override(
        discovered: &DiscoveredReceipt,
    ) -> Result<Option<PathBuf>, RunnerError> {
        let receipt_path = canonicalize_existing_path(&discovered.receipt_path, "receipt path")?;
        let default_path = Path::new(&discovered.receipt.sysroot_path).join("install.json");
        if paths_equivalent(&receipt_path, &default_path) {
            return Ok(None);
        }
        let manifest_dir = receipt_path.parent().ok_or_else(|| {
            runner_error(format!(
                "standalone install receipt path {} has no parent directory",
                receipt_path.display()
            ))
        })?;
        Ok(Some(manifest_dir.to_path_buf()))
    }
}

fn validate_installer(path: &Path, expected_sha256: &str) -> Result<(), RunnerError> {
    let metadata = fs::metadata(path).map_err(|error| {
        runner_error(format!(
            "could not inspect downloaded self-update installer {}: {error}",
            path.display()
        ))
    })?;
    if metadata.len() < MIN_INSTALLER_BYTES {
        return Err(runner_error(format!(
            "downloaded self-update installer {} is too small: {} bytes",
            path.display(),
            metadata.len()
        )));
    }

    let bytes = fs::read(path).map_err(|error| {
        runner_error(format!(
            "could not hash downloaded self-update installer {}: {error}",
            path.display()
        ))
    })?;
    let actual_sha256 = sha256_hex(&bytes);
    if actual_sha256 != expected_sha256 {
        return Err(runner_error(format!(
            "downloaded self-update installer SHA-256 mismatch: expected {expected_sha256}, got {actual_sha256}"
        )));
    }

    let file = fs::File::open(path).map_err(|error| {
        runner_error(format!(
            "could not read downloaded self-update installer {}: {error}",
            path.display()
        ))
    })?;
    let mut reader = io::BufReader::new(file);
    let mut first_line = Vec::new();
    reader.read_until(b'\n', &mut first_line).map_err(|error| {
        runner_error(format!(
            "could not read downloaded self-update installer header {}: {error}",
            path.display()
        ))
    })?;
    if !first_line.starts_with(b"#!") {
        return Err(runner_error(format!(
            "downloaded self-update installer {} does not start with a shebang",
            path.display()
        )));
    }
    Ok(())
}

fn make_executable(path: &Path) -> Result<(), RunnerError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        let mut permissions = fs::metadata(path)
            .map_err(|error| {
                runner_error(format!(
                    "could not inspect downloaded self-update installer {}: {error}",
                    path.display()
                ))
            })?
            .permissions();
        permissions.set_mode(permissions.mode() | 0o700);
        fs::set_permissions(path, permissions).map_err(|error| {
            runner_error(format!(
                "could not make downloaded self-update installer executable {}: {error}",
                path.display()
            ))
        })?;
    }
    Ok(())
}

#[derive(Debug)]
struct InstallLock {
    path: PathBuf,
}

impl InstallLock {
    fn acquire(install_dir: &Path) -> Result<Self, RunnerError> {
        fs::create_dir_all(install_dir).map_err(|error| {
            runner_error(format!(
                "could not create self-update install directory {}: {error}",
                install_dir.display()
            ))
        })?;
        let path = install_dir.join(".sifr-update.lock");
        loop {
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    std::thread::sleep(LOCK_POLL_INTERVAL);
                }
                Err(error) => {
                    return Err(runner_error(format!(
                        "could not acquire self-update lock {}: {error}",
                        path.display()
                    )));
                }
            }
        }
    }
}

impl Drop for InstallLock {
    fn drop(&mut self) {
        let _ = fs::remove_dir(&self.path);
    }
}

#[derive(Debug)]
struct TempWorkDir {
    path: PathBuf,
}

impl TempWorkDir {
    fn create(prefix: &str) -> Result<Self, RunnerError> {
        let mut attempt = 0_u32;
        loop {
            let path = std::env::temp_dir().join(format!(
                "{prefix}-{}-{}-{attempt}",
                std::process::id(),
                nonce()
            ));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    attempt = attempt.saturating_add(1);
                }
                Err(error) => {
                    return Err(runner_error(format!(
                        "could not create self-update temporary directory {}: {error}",
                        path.display()
                    )));
                }
            }
        }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempWorkDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn nonce() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
}

fn canonicalize_existing_path(path: &Path, label: &str) -> Result<PathBuf, RunnerError> {
    path.canonicalize().map_err(|error| {
        runner_error(format!(
            "could not canonicalize self-update {label} {}: {error}",
            path.display()
        ))
    })
}

fn paths_equivalent(left: &Path, right: &Path) -> bool {
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

fn runner_error(message: impl Into<String>) -> RunnerError {
    runner_error_with_exit(message, EXIT_USER_DIAGNOSTIC)
}

fn runner_error_with_exit(message: impl Into<String>, exit_code: i32) -> RunnerError {
    RunnerError {
        diagnostic: Box::new(diagnostic_with_code(
            message,
            DiagnosticCode::SELF_UPDATE_UNMANAGED_RECEIPT,
        )),
        exit_code,
    }
}

#[cfg(test)]
mod tests {
    use super::SelfUpdateRunner;
    use crate::self_update_metadata::{PreviewChannel, PreviewVersion, UpdateAction, UpdatePlan};
    use crate::self_update_receipt::{DiscoveredReceipt, InstallReceipt};
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    struct TestDir(PathBuf);

    impl TestDir {
        fn new(name: &str) -> Self {
            let nonce = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "sifr-self-update-runner-{name}-{}-{nonce}",
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

    fn runner(curl: &Path) -> SelfUpdateRunner {
        SelfUpdateRunner {
            curl_program: curl.to_path_buf(),
        }
    }

    fn plan(force: bool) -> UpdatePlan {
        UpdatePlan {
            current_version: PreviewVersion::parse("0.1.0-beta.1").unwrap(),
            target_version: PreviewVersion::parse("0.1.0-beta.2").unwrap(),
            receipt_channel: PreviewChannel::Beta,
            requested_channel: None,
            resolved_channel: PreviewChannel::Beta,
            action: UpdateAction::Update,
            force,
            installer_sha256: "d".repeat(64),
        }
    }

    fn plan_for_installer(force: bool, installer: &Path) -> UpdatePlan {
        let mut plan = plan(force);
        plan.installer_sha256 =
            sifr_sysroot::sha256_hex(&fs::read(installer).expect("read installer fixture"));
        plan
    }

    fn discovered(root: &Path, modify_path: bool, default_manifest: bool) -> DiscoveredReceipt {
        let install_dir = root.join("home/.sifr/bin");
        fs::create_dir_all(&install_dir).expect("create install dir");
        let binary_path = install_dir.join("sifr");
        let sysroot_path = root.join("home/.sifr");
        fs::write(&binary_path, "sifr").expect("write binary");
        fs::write(sysroot_path.join("sysroot.toml"), "").expect("write sysroot manifest");
        let receipt_path = if default_manifest {
            root.join("home/.sifr/install.json")
        } else {
            root.join("receipts/install.json")
        };
        fs::create_dir_all(receipt_path.parent().expect("receipt parent"))
            .expect("create receipt parent");
        fs::write(&receipt_path, "{}").expect("write receipt");
        DiscoveredReceipt {
            receipt: InstallReceipt {
                name: "sifr".to_owned(),
                version: "0.1.0-beta.1".to_owned(),
                channel: "beta".to_owned(),
                target: "aarch64-apple-darwin".to_owned(),
                install_dir: install_dir.display().to_string(),
                binary_path: binary_path.display().to_string(),
                sysroot_path: sysroot_path.display().to_string(),
                sysroot_schema_version: 1,
                sysroot_sifr_version: "0.1.0-beta.1".to_owned(),
                sysroot_target_triple: "aarch64-apple-darwin".to_owned(),
                sysroot_content_sha256:
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_owned(),
                artifact: "sifr-0.1.0-beta.1-aarch64-apple-darwin.tar.gz".to_owned(),
                modify_path,
            },
            receipt_path,
            current_executable: binary_path,
            matches_receipt: true,
        }
    }

    fn write_fake_curl(root: &Path, installer_source: &Path) -> PathBuf {
        let curl = root.join("fake-curl.sh");
        fs::write(
            &curl,
            format!(
                r#"#!/bin/sh
set -eu
out=""
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-o" ]; then
    shift
    out="$1"
  fi
  shift || true
done
cp "{}" "$out"
"#,
                installer_source.display()
            ),
        )
        .expect("write fake curl");
        make_executable(&curl);
        curl
    }

    fn write_installer(root: &Path, body: &str) -> PathBuf {
        let installer = root.join("downloaded-installer.sh");
        let padding = "#".repeat(1100);
        fs::write(&installer, format!("#!/bin/sh\n{padding}\n{body}\n")).expect("write installer");
        make_executable(&installer);
        installer
    }

    #[cfg(unix)]
    fn make_executable(path: &Path) {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("chmod");
    }

    #[cfg(not(unix))]
    fn make_executable(_path: &Path) {}

    #[test]
    fn passes_receipt_environment_force_and_manifest_override_to_installer() {
        let root = TestDir::new("env");
        let record = root.path().join("record.txt");
        let installer = write_installer(
            root.path(),
            &format!(
                r#"printf 'dir=%s\nsysroot=%s\nmanifest=%s\nno_path=%s\nlock=%s\nargs=%s\n' \
  "$SIFR_INSTALL_DIR" "${{SIFR_SYSROOT_INSTALL_DIR:-}}" "${{SIFR_INSTALL_MANIFEST_DIR:-}}" "${{SIFR_NO_MODIFY_PATH:-}}" "${{SIFR_INSTALL_LOCK_HELD:-}}" "$*" > "{}"
"#,
                record.display()
            ),
        );
        let curl = write_fake_curl(root.path(), &installer);
        let discovered = discovered(root.path(), false, false);

        let exit = runner(&curl)
            .run(&plan_for_installer(true, &installer), &discovered)
            .expect("runner succeeds");

        assert_eq!(exit, 0);
        let output = fs::read_to_string(record).expect("read record");
        assert!(output.contains(&format!("dir={}", discovered.receipt.install_dir)));
        assert!(output.contains(&format!("sysroot={}", discovered.receipt.sysroot_path)));
        let expected_manifest_dir = discovered
            .receipt_path
            .parent()
            .unwrap()
            .canonicalize()
            .expect("canonical receipt parent");
        assert!(output.contains(&format!("manifest={}", expected_manifest_dir.display())));
        assert!(output.contains("no_path=1"));
        assert!(output.contains("lock=1"));
        assert!(output.contains("args=--force"));
    }

    #[test]
    fn omits_manifest_override_for_default_home_manifest() {
        let root = TestDir::new("default-manifest");
        let record = root.path().join("record.txt");
        let installer = write_installer(
            root.path(),
            &format!(
                r#"printf 'manifest=%s\n' "${{SIFR_INSTALL_MANIFEST_DIR:-}}" > "{}""#,
                record.display()
            ),
        );
        let curl = write_fake_curl(root.path(), &installer);
        let discovered = discovered(root.path(), true, true);

        runner(&curl)
            .run(&plan_for_installer(false, &installer), &discovered)
            .expect("runner succeeds");

        let output = fs::read_to_string(record).expect("read record");
        assert_eq!(output, "manifest=\n");
    }

    #[test]
    fn rejects_tiny_download_before_execution() {
        let root = TestDir::new("tiny");
        let tiny = root.path().join("tiny.sh");
        fs::write(&tiny, "#!/bin/sh\n").expect("write tiny installer");
        let curl = write_fake_curl(root.path(), &tiny);
        let error = runner(&curl)
            .run(
                &plan_for_installer(false, &tiny),
                &discovered(root.path(), true, true),
            )
            .expect_err("tiny downloads are rejected");
        assert!(error.diagnostic.message.contains("too small"));
    }

    #[test]
    fn rejects_download_without_shebang_before_execution() {
        let root = TestDir::new("no-shebang");
        let bad = root.path().join("bad.sh");
        fs::write(&bad, format!("{}\n", "x".repeat(1100))).expect("write bad installer");
        let curl = write_fake_curl(root.path(), &bad);
        let error = runner(&curl)
            .run(
                &plan_for_installer(false, &bad),
                &discovered(root.path(), true, true),
            )
            .expect_err("non-shell downloads are rejected");
        assert!(error.diagnostic.message.contains("shebang"));
    }

    #[test]
    fn rejects_installer_digest_mismatch_before_execution() {
        let root = TestDir::new("digest");
        let record = root.path().join("executed.txt");
        let installer = write_installer(
            root.path(),
            &format!("printf executed > \"{}\"", record.display()),
        );
        let curl = write_fake_curl(root.path(), &installer);
        let error = runner(&curl)
            .run(&plan(false), &discovered(root.path(), true, true))
            .expect_err("digest mismatch is rejected");
        assert!(error.diagnostic.message.contains("SHA-256 mismatch"));
        assert!(!record.exists());
    }

    #[test]
    fn maps_installer_failure_to_diagnostic_and_exit_code() {
        let root = TestDir::new("failure");
        let installer = write_installer(root.path(), "exit 7");
        let curl = write_fake_curl(root.path(), &installer);
        let error = runner(&curl)
            .run(
                &plan_for_installer(false, &installer),
                &discovered(root.path(), true, true),
            )
            .expect_err("installer failure is mapped");
        assert_eq!(error.exit_code, 7);
        assert!(error.diagnostic.message.contains("installer exited"));
    }

    #[test]
    fn serializes_concurrent_updates_on_install_lock() {
        let root = TestDir::new("lock");
        let record = root.path().join("record.txt");
        let installer = write_installer(
            root.path(),
            &format!(
                r#"sleep 1
printf '%s\n' "$SIFR_INSTALL_DIR" >> "{}"
"#,
                record.display()
            ),
        );
        let curl = write_fake_curl(root.path(), &installer);
        let discovered = discovered(root.path(), true, true);
        let first_runner = runner(&curl);
        let second_runner = first_runner.clone();
        let first_discovered = discovered.clone();
        let second_discovered = discovered.clone();

        let first_plan = plan_for_installer(false, &installer);
        let second_plan = first_plan.clone();
        let first = thread::spawn(move || first_runner.run(&first_plan, &first_discovered));
        thread::sleep(Duration::from_millis(150));
        let second = thread::spawn(move || second_runner.run(&second_plan, &second_discovered));

        first.join().expect("first thread").expect("first update");
        second
            .join()
            .expect("second thread")
            .expect("second update");

        let output = fs::read_to_string(record).expect("read record");
        assert_eq!(output.lines().count(), 2);
        assert!(
            !Path::new(&discovered.receipt.install_dir)
                .join(".sifr-update.lock")
                .exists()
        );
    }

    #[test]
    fn no_op_plan_skips_download_and_lock() {
        let root = TestDir::new("noop");
        let mut no_op_plan = plan(false);
        no_op_plan.action = UpdateAction::NoOp;
        let missing_curl = root.path().join("missing-curl");
        let discovered = discovered(root.path(), true, true);

        let exit = runner(&missing_curl)
            .run(&no_op_plan, &discovered)
            .expect("no-op skips network");

        assert_eq!(exit, 0);
        assert!(
            !Path::new(&discovered.receipt.install_dir)
                .join(".sifr-update.lock")
                .exists()
        );
    }
}
