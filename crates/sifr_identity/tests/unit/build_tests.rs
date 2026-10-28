use super::{IdentityEncoder, input_tree::hash_tree, production_inputs::visit_package};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "sifr-identity-watch-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, bytes: &str) {
        let path = self.0.join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn production_ownership_requires_complete_valid_explicit_roots() {
    let tree = Scratch::new();
    tree.write("src/lib.rs", "production");
    let hash = || {
        let mut encoder = IdentityEncoder::new("ownership-test");
        visit_package(&tree.0, &tree.0, &mut encoder).map(|()| encoder.finish())
    };
    for manifest in [
        "[package]\nname='probe'\n",
        "[package.metadata.sifr]\nproduction-inputs=['../outside']\n",
        "[package.metadata.sifr]\nproduction-inputs=['/outside']\n",
        "[package.metadata.sifr]\nproduction-inputs=['src','src/lib.rs']\n",
        "[package.metadata.sifr]\nproduction-inputs=['src','missing.asset']\n",
        "[package.metadata.sifr]\nproduction-inputs=['src',false]\n",
        "[package.metadata.sifr]\nproduction-inputs=['Cargo.toml','src']\n",
        "[package.metadata.sifr]\nproduction-inputs=[]\n",
    ] {
        tree.write("Cargo.toml", manifest);
        assert!(hash().is_err(), "accepted invalid ownership: {manifest}");
    }
    tree.write(
        "Cargo.toml",
        "[package.metadata.sifr]\nproduction-inputs=['src']\n",
    );
    let before = hash().unwrap();
    tree.write("tests/unit/assertions.rs", "private assertion");
    assert_eq!(before, hash().unwrap());
    tree.write("build.rs", "fn main() {} ");
    assert!(hash().is_err(), "an undeclared build script is not safe");
    tree.write(
        "Cargo.toml",
        "[package.metadata.sifr]\nproduction-inputs=['src','build.rs']\n",
    );
    assert_ne!(before, hash().unwrap());
}

#[test]
fn workspace_input_ownership_rejects_missing_assets_and_package_cycles() {
    let tree = Scratch::new();
    tree.write("crates/owner/src/lib.rs", "production");
    tree.write("shared/asset.md", "embedded");
    for inputs in [
        "false",
        "['missing']",
        "['../escape']",
        "['shared','shared']",
        "['crates/owner']",
    ] {
        tree.write("crates/owner/Cargo.toml", &format!(
            "[package.metadata.sifr]\nproduction-inputs=['src']\nproduction-workspace-inputs={inputs}\n"
        ));
        let mut hash = IdentityEncoder::new("workspace-ownership-test");
        assert!(
            visit_package(&tree.0, &tree.0.join("crates/owner"), &mut hash).is_err(),
            "accepted invalid workspace ownership: {inputs}"
        );
    }
    tree.write("crates/owner/Cargo.toml", "[package.metadata.sifr]\nproduction-inputs=['src']\nproduction-workspace-inputs=['shared']\n");
    let mut hash = IdentityEncoder::new("workspace-ownership-test");
    visit_package(&tree.0, &tree.0.join("crates/owner"), &mut hash).unwrap();
}

#[test]
fn directory_watches_preserve_framing_order_and_own_embedded_assets() {
    let tree = Scratch::new();
    for (name, bytes) in [
        ("inputs/z.rs", "last"),
        ("inputs/a/deep.rs", "first"),
        ("inputs/readme.md", "embedded"),
        ("inputs/note.mdx", "embedded"),
        ("inputs/.git/state", "excluded"),
        ("inputs/target/obj", "excluded"),
        ("inputs/__pycache__/x", "excluded"),
    ] {
        tree.write(name, bytes);
    }
    let mut actual = IdentityEncoder::new("watch-test");
    hash_tree(&tree.0, &tree.0.join("inputs"), &mut actual).unwrap();
    let mut expected = IdentityEncoder::new("watch-test");
    expected.field(
        &["inputs", "a", "deep.rs"].join(std::path::MAIN_SEPARATOR_STR),
        b"first",
    );
    expected.field(
        &["inputs", "note.mdx"].join(std::path::MAIN_SEPARATOR_STR),
        b"embedded",
    );
    expected.field(
        &["inputs", "readme.md"].join(std::path::MAIN_SEPARATOR_STR),
        b"embedded",
    );
    expected.field(
        &["inputs", "z.rs"].join(std::path::MAIN_SEPARATOR_STR),
        b"last",
    );
    assert_eq!(actual.finish(), expected.finish());
    // Missing optional roots previously contributed no fields; preserve that behavior.
    let mut missing = IdentityEncoder::new("watch-test");
    hash_tree(&tree.0, &tree.0.join("absent"), &mut missing).unwrap();
    assert_eq!(
        missing.finish(),
        IdentityEncoder::new("watch-test").finish()
    );
}

