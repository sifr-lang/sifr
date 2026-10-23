use super::{IdentityEncoder, hash_tree};
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
fn directory_watch_refactor_preserves_hash_framing_order_and_exclusions() {
    let tree = Scratch::new();
    for (name, bytes) in [
        ("inputs/z.rs", "last"),
        ("inputs/a/deep.rs", "first"),
        ("inputs/readme.md", "excluded"),
        ("inputs/note.mdx", "excluded"),
        ("inputs/.git/state", "excluded"),
        ("inputs/target/obj", "excluded"),
        ("inputs/__pycache__/x", "excluded"),
    ] {
        tree.write(name, bytes);
    }
    let mut actual = IdentityEncoder::new("watch-test");
    hash_tree(&tree.0, &tree.0.join("inputs"), &mut actual).unwrap();
    let mut expected = IdentityEncoder::new("watch-test");
    expected.field("inputs/a/deep.rs", b"first");
    expected.field("inputs/z.rs", b"last");
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
        let source = fs::read_to_string(identity.join("src/build.rs")).unwrap();
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
