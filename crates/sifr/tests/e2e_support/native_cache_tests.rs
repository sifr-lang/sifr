use super::native_cache::{NativeCache, select_artifact, write_changed};
use std::path::{Path, PathBuf};
use std::process::Command;

struct Project {
    temp: tempfile::TempDir,
    target: PathBuf,
    project: PathBuf,
}
impl Project {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        // Canonicalize /var on macOS before applying the cache storage contract.
        let root = temp.path().canonicalize().unwrap();
        let project = root.join("app");
        sifr_cache_storage::directory(&project.join("src")).unwrap();
        let target = root.join("target");
        write_changed(&project.join("Cargo.toml"), b"[package]\nname='probe'\nversion='0.1.0'\nedition='2024'\n[workspace]\n[features]\nchanged=[]\n").unwrap();
        write_changed(
            &project.join("src/main.rs"),
            b"fn main() { println!(\"one\"); }\n",
        )
        .unwrap();
        Self {
            temp,
            target,
            project,
        }
    }
    fn cargo(&self, arguments: &[&str]) -> std::process::Output {
        Command::new("cargo")
            .args([
                "build",
                "--offline",
                "--message-format=json-render-diagnostics",
                "-j",
                "1",
            ])
            .args(arguments)
            .env("CARGO_TARGET_DIR", &self.target)
            .current_dir(&self.project)
            .output()
            .unwrap()
    }
    fn build(&self, arguments: &[&str]) -> Result<super::native_cache::NativeArtifact, String> {
        let output = self.cargo(arguments);
        select_artifact(
            output.status.success(),
            &String::from_utf8_lossy(&output.stdout),
            &self.project,
            &self.target,
            "probe",
        )
        .map_err(|e| format!("{e}\n{}", String::from_utf8_lossy(&output.stderr)))
    }
}
fn run(path: &Path) -> String {
    let result = Command::new(path).output().unwrap();
    assert!(result.status.success());
    String::from_utf8(result.stdout).unwrap()
}

#[test]
fn generated_writes_compare_content_and_preserve_unchanged_timestamps() {
    let p = Project::new();
    let source = p.project.join("src/main.rs");
    let old = std::fs::metadata(&source).unwrap().modified().unwrap();
    let bytes = std::fs::read(&source).unwrap();
    write_changed(&source, &bytes).unwrap();
    assert_eq!(old, std::fs::metadata(&source).unwrap().modified().unwrap());
    std::fs::write(&source, b"different content").unwrap();
    std::fs::File::options()
        .write(true)
        .open(&source)
        .unwrap()
        .set_times(std::fs::FileTimes::new().set_modified(old))
        .unwrap();
    write_changed(&source, &bytes).unwrap();
    assert_eq!(std::fs::read(source).unwrap(), bytes);
}

#[test]
fn cargo_validates_noop_source_features_configuration_and_profile() {
    let p = Project::new();
    let first = p.build(&[]).unwrap();
    assert!(!first.fresh);
    assert_eq!(run(&first.path), "one\n");
    assert!(p.build(&[]).unwrap().fresh);
    write_changed(
        &p.project.join("src/main.rs"),
        b"fn main() { println!(\"two\"); }\n",
    )
    .unwrap();
    let edited = p.build(&[]).unwrap();
    assert!(!edited.fresh);
    assert_eq!(run(&edited.path), "two\n");
    assert!(!p.build(&["--features", "changed"]).unwrap().fresh);
    assert!(p.build(&[]).unwrap().fresh);
    assert!(
        !p.build(&[
            "--config",
            "build.rustflags=['--cfg', 'configuration_control']"
        ])
        .unwrap()
        .fresh
    );
    let release = p.build(&["--release"]).unwrap();
    assert!(!release.fresh);
    assert!(
        release
            .path
            .components()
            .any(|c| c.as_os_str() == "release")
    );
    assert!(p.build(&["--release"]).unwrap().fresh);
}