struct CargoProbe(Scratch);
impl CargoProbe {
    fn new() -> Self {
        let tree = Scratch::new();
        let identity = Path::new(env!("CARGO_MANIFEST_DIR"));
        // A source archive fixture: no Git inventory and a separate, owned Cargo target.
        tree.write(
            "Cargo.toml",
            &format!(
                "[package]\nname='identity_watch_probe'\nversion='0.0.0'\nedition='2024'\n\
             [workspace]\n[profile.dev]\ndebug=0\n\
             [build-dependencies]\nsifr_identity={{path={:?}}}\n",
                identity.to_str().unwrap()
            ),
        );
        tree.write("src/main.rs", "fn main() { assert_eq!(env!(\"TRACKED_HASH\"), std::env::var(\"EXPECTED_HASH\").unwrap()); println!(\"input-verified\"); }\n");
        tree.write("build.rs", "use sifr_identity::IdentityEncoder;\n#[allow(dead_code)] mod inventory;\nfn main() { let root=std::path::PathBuf::from(std::env::var_os(\"CARGO_MANIFEST_DIR\").unwrap()); println!(\"cargo:rustc-env=TRACKED_HASH={}\", inventory::probe(&root).unwrap()); }\n");
        let source = fs::read_to_string(identity.join("src/input_tree.rs")).unwrap();
        tree.write("inventory.rs", &(source + "\npub fn probe(root: &Path) -> io::Result<String> { let mut hash = IdentityEncoder::new(\"watch-test\"); visit(root, &root.join(\"inputs\"), &mut hash)?; visit(root, &root.join(\"config.txt\"), &mut hash)?; Ok(hash.finish()) }\n"));
        tree.write("inputs/nested/deep/value.txt", "before");
        tree.write("config.txt", "configuration");
        Self(tree)
    }
    fn run(&self, label: &str, fresh: bool) {
        let tree = &self.0;
        let mut hash = IdentityEncoder::new("watch-test");
        hash_tree(&tree.0, &tree.0.join("inputs"), &mut hash).unwrap();
        hash_tree(&tree.0, &tree.0.join("config.txt"), &mut hash).unwrap();
        let mut cargo = Command::new(env!("CARGO"));
        cargo
            .current_dir(&tree.0)
            .args(["run", "--offline", "--message-format=json"]);
        if tree.0.join("Cargo.lock").exists() {
            cargo.arg("--locked");
        }
        // Parent package metadata must not enter this independent Cargo invocation.
        for (name, _) in std::env::vars_os() {
            let text = name.to_string_lossy();
            if text.starts_with("CARGO_PKG_")
                || matches!(
                    text.as_ref(),
                    "CARGO_MANIFEST_DIR" | "CARGO_MANIFEST_PATH" | "OUT_DIR"
                )
            {
                cargo.env_remove(name);
            }
        }
        cargo
            .env("CARGO_TARGET_DIR", tree.0.join("target"))
            .env("EXPECTED_HASH", hash.finish());
        let output = cargo.output().unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(output.status.success(), "{label}: {stdout}\n{stderr}");
        assert!(
            stdout.lines().any(|line| line == "input-verified"),
            "{label}: no execution"
        );
        let artifacts: Vec<_> = stdout
            .lines()
            .filter(|line| line.contains("\"reason\":\"compiler-artifact\""))
            .collect();
        assert!(!artifacts.is_empty(), "{label}: no Cargo artifact evidence");
        assert_eq!(
            artifacts.iter().all(|line| line.contains("\"fresh\":true")),
            fresh,
            "{label}: {stdout}"
        );
    }
}

