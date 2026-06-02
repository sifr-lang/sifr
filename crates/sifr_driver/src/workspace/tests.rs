use super::*;
use std::sync::{Mutex, MutexGuard, OnceLock};

static CWD_TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct CurrentDirGuard {
    previous: PathBuf,
    _lock: MutexGuard<'static, ()>,
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        std::env::set_current_dir(&self.previous).expect("restore cwd");
    }
}

fn enter_test_cwd(path: &Path) -> CurrentDirGuard {
    let lock = CWD_TEST_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("cwd test lock should not be poisoned");
    let previous = std::env::current_dir().expect("cwd exists");
    std::env::set_current_dir(path).expect("chdir to test cwd");
    CurrentDirGuard {
        previous,
        _lock: lock,
    }
}

struct TempWorkspace {
    path: PathBuf,
}

impl TempWorkspace {
    fn new(name: &str) -> Self {
        let unique = format!(
            "sifr_workspace_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("time should move forward")
                .as_nanos()
        );
        let path = std::env::temp_dir().join(unique);
        std::fs::create_dir_all(&path).expect("temp workspace should be created");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn write(&self, relative: &str, contents: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("parent should be created");
        }
        std::fs::write(path, contents).expect("file should be written");
    }

    fn mkdir(&self, relative: &str) {
        std::fs::create_dir_all(self.path.join(relative)).expect("dir should be created");
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn discovered(entry: &Path) -> WorkspaceRoot {
    find_workspace_root(entry)
        .expect("workspace discovery should succeed")
        .expect("workspace should be found")
}

fn discovery_error(entry: &Path) -> String {
    find_workspace_root(entry)
        .expect_err("workspace discovery should fail")
        .into_iter()
        .map(|error| error.message)
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn test_find_workspace_root_returns_nearest_manifest() {
    let tmp = TempWorkspace::new("nearest");
    tmp.mkdir("outer/inner/src");
    tmp.write("outer/sifr.toml", "[package]\nname = \"outer\"\n");
    tmp.write("outer/inner/sifr.toml", "[package]\nname = \"inner\"\n");
    tmp.write("outer/inner/src/main.sifr", "");

    let root = discovered(&tmp.path().join("outer/inner/src/main.sifr"));

    assert_eq!(root.dir, tmp.path().join("outer/inner"));
    assert_eq!(root.config.package_name.as_deref(), Some("inner"));
    assert_eq!(root.config.source_roots, vec![PathBuf::from(".")]);
}

#[test]
fn test_find_workspace_root_normalizes_relative_root_manifest_to_curdir() {
    let tmp = TempWorkspace::new("relative_root");
    tmp.write("sifr.toml", "[package]\nname = \"relative\"\n");
    tmp.write("main.sifr", "");
    let _cwd = enter_test_cwd(tmp.path());

    let root = discovered(Path::new("main.sifr"));

    assert_eq!(root.dir, PathBuf::from("."));
    assert_eq!(root.config.package_name.as_deref(), Some("relative"));
}

#[test]
fn test_find_workspace_root_returns_none_without_manifest() {
    let tmp = TempWorkspace::new("missing");
    tmp.write("src/main.sifr", "");

    assert_eq!(
        find_workspace_root(&tmp.path().join("src/main.sifr")).expect("discovery should succeed"),
        None
    );
}

#[test]
fn test_malformed_manifest_is_hard_error() {
    let tmp = TempWorkspace::new("malformed");
    tmp.write("sifr.toml", "[package\nname = \"bad\"\n");
    tmp.write("main.sifr", "");

    let message = discovery_error(&tmp.path().join("main.sifr"));

    assert!(message.contains("could not parse sifr.toml"));
}

#[test]
fn test_package_name_must_be_string() {
    let tmp = TempWorkspace::new("package_name_type");
    tmp.write("sifr.toml", "[package]\nname = 1\n");
    tmp.write("main.sifr", "");

    let message = discovery_error(&tmp.path().join("main.sifr"));

    assert!(message.contains("package.name must be a string"));
}

#[test]
fn test_source_roots_must_be_list_of_strings() {
    let tmp = TempWorkspace::new("roots_type");
    tmp.write("sifr.toml", "[source]\nroots = \"src\"\n");
    tmp.write("main.sifr", "");

    let message = discovery_error(&tmp.path().join("main.sifr"));

    assert!(message.contains("source.roots must be a list of strings"));
}

#[test]
fn test_source_root_entries_must_be_strings() {
    let tmp = TempWorkspace::new("root_entry_type");
    tmp.write("sifr.toml", "[source]\nroots = [1]\n");
    tmp.write("main.sifr", "");

    let message = discovery_error(&tmp.path().join("main.sifr"));

    assert!(message.contains("source.roots must be a list of strings"));
}

#[test]
fn test_omitted_roots_missing_package_and_empty_manifest_are_valid() {
    let tmp = TempWorkspace::new("defaults");
    tmp.write("src/main.sifr", "");

    tmp.write("sifr.toml", "");
    let empty = discovered(&tmp.path().join("src/main.sifr"));
    assert_eq!(empty.config.source_roots, vec![PathBuf::from(".")]);
    assert_eq!(empty.config.package_name, None);

    tmp.write("sifr.toml", "[source]\n");
    let omitted = discovered(&tmp.path().join("src/main.sifr"));
    assert_eq!(omitted.config.source_roots, vec![PathBuf::from(".")]);
    assert_eq!(omitted.config.package_name, None);
}

#[test]
fn test_unknown_tables_and_keys_are_ignored() {
    let tmp = TempWorkspace::new("unknown");
    tmp.mkdir("src");
    tmp.write(
            "sifr.toml",
            "[workspace]\nresolver = \"1\"\n[dependencies]\nfoo = \"0.1\"\n[source]\nroots = [\"src\"]\nextra = true\n",
        );
    tmp.write("src/main.sifr", "");

    let root = discovered(&tmp.path().join("src/main.sifr"));

    assert_eq!(root.config.source_roots, vec![PathBuf::from("src")]);
}

#[test]
fn test_source_roots_reject_escape_absolute_empty_missing_and_file_paths() {
    let tmp = TempWorkspace::new("invalid_roots");
    tmp.write("main.sifr", "");
    tmp.write("not_dir", "");

    for (manifest, expected) in [
        (
            "[source]\nroots = [\"../outside\"]\n",
            "escapes the workspace root",
        ),
        (
            "[source]\nroots = [\"/tmp\"]\n",
            "must be a relative non-empty path",
        ),
        (
            "[source]\nroots = [\"\"]\n",
            "must be a relative non-empty path",
        ),
        ("[source]\nroots = [\"missing\"]\n", "is not a directory"),
        ("[source]\nroots = [\"not_dir\"]\n", "is not a directory"),
    ] {
        tmp.write("sifr.toml", manifest);
        let message = discovery_error(&tmp.path().join("main.sifr"));
        assert!(
            message.contains(expected),
            "expected '{message}' to contain '{expected}'"
        );
    }
}

#[test]
fn test_leading_curdir_source_root_is_normalized() {
    let tmp = TempWorkspace::new("curdir");
    tmp.mkdir("src");
    tmp.write("sifr.toml", "[source]\nroots = [\"./src\", \".\"]\n");
    tmp.write("src/main.sifr", "");

    let root = discovered(&tmp.path().join("src/main.sifr"));

    assert_eq!(
        root.config.source_roots,
        vec![PathBuf::from("src"), PathBuf::from(".")]
    );
}

#[test]
fn test_path_separator_source_root_uses_platform_components() {
    let tmp = TempWorkspace::new("separator");
    let nested = PathBuf::from("src").join("nested");
    tmp.mkdir(nested.to_str().expect("test path should be unicode"));
    tmp.write(
        "sifr.toml",
        &format!(
            "[source]\nroots = [\"{}\"]\n",
            nested.to_string_lossy().replace('\\', "\\\\")
        ),
    );
    tmp.write("src/nested/main.sifr", "");

    let root = discovered(&tmp.path().join("src/nested/main.sifr"));

    assert_eq!(root.config.source_roots, vec![nested]);
}

#[test]
fn test_closer_valid_manifest_ignores_farther_malformed_manifest() {
    let tmp = TempWorkspace::new("nearest_wins");
    tmp.mkdir("outer/inner");
    tmp.write("sifr.toml", "[package\nname = \"bad\"\n");
    tmp.write("outer/inner/sifr.toml", "[package]\nname = \"ok\"\n");
    tmp.write("outer/inner/main.sifr", "");

    let root = discovered(&tmp.path().join("outer/inner/main.sifr"));

    assert_eq!(root.dir, tmp.path().join("outer/inner"));
    assert_eq!(root.config.package_name.as_deref(), Some("ok"));
}