#[test]
fn failed_runtime_edit_never_uses_previous_executable_and_dependencies_are_shared() {
    let p = Project::new();
    let runtime = p.temp.path().join("runtime");
    std::fs::create_dir_all(runtime.join("src")).unwrap();
    std::fs::write(
        runtime.join("Cargo.toml"),
        "[package]\nname='runtime_probe'\nversion='0.1.0'\nedition='2024'\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(runtime.join("src/lib.rs"), "pub fn value() -> u8 { 1 }\n").unwrap();
    let manifest = p.project.join("Cargo.toml");
    let mut content = std::fs::read_to_string(&manifest).unwrap();
    content.push_str("[dependencies]\nruntime_probe={path='../runtime'}\n");
    write_changed(&manifest, content.as_bytes()).unwrap();
    write_changed(
        &p.project.join("src/main.rs"),
        b"fn main() { println!(\"{}\", runtime_probe::value()); }\n",
    )
    .unwrap();
    let first = p.build(&[]).unwrap();
    assert_eq!(run(&first.path), "1\n");
    let second_project = p.temp.path().join("second");
    std::fs::create_dir_all(second_project.join("src")).unwrap();
    std::fs::write(
        second_project.join("Cargo.toml"),
        content.replace("name='probe'", "name='second'"),
    )
    .unwrap();
    std::fs::write(
        second_project.join("src/main.rs"),
        "fn main() { let _ = runtime_probe::value(); }",
    )
    .unwrap();
    let second = Command::new("cargo")
        .args(["build", "--offline", "--message-format=json", "-j", "1"])
        .env("CARGO_TARGET_DIR", &p.target)
        .current_dir(&second_project)
        .output()
        .unwrap();
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    assert!(String::from_utf8_lossy(&second.stdout).lines().any(|line| {
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        v["reason"] == "compiler-artifact"
            && v["target"]["name"] == "runtime_probe"
            && v["fresh"] == true
    }));
    // A real path dependency edit must pass through Cargo and reject the old pass.
    std::fs::write(
        runtime.join("src/lib.rs"),
        "compile_error!(\"intentional runtime control\");\n",
    )
    .unwrap();
    assert!(p.build(&[]).unwrap_err().contains("Cargo failed"));
    assert!(first.path.exists());
    assert!(p.target.join("debug/deps").is_dir());
    std::fs::write(runtime.join("src/lib.rs"), "pub fn value() -> u8 { 2 }\n").unwrap();
    assert_eq!(run(&p.build(&[]).unwrap().path), "2\n");
}

#[test]
fn missing_and_corrupt_artifacts_cannot_run_as_a_cached_pass() {
    let p = Project::new();
    let artifact = p.build(&[]).unwrap();
    std::fs::remove_file(&artifact.path).unwrap();
    // Cargo either rebuilds or the selector rejects a missing fresh receipt.
    let rebuilt = p.build(&[]).unwrap();
    assert_eq!(run(&rebuilt.path), "one\n");
    std::fs::write(&rebuilt.path, b"corrupt executable").unwrap();
    assert!(p.build(&[]).unwrap_err().contains("integrity mismatch"));
}

#[test]
fn artifact_selection_rejects_wrong_package_failed_or_incomplete_stream() {
    let p = Project::new();
    let output = p.cargo(&[]);
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(select_artifact(false, &stdout, &p.project, &p.target, "probe").is_err());
    assert!(select_artifact(true, &stdout, &p.project, &p.target, "other").is_err());
    assert!(
        select_artifact(
            true,
            &stdout.replace("probe@0.1.0", "other@0.1.0"),
            &p.project,
            &p.target,
            "probe"
        )
        .is_err()
    );
    let incomplete = stdout
        .lines()
        .filter(|l| !l.contains("build-finished"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(select_artifact(true, &incomplete, &p.project, &p.target, "probe").is_err());
    assert!(select_artifact(true, "broken JSON", &p.project, &p.target, "probe").is_err());
}

#[test]
fn suite_lease_excludes_another_run_and_checks_checkout_ownership() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let first = NativeCache::acquire(&root).unwrap();
    assert!(NativeCache::acquire(&root).is_err());
    let owner_path = first.root.join("owner");
    drop(first);
    assert!(NativeCache::acquire(&root).is_ok());
    write_changed(&owner_path, b"another checkout").unwrap();
    assert!(NativeCache::acquire(&root).is_err());
}

#[cfg(unix)]
#[test]
fn interrupted_owner_cannot_release_a_live_native_child_lease() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let lease = NativeCache::acquire(&root).unwrap();
    let mut child = Command::new("sleep").arg("30").spawn().unwrap();
    drop(lease); // Equivalent to the owner process losing its descriptor.
    assert!(NativeCache::acquire(&root).is_err());
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(NativeCache::acquire(&root).is_ok());
}