#[test]
fn cargo_directory_watches_track_the_hashed_inventory() {
    let probe = CargoProbe::new();
    let tree = &probe.0;
    probe.run("initial", false);
    probe.run("unchanged", true);
    tree.write("inputs/nested/deep/value.txt", "edited");
    probe.run("nested edit", false);
    tree.write("inputs/new/deeper/added.txt", "added");
    probe.run("new nested directory and file", false);
    fs::rename(
        tree.0.join("inputs/new/deeper/added.txt"),
        tree.0.join("inputs/new/deeper/renamed.txt"),
    )
    .unwrap();
    probe.run("rename", false);
    fs::remove_file(tree.0.join("inputs/new/deeper/renamed.txt")).unwrap();
    probe.run("remove file", false);
    fs::remove_dir_all(tree.0.join("inputs/new")).unwrap();
    probe.run("remove directory", false);
    tree.write("config.txt", "changed configuration");
    probe.run("configuration", false);
    tree.write("outside/unrelated.txt", "not a watched input");
    probe.run("unrelated file", true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        tree.write("outside/file.txt", "link-before");
        symlink(
            tree.0.join("outside/file.txt"),
            tree.0.join("inputs/file-link"),
        )
        .unwrap();
        probe.run("new file symlink", false);
        tree.write("outside/file.txt", "link-after");
        probe.run("edit linked file", false);
        tree.write("outside/directory/deep/file.txt", "directory-before");
        symlink(
            tree.0.join("outside/directory"),
            tree.0.join("inputs/directory-link"),
        )
        .unwrap();
        probe.run("new directory symlink", false);
        tree.write("outside/directory/deep/file.txt", "directory-after");
        probe.run("edit linked directory file", false);
        tree.write("outside/directory/deep/new.txt", "new");
        probe.run("add linked directory file", false);
        fs::remove_file(tree.0.join("outside/directory/deep/new.txt")).unwrap();
        probe.run("remove linked directory file", false);
        probe.run("unchanged symlinks", true);
    }
}

