use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const TEST_METADATA_PATH_ENV: &str = "SIFR_TEST_CHANNEL_METADATA_PATH";

pub(crate) fn load(public_url: &str, dry_run: bool) -> Result<String, String> {
    match resolve_test_fixture(std::env::var_os(TEST_METADATA_PATH_ENV), dry_run)? {
        Some(path) => read_fixture(&path),
        None => fetch_public(public_url),
    }
}

fn resolve_test_fixture(path: Option<OsString>, dry_run: bool) -> Result<Option<PathBuf>, String> {
    let Some(path) = path else {
        return Ok(None);
    };
    if !dry_run {
        return Err(format!(
            "{TEST_METADATA_PATH_ENV} is permitted only with self update --dry-run"
        ));
    }
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        return Err(format!(
            "{TEST_METADATA_PATH_ENV} must name an absolute test fixture"
        ));
    }
    Ok(Some(path))
}

fn read_fixture(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        format!("{TEST_METADATA_PATH_ENV} test fixture cannot be inspected: {error}")
    })?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "{TEST_METADATA_PATH_ENV} does not name a regular file"
        ));
    }
    fs::read_to_string(path)
        .map_err(|error| format!("could not read self-update test metadata fixture: {error}"))
}

fn fetch_public(public_url: &str) -> Result<String, String> {
    let output = Command::new("curl")
        .args(["-fsSL", public_url])
        .output()
        .map_err(|error| format!("could not run curl to fetch self-update metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!("self-update metadata unavailable at {public_url}"));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| format!("self-update metadata was not UTF-8: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{read_fixture, resolve_test_fixture};
    use std::collections::HashSet;
    use std::ffi::OsString;
    use std::fs;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn test_fixture_must_be_absolute() {
        let error = resolve_test_fixture(Some(OsString::from("channels.json")), true)
            .expect_err("relative fixture must fail");
        assert!(error.contains("absolute test fixture"));
    }

    #[test]
    fn test_fixture_is_dry_run_only() {
        let error = resolve_test_fixture(Some(OsString::from("/tmp/channels.json")), false)
            .expect_err("real update must reject the fixture");
        assert!(error.contains("only with self update --dry-run"));
        assert_eq!(
            resolve_test_fixture(None, false).expect("public source"),
            None
        );
    }

    #[test]
    fn reads_utf8_test_fixture_and_rejects_invalid_inputs() {
        let root = unique_test_root();
        let path = root.path().join("channels.json");
        let non_utf8 = root.path().join("non-utf8.json");
        fs::write(&path, "{\"schema_version\":2}\n").expect("write fixture");
        fs::write(&non_utf8, [0xff]).expect("write non-UTF-8 fixture");
        let result = read_fixture(&path).expect("read fixture");
        assert_eq!(result, "{\"schema_version\":2}\n");
        assert!(
            read_fixture(root.path())
                .expect_err("directory must fail")
                .contains("regular file")
        );
        assert!(
            read_fixture(&non_utf8)
                .expect_err("non-UTF-8 must fail")
                .contains("could not read")
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlink_fixture() {
        use std::os::unix::fs::symlink;

        let root = unique_test_root();
        let target = root.path().join("target.json");
        let link = root.path().join("channels.json");
        fs::write(&target, "{}\n").expect("write target");
        symlink(&target, &link).expect("create symlink");
        assert!(
            read_fixture(&link)
                .expect_err("symlink must fail")
                .contains("regular file")
        );
    }

    #[test]
    fn allocates_distinct_test_roots_concurrently() {
        const ROOT_COUNT: usize = 32;

        let barrier = Arc::new(Barrier::new(ROOT_COUNT));
        let handles = (0..ROOT_COUNT)
            .map(|_| {
                let barrier = Arc::clone(&barrier);
                thread::spawn(move || {
                    barrier.wait();
                    unique_test_root()
                })
            })
            .collect::<Vec<_>>();
        let roots = handles
            .into_iter()
            .map(|handle| handle.join().expect("test-root allocation thread"))
            .collect::<Vec<_>>();
        let paths = roots
            .iter()
            .map(|root| root.path().to_path_buf())
            .collect::<HashSet<_>>();

        assert_eq!(paths.len(), ROOT_COUNT);
    }

    fn unique_test_root() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("sifr-self-update-metadata-")
            .tempdir()
            .expect("create unique test root")
    }
}