#[test]
fn product_source_identity_tracks_sources_independently_of_configuration() {
    let tree = Scratch::new();
    let identity = Path::new(env!("CARGO_MANIFEST_DIR"));
    tree.write(
        "Cargo.toml",
        "[workspace]\nmembers=['crates/probe']\nresolver='3'\n[profile.dev]\ndebug=0\n",
    );
    tree.write(
        "crates/probe/Cargo.toml",
        &format!(
            "[package]\nname='product_identity_probe'\nversion='0.0.0'\nedition='2024'\n\
             [package.metadata.sifr]\nproduction-inputs=['src','build.rs','semantic-input.txt']\nproduction-workspace-inputs=['shared']\n\
             [features]\nalternate=[]\n[build-dependencies]\n\
             sifr_identity={{path={:?},features=['build']}}\n",
            identity.to_str().unwrap()
        ),
    );
    tree.write(
        "crates/probe/build.rs",
        "fn main() { sifr_identity::build::emit_product_identity().unwrap(); }\n",
    );
    tree.write(
        "crates/probe/src/main.rs",
        "fn main() { assert_eq!(include_str!(\"../../../shared/embedded.md\"), std::env::var(\"EXPECTED_ASSET\").unwrap()); println!(\"{} {}\", env!(\"SIFR_COMPILER_SOURCE_ID\"), env!(\"SIFR_COMPILER_BUILD_ID\")); }\n",
    );
    tree.write("crates/probe/semantic-input.txt", "initial");
    tree.write("shared/embedded.md", "embedded-before");
    tree.write("crates/probe/tests/unit/assertions.rs", "assertion before");
    tree.write(".cargo/config.toml", "");
    tree.write("third_party/owned.txt", "third party input");
    tree.write(
        "rust-toolchain.toml",
        &fs::read_to_string(identity.join("../../rust-toolchain.toml")).unwrap(),
    );
    let run =
        |alternate: bool, fresh: Option<bool>| {
            let mut cargo = Command::new(env!("CARGO"));
            cargo
                .current_dir(&tree.0)
                .args([
                    "run",
                    "--quiet",
                    "--message-format=json",
                    "--offline",
                    "-p",
                    "product_identity_probe",
                ])
                .env("CARGO_TARGET_DIR", tree.0.join("target"))
                .env(
                    "EXPECTED_ASSET",
                    fs::read_to_string(tree.0.join("shared/embedded.md")).unwrap(),
                );
            if alternate {
                cargo.args(["--features", "alternate"]);
            }
            if tree.0.join("Cargo.lock").exists() {
                cargo.arg("--locked");
            }
            for (name, _) in std::env::vars_os() {
                let name_text = name.to_string_lossy();
                if name_text.starts_with("CARGO_PKG_")
                    || name_text.starts_with("CARGO_FEATURE_")
                    || name_text.starts_with("CARGO_CFG_")
                    || name_text.starts_with("DEP_")
                    || matches!(
                        name_text.as_ref(),
                        "CARGO_MANIFEST_DIR" | "CARGO_MANIFEST_PATH" | "OUT_DIR"
                    )
                {
                    cargo.env_remove(name);
                }
            }
            let output = cargo.output().unwrap();
            assert!(
                output.status.success(),
                "identity probe: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let stdout = String::from_utf8(output.stdout).unwrap();
            if let Some(expected) = fresh {
                let artifacts: Vec<_> = stdout
                    .lines()
                    .filter(|line| line.contains("\"reason\":\"compiler-artifact\""))
                    .collect();
                assert!(!artifacts.is_empty(), "missing Cargo freshness evidence");
                assert_eq!(
                    artifacts.iter().all(|line| line.contains("\"fresh\":true")),
                    expected,
                    "{stdout}"
                );
            }
            let fields: Vec<_> = stdout
                .lines()
                .last()
                .unwrap()
                .split_whitespace()
                .map(str::to_string)
                .collect();
            assert_eq!(fields.len(), 2, "{stdout}");
            assert!(fields.iter().all(
                |value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
            ));
            (fields[0].clone(), fields[1].clone())
        };
    let initial = run(false, Some(false));
    let configured = run(true, None);
    assert_eq!(initial.0, configured.0, "same source, different features");
    assert_ne!(
        initial.1, configured.1,
        "configured identity remains distinct"
    );
    assert_eq!(
        configured,
        run(true, Some(true)),
        "unchanged build is fresh"
    );
    tree.write("crates/probe/tests/unit/assertions.rs", "assertion after");
    assert_eq!(
        configured,
        run(true, Some(true)),
        "private assertions are not production inputs"
    );
    tree.write("crates/probe/tests/unit/support.rs", "new private support");
    assert_eq!(
        configured,
        run(true, Some(true)),
        "new private support preserves production"
    );
    tree.write(".git/HEAD", "new-revision-with-the-same-sources");
    assert_eq!(
        configured,
        run(true, None),
        "revision labels are not source inputs"
    );
    tree.write("crates/probe/semantic-input.txt", "edited");
    let edited = run(true, Some(false));
    assert_ne!(configured.0, edited.0, "semantic inputs invalidate pairing");
    assert_ne!(
        configured.1, edited.1,
        "configured identity still tracks source"
    );
    tree.write("crates/probe/src/embedded.md", "new production asset");
    let added = run(true, Some(false));
    assert_ne!(edited, added, "new source assets invalidate");
    fs::remove_file(tree.0.join("crates/probe/src/embedded.md")).unwrap();
    assert_eq!(
        edited,
        run(true, Some(false)),
        "removed source assets invalidate and restore identity"
    );
    tree.write("shared/embedded.md", "embedded-after");
    assert_ne!(
        edited,
        run(true, Some(false)),
        "declared external assets reach the actual consumer"
    );
}
